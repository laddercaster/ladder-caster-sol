use core::mem::size_of;

use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};
use solana_maths::{Rate, TryMul};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::{
    generate_new_equipment, get_current_tile, get_player_bonuses,
    give_exp_to_caster_resources_burned, give_exp_to_caster_spell, is_spell_successful,
    zombify_account, ItemRarity, ItemType, RandomValue, SpellType, DECIMALS_PRECISION, EARTH_INDEX,
    FIRE_INDEX, GAME_CREATOR_AUTHORITY_PUBKEY, LADA_DISTRIBUTION_PER_TURN, WATER_INDEX,
};
use crate::{Tile, TileType};

#[derive(Accounts)]
pub struct Action<'info> {
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub game: Box<Account<'info, Game>>,
    #[account(mut, has_one = authority, has_one = game)]
    pub player: Box<Account<'info, Player>>,
    #[account(mut, constraint = caster.owner == player.key())]
    pub caster: Box<Account<'info, Caster>>,

    #[account(mut, seeds = [b"game_signer"], bump)]
    pub game_signer: UncheckedAccount<'info>,

    #[account(address = sysvar::slot_hashes::id())]
    pub slots: UncheckedAccount<'info>,

    #[account(mut, constraint = resource_1_mint_account.to_account_info().key() == game.resource_1_mint_account)]
    pub resource_1_mint_account: Box<Account<'info, Mint>>,
    #[account(mut, constraint = resource_2_mint_account.to_account_info().key() == game.resource_2_mint_account)]
    pub resource_2_mint_account: Box<Account<'info, Mint>>,
    #[account(mut, constraint = resource_3_mint_account.to_account_info().key() == game.resource_3_mint_account)]
    pub resource_3_mint_account: Box<Account<'info, Mint>>,

    #[account(init_if_needed,
    associated_token::mint = resource_1_mint_account,
    associated_token::authority = authority,
    payer = authority)]
    pub resource_1_token_account: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed,
    associated_token::mint = resource_2_mint_account,
    associated_token::authority = authority,
    payer = authority)]
    pub resource_2_token_account: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed,
    associated_token::mint = resource_3_mint_account,
    associated_token::authority = authority,
    payer = authority)]
    pub resource_3_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = lada_mint_account.to_account_info().key() == game.lada_mint_account)]
    pub lada_mint_account: Box<Account<'info, Mint>>,

    #[account(mut, constraint = game_lada_token_account.key() == game.lada_token_account)]
    pub game_lada_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub lada_token_account: Box<Account<'info, TokenAccount>>,

    //Got to do -1 to the turn because we want the turn BEFORE the crank
    #[account(mut, seeds = [
    b"turn_data",
    game.to_account_info().key().as_ref(),
    (caster.turn_commit.unwrap().turn).to_string().as_ref()
    ], bump = game_turn_data.bump)]
    pub game_turn_data: Box<Account<'info, TurnData>>,

    #[account(init, space =  Item::SIZE, payer = authority)]
    pub item: Box<Account<'info, Item>>, //This will represent the item that could potentially be created by the casted spell

    //There will be a remaining account that represents the spell book item, so that we can
    //zero it out (burn it) at index 0
    //We will also use sub instructions for this,

    // Optional accounts for player bonuses
    pub staff: UncheckedAccount<'info>,
    pub head: UncheckedAccount<'info>,
    pub robe: UncheckedAccount<'info>,
}

//noinspection RsExternalLinter
pub fn caster_redeem_action<'info>(
    ctx: Context<'_, '_, '_, 'info, Action<'info>>,
) -> ProgramResult {
    let caster = &mut ctx.accounts.caster;
    let game = &ctx.accounts.game;
    let player = &ctx.accounts.player;
    let turn_data = &ctx.accounts.game_turn_data;

    // Flag used to burn item account if not populated
    let mut burn_item_account = true;

    match caster.turn_commit {
        None => {
            return Err(ErrorCode::EmptyTurnCommit.into());
        }
        Some(turn_commit) => {
            if game.turn_info.turn == turn_commit.turn {
                return Err(ErrorCode::SameTurnRedeem.into());
            }

            let slots_ref = ctx.accounts.slots.data.borrow();
            let slots = &**slots_ref;
            let mut offset: usize = 1;

            let mut current_action_idx: u8 = 1;

            let number_of_actions_performed = turn_commit
                .actions
                .action_order
                .iter()
                .filter(|value| **value != 0)
                .count() as u8;

            let seeds = &[b"game_signer".as_ref(), &[ctx.accounts.game.signer_bump]];

            let signer = &[&seeds[..]];

            while current_action_idx <= number_of_actions_performed {
                //Everytime one of the action is figured out, we need to break out and restart
                //from the beginning, until all actions are redeemed
                for (index, item) in turn_commit.actions.action_order.iter().enumerate() {
                    if *item == current_action_idx {
                        match index {
                            0 => {
                                //Loot
                                let tile_level = caster.modifiers.tile_level;
                                let potential_looted_tile: Option<&Tile> = get_current_tile(
                                    &turn_data.map,
                                    tile_level,
                                    caster.modifiers.tile_column,
                                );

                                if potential_looted_tile == None {
                                    return Err(ErrorCode::TileNotExists.into());
                                }

                                let looted_tile = potential_looted_tile.unwrap();

                                let range_min_resource: u64 = 1;
                                let mut range_max_resource: u64 = 10 * (tile_level + 1) as u64; // +1 since 0 based

                                let staff_account: Result<Account<Item>, ProgramError> =
                                    Account::try_from(&ctx.accounts.staff);

                                let head_account: Result<Account<Item>, ProgramError> =
                                    Account::try_from(&ctx.accounts.head);

                                let robe_account: Result<Account<Item>, ProgramError> =
                                    Account::try_from(&ctx.accounts.robe);

                                let player_bonuses = get_player_bonuses(
                                    &caster.modifiers,
                                    vec![&staff_account, &head_account, &robe_account],
                                    &game,
                                    &player,
                                    &caster,
                                );

                                match looted_tile.tile_type {
                                    TileType::Earth => {
                                        range_max_resource += player_bonuses.earth_chance as u64;
                                    }
                                    TileType::Fire => {
                                        range_max_resource += player_bonuses.fire_chance as u64;
                                    }
                                    TileType::Water => {
                                        range_max_resource += player_bonuses.water_chance as u64;
                                    }
                                    _ => {}
                                }

                                let mut number_of_resources_given = u64::random_within_range(
                                    slots,
                                    &mut offset,
                                    range_min_resource,
                                    range_max_resource,
                                );

                                if u16::random_within_range(slots, &mut offset, 100, 10000)
                                    < player_bonuses.critical_chance
                                {
                                    number_of_resources_given *= 2;
                                }

                                let resource_token_account: &Account<TokenAccount>;
                                let resource_mint_account: &Account<Mint>;

                                match looted_tile.tile_type {
                                    TileType::Fire => {
                                        resource_token_account =
                                            &ctx.accounts.resource_1_token_account;
                                        resource_mint_account =
                                            &ctx.accounts.resource_1_mint_account;
                                    }
                                    TileType::Water => {
                                        resource_token_account =
                                            &ctx.accounts.resource_2_token_account;
                                        resource_mint_account =
                                            &ctx.accounts.resource_2_mint_account;
                                    }
                                    TileType::Earth => {
                                        resource_token_account =
                                            &ctx.accounts.resource_3_token_account;
                                        resource_mint_account =
                                            &ctx.accounts.resource_3_mint_account;
                                    }
                                    _ => {
                                        return Err(ErrorCode::InvalidTileForLooting.into());
                                    }
                                }

                                token::mint_to(
                                    CpiContext::new(
                                        ctx.accounts.token_program.to_account_info().clone(),
                                        token::MintTo {
                                            mint: resource_mint_account.to_account_info(),
                                            to: resource_token_account.to_account_info(),
                                            authority: ctx.accounts.game_signer.to_account_info(),
                                        },
                                    )
                                    .with_signer(signer),
                                    number_of_resources_given,
                                )?;

                                //Chance of finding a chest is 10% on a resource tile
                                match looted_tile.tile_type {
                                    TileType::Fire | TileType::Water | TileType::Earth => {
                                        //default is 10% so 1000 since we work in % (to not have floating)
                                        let magic_find_chance =
                                            1000 + player_bonuses.magic_find_chance;

                                        if u16::random_within_range(slots, &mut offset, 100, 10000)
                                            < magic_find_chance
                                        {
                                            let item = &mut ctx.accounts.item;
                                            item.game = game.key();
                                            item.owner = ctx.accounts.player.key();
                                            item.equipped_owner = None;
                                            item.item_type = ItemType::Chest {
                                                tier: match tile_level {
                                                    0..=5 => 1,
                                                    6..=10 => 2,
                                                    11..=15 => 3,
                                                    16..=30 => 4,
                                                    _ => 1,
                                                },
                                            };
                                            //Since 0 based, +1
                                            item.level = tile_level + 1;
                                            burn_item_account = false;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            1 => {
                                //Spell
                                if ctx.remaining_accounts.len() == 0 {
                                    return Err(ErrorCode::SpellAccountMissing.into());
                                }

                                match ctx.remaining_accounts.get(0) {
                                    None => {
                                        return Err(ErrorCode::ProvidedSpellBookIsNull.into());
                                    }
                                    _ => {}
                                }

                                let spell_book_account =
                                    &mut ctx.remaining_accounts.get(0).unwrap();

                                if spell_book_account.key() != caster.modifiers.spell_book.unwrap()
                                {
                                    return Err(ErrorCode::SpellKeyMismatch.into());
                                }

                                let mut spell_book_account: &mut Account<Item> =
                                    &mut Account::try_from(&spell_book_account.clone()).unwrap();

                                if let ItemType::SpellBook {
                                    spell,
                                    value,
                                    rarity,
                                    ..
                                } = spell_book_account.item_type
                                {
                                    if is_spell_successful(slots, &mut offset, rarity) {
                                        match spell {
                                            SpellType::Fire => {
                                                token::mint_to(
                                                    CpiContext::new(
                                                        ctx.accounts
                                                            .token_program
                                                            .to_account_info()
                                                            .clone(),
                                                        token::MintTo {
                                                            mint: ctx
                                                                .accounts
                                                                .resource_1_mint_account
                                                                .to_account_info(),
                                                            to: ctx
                                                                .accounts
                                                                .resource_1_token_account
                                                                .to_account_info(),
                                                            authority: ctx
                                                                .accounts
                                                                .game_signer
                                                                .to_account_info(),
                                                        },
                                                    )
                                                    .with_signer(signer),
                                                    value as u64,
                                                )?;
                                            }
                                            SpellType::Water => {
                                                token::mint_to(
                                                    CpiContext::new(
                                                        ctx.accounts
                                                            .token_program
                                                            .to_account_info()
                                                            .clone(),
                                                        token::MintTo {
                                                            mint: ctx
                                                                .accounts
                                                                .resource_2_mint_account
                                                                .to_account_info(),
                                                            to: ctx
                                                                .accounts
                                                                .resource_2_token_account
                                                                .to_account_info(),
                                                            authority: ctx
                                                                .accounts
                                                                .game_signer
                                                                .to_account_info(),
                                                        },
                                                    )
                                                    .with_signer(signer),
                                                    value as u64,
                                                )?;
                                            }
                                            SpellType::Earth => {
                                                token::mint_to(
                                                    CpiContext::new(
                                                        ctx.accounts
                                                            .token_program
                                                            .to_account_info()
                                                            .clone(),
                                                        token::MintTo {
                                                            mint: ctx
                                                                .accounts
                                                                .resource_3_mint_account
                                                                .to_account_info(),
                                                            to: ctx
                                                                .accounts
                                                                .resource_3_token_account
                                                                .to_account_info(),
                                                            authority: ctx
                                                                .accounts
                                                                .game_signer
                                                                .to_account_info(),
                                                        },
                                                    )
                                                    .with_signer(signer),
                                                    value as u64,
                                                )?;
                                            }
                                            SpellType::Experience => {
                                                give_exp_to_caster_spell(caster, value as u64);
                                            }
                                            SpellType::Item => {
                                                let item = &mut ctx.accounts.item;
                                                generate_new_equipment(
                                                    item,
                                                    &ctx.accounts.game,
                                                    &ctx.accounts.player,
                                                    spell_book_account.level,
                                                    Some(rarity),
                                                    slots,
                                                    &mut offset,
                                                )?;
                                                burn_item_account = false;
                                            }
                                            _ => {}
                                        }
                                    }
                                }

                                zombify_account(
                                    &mut spell_book_account,
                                    ctx.accounts.authority.to_account_info(),
                                    ctx.program_id,
                                )?;
                            }
                            2 => {
                                //Move
                                caster.modifiers.tile_level = turn_commit.actions.mv.unwrap()[0];
                                caster.modifiers.tile_column = turn_commit.actions.mv.unwrap()[1];
                            }
                            3 => {
                                //Crafting
                                let crafting_snapshot = turn_commit.actions.crafting.unwrap();
                                let spell_snapshot = turn_commit.actions.spell.clone();

                                //Item level or rarity has a 10% chance of going up
                                let mut new_item_level = crafting_snapshot.min_level;
                                let mut new_item_rarity = crafting_snapshot.min_rarity;

                                //If you have a spell that increases the level, it defaults to common for rarity
                                if spell_snapshot != None
                                    && spell_snapshot.unwrap().is_extra_level_bonus
                                    && crafting_snapshot.min_level < 30
                                {
                                    new_item_level += 1;
                                    new_item_rarity = ItemRarity::Common;
                                } else if u8::random_within_range(slots, &mut offset, 0, 10) == 5 {
                                    if u8::random_within_range(slots, &mut offset, 1, 2) == 1
                                        && crafting_snapshot.min_level < 30
                                    {
                                        new_item_level += 1;
                                        new_item_rarity = ItemRarity::Common;
                                    } else {
                                        match new_item_rarity {
                                            ItemRarity::Common => {
                                                new_item_rarity = ItemRarity::Rare;
                                            }
                                            ItemRarity::Rare => {
                                                new_item_rarity = ItemRarity::Epic;
                                            }
                                            ItemRarity::Epic => {
                                                if crafting_snapshot.max_rarity
                                                    == ItemRarity::Legendary
                                                {
                                                    new_item_rarity = ItemRarity::Legendary;
                                                }
                                            }
                                            ItemRarity::Legendary => {}
                                        }
                                    }
                                }
                                let item = &mut ctx.accounts.item;
                                generate_new_equipment(
                                    item,
                                    game,
                                    player,
                                    new_item_level,
                                    Some(new_item_rarity),
                                    slots,
                                    &mut offset,
                                )?;
                                burn_item_account = false;
                            }
                            _ => {}
                        }

                        current_action_idx += 1;
                        //Break out to reset the for loop since action was completed successfully
                        break;
                    }
                }
            }

            //Give the experience to the caster based on burned resources
            give_exp_to_caster_resources_burned(
                caster,
                Some(turn_commit.resources_burned[FIRE_INDEX]),
                Some(turn_commit.resources_burned[EARTH_INDEX]),
                Some(turn_commit.resources_burned[WATER_INDEX]),
            );

            //Send LADA tokens based on proportion of resources burned by the user vs total resources
            let fire_resources_burned = turn_data.resource_1_burned;
            let water_resources_burned = turn_data.resource_2_burned;
            let earth_resources_burned = turn_data.resource_3_burned;

            let proportion_burned_by_user: f64 = (turn_commit.resources_burned[FIRE_INDEX]
                / non_zero(fire_resources_burned)
                + turn_commit.resources_burned[EARTH_INDEX] / non_zero(earth_resources_burned)
                + turn_commit.resources_burned[WATER_INDEX] / non_zero(water_resources_burned))
                as f64
                / 3.0;

            //Precision of 9 decimals DECIMALS_PRECISION
            let scaled_proportion_burned_by_user =
                (proportion_burned_by_user * DECIMALS_PRECISION as f64) as u64;

            let amount = Rate::from_scaled_val(scaled_proportion_burned_by_user)
                .try_mul(LADA_DISTRIBUTION_PER_TURN)
                .unwrap()
                .try_mul(DECIMALS_PRECISION)
                .unwrap()
                .try_round_u64()
                .unwrap();

            let cpi_accounts = Transfer {
                from: ctx
                    .accounts
                    .game_lada_token_account
                    .to_account_info()
                    .clone(),
                to: ctx.accounts.lada_token_account.to_account_info().clone(),
                authority: ctx.accounts.game_signer.to_account_info().clone(),
            };

            let transfer_cpi = CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                cpi_accounts,
            );

            let seeds = &[b"game_signer".as_ref(), &[ctx.accounts.game.signer_bump]];
            let signer = &[&seeds[..]];

            token::transfer(transfer_cpi.with_signer(signer), amount)?;

            //Reset caster's turn commit
            caster.turn_commit = None;
        }
    }

    // Burn item if not used
    if burn_item_account {
        let item = &mut ctx.accounts.item;
        zombify_account(
            item,
            ctx.accounts.authority.to_account_info(),
            ctx.program_id,
        )?;
    }
    Ok(())
}

pub fn non_zero(number: u64) -> u64 {
    match number {
        0 => 1,
        _ => number,
    }
}

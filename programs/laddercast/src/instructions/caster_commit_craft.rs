use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{Tile, TileType};
use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::{GAME_CREATOR_AUTHORITY_PUBKEY, get_current_tile, ItemRarity, ItemType, zombify_account};
use crate::utils::CraftingSnapshot;
use crate::utils::TurnCommit;

#[derive(Accounts)]
pub struct Craft<'info> {
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

    #[account(mut, constraint = item_1.game == game.key(), constraint = item_1.owner == player.key(), constraint = item_1.equipped_owner == None, close = authority)]
    pub item_1: Box<Account<'info, Item>>,
    #[account(mut, constraint = item_2.game == game.key(), constraint = item_2.owner == player.key(), constraint = item_2.equipped_owner == None, close = authority)]
    pub item_2: Box<Account<'info, Item>>,
    #[account(mut, constraint = item_3.game == game.key(), constraint = item_3.owner == player.key(), constraint = item_3.equipped_owner == None, close = authority)]
    pub item_3: Box<Account<'info, Item>>,

    #[account(mut, seeds = [b"turn_data", game.to_account_info().key().as_ref(), game.turn_info.turn.to_string().as_ref()], bump = game_turn_data.bump)]
    pub game_turn_data: Box<Account<'info, TurnData>>,
}

pub fn caster_commit_craft(ctx: Context<Craft>) -> ProgramResult {
    let map = ctx.accounts.game.map.clone();
    let game_turn = ctx.accounts.game.turn_info.turn;

    let caster = &mut ctx.accounts.caster;
    let turn_data = &mut ctx.accounts.game_turn_data;

    let mut caster_turn_commit: TurnCommit = match caster.turn_commit.clone() {
        Some(turn_commit) => turn_commit,
        None => TurnCommit {
            turn: game_turn,
            ..Default::default()
        },
    };

    if caster_turn_commit.turn != game_turn {
        return Err(ErrorCode::PendingTurn.into());
    }

    if caster_turn_commit.actions.crafting != None {
        return Err(ErrorCode::ActionAlreadyDone.into());
    }

    let mut crafting_snapshot: CraftingSnapshot = CraftingSnapshot {
        min_level: u8::MAX, //Starts at the max because we want the lowest
        min_rarity: ItemRarity::Legendary, //Starts legendary because we take lowest of all 3
        max_rarity: ItemRarity::Epic,
    };

    //Need to do this because if user moved to a crafting tile during that turn, then crafting is authorized
    let (dest_level, dest_column): (u8, u8) = {
        if caster_turn_commit.actions.mv != None {
            (
                caster_turn_commit.actions.mv.unwrap()[0],
                caster_turn_commit.actions.mv.unwrap()[1],
            )
        } else {
            (caster.modifiers.tile_level, caster.modifiers.tile_column)
        }
    };

    let potential_current_tile: Option<&Tile> = get_current_tile(&map, dest_level, dest_column);

    if potential_current_tile == None {
        return Err(ErrorCode::TileNotExists.into());
    }

    let current_tile = potential_current_tile.unwrap();

    if current_tile.tile_type != TileType::Crafting && current_tile.tile_type != TileType::Legendary {
        return Err(ErrorCode::NotCraftingTile.into());
    }

    //Take the resources cost for crafting (since 0 based, gotta add 1)
    let per_resource_burn = caster.modifiers.tile_level.checked_add(1).unwrap().checked_mul(5).unwrap() as u64;

    let resource_1_token_account = &ctx.accounts.resource_1_token_account;
    let resource_2_token_account = &ctx.accounts.resource_2_token_account;
    let resource_3_token_account = &ctx.accounts.resource_3_token_account;

    if resource_1_token_account
        .amount
        .checked_sub(per_resource_burn)
        == None
        || resource_2_token_account
        .amount
        .checked_sub(per_resource_burn)
        == None
        || resource_3_token_account
        .amount
        .checked_sub(per_resource_burn)
        == None
    {
        return Err(ErrorCode::PlayerIsPoor.into());
    }

    turn_data.resource_1_burned += per_resource_burn;
    turn_data.resource_2_burned += per_resource_burn;
    turn_data.resource_3_burned += per_resource_burn;

    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::Burn {
                mint: ctx
                    .accounts
                    .resource_1_mint_account
                    .to_account_info()
                    .clone(),
                to: resource_1_token_account.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        per_resource_burn,
    )?;
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::Burn {
                mint: ctx
                    .accounts
                    .resource_2_mint_account
                    .to_account_info()
                    .clone(),
                to: resource_2_token_account.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        per_resource_burn,
    )?;
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::Burn {
                mint: ctx
                    .accounts
                    .resource_3_mint_account
                    .to_account_info()
                    .clone(),
                to: resource_3_token_account.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        per_resource_burn,
    )?;

    //If not a legendary crafting tile, can only go up to epic, if it is can go up to legendary
    if current_tile.tile_type == TileType::Legendary {
        crafting_snapshot.max_rarity = ItemRarity::Legendary;
    } else {
        crafting_snapshot.max_rarity = ItemRarity::Epic;
    }

    for removed_item in [
        ctx.accounts.item_1.clone(),
        ctx.accounts.item_2.clone(),
        ctx.accounts.item_3.clone(),
    ]
        .iter()
    {
        if removed_item.level < crafting_snapshot.min_level {
            crafting_snapshot.min_level = removed_item.level;
        }

        match removed_item.item_type {
            ItemType::Equipment { rarity, .. } => match rarity {
                ItemRarity::Common => {
                    crafting_snapshot.min_rarity = rarity;
                }
                ItemRarity::Rare => {
                    if crafting_snapshot.min_rarity != ItemRarity::Common {
                        crafting_snapshot.min_rarity = rarity;
                    }
                }
                ItemRarity::Epic => {
                    if crafting_snapshot.min_rarity != ItemRarity::Common
                        && crafting_snapshot.min_rarity != ItemRarity::Rare
                    {
                        crafting_snapshot.min_rarity = rarity;
                    }
                }
                ItemRarity::Legendary => {}
            },
            _ => {
                return Err(ErrorCode::InvalidItemType.into());
            }
        }
    }

    caster_turn_commit.actions.crafting = Some(crafting_snapshot);

    caster_turn_commit.resources_burned = [
        caster_turn_commit.resources_burned[0]
            .checked_add(per_resource_burn)
            .unwrap(),
        caster_turn_commit.resources_burned[1]
            .checked_add(per_resource_burn)
            .unwrap(),
        caster_turn_commit.resources_burned[2]
            .checked_add(per_resource_burn)
            .unwrap(),
    ];

    caster_turn_commit.actions.add_new_action_order(3);

    ctx.accounts.caster.turn_commit = Some(caster_turn_commit);

    //Zombifies the 3 item accounts
    zombify_account(
        &mut ctx.accounts.item_1,
        ctx.accounts.authority.to_account_info(),
        ctx.program_id,
    )?;
    zombify_account(
        &mut ctx.accounts.item_2,
        ctx.accounts.authority.to_account_info(),
        ctx.program_id,
    )?;
    zombify_account(
        &mut ctx.accounts.item_3,
        ctx.accounts.authority.to_account_info(),
        ctx.program_id,
    )?;

    Ok(())
}

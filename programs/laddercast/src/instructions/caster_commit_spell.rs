use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::{EARTH_INDEX, FIRE_INDEX, GAME_CREATOR_AUTHORITY_PUBKEY, is_spell_successful, ItemFeature, ItemType, SpellSnapshot, SpellType, WATER_INDEX};
use crate::utils::TurnCommit;

#[derive(Accounts)]
pub struct Spell<'info> {
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

    #[account(mut, constraint = spellbook.game == game.key(), constraint = spellbook.owner == player.key(), constraint = spellbook.equipped_owner.unwrap() == caster.key())]
    pub spellbook: Box<Account<'info, Item>>,

    #[account(mut, seeds = [b"turn_data", game.to_account_info().key().as_ref(), game.turn_info.turn.to_string().as_ref()], bump = game_turn_data.bump)]
    pub game_turn_data: Box<Account<'info, TurnData>>,
}

pub fn caster_commit_spell(ctx: Context<Spell>) -> ProgramResult {
    let game_turn = ctx.accounts.game.turn_info.turn;

    let caster = &mut ctx.accounts.caster;
    let turn_data = &mut ctx.accounts.game_turn_data;

    let mut caster_turn_commit: TurnCommit = match &caster.turn_commit {
        Some(turn_commit) => *turn_commit,
        None => TurnCommit {
            turn: game_turn,
            ..Default::default()
        },
    };

    if caster_turn_commit.turn != game_turn {
        return Err(ErrorCode::PendingTurn.into());
    }

    if caster_turn_commit.actions.spell != None {
        return Err(ErrorCode::ActionAlreadyDone.into());
    }

    if let ItemType::SpellBook {
        cost_feature, cost, spell, rarity, ..
    } = ctx.accounts.spellbook.item_type
    {
        let resource_burned = cost as u64;

        match cost_feature {
            ItemFeature::Fire => {
                if ctx.accounts.resource_1_token_account.amount.checked_sub(resource_burned) == None {
                    return Err(ErrorCode::PlayerIsPoor.into());
                }

                turn_data.resource_1_burned += resource_burned;

                caster_turn_commit.resources_burned[FIRE_INDEX] = caster_turn_commit.resources_burned[FIRE_INDEX]
                    .checked_add(resource_burned)
                    .unwrap();

                token::burn(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info().clone(),
                        token::Burn {
                            mint: ctx.accounts.resource_1_mint_account.to_account_info().clone(),
                            to: ctx.accounts.resource_1_token_account.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                            authority: ctx.accounts.authority.to_account_info(),
                        },
                    ),
                    resource_burned,
                )?;
            }
            ItemFeature::Water => {
                if ctx.accounts.resource_2_token_account.amount.checked_sub(resource_burned) == None {
                    return Err(ErrorCode::PlayerIsPoor.into());
                }

                turn_data.resource_2_burned += resource_burned;

                caster_turn_commit.resources_burned[WATER_INDEX] = caster_turn_commit.resources_burned[WATER_INDEX]
                    .checked_add(resource_burned)
                    .unwrap();

                token::burn(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info().clone(),
                        token::Burn {
                            mint: ctx.accounts.resource_2_mint_account.to_account_info().clone(),
                            to: ctx.accounts.resource_2_token_account.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                            authority: ctx.accounts.authority.to_account_info(),
                        },
                    ),
                    resource_burned,
                )?;
            }
            ItemFeature::Earth => {
                if ctx.accounts.resource_3_token_account.amount.checked_sub(resource_burned) == None {
                    return Err(ErrorCode::PlayerIsPoor.into());
                }

                turn_data.resource_3_burned += resource_burned;

                caster_turn_commit.resources_burned[EARTH_INDEX] = caster_turn_commit.resources_burned[EARTH_INDEX]
                    .checked_add(resource_burned)
                    .unwrap();

                token::burn(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info().clone(),
                        token::Burn {
                            mint: ctx.accounts.resource_3_mint_account.to_account_info().clone(),
                            to: ctx.accounts.resource_3_token_account.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                            authority: ctx.accounts.authority.to_account_info(),
                        },
                    ),
                    resource_burned,
                )?;
            }
            _ => {
                return Err(ErrorCode::InvalidSpellCost.into());
            }
        }

        match spell {
            SpellType::Craft => {
                let slots_ref = ctx.accounts.slots.data.borrow();
                let slots = &**slots_ref;

                let mut offset: usize = 1;

                if is_spell_successful(slots, &mut offset, rarity) {
                    caster_turn_commit.actions.spell = Some(SpellSnapshot {
                        is_extra_level_bonus: true
                    });
                } else {
                    caster_turn_commit.actions.spell = Some(SpellSnapshot {
                        is_extra_level_bonus: false
                    });
                }
            }
            _ => {
                caster_turn_commit.actions.spell = Some(SpellSnapshot {
                    is_extra_level_bonus: false
                });
            }
        }
    }

    caster_turn_commit.actions.add_new_action_order(1);

    ctx.accounts.caster.turn_commit = Some(caster_turn_commit);

    Ok(())
}

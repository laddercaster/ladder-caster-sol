use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{Tile, TileType};
use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::{EARTH_INDEX, FIRE_INDEX, GAME_CREATOR_AUTHORITY_PUBKEY, get_current_tile, WATER_INDEX};
use crate::utils::TurnCommit;

#[derive(Accounts)]
pub struct Move<'info> {
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

    #[account(mut, seeds = [b"turn_data", game.to_account_info().key().as_ref(), game.turn_info.turn.to_string().as_ref()], bump = game_turn_data.bump)]
    pub game_turn_data: Box<Account<'info, TurnData>>,
}

pub fn caster_commit_move(ctx: Context<Move>, lvl: u8, clm: u8) -> ProgramResult {
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

    if caster_turn_commit.actions.mv != None {
        return Err(ErrorCode::ActionAlreadyDone.into());
    }

    let (dest_level, dest_column): (u8, u8) = (lvl, clm);

    //Can only go up the ladder, and can only go if your caster is the right level
    if dest_level < caster.modifiers.tile_level || dest_level > caster.level {
        return Err(ErrorCode::InvalidMove.into());
    }

    let d_col = dest_column as i8;
    let f_col = caster.modifiers.tile_column as i8;
    let distance = (d_col - f_col).abs();

    //Can only move to the column next to you or move up by 1
    if distance > 1 || (dest_level > caster.modifiers.tile_level && distance != 0) {
        return Err(ErrorCode::InvalidMove.into());
    }

    let potential_dest_tile: Option<&Tile> = get_current_tile(&map, dest_level, dest_column);

    if potential_dest_tile == None {
        return Err(ErrorCode::TileNotExists.into());
    }

    let dest_tile = potential_dest_tile.unwrap();

    //MOVE costs 10*resource of tile you're moving too
    let resource_burned = (dest_level + 1).checked_mul(10).unwrap() as u64;

    match dest_tile.tile_type {
        TileType::Fire => {
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
                        mint: ctx
                            .accounts
                            .resource_1_mint_account
                            .to_account_info()
                            .clone(),
                        to: ctx
                            .accounts
                            .resource_1_token_account
                            .to_account_info()
                            .clone(), // STUPID NAME: its the token account you are burning the tokens from
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                ),
                resource_burned,
            )?;
        }
        TileType::Water => {
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
                        mint: ctx
                            .accounts
                            .resource_2_mint_account
                            .to_account_info()
                            .clone(),
                        to: ctx
                            .accounts
                            .resource_2_token_account
                            .to_account_info()
                            .clone(), // STUPID NAME: its the token account you are burning the tokens from
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                ),
                resource_burned,
            )?;
        }
        TileType::Earth => {
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
                        mint: ctx
                            .accounts
                            .resource_3_mint_account
                            .to_account_info()
                            .clone(),
                        to: ctx
                            .accounts
                            .resource_3_token_account
                            .to_account_info()
                            .clone(), // STUPID NAME: its the token account you are burning the tokens from
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                ),
                resource_burned,
            )?;
        }
        _ => {}
    }

    caster_turn_commit.actions.mv = Some([dest_level, dest_column]);

    caster_turn_commit.actions.add_new_action_order(2);

    ctx.accounts.caster.turn_commit = Some(caster_turn_commit);

    Ok(())
}

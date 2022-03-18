use anchor_lang::prelude::*;

use crate::{Tile, TileType};
use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::{GAME_CREATOR_AUTHORITY_PUBKEY, get_current_tile, TurnCommit};

#[derive(Accounts)]
pub struct Loot<'info> {
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub game: Box<Account<'info, Game>>,
    #[account(mut, has_one = authority, has_one = game)]
    pub player: Account<'info, Player>,
    #[account(mut, constraint = caster.owner == player.key())]
    pub caster: Account<'info, Caster>,

    #[account(mut, seeds = [b"turn_data", game.to_account_info().key().as_ref(), game.turn_info.turn.to_string().as_ref()], bump = game_turn_data.bump)]
    pub game_turn_data: Account<'info, TurnData>,
}

pub fn caster_commit_loot(ctx: Context<Loot>) -> ProgramResult {
    let game_turn = ctx.accounts.game.turn_info.turn;

    let caster = &mut ctx.accounts.caster;

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

    //Need to do this because if user moved to a looting tile during that turn, then looting is authorized
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

    let potential_looted_tile: Option<&Tile> = get_current_tile(
        &ctx.accounts.game.map,
        dest_level,
        dest_column,
    );

    if potential_looted_tile == None {
        return Err(ErrorCode::TileNotExists.into());
    }

    let looted_tile = potential_looted_tile.unwrap();

    match looted_tile.tile_type {
        TileType::Earth | TileType::Fire | TileType::Water => {}
        _ => {
            return Err(ErrorCode::InvalidTileForLooting.into());
        }
    }

    //Loot Action
    if caster_turn_commit.actions.loot {
        return Err(ErrorCode::ActionAlreadyDone.into());
    }

    caster_turn_commit.actions.loot = true;

    caster_turn_commit.actions.add_new_action_order(0);

    ctx.accounts.caster.turn_commit = Some(caster_turn_commit);

    Ok(())
}

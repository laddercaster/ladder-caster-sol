use core::mem::size_of;

use anchor_lang::{prelude::*, solana_program::sysvar};

use crate::account::{Game, TurnData};
use crate::error::ErrorCode;
use crate::event::NewTurn;
use crate::utils::{cycle_tile, get_highest_level_and_column, GAME_CREATOR_AUTHORITY_PUBKEY};
use crate::Tile;

#[derive(Accounts)]
pub struct Crank<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub game_account: Box<Account<'info, Game>>,

    #[account(address = sysvar::slot_hashes::id())]
    pub slots: UncheckedAccount<'info>,

    #[account(mut, seeds = [b"turn_data", game_account.to_account_info().key().as_ref(), game_account.turn_info.turn.to_string().as_ref()], bump = current_game_turn_data.bump)]
    pub current_game_turn_data: Account<'info, TurnData>,

    #[account(init,
    seeds = [b"turn_data", game_account.to_account_info().key().as_ref(), game_account.turn_info.turn.checked_add(1).unwrap().to_string().as_ref()],
    bump,
    payer = authority,
    space =  TurnData::SIZE
    )]
    pub game_turn_data: Account<'info, TurnData>,
}

pub fn crank(ctx: Context<Crank>) -> ProgramResult {
    let game = &mut ctx.accounts.game_account;

    // if game.authority.to_string() != GAME_CREATOR_AUTHORITY_PUBKEY {
    //     return Err(ErrorCode::InvalidGame.into());
    // }

    let clock = Clock::get().unwrap();

    //Make sure enough time has passed to move on to next turn
    if clock.unix_timestamp < (game.turn_info.last_crank_seconds + game.turn_info.turn_delay as i64)
    {
        return Err(ErrorCode::PrematureCrankPull.into());
    }

    let turn_data = &mut ctx.accounts.game_turn_data;

    turn_data.bump = *ctx.bumps.get("game_turn_data").unwrap();

    let current_game_turn_data = &mut ctx.accounts.current_game_turn_data;

    current_game_turn_data.map = game.map.clone();

    // iterate through array and decrement life turns
    // if turn = 0, spawn a new tile in it's place
    // if a new tile needs to be spawned, spawn a new tile
    let slots_ref = ctx.accounts.slots.data.borrow();
    let slots = &**slots_ref;
    let mut offset: usize = 1;

    for i in 0..game.map.len() {
        for j in 0..game.map[i].len() {
            let tile = &mut game.map[i][j];

            match tile {
                None => {}
                Some(tile) => {
                    if tile.life - 1 == 0 {
                        *tile = cycle_tile(Some(*tile), i as u8, slots, &mut offset);
                    } else {
                        tile.life -= 1;
                    }
                }
            }
        }
    }

    //Spawn a new tile if enough turns have passed, don't push new tiles if map is full
    let (highest_lvl, highest_col) = get_highest_level_and_column(&game.map);

    if (game.turn_info.turn + 1)
        >= (game.turn_info.last_tile_spawn + game.turn_info.tile_spawn_delay)
        && !(highest_lvl == 30 && highest_col == 3)
    {
        let new_tile: Tile;

        if highest_col < 3 {
            new_tile = cycle_tile(None, highest_lvl, slots, &mut offset);
            game.map[highest_lvl as usize][(highest_col + 1) as usize] = Some(new_tile);
        } else {
            new_tile = cycle_tile(None, highest_lvl + 1, slots, &mut offset);
            game.map[(highest_lvl + 1) as usize][highest_col as usize] = Some(new_tile);
        }

        game.turn_info.last_tile_spawn = game.turn_info.turn + 1;
    }

    game.turn_info.turn += 1;
    game.turn_info.last_crank_seconds = clock.unix_timestamp;
    game.last_turn_added = game.turn_info.turn;

    emit!(NewTurn {
        turn: game.turn_info.turn,
        tile_map: game.map.clone()
    });

    Ok(())
}

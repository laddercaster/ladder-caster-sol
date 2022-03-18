use core::mem::size_of;

use anchor_lang::prelude::*;

use crate::account::{Game, Player};
use crate::error::ErrorCode;
use crate::utils::GAME_CREATOR_AUTHORITY_PUBKEY;

#[derive(Accounts)]
pub struct InitPlayer<'info> {
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub game: Box<Account<'info, Game>>,

    #[account(init,
    seeds = [game.key().as_ref(), authority.key().as_ref()],
    bump,
    payer = authority,
    space = Player::SIZE
    )]
    pub player_account: Account<'info, Player>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub struct PlayerBonuses {
    //Is 0-100 which represents a %
    pub critical_chance: u16,
    //Is 0-100 which represents a %
    pub magic_find_chance: u16,
    //Is a value, not a %
    pub fire_chance: u16,
    //Is a value, not a %
    pub water_chance: u16,
    //Is a value, not a %
    pub earth_chance: u16,
}

pub fn init_player(ctx: Context<InitPlayer>) -> ProgramResult {
    let player_account = &mut ctx.accounts.player_account;

    player_account.authority = ctx.accounts.authority.key();
    player_account.game = ctx.accounts.game.key();
    player_account.bump = *ctx.bumps.get("player_account").unwrap();

    Ok(())
}

use core::mem::size_of;

use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::token::{Mint, Token, TokenAccount};
use strum::{EnumCount, EnumIter};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::constants::*;
use crate::utils::cycle_tile;

#[derive(Accounts)]
#[instruction(turn_info: GameTurnInfo)]
pub struct InitGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,

    #[account(init,
    payer = authority,
    space = Game::SIZE
    )]
    pub game_account: Box<Account<'info, Game>>,

    #[account(mut, seeds = [b"game_signer"], bump)]
    pub game_signer: UncheckedAccount<'info>,

    #[account(init,
    seeds = [b"turn_data", game_account.key().as_ref(), turn_info.turn.to_string().as_ref()],
    bump,
    payer = authority,
    space = TurnData::SIZE
    )]
    pub game_turn_data: Box<Account<'info, TurnData>>,

    #[account(address = sysvar::slot_hashes::id())]
    pub slots: UncheckedAccount<'info>,

    #[account(init,
    mint::decimals = 0,
    mint::authority = game_signer,
    payer = authority)]
    pub resource_1_mint: Account<'info, Mint>,
    #[account(init,
    mint::decimals = 0,
    mint::authority = game_signer,
    payer = authority)]
    pub resource_2_mint: Account<'info, Mint>,
    #[account(init,
    mint::decimals = 0,
    mint::authority = game_signer,
    payer = authority)]
    pub resource_3_mint: Account<'info, Mint>,

    pub lada_mint: Account<'info, Mint>,
    pub lada_token_account: Account<'info, TokenAccount>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub struct GameTurnInfo {
    /// current turn
    pub turn: u32,
    /// how many seconds till next turn
    pub turn_delay: u16,
    /// last timestamp the crank was pulled
    pub last_crank_seconds: i64,
    /// last turn a tile was spawned
    pub last_tile_spawn: u32,
    /// how many turns til next tile should spawn
    pub tile_spawn_delay: u32,
}

impl GameTurnInfo {
    pub const SIZE: usize = 8 + 4 + 2 + 8 + 4 + 4;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub struct Tile {
    pub tile_type: TileType,
    pub life: u8,
    /// First time crafting tile is spawned, it will be legendary (only once, then goes to normal crafting)
    pub is_first_time_spawning: bool,
}

impl Tile {
    pub const SIZE: usize = 8 + size_of::<TileType>() + 1 + 1;
}

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy, EnumCount, EnumIter,
)]
pub enum TileType {
    Earth,
    Water,
    Fire,
    Crafting,
    Legendary,
}

pub fn init_game(
    ctx: Context<InitGame>,
    turn_info: GameTurnInfo,
    merkle_root_nft: [u8; 32],
) -> ProgramResult {
    let game = &mut ctx.accounts.game_account;
    let turn_data = &mut ctx.accounts.game_turn_data;

    // if ctx.accounts.authority.key().to_string() != GAME_CREATOR_AUTHORITY_PUBKEY {
    //     return Err(ErrorCode::NotSuperAdmin.into());
    // }
    //
    // if ctx.accounts.lada_mint.key().to_string() != LADA_MINT_PUBKEY {
    //     return Err(ErrorCode::NotSuperAdmin.into());
    // }
    //
    // if ctx.accounts.lada_mint.key().to_string() != LADA_MINT_PUBKEY {
    //     return Err(ErrorCode::NotSuperAdmin.into());
    // }

    game.authority = ctx.accounts.authority.key();
    game.turn_info = turn_info;
    game.last_turn_added = 1;
    game.version = 1;
    game.merkle_root_nft = merkle_root_nft;

    game.signer_bump = *ctx.bumps.get("game_signer").unwrap();
    turn_data.bump = *ctx.bumps.get("game_turn_data").unwrap();

    let slots_ref = ctx.accounts.slots.data.borrow();
    let slots = &**slots_ref;

    let mut offset: usize = 1;

    let t1 = cycle_tile(None, 1, slots, &mut offset);
    let t2 = cycle_tile(None, 1, slots, &mut offset);
    let t3 = cycle_tile(None, 1, slots, &mut offset);

    game.map[0][0] = Some(t1);
    game.map[0][1] = Some(t2);
    game.map[0][2] = Some(t3);

    game.resource_1_mint_account = ctx.accounts.resource_1_mint.to_account_info().key();
    game.resource_2_mint_account = ctx.accounts.resource_2_mint.to_account_info().key();
    game.resource_3_mint_account = ctx.accounts.resource_3_mint.to_account_info().key();

    game.lada_mint_account = ctx.accounts.lada_mint.to_account_info().key();
    game.lada_token_account = ctx.accounts.lada_token_account.to_account_info().key();

    turn_data.map = game.map.clone();

    Ok(())
}

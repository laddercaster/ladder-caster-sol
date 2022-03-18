use anchor_lang::prelude::*;

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::GAME_CREATOR_AUTHORITY_PUBKEY;

#[derive(Accounts)]
pub struct UpdateMerkleRoot<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub game_account: Box<Account<'info, Game>>,
}

pub fn update_merkle_root(
    ctx: Context<UpdateMerkleRoot>,
    merkle_root_nft: [u8; 32],
) -> ProgramResult {
    let game = &mut ctx.accounts.game_account;

    // if ctx.accounts.authority.key().to_string() != GAME_CREATOR_AUTHORITY_PUBKEY {
    //     return Err(ErrorCode::NotSuperAdmin.into());
    // }
    //
    // if ctx.accounts.game_account.authority.to_string() != GAME_CREATOR_AUTHORITY_PUBKEY {
    //     return Err(ErrorCode::InvalidGame.into());
    // }

    game.merkle_root_nft = merkle_root_nft;
    Ok(())
}

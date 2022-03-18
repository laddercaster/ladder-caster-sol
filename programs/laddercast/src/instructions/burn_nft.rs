use core::mem::size_of;

use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::account::{Caster, Game, Item, MetadataNFTCaster, MetadataNFTItem, Player};
use crate::error::ErrorCode;
use crate::utils::{Modifiers, GAME_CREATOR_AUTHORITY_PUBKEY};

#[derive(Accounts)]
pub struct RedeemItem<'info> {
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub game: Box<Account<'info, Game>>,
    #[account(mut, has_one = authority, has_one = game)]
    pub player: Account<'info, Player>,
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    #[account(mut)]
    pub nft_token: Account<'info, TokenAccount>,
    #[account(mut,
    seeds = [b"metadata".as_ref(), nft_mint.key().as_ref()],
    bump = nft_metadata.self_bump,
    close = authority)]
    pub nft_metadata: Account<'info, MetadataNFTItem>,
    #[account(init, payer = authority, space = Item::SIZE)]
    pub item: Box<Account<'info, Item>>,
}

pub fn redeem_item(ctx: Context<RedeemItem>) -> ProgramResult {
    if ctx.accounts.nft_token.amount != 1 {
        return Err(ErrorCode::InvalidTokenAmount.into());
    }

    let item_metadata = ctx.accounts.nft_metadata.item;

    let item = &mut ctx.accounts.item;
    item.item_type = item_metadata.item_type;
    item.level = item_metadata.level;
    item.game = ctx.accounts.game.key();
    item.owner = ctx.accounts.player.key();
    item.equipped_owner = None;

    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::Burn {
                mint: ctx.accounts.nft_mint.to_account_info().clone(),
                to: ctx.accounts.nft_token.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        1,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct RedeemCaster<'info> {
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub game: Box<Account<'info, Game>>,
    #[account(mut, has_one = authority, has_one = game)]
    pub player: Account<'info, Player>,
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    #[account(mut)]
    pub nft_token: Account<'info, TokenAccount>,
    #[account(mut,
    seeds = [b"metadata".as_ref(), nft_mint.key().as_ref()],
    bump = nft_metadata.self_bump,
    close = authority)]
    pub nft_metadata: Account<'info, MetadataNFTCaster>,
    #[account(init, payer = authority, space=Caster::SIZE)]
    pub caster: Box<Account<'info, Caster>>,
}

pub fn redeem_caster(ctx: Context<RedeemCaster>) -> ProgramResult {
    if ctx.accounts.nft_token.amount != 1 {
        return Err(ErrorCode::InvalidTokenAmount.into());
    }

    let caster_metadata = &ctx.accounts.nft_metadata.caster;

    let caster = &mut ctx.accounts.caster;
    caster.experience = caster_metadata.experience;
    caster.level = caster_metadata.level;
    caster.version = caster_metadata.version;
    caster.turn_commit = None;
    caster.modifiers = Modifiers {
        tile_level: caster_metadata.modifiers.tile_level,
        tile_column: caster_metadata.modifiers.tile_column,
        head: None,
        robe: None,
        staff: None,
        spell_book: None,
    };
    caster.owner = ctx.accounts.player.key();

    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::Burn {
                mint: ctx.accounts.nft_mint.to_account_info().clone(),
                to: ctx.accounts.nft_token.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        1,
    )?;

    Ok(())
}

use core::mem::size_of;

use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::ItemType;
use crate::{Tile, TileType};

#[derive(Accounts)]
pub struct GiveResources<'info> {
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"game_signer"], bump)]
    pub game_signer: UncheckedAccount<'info>,

    pub game: Box<Account<'info, Game>>,

    #[account(mut, has_one = authority, has_one = game)]
    pub player: Box<Account<'info, Player>>,
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
}

#[derive(Accounts)]
pub struct GiveItems<'info> {
    pub system_program: Program<'info, System>,
    pub game: Box<Account<'info, Game>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority, has_one = game)]
    pub player: Account<'info, Player>,
    #[account(address = sysvar::slot_hashes::id())]
    pub slots: UncheckedAccount<'info>,
    #[account(init, payer = authority, space = Item::SIZE)]
    pub item: Box<Account<'info, Item>>,
}

#[derive(Accounts)]
pub struct GiveLada<'info> {
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"game_signer"], bump)]
    pub game_signer: UncheckedAccount<'info>,

    pub game: Box<Account<'info, Game>>,

    #[account(constraint = lada_mint_account.to_account_info().key() == game.lada_mint_account)]
    pub lada_mint_account: Account<'info, Mint>,

    #[account(mut)]
    pub game_lada_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub lada_token_account: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct ChangeTile<'info> {
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub game: Box<Account<'info, Game>>,
}

#[allow(unused_variables)]
pub fn give_resources(ctx: Context<GiveResources>, amount: u64) -> ProgramResult {
    // #[cfg(feature = "debug")]
    // {
    let seeds = &[b"game_signer".as_ref(), &[ctx.accounts.game.signer_bump]];
    let signer = &[&seeds[..]];
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::MintTo {
                mint: ctx.accounts.resource_1_mint_account.to_account_info(),
                to: ctx.accounts.resource_1_token_account.to_account_info(),
                authority: ctx.accounts.game_signer.to_account_info(),
            },
        )
        .with_signer(signer),
        amount,
    )?;
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::MintTo {
                mint: ctx.accounts.resource_2_mint_account.to_account_info(),
                to: ctx.accounts.resource_2_token_account.to_account_info(),
                authority: ctx.accounts.game_signer.to_account_info(),
            },
        )
        .with_signer(signer),
        amount,
    )?;
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::MintTo {
                mint: ctx.accounts.resource_3_mint_account.to_account_info(),
                to: ctx.accounts.resource_3_token_account.to_account_info(),
                authority: ctx.accounts.game_signer.to_account_info(),
            },
        )
        .with_signer(signer),
        amount,
    )?;

    Ok(())
    // }
    // #[cfg(not(feature = "debug"))]
    // {
    //     Err(ErrorCode::InvalidFunction.into())
    // }
}

#[allow(unused_variables)]
pub fn give_lada(ctx: Context<GiveLada>, amount: u64) -> ProgramResult {
    let seeds = &[b"game_signer".as_ref(), &[ctx.accounts.game.signer_bump]];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx
            .accounts
            .game_lada_token_account
            .to_account_info()
            .clone(),
        to: ctx.accounts.lada_token_account.to_account_info().clone(),
        authority: ctx.accounts.game_signer.to_account_info(),
    };

    let transfer_cpi = CpiContext::new(
        ctx.accounts.token_program.to_account_info().clone(),
        cpi_accounts,
    )
    .with_signer(signer);

    token::transfer(transfer_cpi, amount)?;
    Ok(())
}

#[allow(unused_variables)]
pub fn give_item(ctx: Context<GiveItems>, item_type: ItemType, level: u8) -> ProgramResult {
    // #[cfg(feature = "debug")]
    // {
    let player_acc = &mut ctx.accounts.player;
    let game = &ctx.accounts.game;

    match item_type {
        ItemType::Chest { .. } => {
            // Item {
            let item = &mut ctx.accounts.item;
            item.game = ctx.accounts.game.key();
            item.owner = player_acc.key();
            item.equipped_owner = None;
            item.level = level;
            item.item_type = ItemType::Chest {
                tier: match level {
                    1..=5 => 1,
                    6..=10 => 2,
                    11..=15 => 3,
                    16..=30 => 4,
                    _ => 1,
                },
            };
        }
        ItemType::SpellBook { .. } => {
            let item = &mut ctx.accounts.item;
            item.game = ctx.accounts.game.key();
            item.owner = player_acc.key();
            item.equipped_owner = None;
            item.level = level;
            item.item_type = item_type.clone();
        }
        ItemType::Equipment { .. } => {
            let item = &mut ctx.accounts.item;
            item.game = ctx.accounts.game.key();
            item.owner = player_acc.key();
            item.equipped_owner = None;
            item.level = level;
            item.item_type = item_type.clone();
        }
        _ => {}
    }
    Ok(())
    // }
    // #[cfg(not(feature = "debug"))]
    // {
    //     Err(ErrorCode::InvalidFunction.into())
    // }
}

#[allow(unused_variables)]
pub fn change_tile(
    ctx: Context<ChangeTile>,
    tile_type: TileType,
    lvl: u8,
    col: u8,
) -> ProgramResult {
    ctx.accounts.game.map[lvl as usize][col as usize] = Some(Tile {
        tile_type,
        life: 1,
        is_first_time_spawning: false,
    });
    Ok(())
    // }
    // #[cfg(not(feature = "debug"))]
    // {
    //     Err(ErrorCode::InvalidFunction.into())
    // }
}

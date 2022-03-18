use core::mem::size_of;

use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::account::{Caster, Game, Player};
use crate::error::ErrorCode;
use crate::utils::{
    Modifiers, RandomValue, COST_IN_LADA_FOR_CASTER, DECIMALS_PRECISION,
    GAME_CREATOR_AUTHORITY_PUBKEY,
};

#[derive(Accounts)]
pub struct InitCaster<'info> {
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub game: Box<Account<'info, Game>>,

    #[account(mut, has_one = authority, has_one = game)]
    pub player: Account<'info, Player>,

    #[account(address = sysvar::slot_hashes::id())]
    pub slots: UncheckedAccount<'info>,

    #[account(mut, constraint = lada_mint.to_account_info().key() == game.lada_mint_account)]
    pub lada_mint: Account<'info, Mint>,

    #[account(init, payer = authority,
    space = Caster::SIZE)]
    pub caster: Box<Account<'info, Caster>>,

    #[account(mut)]
    pub lada_token_account: Account<'info, TokenAccount>,
}

pub fn init_caster(ctx: Context<InitCaster>) -> ProgramResult {
    let player = &ctx.accounts.player;

    let slots_ref = ctx.accounts.slots.data.borrow();
    let slots = &**slots_ref;
    let mut offset: usize = 1;

    let caster = &mut ctx.accounts.caster;

    caster.owner = player.key();
    caster.version = 1;
    caster.level = 1;
    caster.experience = 0;
    caster.turn_commit = None;
    caster.modifiers = Modifiers {
        tile_level: 0,
        tile_column: u8::random_within_range(slots, &mut offset, 0, 2),
        head: None,
        robe: None,
        staff: None,
        spell_book: None,
    };

    //Decimal precision is 9
    let amount: u64 = u64::from(COST_IN_LADA_FOR_CASTER)
        .checked_mul(DECIMALS_PRECISION)
        .unwrap();

    if ctx.accounts.lada_token_account.amount.checked_sub(amount) == None {
        return Err(ErrorCode::PlayerIsPoor.into());
    }

    //We burn the lada tokens instead of giving them back to the game lada account
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::Burn {
                mint: ctx.accounts.lada_mint.to_account_info().clone(),
                to: ctx.accounts.lada_token_account.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}

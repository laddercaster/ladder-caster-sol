use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::{EARTH_INDEX, FIRE_INDEX, GAME_CREATOR_AUTHORITY_PUBKEY, give_exp_to_caster_resources_burned, ItemFeature, TurnCommit, WATER_INDEX};

#[derive(Accounts)]
pub struct ManualResourceBurn<'info> {
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub game: Box<Account<'info, Game>>,

    #[account(mut, has_one = authority, has_one = game)]
    pub player: Box<Account<'info, Player>>,
    #[account(mut, constraint = caster.owner == player.key())]
    pub caster: Box<Account<'info, Caster>>,

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

pub fn manual_resource_burn(
    ctx: Context<ManualResourceBurn>,
    resource_type: ItemFeature,
    amount_to_burn: u64,
) -> ProgramResult {
    // let game_turn = ctx.accounts.game.turn_info.turn;
    let turn_data = &mut ctx.accounts.game_turn_data;
    let caster = &mut ctx.accounts.caster;
    let game_turn = ctx.accounts.game.turn_info.turn;

    let token_account_to_burn_from: &Account<TokenAccount>;
    let mint_account_to_burn_from: &Account<Mint>;

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

    match resource_type {
        ItemFeature::Fire => {
            let resource_1_token_account = &ctx.accounts.resource_1_token_account;

            if resource_1_token_account.amount < amount_to_burn {
                return Err(ErrorCode::PlayerIsPoor.into());
            }

            give_exp_to_caster_resources_burned(caster, Some(amount_to_burn), None, None);

            turn_data.resource_1_burned += amount_to_burn;

            caster_turn_commit.resources_burned[FIRE_INDEX] = caster_turn_commit.resources_burned[FIRE_INDEX]
                .checked_add(amount_to_burn)
                .unwrap();

            token_account_to_burn_from = resource_1_token_account;
            mint_account_to_burn_from = &ctx.accounts.resource_1_mint_account;
        }
        ItemFeature::Water => {
            let resource_2_token_account = &ctx.accounts.resource_2_token_account;

            if resource_2_token_account.amount < amount_to_burn {
                return Err(ErrorCode::PlayerIsPoor.into());
            }

            give_exp_to_caster_resources_burned(caster, None, None, Some(amount_to_burn));

            turn_data.resource_2_burned += amount_to_burn;

            caster_turn_commit.resources_burned[WATER_INDEX] = caster_turn_commit.resources_burned[WATER_INDEX]
                .checked_add(amount_to_burn)
                .unwrap();

            token_account_to_burn_from = resource_2_token_account;
            mint_account_to_burn_from = &ctx.accounts.resource_2_mint_account;
        }
        ItemFeature::Earth => {
            let resource_3_token_account = &ctx.accounts.resource_3_token_account;

            if resource_3_token_account.amount < amount_to_burn {
                return Err(ErrorCode::PlayerIsPoor.into());
            }

            give_exp_to_caster_resources_burned(caster, None, Some(amount_to_burn), None);

            turn_data.resource_3_burned += amount_to_burn;

            caster_turn_commit.resources_burned[EARTH_INDEX] = caster_turn_commit.resources_burned[EARTH_INDEX]
                .checked_add(amount_to_burn)
                .unwrap();

            token_account_to_burn_from = resource_3_token_account;
            mint_account_to_burn_from = &ctx.accounts.resource_3_mint_account;
        }
        _ => {
            return Err(ErrorCode::InvalidResourceTypeForBurn.into());
        }
    }

    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::Burn {
                mint: mint_account_to_burn_from.to_account_info().clone(),
                to: token_account_to_burn_from.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        amount_to_burn,
    )?;

    ctx.accounts.caster.turn_commit = Some(caster_turn_commit);

    Ok(())
}

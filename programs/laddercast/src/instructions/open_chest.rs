use core::mem::size_of;

use anchor_lang::{prelude::*, solana_program::sysvar};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::zombify_account;
use crate::utils::{generate_new_equipment, generate_new_spell_book, ItemType, RandomValue};

#[derive(Accounts)]
pub struct OpenChest<'info> {
    pub system_program: Program<'info, System>,

    pub game: Box<Account<'info, Game>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority, has_one = game)]
    pub player: Account<'info, Player>,

    #[account(address = sysvar::slot_hashes::id())]
    pub slots: UncheckedAccount<'info>,

    #[account(
    mut,
    close = authority,
    constraint = chest.game == game.key(),
    constraint = chest.owner == player.key(),
    )]
    pub chest: Box<Account<'info, Item>>,

    #[account(init, space = Item::SIZE, payer = authority)]
    pub item_1: Box<Account<'info, Item>>,
    #[account(init, space = Item::SIZE, payer = authority)]
    pub item_2: Box<Account<'info, Item>>,
    #[account(init, space = Item::SIZE, payer = authority)]
    pub item_3: Box<Account<'info, Item>>,
}

pub fn open_chest(ctx: Context<OpenChest>) -> ProgramResult {
    let player = &ctx.accounts.player;
    let game = &ctx.accounts.game;
    let chest = &ctx.accounts.chest;

    let mut item_to_create = vec![
        &mut ctx.accounts.item_1,
        &mut ctx.accounts.item_2,
        &mut ctx.accounts.item_3,
    ];

    match chest.item_type {
        ItemType::Chest { .. } => {
            let min_item_level = match chest.item_type {
                ItemType::Chest { tier } => match tier {
                    1 => 1,
                    2 => 6,
                    3 => 11,
                    4 => 16,
                    _ => 1,
                },
                _ => 1,
            };
            let max_item_level = chest.level;

            let slots_ref = ctx.accounts.slots.data.borrow();
            let slots = &**slots_ref;
            let mut offset: usize = 1;

            //Item level of chest gives range 1 to item level
            // tier sets the minimum, tier 1 = 1, tier 2 = 6
            //Each chest generates 3 new item
            for i in 0..3 {
                let item_level =
                    u8::random_within_range(slots, &mut offset, min_item_level, max_item_level);

                if u8::random_within_range(slots, &mut offset, 1, 4) == 1 {
                    //This will generate a spell book (1 in 4 chances, between robe, staff, head and spell book)
                    generate_new_spell_book(
                        &mut item_to_create[i],
                        game,
                        player,
                        item_level,
                        slots,
                        &mut offset,
                    )?;
                } else {
                    generate_new_equipment(
                        &mut item_to_create[i],
                        game,
                        player,
                        item_level,
                        None,
                        slots,
                        &mut offset,
                    )?;
                }
            }
        }
        _ => {
            return Err(ErrorCode::ItemIsNotAChest.into());
        }
    }

    zombify_account(
        &mut ctx.accounts.chest,
        ctx.accounts.authority.to_account_info(),
        ctx.program_id,
    )?;

    Ok(())
}

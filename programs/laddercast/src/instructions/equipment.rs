use anchor_lang::prelude::*;

use crate::account::{Caster, Game, Item, Player};
use crate::error::ErrorCode;
use crate::utils::{EquipmentType, ItemType};

#[derive(Accounts)]
pub struct EquipUnequipItem<'info> {
    pub game: Box<Account<'info, Game>>,
    pub authority: Signer<'info>,
    #[account(
    mut,
    constraint = player.game == game.key(),
    has_one = authority,
    has_one = game
    )]
    pub player: Account<'info, Player>,
    #[account(
    mut,
    constraint = item.owner == player.key(),
    )]
    pub caster: Account<'info, Caster>,
    #[account(
    mut,
    constraint = item.game == game.key(),
    constraint = item.owner == player.key(),
    )]
    pub item: Box<Account<'info, Item>>,
}

pub fn equip_item(ctx: Context<EquipUnequipItem>) -> ProgramResult {
    let caster = &mut ctx.accounts.caster;
    let item = &mut ctx.accounts.item;

    if item.equipped_owner != None {
        return Err(ErrorCode::ItemAlreadyInUse.into());
    }

    if item.level > caster.level {
        return Err(ErrorCode::ItemLevelTooHigh.into());
    }

    if caster.turn_commit != None {
        return Err(ErrorCode::NoEquipUnequipOnPendingTurn.into());
    }

    item.equipped_owner = Some(caster.key());

    match item.item_type {
        ItemType::Equipment {
            feature: _,
            rarity: _,
            equipment_type,
            value: _,
        } => match equipment_type {
            EquipmentType::Head => {
                if caster.modifiers.head != None {
                    return Err(ErrorCode::ItemTypeAlreadyEquipped.into());
                }

                caster.modifiers.head = Some(item.key());
            }
            EquipmentType::Staff => {
                if caster.modifiers.staff != None {
                    return Err(ErrorCode::ItemTypeAlreadyEquipped.into());
                }

                caster.modifiers.staff = Some(item.key());
            }
            EquipmentType::Robe => {
                if caster.modifiers.robe != None {
                    return Err(ErrorCode::ItemTypeAlreadyEquipped.into());
                }

                caster.modifiers.robe = Some(item.key());
            }
        },
        ItemType::SpellBook { .. } => {
            if caster.modifiers.spell_book != None {
                return Err(ErrorCode::ItemTypeAlreadyEquipped.into());
            }

            caster.modifiers.spell_book = Some(item.key());
        }
        _ => {
            return Err(ErrorCode::InvalidEquipItemType.into());
        }
    }

    Ok(())
}

pub fn unequip_item(ctx: Context<EquipUnequipItem>) -> ProgramResult {
    let caster = &mut ctx.accounts.caster;
    let item = &mut ctx.accounts.item;

    if item.equipped_owner == None || item.equipped_owner.unwrap() != caster.key() {
        return Err(ErrorCode::ItemNotExists.into());
    }

    if caster.turn_commit != None {
        return Err(ErrorCode::NoEquipUnequipOnPendingTurn.into());
    }

    match item.item_type {
        ItemType::Equipment { equipment_type, .. } => match equipment_type {
            EquipmentType::Head => {
                caster.modifiers.head = None;
            }
            EquipmentType::Robe => {
                caster.modifiers.robe = None;
            }
            EquipmentType::Staff => {
                caster.modifiers.staff = None;
            }
        },
        ItemType::SpellBook { .. } => {
            caster.modifiers.spell_book = None;
        }
        _ => {
            return Err(ErrorCode::InvalidEquipItemType.into());
        }
    }

    item.equipped_owner = None;

    Ok(())
}

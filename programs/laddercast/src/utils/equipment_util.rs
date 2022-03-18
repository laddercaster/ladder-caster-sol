use anchor_lang::prelude::*;

use crate::account::{Game, Item, Player};
use crate::utils::{
    EquipmentType, ItemFeature, ItemRarity, ItemType, RandomEnumValue, RandomValue, SpellType,
};

pub fn get_item_resource_value(
    item_rarity: ItemRarity,
    item_level: u8,
    slots: &[u8],
    offset: &mut usize,
) -> u16 {
    let multiplier: u16;

    match item_rarity {
        ItemRarity::Common => {
            multiplier = 10;
        }
        ItemRarity::Rare => {
            multiplier = 20;
        }
        ItemRarity::Epic => {
            multiplier = 30;
        }
        ItemRarity::Legendary => {
            multiplier = 40;
        }
    }

    let min: u16 = item_level as u16 * (multiplier - 10) + 1;
    let max: u16 = item_level as u16 * multiplier;

    u16::random_within_range(slots, offset, min, max)
}

pub fn get_item_percentage_value(
    item_rarity: ItemRarity,
    item_level: u8,
    slots: &[u8],
    offset: &mut usize,
) -> u16 {
    let multiplier: u16;

    match item_rarity {
        ItemRarity::Common => {
            multiplier = 100;
        }
        ItemRarity::Rare => {
            multiplier = 200;
        }
        ItemRarity::Epic => {
            multiplier = 300;
        }
        ItemRarity::Legendary => {
            multiplier = 400;
        }
    }

    let min: u16 = multiplier;
    let max = ((item_level as u16 / 3 * 100) + multiplier) as u16;

    u16::random_within_range(slots, offset, min, max)
}

pub fn get_spell_book_value(cost: u16, item_rarity: ItemRarity, multiplier: u16) -> u16 {
    //Multiplier is something that applies to experience only for now (same calculation as the
    //resource spells, but x 2)
    let rarity_odds: u16;
    let rarity_min: u16;
    let rarity_max: u16;

    match item_rarity {
        ItemRarity::Common => {
            rarity_odds = 8;
            rarity_min = 1;
            rarity_max = 10;
        }
        ItemRarity::Rare => {
            rarity_odds = 6;
            rarity_min = 11;
            rarity_max = 20;
        }
        ItemRarity::Epic => {
            rarity_odds = 4;
            rarity_min = 21;
            rarity_max = 30;
        }
        ItemRarity::Legendary => {
            rarity_odds = 2;
            rarity_min = 31;
            rarity_max = 40;
        }
    }

    let avg_resources = (rarity_min as f32 * rarity_max as f32 / 2.0) as u16;
    const SPELL_MULTIPLE: u16 = 4;

    ((cost * (rarity_odds - 1)) + (SPELL_MULTIPLE * avg_resources)) * multiplier
}

pub fn get_item_rarity(slots: &[u8], offset: &mut usize) -> ItemRarity {
    let item_rarity_chance = u8::random_within_range(slots, offset, 1, 100);

    match item_rarity_chance {
        0..=80 => ItemRarity::Common,
        81..=95 => ItemRarity::Rare,
        96..=99 => ItemRarity::Epic,
        100 => ItemRarity::Legendary,
        _ => ItemRarity::Common,
    }
}

pub fn generate_new_equipment(
    item: &mut Account<Item>,
    game: &Account<Game>,
    player: &Account<Player>,
    item_level: u8,
    item_rarity: Option<ItemRarity>,
    slots: &[u8],
    offset: &mut usize,
) -> ProgramResult {
    let item_value: u16;

    let item_feature = ItemFeature::get_random(slots, offset);
    let new_item_rarity: ItemRarity;

    match item_rarity {
        None => {
            new_item_rarity = get_item_rarity(slots, offset);
        }
        Some(item_rarity) => {
            new_item_rarity = item_rarity;
        }
    }

    match item_feature {
        ItemFeature::Fire | ItemFeature::Water | ItemFeature::Earth => {
            item_value = get_item_resource_value(new_item_rarity, item_level, slots, offset);
        }
        ItemFeature::Power | ItemFeature::Magic => {
            item_value = get_item_percentage_value(new_item_rarity, item_level, slots, offset);
        }
    }

    item.game = game.key();
    item.owner = player.key();
    item.equipped_owner = None;
    item.item_type = ItemType::Equipment {
        feature: ItemFeature::get_random(slots, offset),
        rarity: new_item_rarity,
        equipment_type: EquipmentType::get_random(slots, offset),
        value: item_value,
    };
    item.level = item_level;
    Ok(())
}

pub fn generate_new_spell_book(
    item: &mut Account<Item>,
    game: &Account<Game>,
    player: &Account<Player>,
    item_level: u8,
    slots: &[u8],
    offset: &mut usize,
) -> ProgramResult {
    let spell = SpellType::get_random(slots, offset);
    let mut item_value: u16 = 0;

    let cost: u16 =
        u16::random_within_range(slots, offset, item_level as u16, 10 * item_level as u16);
    let cost_feature: ItemFeature = ItemFeature::get_random_within_range(slots, offset, 2, 4);

    let spell_book_rarity = get_item_rarity(slots, offset);

    match spell {
        //These values are based on spell book level and spell book rarity
        SpellType::Fire | SpellType::Water | SpellType::Earth => {
            item_value = get_spell_book_value(cost, spell_book_rarity, 1);
        }
        SpellType::Experience => {
            item_value = get_spell_book_value(cost, spell_book_rarity, 2);
        }
        _ => {}
    }

    item.game = game.key();
    item.owner = player.key();
    item.equipped_owner = None;
    item.item_type = ItemType::SpellBook {
        spell,
        cost_feature,
        cost,
        value: item_value,
        rarity: spell_book_rarity,
    };
    item.level = item_level;
    Ok(())
}

pub fn zombify_account(
    item: &mut Account<Item>,
    authority: AccountInfo,
    program_id: &Pubkey,
) -> ProgramResult {
    // Zombify item so item will be useless if rent is put back in
    item.item_type = ItemType::Zombie;

    // Take that money
    let dest_starting_lamports = authority.lamports();
    **authority.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(item.to_account_info().lamports())
        .unwrap();
    **item.to_account_info().lamports.borrow_mut() = 0;

    // Exit and write
    item.exit(program_id)?;
    Ok(())
}

pub fn get_name_for_mint(item_type: &ItemType) -> Option<String> {
    match item_type {
        ItemType::Equipment {
            equipment_type, ..
        } => {
            match equipment_type {
                EquipmentType::Staff => Some("Staff".to_string()),
                EquipmentType::Robe => Some("Robe".to_string()),
                EquipmentType::Head => Some("Head".to_string())
            }
        }
        ItemType::SpellBook { .. } => Some("Spellbook".to_string()),
        ItemType::Chest { .. } => Some("Chest".to_string()),
        ItemType::Zombie { .. } => None
    }
}
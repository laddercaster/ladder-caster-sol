use crate::{ItemFeature, ItemType};
use crate::account::{Caster, Item};
use crate::utils::{EquipmentType, ItemRarity, SpellType};

//Format for merkle strings
//
// Chest: {uri}:chest:{item_level}:{tier}
// Spellbook: {uri}:spellbook:{item_level}:{spell_type}:{cost_feature}:{rarity}:{cost}:{value}
// Equipment: {uri}:{equipment_type}:{item_level}:{feature}:{rarity}:{value}
// Caster: {uri}:caster:{version}:{level}

const SEPARATOR: &str = ":";
const CHEST_NAME: &str = "chest";
const SPELLBOOK_NAME: &str = "spellbook";
const ROBE_NAME: &str = "robe";
const HEAD_NAME: &str = "head";
const STAFF_NAME: &str = "staff";
const CASTER_NAME: &str = "caster";

fn get_merkle_string_for_spell_type(spell: SpellType) -> String {
    match spell {
        SpellType::Fire => "fire".to_string(),
        SpellType::Water => "water".to_string(),
        SpellType::Earth => "earth".to_string(),
        SpellType::Experience => "experience".to_string(),
        SpellType::Craft => "craft".to_string(),
        SpellType::Item => "item".to_string()
    }
}

fn get_merkle_string_for_item_feature(item_feature: ItemFeature) -> String {
    match item_feature {
        ItemFeature::Power => "power".to_string(),
        ItemFeature::Magic => "magic".to_string(),
        ItemFeature::Fire => "fire".to_string(),
        ItemFeature::Earth => "earth".to_string(),
        ItemFeature::Water => "water".to_string()
    }
}

fn get_merkle_string_for_item_rarity(item_rarity: ItemRarity) -> String {
    match item_rarity {
        ItemRarity::Common => "common".to_string(),
        ItemRarity::Rare => "rare".to_string(),
        ItemRarity::Epic => "epic".to_string(),
        ItemRarity::Legendary => "legendary".to_string()
    }
}

fn get_merkle_string_for_equipment_type(equipment_type: EquipmentType) -> String {
    match equipment_type {
        EquipmentType::Head => HEAD_NAME.to_string(),
        EquipmentType::Robe => ROBE_NAME.to_string(),
        EquipmentType::Staff => STAFF_NAME.to_string()
    }
}

fn get_merkle_string_for_chest(uri: &str, item: Item, tier: u8) -> String {
    let mut merkle_string = "".to_owned();

    merkle_string.push_str(uri);
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(CHEST_NAME);
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&item.level.to_string());
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&tier.to_string());

    merkle_string
}

fn get_merkle_string_for_spell_book(uri: &str, item: Item, spell: SpellType, cost_feature: ItemFeature, rarity: ItemRarity, cost: u16, value: u16) -> String {
    let mut merkle_string = "".to_owned();

    merkle_string.push_str(uri);
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(SPELLBOOK_NAME);
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&item.level.to_string());
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&get_merkle_string_for_spell_type(spell));
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&get_merkle_string_for_item_feature(cost_feature));
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&get_merkle_string_for_item_rarity(rarity));
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&cost.to_string());
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&value.to_string());

    merkle_string
}

fn get_merkle_string_for_equipment(uri: &str, item: Item, equipment_type: EquipmentType, feature: ItemFeature, rarity: ItemRarity, value: u16) -> String {
    let mut merkle_string = "".to_owned();

    merkle_string.push_str(uri);
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&get_merkle_string_for_equipment_type(equipment_type));
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&item.level.to_string());
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&get_merkle_string_for_item_feature(feature));
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&get_merkle_string_for_item_rarity(rarity));
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&value.to_string());

    merkle_string
}

pub fn get_merkle_string_for_item(uri: &str, item: Item) -> Option<String> {
    match item.item_type {
        ItemType::SpellBook { spell, cost_feature, rarity, cost, value } => {
            Some(get_merkle_string_for_spell_book(uri, item, spell, cost_feature, rarity, cost, value))
        }
        ItemType::Chest { tier } => {
            Some(get_merkle_string_for_chest(uri, item, tier))
        }
        ItemType::Equipment { feature, rarity, equipment_type, value } => {
            Some(get_merkle_string_for_equipment(uri, item, equipment_type, feature, rarity, value))
        }
        _ => {
            None
        }
    }
}

pub fn get_merkle_string_for_caster(uri: &str, caster: Caster) -> String {
    let mut merkle_string = "".to_owned();

    merkle_string.push_str(uri);
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(CASTER_NAME);
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&caster.version.to_string());
    merkle_string.push_str(SEPARATOR);
    merkle_string.push_str(&caster.level.to_string());

    merkle_string
}

//Function for merkle proof
//Taken from https://github.com/sayantank/anchor-whitelist
pub fn verify_merkle_proof(proof: Vec<[u8; 32]>, root: [u8; 32], leaf: [u8; 32]) -> bool {
    let mut computed_hash = leaf;
    for proof_element in proof.into_iter() {
        if computed_hash <= proof_element {
            // Hash(current computed hash + current element of the proof)
            computed_hash =
                anchor_lang::solana_program::keccak::hashv(&[&computed_hash, &proof_element]).0;
        } else {
            // Hash(current element of the proof + current computed hash)
            computed_hash =
                anchor_lang::solana_program::keccak::hashv(&[&proof_element, &computed_hash]).0;
        }
    }
    // Check if the computed hash (root) is equal to the provided root
    computed_hash == root
}
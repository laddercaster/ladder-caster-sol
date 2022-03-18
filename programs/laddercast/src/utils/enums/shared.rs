use anchor_lang::prelude::*;
use strum::{EnumCount, EnumIter};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub enum ItemType {
    // For hackers, you could have been cool but nah
    Zombie,
    Chest {
        /// Between 1 to 4 based on level of tile where found 1-5 tier 1, 6-10 tier 2, 11-15 tier 3, 16-30 tier 4
        tier: u8,
    },
    Equipment {
        feature: ItemFeature,
        rarity: ItemRarity,
        equipment_type: EquipmentType,
        /// 0-1200 for resources and 0-1400 for percentage
        value: u16,
    },
    SpellBook {
        spell: SpellType,
        cost_feature: ItemFeature,
        rarity: ItemRarity,
        /// 1-300
        cost: u16,
        /// 0-3.6k
        value: u16,
    },
}

impl Default for ItemType {
    fn default() -> Self {
        ItemType::Zombie
    }
}

#[derive(
AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy, EnumCount, EnumIter,
)]
pub enum ItemFeature {
    Power,
    Magic,
    Fire,
    Earth,
    Water,
}

#[derive(
AnchorSerialize,
AnchorDeserialize,
Clone,
PartialEq,
Debug,
Copy,
EnumCount,
EnumIter,
PartialOrd,
)]
pub enum ItemRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

#[derive(
AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy, EnumCount, EnumIter,
)]
pub enum EquipmentType {
    Head,
    Robe,
    Staff,
}

#[derive(
AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy, EnumCount, EnumIter,
)]
pub enum SpellType {
    Fire,
    Water,
    Earth,
    Experience,
    Craft,
    Item,
}

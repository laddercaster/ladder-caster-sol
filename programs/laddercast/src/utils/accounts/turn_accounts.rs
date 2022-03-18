use std::mem::size_of;

use anchor_lang::prelude::*;

use crate::utils::ItemRarity;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Default, Copy)]
pub struct TurnCommit {
    pub turn: u32,
    pub resources_burned: [u64; 3],
    pub actions: CommittedActions,
}

impl TurnCommit {
    pub const SIZE: usize = 8 + 4 + 8 * 3 + CommittedActions::SIZE;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Default, Copy)]
pub struct CommittedActions {
    pub loot: bool,
    pub spell: Option<SpellSnapshot>,
    pub mv: Option<[u8; 2]>,
    pub crafting: Option<CraftingSnapshot>,
    pub action_order: [u8; 4],
}

impl CommittedActions {
    pub const SIZE: usize =
        8 + 1 + 1 + SpellSnapshot::SIZE + 1 + 1 * 2 + 1 + CraftingSnapshot::SIZE + 1 * 4;

    fn get_highest_value(&self) -> u8 {
        *self
            .action_order
            .iter()
            .enumerate()
            .map(|(x, y)| (y, x))
            .max()
            .unwrap()
            .0
    }
    pub fn add_new_action_order(&mut self, index: u8) {
        if (index as usize) >= self.action_order.len() {
            panic!("WHAT");
        }
        self.action_order[index as usize] = self.get_highest_value() + 1;
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub struct CraftingSnapshot {
    pub min_level: u8,
    pub min_rarity: ItemRarity,
    pub max_rarity: ItemRarity,
}

impl CraftingSnapshot {
    pub const SIZE: usize =
        8 + 1 + size_of::<ItemRarity>() + size_of::<ItemRarity>() + /* padding */ 300;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub struct SpellSnapshot {
    pub is_extra_level_bonus: bool,
}

impl SpellSnapshot {
    pub const SIZE: usize = 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Default, Copy)]
pub struct Modifiers {
    /// 0..29
    pub tile_level: u8,
    /// 0, 1, 2
    pub tile_column: u8,
    /// Item
    pub head: Option<Pubkey>,
    /// Item
    pub robe: Option<Pubkey>,
    /// Item
    pub staff: Option<Pubkey>,
    /// Item
    pub spell_book: Option<Pubkey>,
}

impl Modifiers {
    pub const SIZE: usize = 8 + 1 + 1 + (1 + 32) * 4;
}

use anchor_lang::prelude::Pubkey;

use crate::{GameTurnInfo, ItemFeature, ItemType, Tile, TileType};
use crate::account::{Caster, Game, Item};
use crate::utils::{EquipmentType, ItemRarity, Modifiers, SpellType};

//Testing utilities
pub fn create_caster_for_testing() -> Caster {
    Caster {
        version: 1,
        level: 1,
        experience: 0,
        turn_commit: None,
        modifiers: create_caster_modifiers_for_testing(false),
        owner: Pubkey::new_unique(),
    }
}

pub fn create_caster_modifiers_for_testing(with_spell: bool) -> Modifiers {
    let mut modifiers = Modifiers {
        tile_level: 1,
        tile_column: 1,
        head: Some(Pubkey::new_unique()),
        robe: Some(Pubkey::new_unique()),
        staff: Some(Pubkey::new_unique()),
        spell_book: None,
    };

    if with_spell {
        modifiers.spell_book = Some(Pubkey::new_unique());
    }

    modifiers
}

pub fn create_tile_for_testing(
    feature: TileType,
    life: u8,
    is_first_time_spawning: bool,
) -> Tile {
    Tile {
        life,
        tile_type: feature,
        is_first_time_spawning,
    }
}

pub fn create_game_for_testing() -> Game {
    Game {
        version: 0,
        authority: Pubkey::new_unique(),
        map: [[None; 3]; 30],
        turn_info: GameTurnInfo {
            turn: 1,
            turn_delay: 2,
            last_crank_seconds: 3,
            last_tile_spawn: 4,
            tile_spawn_delay: 5,
        },
        last_turn_added: 0,
        signer_bump: 0,
        resource_1_mint_account: Default::default(),
        resource_2_mint_account: Default::default(),
        resource_3_mint_account: Default::default(),
        lada_mint_account: Default::default(),
        lada_token_account: Default::default(),
        merkle_root_nft: [1; 32],
    }
}

pub fn create_equipment_for_testing(equipment_type: EquipmentType) -> Item {
    Item {
        game: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        level: 3,
        item_type: ItemType::Equipment {
            feature: ItemFeature::Power,
            rarity: ItemRarity::Common,
            equipment_type,
            value: 1,
        },
        equipped_owner: None,
    }
}

pub fn create_spell_book_for_testing() -> Item {
    Item {
        game: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        level: 3,
        item_type: ItemType::SpellBook {
            spell: SpellType::Fire,
            cost_feature: ItemFeature::Fire,
            rarity: ItemRarity::Common,
            cost: 1,
            value: 2,
        },
        equipped_owner: None,
    }
}

pub fn create_chest_for_testing() -> Item {
    Item {
        game: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        level: 3,
        item_type: ItemType::Chest {
            tier: 2
        },
        equipped_owner: None,
    }
}

pub fn create_zombie_for_testing() -> Item {
    Item {
        game: Pubkey::new_unique(),
        owner: Pubkey::new_unique(),
        level: 3,
        item_type: ItemType::Zombie {},
        equipped_owner: None,
    }
}
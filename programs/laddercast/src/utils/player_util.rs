use anchor_lang::{Key, ToAccountInfo};
use anchor_lang::prelude::Account;
use anchor_lang::prelude::ProgramError;

use crate::{ItemFeature, ItemType, PlayerBonuses};
use crate::account::{Caster, Game, Item, Player};
use crate::utils::{EquipmentType, Modifiers};

pub fn get_player_bonuses(
    modifiers: &Modifiers,
    item_accounts: Vec<&Result<Account<Item>, ProgramError>>,
    game: &Account<Game>,
    player: &Account<Player>,
    caster: &Account<Caster>,
) -> PlayerBonuses {
    let mut player_bonuses: PlayerBonuses = PlayerBonuses {
        critical_chance: 200, //By default they have 2% chance of critical strike
        magic_find_chance: 0,
        fire_chance: 0,
        water_chance: 0,
        earth_chance: 0,
    };

    for item in item_accounts {
        match item {
            Ok(equipment) => {
                if equipment.owner != player.to_account_info().key()
                    || equipment.equipped_owner.unwrap() != caster.to_account_info().key()
                    || equipment.game != game.to_account_info().key()
                {
                    continue;
                }

                if let ItemType::Equipment { feature, value, equipment_type, .. } = equipment.item_type {
                    match equipment_type {
                        EquipmentType::Head => {
                            if modifiers.head == None || modifiers.head.unwrap() != equipment.to_account_info().key() {
                                continue;
                            }
                        }
                        EquipmentType::Staff => {
                            if modifiers.staff == None || modifiers.staff.unwrap() != equipment.to_account_info().key() {
                                continue;
                            }
                        }
                        EquipmentType::Robe => {
                            if modifiers.robe == None || modifiers.robe.unwrap() != equipment.to_account_info().key() {
                                continue;
                            }
                        }
                    }

                    match feature {
                        ItemFeature::Fire => player_bonuses.fire_chance += value,
                        ItemFeature::Water => player_bonuses.water_chance += value,
                        ItemFeature::Earth => player_bonuses.earth_chance += value,
                        ItemFeature::Magic => player_bonuses.magic_find_chance += value,
                        ItemFeature::Power => player_bonuses.critical_chance += value,
                    }
                }
            }
            Err(_) => {}
        }
    }

    player_bonuses
}

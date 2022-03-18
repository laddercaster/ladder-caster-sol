#[cfg(test)]
mod test_internal_functions {
    use std::convert::TryInto;

    use lazy_static::lazy_static;
    use rand::random;

    use crate::{ItemFeature, ItemType};
    use crate::utils::{create_game_for_testing, generate_new_equipment, generate_new_spell_book, get_item_percentage_value, get_item_rarity, get_item_resource_value, ItemRarity, SpellType};

    lazy_static! {
        static ref SLOT_HASHES: [u8; 512 * 40] = generate_slot_hashes(true).try_into().unwrap();
    }

    fn generate_slot_hashes(is_random: bool) -> Vec<u8> {
        if is_random {
            (0..512 * 40).map(|_| random()).collect()
        } else {
            vec![1; 512 * 40]
        }
    }

    #[test]
    fn test_get_item_resource_value_is_epic() {
        let mut offset: usize = 1;

        //Multiple tests to make sure it works with different values (random)
        for _i in 0..4 {
            let value =
                get_item_resource_value(ItemRarity::Epic, 2, SLOT_HASHES.as_slice(), &mut offset);
            assert!((41..=60).contains(&value));
        }
    }

    #[test]
    fn test_get_item_percentage_value_is_common() {
        let mut offset: usize = 1;

        //Multiple tests to make sure it works with different values (random)
        for _i in 0..4 {
            let value = get_item_percentage_value(
                ItemRarity::Common,
                4,
                SLOT_HASHES.as_slice(),
                &mut offset,
            );
            assert!((100..=233).contains(&value));
        }

        for _i in 0..4 {
            let value = get_item_percentage_value(
                ItemRarity::Epic,
                20,
                SLOT_HASHES.as_slice(),
                &mut offset,
            );
            assert!((300..=966).contains(&value));
        }
    }

    #[test]
    fn test_get_item_rarity() {
        let mut offset: usize = 1;

        let rarity = get_item_rarity(SLOT_HASHES.as_slice(), &mut offset);

        assert!(matches!(
            rarity,
            ItemRarity::Common | ItemRarity::Rare | ItemRarity::Epic | ItemRarity::Legendary
        ));
    }
}
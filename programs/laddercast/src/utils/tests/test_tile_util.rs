#[cfg(test)]
mod test_internal_functions {
    use std::convert::TryInto;

    use lazy_static::lazy_static;
    use rand::random;

    use crate::{Tile, TileType};
    use crate::utils::{create_tile_for_testing, cycle_tile, get_current_tile, get_highest_level_and_column};

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
    fn test_cycle_tile_tile_is_none() {
        let mut offset: usize = 1;

        let generated_tile = cycle_tile(None, 1, SLOT_HASHES.as_slice(), &mut offset);

        assert!(generated_tile.life >= 3 && generated_tile.life <= 6);
        assert!(matches!(
            generated_tile.tile_type,
            TileType::Fire | TileType::Water | TileType::Earth
        ));
    }

    #[test]
    fn test_cycle_tile_is_crafting_or_legendary() {
        let current_tile = create_tile_for_testing(TileType::Crafting, 1, false);

        let mut offset: usize = 1;

        let generated_tile = cycle_tile(Some(current_tile), 1, SLOT_HASHES.as_slice(), &mut offset);

        assert!(generated_tile.life >= 3 && generated_tile.life <= 6);
        assert!(matches!(
            generated_tile.tile_type,
            TileType::Fire | TileType::Water | TileType::Earth
        ));
    }

    #[test]
    fn test_cycle_tile_is_resource_is_not_first_time_spawning() {
        let current_tile = create_tile_for_testing(TileType::Fire, 1, false);

        let mut offset: usize = 1;

        let generated_tile = cycle_tile(Some(current_tile), 1, SLOT_HASHES.as_slice(), &mut offset);

        assert_eq!(generated_tile.life, 1);
        assert_eq!(generated_tile.tile_type, TileType::Crafting);
    }

    #[test]
    fn test_cycle_tile_is_resource_is_first_time_spawning() {
        let current_tile = create_tile_for_testing(TileType::Fire, 1, true);

        let mut offset: usize = 1;

        let generated_tile = cycle_tile(Some(current_tile), 1, SLOT_HASHES.as_slice(), &mut offset);

        assert_eq!(generated_tile.life, 1);
        assert_eq!(generated_tile.tile_type, TileType::Legendary);
    }

    #[test]
    fn test_cycle_tile_is_resource_is_level_30_tile() {
        let current_tile = create_tile_for_testing(TileType::Fire, 1, false);

        let mut offset: usize = 1;

        let generated_tile =
            cycle_tile(Some(current_tile), 30, SLOT_HASHES.as_slice(), &mut offset);

        assert_eq!(generated_tile.life, 1);
        assert_eq!(generated_tile.tile_type, TileType::Legendary);
    }

    #[test]
    fn test_get_highest_level_and_column() {
        let mut map: [[Option<Tile>; 3]; 30] = [[None; 3]; 30];

        map[0][0] = Some(create_tile_for_testing(TileType::Fire, 1, false));
        map[1][0] = Some(create_tile_for_testing(TileType::Fire, 1, false));
        map[0][1] = Some(create_tile_for_testing(TileType::Fire, 1, false));
        map[2][0] = Some(create_tile_for_testing(TileType::Fire, 1, false));
        map[2][1] = Some(create_tile_for_testing(TileType::Fire, 1, false));

        let (highest_level, highest_column) = get_highest_level_and_column(&map);

        assert_eq!(highest_level, 2);
        assert_eq!(highest_column, 1);
    }

    #[test]
    fn test_get_current_tile() {
        let mut map: [[Option<Tile>; 3]; 30] = [[None; 3]; 30];

        map[0][0] = Some(create_tile_for_testing(TileType::Fire, 1, false));
        map[1][0] = Some(create_tile_for_testing(TileType::Fire, 1, false));
        map[0][1] = Some(create_tile_for_testing(TileType::Fire, 1, false));
        map[2][0] = Some(create_tile_for_testing(TileType::Fire, 1, false));
        map[2][1] = Some(create_tile_for_testing(TileType::Fire, 1, false));

        let current_tile = get_current_tile(&map, 2, 1);

        assert_eq!(current_tile.unwrap().tile_type, map[2][1].unwrap().tile_type);
        assert_eq!(current_tile.unwrap().is_first_time_spawning, map[2][1].unwrap().is_first_time_spawning);
        assert_eq!(current_tile.unwrap().life, map[2][1].unwrap().life);
    }
}
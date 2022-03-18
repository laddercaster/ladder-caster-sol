#[cfg(test)]
mod test_internal_functions {
    use crate::utils::{create_caster_for_testing, create_caster_modifiers_for_testing, create_chest_for_testing, create_equipment_for_testing, create_spell_book_for_testing, create_zombie_for_testing, EquipmentType, get_merkle_string_for_caster, get_merkle_string_for_item, verify_merkle_proof};

    const URI: &str = "https://laddercaster.com";

    //The leaves are [
    //                 "https://laddercaster.com:caster:1:1",
    //                 "https://laddercaster.com:caster:1:2",
    //                 "https://laddercaster.com:caster:1:3",
    //                 "https://laddercaster.com:caster:1:4",
    //             ],
    const MERKLE_ROOT: [u8; 32] = [
        11, 255, 107, 110, 116, 118, 136, 248,
        114, 25, 82, 117, 62, 38, 97, 44,
        30, 164, 55, 200, 193, 211, 235, 226,
        72, 61, 219, 131, 247, 91, 171, 152
    ];

    //Leaf is "https://laddercaster.com:caster:1:1"
    const MERKLE_LEAF: [u8; 32] = [
        248, 244, 117, 59, 23, 247, 236, 40,
        205, 36, 128, 50, 66, 251, 233, 190,
        98, 208, 189, 141, 58, 172, 189, 14,
        120, 255, 162, 108, 189, 97, 3, 144
    ];

    #[test]
    fn test_get_merkle_string_for_item_invalid_item() {
        let item = create_zombie_for_testing();

        assert!(
            get_merkle_string_for_item(URI, item) == None
        );
    }

    #[test]
    fn test_get_merkle_string_for_item_valid_item() {
        let mut item = create_chest_for_testing();

        assert_eq!(
            get_merkle_string_for_item(URI, item).unwrap(),
            "https://laddercaster.com:chest:3:2"
        );

        item = create_spell_book_for_testing();

        assert_eq!(
            get_merkle_string_for_item(URI, item).unwrap(),
            "https://laddercaster.com:spellbook:3:fire:fire:common:1:2"
        );

        item = create_equipment_for_testing(EquipmentType::Staff);

        assert_eq!(
            get_merkle_string_for_item(URI, item).unwrap(),
            "https://laddercaster.com:staff:3:power:common:1"
        );

        item = create_equipment_for_testing(EquipmentType::Head);

        assert_eq!(
            get_merkle_string_for_item(URI, item).unwrap(),
            "https://laddercaster.com:head:3:power:common:1"
        );

        item = create_equipment_for_testing(EquipmentType::Robe);

        assert_eq!(
            get_merkle_string_for_item(URI, item).unwrap(),
            "https://laddercaster.com:robe:3:power:common:1"
        );
    }

    #[test]
    fn test_get_merkle_string_for_caster() {
        let caster = create_caster_for_testing();

        assert_eq!(
            get_merkle_string_for_caster(URI, caster),
            "https://laddercaster.com:caster:1:1"
        );
    }

    #[test]
    fn test_verify_merkle_proof_invalid() {
        let valid_proof = vec![
            [
                250, 205, 52, 23, 50, 239, 114,
                8, 120, 244, 113, 246, 106, 114,
                184, 223, 122, 144, 79, 152, 227,
                221, 101, 250, 14, 230, 29, 249,
                21, 241, 150, 120
            ],
            [
                125, 14, 137, 191, 80, 159, 116, 204,
                186, 34, 148, 52, 223, 30, 167, 198,
                37, 2, 174, 231, 77, 78, 106, 188,
                79, 127, 254, 183, 42, 80, 251, 161
            ],
        ];

        assert!(!verify_merkle_proof(valid_proof, MERKLE_ROOT, MERKLE_LEAF));
    }

    #[test]
    fn test_verify_merkle_proof_valid() {
        let valid_proof = vec![
            [
                250, 205, 52, 23, 50, 239, 114,
                8, 120, 244, 113, 246, 106, 114,
                184, 223, 122, 144, 79, 152, 227,
                221, 101, 250, 14, 230, 29, 249,
                21, 241, 150, 120
            ],
            [
                125, 14, 137, 191, 80, 159, 116, 204,
                186, 34, 148, 52, 223, 30, 167, 198,
                37, 2, 174, 231, 77, 78, 106, 188,
                79, 127, 254, 183, 42, 80, 251, 160
            ],
        ];

        assert!(verify_merkle_proof(valid_proof, MERKLE_ROOT, MERKLE_LEAF));
    }
}
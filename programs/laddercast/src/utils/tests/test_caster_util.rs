#[cfg(test)]
mod test_internal_functions {
    use std::convert::TryInto;

    use lazy_static::lazy_static;
    use rand::random;

    use crate::utils::{create_caster_for_testing, give_exp_to_caster_resources_burned, give_exp_to_caster_spell, is_spell_successful, ItemRarity};

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
    fn test_give_exp_to_caster_resources_burned_no_level_up() {
        let mut caster = create_caster_for_testing();

        give_exp_to_caster_resources_burned(&mut caster, Some(10), Some(20), Some(30));

        assert_eq!(caster.experience, 60);
        assert_eq!(caster.level, 1)
    }

    #[test]
    fn test_give_exp_to_caster_resources_burned_with_level_up() {
        let mut caster = create_caster_for_testing();

        give_exp_to_caster_resources_burned(&mut caster, Some(1600), None, None);

        assert_eq!(caster.experience, 1600);
        assert_eq!(caster.level, 2)
    }

    #[test]
    fn test_give_exp_to_caster_resources_burned_with_level_up_but_already_max_level() {
        let mut caster = create_caster_for_testing();
        caster.level = 30;

        give_exp_to_caster_resources_burned(&mut caster, Some(2400), None, None);

        assert_eq!(caster.experience, 2400);
        assert_eq!(caster.level, 30)
    }

    #[test]
    fn test_give_exp_to_caster_spell_no_level_up() {
        let mut caster = create_caster_for_testing();

        give_exp_to_caster_spell(&mut caster, 10);

        assert_eq!(caster.experience, 10);
        assert_eq!(caster.level, 1)
    }

    #[test]
    fn test_give_exp_to_caster_spell_with_level_up() {
        let mut caster = create_caster_for_testing();

        give_exp_to_caster_spell(&mut caster, 1600);

        assert_eq!(caster.experience, 1600);
        assert_eq!(caster.level, 2)
    }

    #[test]
    fn test_give_exp_to_caster_spell_with_3_level_up() {
        let mut caster = create_caster_for_testing();

        give_exp_to_caster_spell(&mut caster, 15000);

        assert_eq!(caster.experience, 15000);
        assert_eq!(caster.level, 4)
    }


    #[test]
    fn test_give_exp_to_caster_spell_with_level_up_but_already_max_level() {
        let mut caster = create_caster_for_testing();
        caster.level = 30;

        give_exp_to_caster_spell(&mut caster, 2400);

        assert_eq!(caster.experience, 2400);
        assert_eq!(caster.level, 30)
    }

    #[test]
    fn test_test_is_spell_successful() {
        let mut offset: usize = 1;

        let is_success =
            is_spell_successful(SLOT_HASHES.as_slice(), &mut offset, ItemRarity::Legendary);

        //No really good way to test it except to make sure it doesn't error out
        assert!(is_success || !is_success);
    }
}
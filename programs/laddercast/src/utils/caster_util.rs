use crate::account::Caster;
use crate::utils::{ItemRarity, RandomValue};

pub const EXPERIENCE_REQUIRED_PER_LEVEL: [u64; 30] = [
    1508, 6001, 14956, 29851,
    52163, 83369, 124946, 178371,
    245121, 326674, 424507, 540097,
    674921, 830456, 1008179, 1209568,
    1436100, 1689252, 1970501, 2281324,
    2623198, 2997601, 3406010, 3849902,
    4330754, 4850043, 5409246, 6009841,
    6653305, 7341115
];

pub fn give_exp_to_caster_resources_burned(
    caster: &mut Caster,
    fire_burned: Option<u64>,
    earth_burned: Option<u64>,
    water_burned: Option<u64>,
) {
    //Give exp to user based on burned resources
    caster.experience +=
        fire_burned.unwrap_or(0) + earth_burned.unwrap_or(0) + water_burned.unwrap_or(0);


    //Since 0 based, we don't add +1 to level
    while caster.level < 30 && caster.experience >= EXPERIENCE_REQUIRED_PER_LEVEL[(caster.level - 1) as usize] {
        caster.level += 1;
    }
}

pub fn give_exp_to_caster_spell(caster: &mut Caster, value: u64) {
    caster.experience += value;

    while caster.level < 30 && caster.experience >= EXPERIENCE_REQUIRED_PER_LEVEL[(caster.level - 1) as usize] {
        caster.level += 1;
    }
}

pub fn is_spell_successful(slots: &[u8], offset: &mut usize, spell_book_rarity: ItemRarity) -> bool {
    //Spell have a chance of working, they won't always work
    let max_range = match spell_book_rarity {
        ItemRarity::Common => 8,
        ItemRarity::Rare => 6,
        ItemRarity::Epic => 4,
        ItemRarity::Legendary => 2
    };

    u8::random_within_range(slots, offset, 1, max_range) == 1
}
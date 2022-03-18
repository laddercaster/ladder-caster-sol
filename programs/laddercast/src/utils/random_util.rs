use std::convert::TryInto;
use num_traits::Num;
use strum::EnumCount;

//Random for Enums
pub trait RandomEnumValue {
    fn get_random(slots: &[u8], offset: &mut usize) -> Self;
    fn get_random_within_range(slots: &[u8], offset: &mut usize, min: u8, max: u8) -> Self;
}

impl<T> RandomEnumValue for T
    where
        T: EnumCount + strum::IntoEnumIterator,
{
    fn get_random(slots: &[u8], offset: &mut usize) -> Self {
        let random_value = u8::random_within_range(slots, offset, 0, T::COUNT as u8 - 1);

        T::iter().nth(random_value as usize).unwrap()
    }

    fn get_random_within_range(slots: &[u8], offset: &mut usize, min: u8, max: u8) -> Self {
        let random_value = u8::random_within_range(slots, offset, min, max);

        T::iter().nth(random_value as usize).unwrap()
    }
}

//Random based on slot hashes
pub fn get_random_bytes<const N: usize>(slot_hashes: &[u8], offset: &mut usize) -> [u8; N] {
    let out = &slot_hashes[*offset * 40 + 8..][..N];
    *offset += 1;
    out.try_into().expect("Out of data for random.")
}

pub trait FromNE<const N: usize> {
    fn from_ne_bytes(bytes: [u8; N]) -> Self;
}

pub trait RandomValue<const N: usize> {
    fn random(slot_hashes: &[u8], byte_offset: &mut usize) -> Self;
    fn random_within_range(slot_hashes: &[u8], byte_offset: &mut usize, min: Self, max: Self) -> Self;
}

impl<T, const N: usize> RandomValue<N> for T where T: FromNE<N> + Num + Copy {
    fn random(slot_hashes: &[u8], byte_offset: &mut usize) -> Self {
        T::from_ne_bytes(get_random_bytes(slot_hashes, byte_offset))
    }

    //Max is included as a potential number
    fn random_within_range(slot_hashes: &[u8], byte_offset: &mut usize, min: Self, max: Self) -> Self {
        Self::random(slot_hashes, byte_offset) % (max + Self::one() - min) + min
    }
}
macro_rules! impl_from_ne_prim {
    (all $(($ty:ty, $size:expr)),+) => {
        $(impl_from_ne_prim!($ty, $size);)+
    };
    ($ty:ty, $size:expr) => {
        impl FromNE<$size> for $ty{
            fn from_ne_bytes(bytes: [u8; $size]) -> Self{
                Self::from_ne_bytes(bytes)
            }
        }
    }
}
impl_from_ne_prim!(all (u8, 1), (u16, 2), (u32, 4), (u64, 8), (u128, 16));
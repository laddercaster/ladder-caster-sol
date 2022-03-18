use std::ops::AddAssign;
use num_traits::One;

pub trait UpdateOrInsert<U> {
    fn update_or_insert(&mut self, index: usize, value: U);
}

impl<U> UpdateOrInsert<U> for Vec<U> where U: AddAssign + Copy + One {
    fn update_or_insert(&mut self, index: usize, value: U) {
        match self.get_mut(index) {
            None => self.insert(index, value),
            Some(value_mut) => *value_mut += value
        }
    }
}
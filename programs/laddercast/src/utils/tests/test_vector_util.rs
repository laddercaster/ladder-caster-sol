#[cfg(test)]
mod test_internal_functions {
    use crate::utils::UpdateOrInsert;

    #[test]
    fn test_insert_vector() {
        let mut vector: Vec<u8> = vec![];

        vector.update_or_insert(0, 3);

        assert_eq!(*vector.get(0).unwrap(), 3);
    }

    #[test]
    fn test_update_vector() {
        let mut vector: Vec<u8> = vec![];

        vector.update_or_insert(0, 3);

        vector.update_or_insert(0, 4);

        assert_eq!(*vector.get(0).unwrap(), 7);
    }
}
use std::str::FromStr;

use anchor_lang::prelude::Pubkey;

#[derive(Clone)]
pub struct MetaplexTokenMetadata;

impl anchor_lang::Id for MetaplexTokenMetadata {
    fn id() -> Pubkey {
        Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap()
    }
}
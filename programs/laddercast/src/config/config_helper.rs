// use std::str::FromStr;
//
// use crate::config::{dev_config, local_config, prod_config};
// use crate::Pubkey;
//
// pub const LADA_MINT_PUB_KEY: usize = 0;
// pub const LADA_ACCOUNT_PUB_KEY: usize = 1;
// pub const GAME_CREATOR_AUTHORITY_PUB_KEY: usize = 2;
// pub const NFT_CREATOR_PUB_KEY: usize = 3;
// pub const NFT_CREATOR_SPLITTER_PUB_KEY: usize = 4;
//
// fn get_index_based_on_key(key: &str) -> usize {
//     match key {
//         "LADA_MINT_PUB_KEY" => LADA_MINT_PUB_KEY,
//         "LADA_ACCOUNT_PUB_KEY" => LADA_ACCOUNT_PUB_KEY,
//         "GAME_CREATOR_AUTHORITY_PUB_KEY" => GAME_CREATOR_AUTHORITY_PUB_KEY,
//         "NFT_CREATOR_PUB_KEY" => NFT_CREATOR_PUB_KEY,
//         "NFT_CREATOR_SPLITTER_PUB_KEY" => NFT_CREATOR_SPLITTER_PUB_KEY,
//         _ => 999 as usize
//     }
// }
//
// pub fn get_config_value(key: &str) -> String {
//     let index = get_index_based_on_key(key);
//
//     #[cfg(feature = "dev")]
//         {
//             dev_config::CONFIG[index].to_string();
//         }
//     #[cfg(feature = "prod")]
//         {
//             prod_config::CONFIG[index].to_string();
//         }
//
//     local_config::CONFIG[index].to_string()
// }
//
// pub fn get_config_value_as_pubkey(key: &str) -> Pubkey {
//     let index = get_index_based_on_key(key);
//
//     #[cfg(feature = "dev")]
//         {
//             Pubkey::from_str(dev_config::CONFIG[index]).unwrap();
//         }
//     #[cfg(feature = "prod")]
//         {
//             Pubkey::from_str(prod_config::CONFIG[index]).unwrap();
//         }
//
//     Pubkey::from_str(local_config::CONFIG[index]).unwrap()
// }

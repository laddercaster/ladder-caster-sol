use anchor_lang::prelude::*;

use crate::Tile;

#[event]
pub struct NewTurn {
    pub turn: u32,
    pub tile_map: [[Option<Tile>; 3]; 30],
}

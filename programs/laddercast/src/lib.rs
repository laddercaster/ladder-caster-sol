use anchor_lang::prelude::*;

use instructions::*;

use crate::utils::{ItemFeature, ItemType};

mod account;
mod config;
mod error;
mod event;
mod instructions;
mod utils;

declare_id!("LCThBz55Ma7hcueUQA3iiofBhvidQHfNLxxwfLsycxb");

#[program]
pub mod laddercast {
    use super::*;

    //********************************************
    //Initialization functions
    //********************************************
    pub fn init_game(
        ctx: Context<InitGame>,
        turn_info: GameTurnInfo,
        merkle_root_nft: [u8; 32],
    ) -> ProgramResult {
        init_game::init_game(ctx, turn_info, merkle_root_nft)
    }

    pub fn init_player(ctx: Context<InitPlayer>) -> ProgramResult {
        init_player::init_player(ctx)
    }

    pub fn init_caster(ctx: Context<InitCaster>) -> ProgramResult {
        init_caster::init_caster(ctx)
    }

    //********************************************
    //Turn based functions
    //********************************************

    pub fn caster_commit_loot(ctx: Context<Loot>) -> ProgramResult {
        caster_commit_loot::caster_commit_loot(ctx)
    }

    pub fn caster_commit_move(ctx: Context<Move>, lvl: u8, clm: u8) -> ProgramResult {
        caster_commit_move::caster_commit_move(ctx, lvl, clm)
    }

    pub fn caster_commit_craft(ctx: Context<Craft>) -> ProgramResult {
        caster_commit_craft::caster_commit_craft(ctx)
    }

    pub fn caster_commit_spell(ctx: Context<Spell>) -> ProgramResult {
        caster_commit_spell::caster_commit_spell(ctx)
    }

    pub fn caster_redeem_action<'info>(
        ctx: Context<'_, '_, '_, 'info, Action<'info>>,
    ) -> ProgramResult {
        caster_turn_redeem::caster_redeem_action(ctx)
    }

    pub fn crank(ctx: Context<Crank>) -> ProgramResult {
        crank::crank(ctx)
    }

    //********************************************
    //Non-turn based functions
    //********************************************

    pub fn equip_item(ctx: Context<EquipUnequipItem>) -> ProgramResult {
        equipment::equip_item(ctx)
    }

    pub fn unequip_item(ctx: Context<EquipUnequipItem>) -> ProgramResult {
        equipment::unequip_item(ctx)
    }

    pub fn open_chest(ctx: Context<OpenChest>) -> ProgramResult {
        open_chest::open_chest(ctx)
    }

    pub fn manual_resource_burn(
        ctx: Context<ManualResourceBurn>,
        resource_type: ItemFeature,
        amount_to_burn: u64,
    ) -> ProgramResult {
        manual_resource_burn::manual_resource_burn(ctx, resource_type, amount_to_burn)
    }

    //********************************************
    //Functions to mint / burn into NFTs
    //********************************************
    pub fn mint_item(
        ctx: Context<MintItem>,
        nft_uri: String,
        merkle_proof: Vec<[u8; 32]>,
    ) -> ProgramResult {
        mint_nft::mint_item(ctx, nft_uri, merkle_proof)
    }

    pub fn mint_caster(
        ctx: Context<MintCaster>,
        nft_uri: String,
        merkle_proof: Vec<[u8; 32]>,
    ) -> ProgramResult {
        mint_nft::mint_caster(ctx, nft_uri, merkle_proof)
    }

    pub fn redeem_item(ctx: Context<RedeemItem>) -> ProgramResult {
        burn_nft::redeem_item(ctx)
    }

    pub fn redeem_caster(ctx: Context<RedeemCaster>) -> ProgramResult {
        burn_nft::redeem_caster(ctx)
    }

    pub fn update_merkle_root(
        ctx: Context<UpdateMerkleRoot>,
        merkle_root_nft: [u8; 32],
    ) -> ProgramResult {
        update_merkle_root::update_merkle_root(ctx, merkle_root_nft)
    }

    //********************************************
    //Debug functions only for testing
    //********************************************
    pub fn give_resources(ctx: Context<GiveResources>, amount: u64) -> ProgramResult {
        test_helper::give_resources(ctx, amount)
    }

    pub fn give_lada(ctx: Context<GiveLada>, amount: u64) -> ProgramResult {
        test_helper::give_lada(ctx, amount)
    }

    pub fn give_item(ctx: Context<GiveItems>, item_type: ItemType, level: u8) -> ProgramResult {
        test_helper::give_item(ctx, item_type, level)
    }

    pub fn change_tile(
        ctx: Context<ChangeTile>,
        tile_type: TileType,
        lvl: u8,
        col: u8,
    ) -> ProgramResult {
        test_helper::change_tile(ctx, tile_type, lvl, col)
    }
}

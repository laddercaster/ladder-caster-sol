use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("Invalid Function")]
    InvalidFunction,

    #[msg("Turn isn't over yet!")]
    PrematureCrankPull,

    #[msg("Invalid Move.")]
    InvalidMove,

    #[msg("Pending turn needs to be redeemed first.")]
    PendingTurn,

    #[msg("No turn to redeem.")]
    EmptyTurnCommit,

    #[msg("Turn needs to advance before you can redeem!")]
    SameTurnRedeem,

    #[msg("Not enough resources for action.")]
    PlayerIsPoor,

    #[msg("You already did this action, wait for next turn.")]
    ActionAlreadyDone,

    #[msg("Item already equipped by another caster.")]
    ItemAlreadyInUse,

    #[msg("Item does not exist.")]
    ItemNotExists,

    #[msg("Already have an item of that type equipped.")]
    ItemTypeAlreadyEquipped,

    #[msg("Item level is higher than your caster's level.")]
    ItemLevelTooHigh,

    #[msg("Item can't be equipped / unequipped.")]
    InvalidEquipItemType,

    #[msg("Item can't be used for crafting.")]
    InvalidItemType,

    #[msg("This is not a crafting tile.")]
    NotCraftingTile,

    #[msg("Item is not a chest.")]
    ItemIsNotAChest,

    #[msg("Invalid resource for manual burn.")]
    InvalidResourceTypeForBurn,

    #[msg("Invalid token amount for burn.")]
    InvalidTokenAmount,

    #[msg("You can't equip or unequip while you have a pending turn.")]
    NoEquipUnequipOnPendingTurn,

    #[msg("Invalid location.")]
    TileNotExists,

    #[msg("Invalid tile type for looting.")]
    InvalidTileForLooting,

    #[msg("To redeem a spell you need to provide spell account.")]
    SpellAccountMissing,

    #[msg("Spell key mismatch.")]
    SpellKeyMismatch,

    #[msg("Only super admin can create new game.")]
    NotSuperAdmin,

    #[msg("Caster can't be minted if it has equipped items.")]
    InvalidCasterMintEquipped,

    #[msg("Caster can't be minted if it has a pending turn.")]
    InvalidCasterMintPendingTurn,

    #[msg("Invalid cost type for spell.")]
    InvalidSpellCost,

    #[msg("Provided spell book for redeem is null.")]
    ProvidedSpellBookIsNull,

    #[msg("Invalid game provided.")]
    InvalidGame,

    #[msg("Invalid lada mint provided.")]
    InvalidLadaMint,

    #[msg("Invalid lada token for game provided.")]
    InvalidLadaTokenGameAccount,

    #[msg("Invalid item for merkle proof.")]
    InvalidItemForMerkleProof,

    #[msg("Invalid uri passed for NFT minting.")]
    InvalidNFTURI,

    #[msg("Item can't be equipped to mint it.")]
    ItemCantBeMintIfEquipped,
}

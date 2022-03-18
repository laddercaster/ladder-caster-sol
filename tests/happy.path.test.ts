import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Laddercast } from "../target/types/laddercast";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_SLOT_HASHES_PUBKEY,
} from "@solana/web3.js";
import assert from "assert";
import { GameTurnInfo } from "./interface";
import { MerkleTree } from "merkletreejs";
import keccak256 from "keccak256";

describe("laddercast", () => {
  //Connection
  let connection;

  //Programs
  const program = anchor.workspace.Laddercast as Program<Laddercast>;

  //Game related
  const gameAuthority = anchor.web3.Keypair.generate();

  let gameAccount: anchor.web3.Keypair;
  let gameLADATokenAccount: anchor.web3.PublicKey;

  //Player
  const someGuy = anchor.web3.Keypair.generate();

  let someGuyLADATokenAccount: anchor.web3.PublicKey;

  //Lada
  const mintAuthority = anchor.web3.Keypair.generate();

  let ladaMint: Token;

  //Resources
  const mintResource1 = anchor.web3.Keypair.generate();
  const mintResource2 = anchor.web3.Keypair.generate();
  const mintResource3 = anchor.web3.Keypair.generate();

  let ATAResource1: anchor.web3.PublicKey;
  let ATAResource2: anchor.web3.PublicKey;
  let ATAResource3: anchor.web3.PublicKey;

  //Caster
  let caster: anchor.web3.Keypair;

  //Items
  let spellBook: anchor.web3.Keypair;

  //Constants
  const DECIMALS = 1_000_000_000;

  const gameTurnInfo = {
    turn: 1,
    turnDelay: 1200,
    lastCrankSeconds: new anchor.BN(0),
    lastTileSpawn: 0,
    tileSpawnDelay: 20,
  } as GameTurnInfo;

  async function getTokenAccountBalance(pubkey: anchor.web3.PublicKey) {
    return Number.parseInt(
      (await connection.getTokenAccountBalance(pubkey as PublicKey)).value
        .amount
    );
  }

  async function getTokenAccountBalances() {
    return [
      await getTokenAccountBalance(ATAResource1),
      await getTokenAccountBalance(ATAResource2),
      await getTokenAccountBalance(ATAResource3),
    ];
  }

  async function getFeatureForTile(lvl, column) {
    return Object.keys(
      (await program.account.game.fetch(gameAccount.publicKey)).map[lvl][column]
        .tileType
    )[0];
  }

  it("Initializes game", async () => {
    const con = anchor.Provider.local();

    anchor.setProvider(con);

    await con.connection.confirmTransaction(
      await con.connection.requestAirdrop(mintAuthority.publicKey, 10000000000),
      "confirmed"
    );

    await con.connection.confirmTransaction(
      await con.connection.requestAirdrop(gameAuthority.publicKey, 10000000000),
      "confirmed"
    );

    await con.connection.confirmTransaction(
      await con.connection.requestAirdrop(someGuy.publicKey, 10000000000),
      "confirmed"
    );

    connection = con.connection;

    gameAccount = anchor.web3.Keypair.generate();

    ladaMint = await Token.createMint(
      con.connection,
      mintAuthority,
      mintAuthority.publicKey,
      null,
      9,
      TOKEN_PROGRAM_ID
    );

    const [gameSigner] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("game_signer")],
      program.programId
    );

    gameLADATokenAccount = await ladaMint.createAccount(gameSigner);
    someGuyLADATokenAccount = await ladaMint.createAccount(someGuy.publicKey);

    await ladaMint.mintTo(
      gameLADATokenAccount,
      mintAuthority.publicKey,
      [mintAuthority],
      3000 * DECIMALS
    );
    await ladaMint.mintTo(
      someGuyLADATokenAccount,
      mintAuthority.publicKey,
      [mintAuthority],
      1000 * DECIMALS
    );

    ATAResource1 = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mintResource1.publicKey,
      someGuy.publicKey
    );
    ATAResource2 = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mintResource2.publicKey,
      someGuy.publicKey
    );
    ATAResource3 = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mintResource3.publicKey,
      someGuy.publicKey
    );

    const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("turn_data"),
        gameAccount.publicKey.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn))),
      ],
      program.programId
    );

    //Create random merkle tree, as we don't need a valid one (no mint testing here)
    const tree = new MerkleTree(["random1", "random2"], keccak256, {
      sortPairs: true,
      hashLeaves: true,
    });
    const root = tree.getRoot();

    await program.rpc.initGame(gameTurnInfo, [...root], {
      accounts: {
        authority: gameAuthority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: "SysvarRent111111111111111111111111111111111",
        gameAccount: gameAccount.publicKey,
        gameSigner: gameSigner,
        gameTurnData: gameTurnData,
        slots: "SysvarS1otHashes111111111111111111111111111",
        resource1Mint: mintResource1.publicKey,
        resource2Mint: mintResource2.publicKey,
        resource3Mint: mintResource3.publicKey,
        ladaMint: ladaMint.publicKey,
        ladaTokenAccount: gameLADATokenAccount,
      },
      signers: [
        gameAuthority,
        gameAccount,
        mintResource1,
        mintResource2,
        mintResource3,
      ],
    });

    //Assertions
    let createdGame = await program.account.game.fetch(gameAccount.publicKey);

    assert.equal(createdGame.map[0][0] !== null, true);
    assert.equal(createdGame.map[0][1] !== null, true);
    assert.equal(createdGame.map[0][2] !== null, true);

    for (let i = 1; i < createdGame.map; i++) {
      for (let j = 0; j < createdGame.map[j]; j++) {
        assert.equal(createdGame.map[0][2] === null, true);
      }
    }

    assert.deepEqual(createdGame.turnInfo, gameTurnInfo);
    assert.equal(createdGame.lastTurnAdded, 1);
  });

  it("Initializes player", async () => {
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );

    await program.rpc.initPlayer({
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        authority: someGuy.publicKey,
        game: gameAccount.publicKey,
        playerAccount: playerAccount,
      },
      signers: [someGuy],
    });

    const createdPlayer = await program.account.player.fetch(playerAccount);

    assert.deepEqual(createdPlayer.game, gameAccount.publicKey);
  });

  it("Initializes caster", async () => {
    caster = anchor.web3.Keypair.generate();
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );

    const playerLadaBalanceBefore = (
      await ladaMint.getAccountInfo(someGuyLADATokenAccount)
    ).amount.toNumber();
    const gameLadaBalanceBefore = (
      await ladaMint.getAccountInfo(gameLADATokenAccount)
    ).amount.toNumber();

    await program.rpc.initCaster({
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        authority: someGuy.publicKey,
        game: gameAccount.publicKey,
        player: playerAccount,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        ladaMint: ladaMint.publicKey,
        caster: caster.publicKey,
        ladaTokenAccount: someGuyLADATokenAccount,
      },
      signers: [someGuy, caster],
    });

    const createdCaster = await program.account.caster.fetch(caster.publicKey);

    assert.deepEqual(createdCaster.owner, playerAccount);
    assert.equal(createdCaster.version, 1);
    assert.equal(createdCaster.level, 1);
    assert.equal(createdCaster.experience, 0);
    assert.equal(createdCaster.turnCommit, null);

    assert.equal(
      createdCaster.modifiers.tileColumn <= 2 &&
        createdCaster.modifiers.tileColumn >= 0,
      true
    );

    delete createdCaster.modifiers.tileColumn; //Since random

    assert.deepEqual(createdCaster.modifiers, {
      tileLevel: 0,
      head: null,
      robe: null,
      staff: null,
      spellBook: null,
    });

    assert.equal(
      (
        await ladaMint.getAccountInfo(someGuyLADATokenAccount)
      ).amount.toNumber(),
      playerLadaBalanceBefore - 1_000 * DECIMALS
    );
    //Should stay the same number as we're burning instead of transferring back
    assert.equal(
      (await ladaMint.getAccountInfo(gameLADATokenAccount)).amount.toNumber(),
      gameLadaBalanceBefore
    );
  });

  it("can open chest", async () => {
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );

    const chest = { chest: { tier: 1 } };
    const chestItem = anchor.web3.Keypair.generate();

    await program.rpc.giveItem(chest, new anchor.BN(2), {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        item: chestItem.publicKey,
      },
      signers: [someGuy, chestItem],
    });

    const newItems = [
      anchor.web3.Keypair.generate(),
      anchor.web3.Keypair.generate(),
      anchor.web3.Keypair.generate(),
    ];

    await program.rpc.openChest({
      accounts: {
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        slots: "SysvarS1otHashes111111111111111111111111111",
        chest: chestItem.publicKey,
        item1: newItems[0].publicKey,
        item2: newItems[1].publicKey,
        item3: newItems[2].publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [someGuy, newItems[0], newItems[1], newItems[2]],
    });

    const openedChest = await program.account.item.fetchNullable(
      chestItem.publicKey
    );

    //Is null because we close the account
    assert.equal(openedChest, null);

    for (let i = 0; i < newItems.length; i++) {
      let newItem = await program.account.item.fetch(newItems[i].publicKey);

      assert.equal(newItem.level <= 2 && newItem.level >= 1, true);
      assert.deepEqual(newItem.game, gameAccount.publicKey);
      assert.deepEqual(newItem.owner, playerAccount);
      assert.equal(newItem.equippedOwner, null);
    }
  });

  it("equip", async () => {
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );

    const equipment = {
      equipment: {
        feature: { power: {} },
        rarity: { common: {} },
        equipmentType: { head: {} },
        value: 1,
      },
    };

    const equipmentItem = anchor.web3.Keypair.generate();

    await program.rpc.giveItem(equipment, new anchor.BN(1), {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        item: equipmentItem.publicKey,
      },
      signers: [someGuy, equipmentItem],
    });

    await program.rpc.equipItem({
      accounts: {
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        item: equipmentItem.publicKey,
      },
      signers: [someGuy],
    });

    let equippedItem = await program.account.item.fetch(
      equipmentItem.publicKey
    );
    assert.deepEqual(equippedItem.equippedOwner, caster.publicKey);

    await program.rpc.unequipItem({
      accounts: {
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        item: equipmentItem.publicKey,
      },
      signers: [someGuy],
    });

    equippedItem = await program.account.item.fetch(equipmentItem.publicKey);
    assert.deepEqual(equippedItem.equippedOwner, null);

    const spellbook = {
      spellBook: {
        spell: { fire: {} },
        costFeature: { fire: {} },
        rarity: { common: {} },
        cost: 1,
        value: 1,
      },
    };
    const spellbookItem = anchor.web3.Keypair.generate();

    await program.rpc.giveItem(spellbook, new anchor.BN(1), {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        item: spellbookItem.publicKey,
      },
      signers: [someGuy, spellbookItem],
    });

    await program.rpc.equipItem({
      accounts: {
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        item: spellbookItem.publicKey,
      },
      signers: [someGuy],
    });

    let equippedSpellBook = await program.account.item.fetch(
      spellbookItem.publicKey
    );
    assert.deepEqual(equippedSpellBook.equippedOwner, caster.publicKey);

    await program.rpc.unequipItem({
      accounts: {
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        item: spellbookItem.publicKey,
      },
      signers: [someGuy],
    });

    equippedSpellBook = await program.account.item.fetch(
      spellbookItem.publicKey
    );
    assert.deepEqual(equippedSpellBook.equippedOwner, null);
  });

  it("commit spell", async () => {
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );
    const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("turn_data"),
        gameAccount.publicKey.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn))),
      ],
      program.programId
    );

    const [gameSigner] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("game_signer")],
      program.programId
    );

    await program.rpc.giveResources(new anchor.BN(500), {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        authority: someGuy.publicKey,
        gameSigner: gameSigner,
        game: gameAccount.publicKey,
        player: playerAccount,
        resource1MintAccount: mintResource1.publicKey,
        resource2MintAccount: mintResource2.publicKey,
        resource3MintAccount: mintResource3.publicKey,
        resource1TokenAccount: ATAResource1,
        resource2TokenAccount: ATAResource2,
        resource3TokenAccount: ATAResource3,
      },
      signers: [someGuy],
    });

    const spellbook = {
      spellBook: {
        spell: { fire: {} },
        costFeature: { fire: {} },
        rarity: { common: {} },
        cost: 3,
        value: 1,
      },
    };
    spellBook = anchor.web3.Keypair.generate();

    await program.rpc.giveItem(spellbook, new anchor.BN(1), {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        item: spellBook.publicKey,
      },
      signers: [someGuy, spellBook],
    });

    await program.rpc.equipItem({
      accounts: {
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        item: spellBook.publicKey,
      },
      signers: [someGuy],
    });

    const turnResource1Burned = (
      await program.account.turnData.fetch(gameTurnData)
    ).resource1Burned;

    const someGuyFireResource = await getTokenAccountBalance(ATAResource1);

    await program.rpc.casterCommitSpell({
      accounts: {
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        authority: someGuy.publicKey,
        game: gameAccount.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        resource1MintAccount: mintResource1.publicKey,
        resource2MintAccount: mintResource2.publicKey,
        resource3MintAccount: mintResource3.publicKey,
        resource1TokenAccount: ATAResource1,
        resource2TokenAccount: ATAResource2,
        resource3TokenAccount: ATAResource3,
        spellbook: spellBook.publicKey,
        gameTurnData: gameTurnData,
      },
      signers: [someGuy],
    });

    //Assert game turn data resource burned higher
    assert.equal(
      (
        await program.account.turnData.fetch(gameTurnData)
      ).resource1Burned.toNumber(),
      turnResource1Burned.add(new anchor.BN(3)).toNumber()
    );

    //Assert player has burned some resources
    assert.equal(
      await getTokenAccountBalance(ATAResource1),
      someGuyFireResource - 3
    );

    //Assert turn commit created (deep equal) (actions & order actions)
    const fetchedCaster = await program.account.caster.fetch(caster.publicKey);

    assert.equal(fetchedCaster.turnCommit.resourcesBurned[0].toNumber(), 3);
    assert.equal(fetchedCaster.turnCommit.resourcesBurned[1].toNumber(), 0);
    assert.equal(fetchedCaster.turnCommit.resourcesBurned[2].toNumber(), 0);

    delete fetchedCaster.turnCommit.resourcesBurned;

    assert.deepEqual(fetchedCaster.turnCommit, {
      turn: 1,
      actions: {
        loot: false,
        spell: {
          isExtraLevelBonus: false,
        },
        mv: null,
        crafting: null,
        actionOrder: [0, 1, 0, 0],
      },
    });
  });

  it("commit craft", async () => {
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );
    const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("turn_data"),
        gameAccount.publicKey.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn))),
      ],
      program.programId
    );

    const equipment = {
      equipment: {
        feature: { power: {} },
        rarity: { common: {} },
        equipmentType: { head: {} },
        value: 1,
      },
    };
    const item1 = anchor.web3.Keypair.generate();

    await program.rpc.giveItem(equipment, new anchor.BN(1), {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        item: item1.publicKey,
      },
      signers: [someGuy, item1],
    });

    const item2 = anchor.web3.Keypair.generate();

    await program.rpc.giveItem(equipment, new anchor.BN(1), {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        item: item2.publicKey,
      },
      signers: [someGuy, item2],
    });

    const item3 = anchor.web3.Keypair.generate();

    await program.rpc.giveItem(equipment, new anchor.BN(1), {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        game: gameAccount.publicKey,
        authority: someGuy.publicKey,
        player: playerAccount,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        item: item3.publicKey,
      },
      signers: [someGuy, item3],
    });

    const currentModifiers = (await program.account.caster.all())[0].account
      .modifiers;

    //Sets the tile as a crafting tile
    await program.rpc.changeTile(
      { crafting: {} },
      currentModifiers.tileLevel,
      currentModifiers.tileColumn,
      {
        accounts: {
          systemProgram: anchor.web3.SystemProgram.programId,
          game: gameAccount.publicKey,
        },
      }
    );

    let preGameTurnDataResourcesBurned = await program.account.turnData.fetch(
      gameTurnData
    );
    let preTurnCommitResourceBurned = (
      await program.account.caster.fetch(caster.publicKey)
    ).turnCommit.resourcesBurned;
    let preSomeGuyResources = await getTokenAccountBalances();

    await program.rpc.casterCommitCraft({
      accounts: {
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        authority: someGuy.publicKey,
        game: gameAccount.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        resource1MintAccount: mintResource1.publicKey,
        resource2MintAccount: mintResource2.publicKey,
        resource3MintAccount: mintResource3.publicKey,
        resource1TokenAccount: ATAResource1,
        resource2TokenAccount: ATAResource2,
        resource3TokenAccount: ATAResource3,
        item1: item1.publicKey,
        item2: item2.publicKey,
        item3: item3.publicKey,
        gameTurnData: gameTurnData,
      },
      signers: [someGuy],
    });

    let postGameTurnDataResourcesBurned = await program.account.turnData.fetch(
      gameTurnData
    );
    let postTurnCommitResourceBurned = (
      await program.account.caster.fetch(caster.publicKey)
    ).turnCommit.resourcesBurned;
    let postSomeGuyResources = await getTokenAccountBalances();

    const multiplier =
      (await program.account.caster.fetch(caster.publicKey)).modifiers
        .tileLevel + 1; //0 based so + 1

    //Assert game turn data resource burned higher
    assert.equal(
      postGameTurnDataResourcesBurned.resource1Burned.toNumber(),
      preGameTurnDataResourcesBurned.resource1Burned
        .add(new anchor.BN(5 * multiplier))
        .toNumber()
    );
    assert.equal(
      postGameTurnDataResourcesBurned.resource2Burned.toNumber(),
      preGameTurnDataResourcesBurned.resource2Burned
        .add(new anchor.BN(5 * multiplier))
        .toNumber()
    );
    assert.equal(
      postGameTurnDataResourcesBurned.resource3Burned.toNumber(),
      preGameTurnDataResourcesBurned.resource3Burned
        .add(new anchor.BN(5 * multiplier))
        .toNumber()
    );

    //Assert player has burned some resources
    assert.equal(
      preSomeGuyResources[0] - 5 * multiplier,
      postSomeGuyResources[0]
    );
    assert.equal(
      preSomeGuyResources[1] - 5 * multiplier,
      postSomeGuyResources[1]
    );
    assert.equal(
      preSomeGuyResources[2] - 5 * multiplier,
      postSomeGuyResources[2]
    );

    //Assert items are deleted
    assert.equal(
      await program.account.item.fetchNullable(item1.publicKey),
      null
    );
    assert.equal(
      await program.account.item.fetchNullable(item2.publicKey),
      null
    );
    assert.equal(
      await program.account.item.fetchNullable(item3.publicKey),
      null
    );

    //Assert turn commit created (deep equal) (actions & order actions)
    const fetchedCaster = await program.account.caster.fetch(caster.publicKey);

    assert.equal(
      preTurnCommitResourceBurned[0].toNumber() + 5 * multiplier,
      postTurnCommitResourceBurned[0]
    );
    assert.equal(
      preTurnCommitResourceBurned[1].toNumber() + 5 * multiplier,
      postTurnCommitResourceBurned[1]
    );
    assert.equal(
      preTurnCommitResourceBurned[2].toNumber() + 5 * multiplier,
      postTurnCommitResourceBurned[2]
    );

    delete fetchedCaster.turnCommit.resourcesBurned;

    assert.deepEqual(fetchedCaster.turnCommit, {
      turn: 1,
      actions: {
        loot: false,
        spell: {
          isExtraLevelBonus: false,
        },
        mv: null,
        crafting: {
          minLevel: 1,
          minRarity: { common: {} },
          maxRarity: { epic: {} },
        },
        actionOrder: [0, 1, 0, 2],
      },
    });
  });

  it("commit move", async () => {
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );
    const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("turn_data"),
        gameAccount.publicKey.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn))),
      ],
      program.programId
    );

    let preGameTurnDataResourcesBurned = await program.account.turnData.fetch(
      gameTurnData
    );
    let preTurnCommitResourceBurned = (
      await program.account.caster.fetch(caster.publicKey)
    ).turnCommit.resourcesBurned;
    let preSomeGuyResources = await getTokenAccountBalances();

    await program.rpc.casterCommitMove(0, 1, {
      accounts: {
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        authority: someGuy.publicKey,
        game: gameAccount.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        resource1MintAccount: mintResource1.publicKey,
        resource2MintAccount: mintResource2.publicKey,
        resource3MintAccount: mintResource3.publicKey,
        resource1TokenAccount: ATAResource1,
        resource2TokenAccount: ATAResource2,
        resource3TokenAccount: ATAResource3,
        gameTurnData: gameTurnData,
      },
      signers: [someGuy],
    });

    await new Promise((f) => setTimeout(f, 10000)); //Give time to sol to update the accounts

    let postGameTurnDataResourcesBurned = await program.account.turnData.fetch(
      gameTurnData
    );
    let postTurnCommitResourceBurned = (
      await program.account.caster.fetch(caster.publicKey)
    ).turnCommit.resourcesBurned;
    let postSomeGuyResources = await getTokenAccountBalances();

    let resourceTypeWhereMoved = await getFeatureForTile(0, 1);

    //Assert game turn data resource burned higher
    const resourceTypeBurned =
      resourceTypeWhereMoved == "fire"
        ? "resource1Burned"
        : resourceTypeWhereMoved == "water"
        ? "resource2Burned"
        : "resource3Burned";

    assert.equal(
      postGameTurnDataResourcesBurned[resourceTypeBurned].toNumber(),
      preGameTurnDataResourcesBurned[resourceTypeBurned]
        .add(new anchor.BN(10))
        .toNumber()
    );

    //Assert player has burned some resources
    const resourceTypeBurnedIndex =
      resourceTypeWhereMoved == "fire"
        ? 0
        : resourceTypeWhereMoved == "water"
        ? 1
        : 2;

    assert.equal(
      preSomeGuyResources[resourceTypeBurnedIndex] - 10,
      postSomeGuyResources[resourceTypeBurnedIndex]
    );

    //Assert turn commit created (deep equal) (actions & order actions)
    const fetchedCaster = await program.account.caster.fetch(caster.publicKey);

    assert.equal(
      preTurnCommitResourceBurned[resourceTypeBurnedIndex].toNumber() + 10,
      postTurnCommitResourceBurned[resourceTypeBurnedIndex]
    );

    delete fetchedCaster.turnCommit.resourcesBurned;

    assert.deepEqual(fetchedCaster.turnCommit, {
      turn: 1,
      actions: {
        loot: false,
        spell: {
          isExtraLevelBonus: false,
        },
        mv: [0, 1],
        crafting: {
          minLevel: 1,
          minRarity: { common: {} },
          maxRarity: { epic: {} },
        },
        actionOrder: [0, 1, 3, 2],
      },
    });
  });

  it("commit loot", async () => {
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );
    const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("turn_data"),
        gameAccount.publicKey.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn))),
      ],
      program.programId
    );

    //Sets the tile as a crafting tile
    await program.rpc.changeTile({ fire: {} }, 0, 1, {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        game: gameAccount.publicKey,
      },
    });

    await program.rpc.casterCommitLoot({
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        authority: someGuy.publicKey,
        game: gameAccount.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        gameTurnData: gameTurnData,
      },
      signers: [someGuy],
    });

    //Assert turn commit created (deep equal) (actions & order actions)
    const fetchedCaster = await program.account.caster.fetch(caster.publicKey);

    delete fetchedCaster.turnCommit.resourcesBurned;

    assert.deepEqual(fetchedCaster.turnCommit, {
      turn: 1,
      actions: {
        loot: true,
        spell: {
          isExtraLevelBonus: false,
        },
        mv: [0, 1],
        crafting: {
          minLevel: 1,
          minRarity: { common: {} },
          maxRarity: { epic: {} },
        },
        actionOrder: [4, 1, 3, 2],
      },
    });
  });

  it("can manual_resource_burn", async () => {
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );

    const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("turn_data"),
        gameAccount.publicKey.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn))),
      ],
      program.programId
    );

    let preGameTurnDataResourcesBurned = await program.account.turnData.fetch(
      gameTurnData
    );
    let preTurnCommitResourceBurned = (
      await program.account.caster.fetch(caster.publicKey)
    ).turnCommit.resourcesBurned;
    let preSomeGuyResources = await getTokenAccountBalances();

    await program.rpc.manualResourceBurn({ water: {} }, new anchor.BN(10), {
      accounts: {
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        authority: someGuy.publicKey,
        game: gameAccount.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        resource1MintAccount: mintResource1.publicKey,
        resource2MintAccount: mintResource2.publicKey,
        resource3MintAccount: mintResource3.publicKey,
        resource1TokenAccount: ATAResource1,
        resource2TokenAccount: ATAResource2,
        resource3TokenAccount: ATAResource3,
        gameTurnData: gameTurnData,
      },
      signers: [someGuy],
    });

    let postGameTurnDataResourcesBurned = await program.account.turnData.fetch(
      gameTurnData
    );
    let postTurnCommitResourceBurned = (
      await program.account.caster.fetch(caster.publicKey)
    ).turnCommit.resourcesBurned;
    let postSomeGuyResources = await getTokenAccountBalances();

    //Assert game turn data resource burned higher
    assert.equal(
      postGameTurnDataResourcesBurned.resource2Burned.toNumber(),
      preGameTurnDataResourcesBurned.resource2Burned
        .add(new anchor.BN(10))
        .toNumber()
    );

    //Assert player has burned some resources
    assert.equal(preSomeGuyResources[1] - 10, postSomeGuyResources[1]);

    //Assert turn commit burned resources for caster
    assert.equal(
      preTurnCommitResourceBurned[1].toNumber() + 10,
      postTurnCommitResourceBurned[1]
    );
  });

  it("crank", async () => {
    const [currentGameTurnData] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("turn_data"),
          gameAccount.publicKey.toBuffer(),
          Buffer.from(
            anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn))
          ),
        ],
        program.programId
      );

    const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("turn_data"),
        gameAccount.publicKey.toBuffer(),
        Buffer.from(
          anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn + 1))
        ),
      ],
      program.programId
    );

    let oldFetchedGame = await program.account.game.fetch(
      gameAccount.publicKey
    );

    await program.rpc.crank({
      accounts: {
        authority: someGuy.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        gameAccount: gameAccount.publicKey,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        gameTurnData: gameTurnData,
        currentGameTurnData: currentGameTurnData,
      },
      signers: [someGuy],
    });

    let fetchedGame = await program.account.game.fetch(gameAccount.publicKey);

    assert.equal(fetchedGame.turnInfo.turn, 2);
    assert.equal(
      fetchedGame.turnInfo.lastCrankSeconds >
        oldFetchedGame.turnInfo.lastCrankSeconds,
      true
    );
    assert.equal(fetchedGame.lastTurnAdded, 2);
  });

  it("redeem", async () => {
    const [playerAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [gameAccount.publicKey.toBuffer(), someGuy.publicKey.toBuffer()],
      program.programId
    );
    const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("turn_data"),
        gameAccount.publicKey.toBuffer(),
        Buffer.from(
          //we don't add 1 to the turn because we want the game turn data of the turn BEFORE the crank
          anchor.utils.bytes.utf8.encode(
            String(
              (
                await program.account.caster.fetch(caster.publicKey)
              ).turnCommit.turn
            )
          )
        ),
      ],
      program.programId
    );
    const [gameSigner] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("game_signer")],
      program.programId
    );

    const item = anchor.web3.Keypair.generate();
    const empty = anchor.web3.Keypair.generate();

    const oldFetchedCaster = await program.account.caster.fetch(
      caster.publicKey
    );
    const itemPreCreation = await program.account.item.fetchNullable(
      item.publicKey
    );
    const playerLadaBalanceBefore = await getTokenAccountBalance(
      someGuyLADATokenAccount
    );
    const gameLadaBalanceBefore = await getTokenAccountBalance(
      gameLADATokenAccount
    );
    const preSomeGuyResources = await getTokenAccountBalances();

    await program.rpc.casterRedeemAction({
      accounts: {
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        authority: someGuy.publicKey,
        game: gameAccount.publicKey,
        player: playerAccount,
        caster: caster.publicKey,
        gameSigner: gameSigner,
        slots: SYSVAR_SLOT_HASHES_PUBKEY,
        resource1MintAccount: mintResource1.publicKey,
        resource2MintAccount: mintResource2.publicKey,
        resource3MintAccount: mintResource3.publicKey,
        resource1TokenAccount: ATAResource1,
        resource2TokenAccount: ATAResource2,
        resource3TokenAccount: ATAResource3,
        ladaMintAccount: ladaMint.publicKey,
        gameLadaTokenAccount: gameLADATokenAccount,
        ladaTokenAccount: someGuyLADATokenAccount,
        gameTurnData: gameTurnData,
        item: item.publicKey,
        staff: empty.publicKey,
        head: empty.publicKey,
        robe: empty.publicKey,
      },
      signers: [item, someGuy],
      remainingAccounts: [
        {
          pubkey: spellBook.publicKey,
          isSigner: false,
          isWritable: true,
        },
      ],
    });

    //Check caster changes
    const fetchedCaster = await program.account.caster.fetch(caster.publicKey);

    assert.equal(fetchedCaster.turnCommit, null);
    assert.equal(oldFetchedCaster.experience < fetchedCaster.experience, true);

    //Assert the move
    assert.equal(fetchedCaster.modifiers.tileLevel, 0);
    assert.equal(fetchedCaster.modifiers.tileColumn, 1);

    //Assert craft
    assert.equal(itemPreCreation, null);
    assert.equal(
      (await program.account.item.fetchNullable(item.publicKey)) !== null,
      true
    );

    //Assert spell (get zombified)
    assert.equal(
      await program.account.item.fetchNullable(spellBook.publicKey),
      null
    );

    //Assert loot
    const postSomeGuyResources = await getTokenAccountBalances();

    assert.equal(postSomeGuyResources[0] > preSomeGuyResources[0], true);

    //Assert received LADA and removed from game
    //Gets all of it since the only player in the round
    assert.equal(
      (
        await ladaMint.getAccountInfo(someGuyLADATokenAccount)
      ).amount.toNumber(),
      playerLadaBalanceBefore + 1_984_126_984_130 //constant for LADA per turn
    );
    assert.equal(
      (await ladaMint.getAccountInfo(gameLADATokenAccount)).amount.toNumber(),
      gameLadaBalanceBefore - 1_984_126_984_130
    );
  });
});

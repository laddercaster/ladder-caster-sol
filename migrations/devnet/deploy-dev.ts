// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

// warning! run minting.ts before this script

import fs from "fs/promises";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import NodeWallet from "../NodeWallet";
import { Laddercast } from "../../target/types/laddercast";
import { Connection, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import MerkleTree from "merkletreejs";
import keccak256 from "keccak256";
import axios from "axios";

async function buildMerkleTree(): Promise<MerkleTree> {
  const merkelTree = await axios.get(
    "https://arweave.net/T1L0wI3oCt6exqSQIPK4bO0fYy24SHR5i9DAiK66c64"
  );

  return new MerkleTree(merkelTree.data, keccak256, {
    sortPairs: true,
    hashLeaves: true,
  });
}

async function createGame() {
  let keys, keysPK, idl;
  try {
    keys = JSON.parse(
      (await fs.readFile("migrations/devnet/sk.json")).toString()
    );

    keysPK = JSON.parse(
      (await fs.readFile("migrations/devnet/pk.json")).toString()
    );

    idl = JSON.parse(
      (await fs.readFile("target/idl/laddercast.json")).toString()
    );
  } catch (e) {
    console.log(e);
  }

  const mintRes1 = Keypair.fromSecretKey(Uint8Array.from(keys.res1SK));
  const mintRes2 = Keypair.fromSecretKey(Uint8Array.from(keys.res2SK));
  const mintRes3 = Keypair.fromSecretKey(Uint8Array.from(keys.res3SK));
  const gameAuthority = Keypair.fromSecretKey(
    Uint8Array.from(keys.gameAuthoritySK)
  );
  const gameAccount = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(keys.gameAccountSK)
  );

  const CONTRACT_ADDRESS = idl.metadata.address;
  const connection = new Connection(
    "https://api.devnet.solana.com",
    "confirmed"
  );
  anchor.setProvider(
    new anchor.Provider(
      connection,
      new NodeWallet(gameAuthority),
      anchor.Provider.defaultOptions()
    )
  );
  const game = new Program<Laddercast>(idl, CONTRACT_ADDRESS);

  // RPC New Game
  //RANDOM generate
  const [gameSigner] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("game_signer")],
    game.programId
  );

  const gameTurnInfo = {
    turn: 1,
    turnDelay: 1200,
    lastCrankSeconds: new anchor.BN(Date.now() / 1000),
    lastTileSpawn: 0,
    tileSpawnDelay: 20,
  } as GameTurnInfo;

  const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from("turn_data"),
      gameAccount.publicKey.toBuffer(),
      Buffer.from(anchor.utils.bytes.utf8.encode(String(1))),
    ],
    game.programId
  );
  let tree: MerkleTree = await buildMerkleTree();
  const root = tree.getRoot();

  await connection.confirmTransaction(
    await game.rpc.initGame(gameTurnInfo, [...root], {
      accounts: {
        authority: gameAuthority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: "SysvarRent111111111111111111111111111111111",
        gameAccount: gameAccount.publicKey,
        gameSigner: gameSigner,
        gameTurnData,
        slots: "SysvarS1otHashes111111111111111111111111111",
        resource1Mint: mintRes1.publicKey,
        resource2Mint: mintRes2.publicKey,
        resource3Mint: mintRes3.publicKey,
        ladaMint: keysPK.mintLadaPK,
        ladaTokenAccount: keysPK.tokenAccount,
      },
      signers: [gameAuthority, gameAccount, mintRes1, mintRes2, mintRes3],
    })
  );
  console.log("Initalized Game");
  console.log(
    JSON.stringify(await game.account.game.fetch(gameAccount.publicKey))
  );

  await fs.writeFile(
    "migrations/devnet/pk.json",
    JSON.stringify({
      ...keysPK,
      mintRes1PK: mintRes1.publicKey.toString(),
      mintRes2PK: mintRes2.publicKey.toString(),
      mintRes3PK: mintRes3.publicKey.toString(),
    })
  );
}

createGame();

interface GameTurnInfo {
  //current turn
  turn: number; // 64
  //how many slots til next turn
  turnDelay: number; // 64
  //last slot the crank was pulled
  lastCrankSeconds: anchor.BN; // 64
  // last turn a tile was spawned
  lastTileSpawn: number; // 64
  // how many turns til next tile should spawn
  tileSpawnDelay: number; // 64
}

//BEFORE DEPLOY SCRIPT
//rm -rf solana test validator
//solana-test-validator
//solana deploy target/deploy/legacy_sol.so target/deploy/legacy_sol-keypair.json
//Deploy with local id.json THEN use a different address for managing the game deployment

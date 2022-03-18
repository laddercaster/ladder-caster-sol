// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

// warning! run minting.ts before this script

import fs from "fs/promises";
import * as anchor from "@project-serum/anchor";
import NodeWallet from "./NodeWallet";
import { Laddercast } from "../target/types/laddercast";
import { Program } from "@project-serum/anchor";
import { Keypair, Connection } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { bs58 } from "@project-serum/anchor/dist/cjs/utils/bytes";

async function createGame() {
  console.log(`Setting up New Game`);
  const idl = JSON.parse(
    (await fs.readFile("target/idl/laddercast.json")).toString()
  );
  const keys = JSON.parse(
    (await fs.readFile("migrations/keys.json")).toString()
  );

  const mintRes1 = keys.mintRes1PK
    ? Keypair.fromSecretKey(bs58.decode(keys.mintRes1PK))
    : Keypair.generate();
  const mintRes2 = keys.mintRes2PK
    ? Keypair.fromSecretKey(bs58.decode(keys.mintRes2PK))
    : Keypair.generate();
  const mintRes3 = keys.mintRes3PK
    ? Keypair.fromSecretKey(bs58.decode(keys.mintRes3PK))
    : Keypair.generate();
  const CONTRACT_ADDRESS = idl.metadata.address;
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");
  const gameAuthority = anchor.web3.Keypair.generate();
  const gameAccount = anchor.web3.Keypair.fromSecretKey(
    bs58.decode(keys.gameAccountSK)
  );

  await connection.requestAirdrop(gameAccount.publicKey, 10e9 * 1000);
  await connection.requestAirdrop(gameAuthority.publicKey, 10e9 * 1000);
  console.log(gameAuthority.publicKey.toString());
  console.log("Starting 25s sleep to confirm the airdrop went through");
  await new Promise((f) => setTimeout(f, 25000)); //wait for airdrop to go through

  // const provider = new anchor.Provider(connection, new NodeWallet(keypair), {});
  // console.log(CONTRACT_ADDRESS, idl, keys);
  // const game: Program<Laddercast> = new anchor.Program<Laddercast>(
  //   idl,
  //   CONTRACT_ADDRESS,
  //   provider
  // );
  anchor.setProvider(
    new anchor.Provider(
      connection,
      new NodeWallet(gameAccount),
      anchor.Provider.defaultOptions()
    )
  );

  const game = new Program<Laddercast>(idl, CONTRACT_ADDRESS);

  //Devnet Deploy
  /*
  const connection = new anchor.web3.Connection('http://api.devnet.solana.com');
  const provider = new anchor.Provider(connection, new NodeWallet(anchor.web3.Keypair.fromSecretKey(bs58.decode('2Q1ComiijcAgk5ZhrzkXB3qffFhK23TMV1tw9ZzUcHo3f3QN4q5erd2SVaq12kuX23YU6KKtnyKt53N8kVNULBVn'))), {});

  */

  // RPC New Game
  //RANDOM generate
  const [gameSigner] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("game_signer"), gameAccount.publicKey.toBuffer()],
    game.programId
  );

  const gameTurnInfo = {
    turn: 1,
    turnDelay: 1200,
    lastCrankSeconds: new anchor.BN(0),
    lastTileSpawn: 0,
    tileSpawnDelay: 20,
  } as GameTurnInfo;

  const [gameTurnData] = await anchor.web3.PublicKey.findProgramAddress(
    [
      Buffer.from("turn_data"),
      gameAccount.publicKey.toBuffer(),
      Buffer.from(anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn))),
    ],
    game.programId
  );

  await connection.confirmTransaction(
    await game.rpc.initGame(gameTurnInfo, {
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
        ladaMint: keys.mintLadaPK,
        ladaTokenAccount: keys.tokenAccount,
      },
      signers: [gameAuthority, gameAccount, mintRes1, mintRes2, mintRes3],
    })
  );
  console.log("Initalized Game");
  console.log(
    JSON.stringify(await game.account.game.fetch(gameAccount.publicKey))
  );

  await fs.writeFile(
    "migrations/keys.json",
    JSON.stringify({
      ...keys,
      mintRes1PK: mintRes1.publicKey.toString(),
      mintRes2PK: mintRes2.publicKey.toString(),
      mintRes3PK: mintRes3.publicKey.toString(),
    })
  );
}

new Promise((resolve) => {
  createGame().then((resp) => {
    resolve(resp);
  });
});

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

import fs from "fs/promises";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import NodeWallet from "../NodeWallet";
import { Laddercast } from "../../target/types/laddercast";
import { Connection, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

async function createGame() {
  let keys, idl;
  try {
    keys = JSON.parse(
      (await fs.readFile("migrations/mainnet/sk.json")).toString()
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
    "https://api.mainnet-beta.solana.com",
    "confirmed"
  );

  anchor.setProvider(
    new anchor.Provider(
      connection,
      new NodeWallet(gameAccount),
      anchor.Provider.defaultOptions()
    )
  );

  const program = new Program<Laddercast>(idl, CONTRACT_ADDRESS);

  const [gameSigner] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("game_signer")],
    program.programId
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
      Buffer.from(anchor.utils.bytes.utf8.encode(String(gameTurnInfo.turn))),
    ],
    program.programId
  );

  await connection.confirmTransaction(
    await program.rpc.initGame(gameTurnInfo, {
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
  console.log(
    JSON.stringify(await program.account.game.fetch(gameAccount.publicKey))
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

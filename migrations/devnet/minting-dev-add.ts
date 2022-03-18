import {
  TOKEN_PROGRAM_ID,
  Token,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, Connection, PublicKey } from "@solana/web3.js";
import { bs58 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import fs from "fs/promises";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Laddercast } from "../../target/types/laddercast";
import NodeWallet from "../NodeWallet";

async function mintTokens() {
  let keys, idl;
  try {
    keys = JSON.parse(
      (await fs.readFile("migrations/devnet/sk.json")).toString()
    );
    idl = JSON.parse(
      (await fs.readFile("target/idl/laddercast.json")).toString()
    );
  } catch (e) {
    console.log(e);
  }

  const mintAuthority = Keypair.fromSecretKey(
    Uint8Array.from(keys.mintAuthoritySK)
  );
  const gameAccount = Keypair.fromSecretKey(
    Uint8Array.from(keys.gameAccountSK)
  );

  const connection = new Connection(
    "https://api.devnet.solana.com",
    "confirmed"
  );
  const provider = new anchor.Provider(
    connection,
    new NodeWallet(gameAccount),
    {}
  );
  const program: Program<Laddercast> = new anchor.Program<Laddercast>(
    idl,
    idl.metadata.address,
    provider
  );

  const mintAmount = 1000000 * 1_000_000_000;

  // const mintAccount = await Token.createMint(
  //   connection,
  //   mintAuthority,
  //   mintAuthority.publicKey,
  //   null,
  //   9,
  //   TOKEN_PROGRAM_ID
  // );

  const [gameSigner] = await PublicKey.findProgramAddress(
    [Buffer.from("game_signer"), gameAccount.publicKey.toBuffer()],
    program.programId
  );

  // //LADA mint
  // const tokenAccount = await mintAccount.createAccount(gameSigner);

  const mintAccount = new Token(
    connection,
    new PublicKey("4fGRYQjNfWagLjzpycuYu5MLpAKd4g2A3gBBHW1gEnjH"),
    TOKEN_PROGRAM_ID,
    mintAuthority
  );

  await mintAccount.mintTo(
    new PublicKey("9BJbFPViAdCSsCArnGqgx9BDBFt498GLms9VbCQZw1Ko"),
    mintAuthority.publicKey,
    [mintAuthority],
    mintAmount
  );

  // let _initializerTokenAccountLada = await mintAccount.getAccountInfo(
  //   tokenAccount
  // );

  // console.log(
  //   "Lada token account amount",
  //   _initializerTokenAccountLada.amount.toNumber()
  // );

  console.log("minting successful");

  // await fs.writeFile(
  //   "migrations/devnet/pk.json",
  //   JSON.stringify({
  //     gameAccount: gameAccount.publicKey.toString(),
  //     mintLadaPK: mintAccount.publicKey.toString(),
  //     // tokenAccount: tokenAccount.toString(),
  //   })
  // );
}

mintTokens();

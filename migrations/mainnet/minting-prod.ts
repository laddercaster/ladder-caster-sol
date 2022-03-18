import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
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
      (await fs.readFile("migrations/mainnet/keys.json")).toString()
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
    "https://api.mainnet-beta.solana.com",
    "confirmed"
  );
  const provider = new anchor.Provider(
    connection,
    new NodeWallet(gameAccount),
    {}
  );

  const [gameSigner] = await PublicKey.findProgramAddress(
    [Buffer.from("game_signer")],
    new PublicKey("LCThBz55Ma7hcueUQA3iiofBhvidQHfNLxxwfLsycxb")
  );

  console.log(gameSigner.toString());

  const tokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    mintAuthority,
    new PublicKey("25YC9dJjKkT4uyVUvL6WHTnpbRXfS8AEXvJp7FQQs9Yj"),
    gameSigner,
    true
  );

  console.log(tokenAccount.address.toBase58());

  // await mintAccount.mintTo(
  //   tokenAccount,
  //   mintAuthority.publicKey,
  //   [mintAuthority],
  //   500_000_000 * 1_000_000_000
  // );

  console.log("minting successful");
}

mintTokens();

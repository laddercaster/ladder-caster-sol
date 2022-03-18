import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { Keypair, Connection, PublicKey } from "@solana/web3.js";
import { bs58 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import fs from "fs/promises";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Laddercast } from "../target/types/laddercast";
import NodeWallet from "./NodeWallet";

async function mintTokens() {
  let keys, idl;
  try {
    keys = JSON.parse((await fs.readFile("migrations/keys.json")).toString());
    idl = JSON.parse(
      (await fs.readFile("target/idl/laddercast.json")).toString()
    );
  } catch (e) {
    console.log(e);
  }

  const mintAuthority = keys.mintAuthoritySK
    ? Keypair.fromSecretKey(bs58.decode(keys.mintAuthoritySK))
    : Keypair.generate();
  const gameAccount = keys.gameAccountSK
    ? Keypair.fromSecretKey(bs58.decode(keys.gameAccountSK))
    : Keypair.generate();

  const connection = new Connection("http://127.0.0.1:8899", "confirmed");
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

  console.log(gameAccount.publicKey.toString());
  console.log("Starting 25s sleep to confirm the airdrop went through");
  await connection.confirmTransaction(
    await connection.requestAirdrop(mintAuthority.publicKey, 10e9 * 1000)
  );

  const mintAmount = 250_000_000 * 1_000_000_000;

  const mintAccount = await Token.createMint(
    connection,
    mintAuthority,
    mintAuthority.publicKey,
    null,
    9,
    TOKEN_PROGRAM_ID
  );

  console.log("Lada mint account", mintAccount.publicKey.toBase58());

  const [gameSigner] = await PublicKey.findProgramAddress(
    [Buffer.from("game_signer"), gameAccount.publicKey.toBuffer()],
    program.programId
  );

  //LADA mint
  const tokenAccount = await mintAccount.createAccount(gameSigner);

  await mintAccount.mintTo(
    tokenAccount,
    mintAuthority.publicKey,
    [mintAuthority],
    mintAmount
  );

  let _initializerTokenAccountLada = await mintAccount.getAccountInfo(
    tokenAccount
  );

  console.log(
    "Lada token account amount",
    _initializerTokenAccountLada.amount.toNumber()
  );

  console.log("minting successful");

  await fs.writeFile(
    "migrations/keys.json",
    JSON.stringify({
      gameAccountSK: bs58.encode(gameAccount.secretKey),
      gameAccount: gameAccount.publicKey.toString(),
      mintAuthoritySK: bs58.encode(mintAuthority.secretKey),
      mintLadaPK: mintAccount.publicKey.toString(),
      tokenAccount: tokenAccount.toString(),
    })
  );
}

mintTokens();

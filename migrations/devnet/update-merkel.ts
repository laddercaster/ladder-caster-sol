import fs from "fs/promises";
import * as anchor from "@project-serum/anchor";
import NodeWallet from "../NodeWallet";
import { Laddercast } from "../../target/types/laddercast";
import { Program } from "@project-serum/anchor";
import { Keypair, Connection } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { bs58 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import { MerkleTree } from "merkletreejs";
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

async function updateMerkel() {
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

  const gameAccount = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(keys.gameAccountSK)
  );
  const gameAuthority = Keypair.fromSecretKey(
    Uint8Array.from(keys.gameAuthoritySK)
  );
  console.log(gameAuthority.publicKey.toString());
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
  let tree: MerkleTree = await buildMerkleTree();
  const root = tree.getRoot();

  await connection.confirmTransaction(
    await game.rpc.updateMerkleRoot([...root], {
      accounts: {
        authority: gameAuthority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        gameAccount: gameAccount.publicKey,
      },
      signers: [gameAuthority],
    })
  );

  console.log("yooooo we done");
}

updateMerkel();

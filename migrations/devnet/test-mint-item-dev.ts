import {Connection, Keypair} from "@solana/web3.js";
import fs from "fs/promises";
import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {Laddercast} from "../../target/types/laddercast";
import NodeWallet from "../NodeWallet";
import {ASSOCIATED_TOKEN_PROGRAM_ID, Token, TOKEN_PROGRAM_ID,} from "@solana/spl-token";
import {Metadata, MetadataProgram,} from "@metaplex-foundation/mpl-token-metadata";
import {MerkleTree} from "merkletreejs";
import keccak256 from "keccak256";

function buildMerkleTree(): MerkleTree {
    return new MerkleTree(
        [
            "chest:1:http://google.com",
            "chest:2:http://google2.com",
        ],
        keccak256,
        {sortPairs: true, hashLeaves: true}
    );
}

async function mintItem() {
    let keys, idl, phantomKeys;
    try {
        // keys = JSON.parse(
        //     (await fs.readFile("migrations/devnet/sk.json")).toString()
        // );

        idl = JSON.parse(
            (await fs.readFile("target/idl/laddercast.json")).toString()
        );

        phantomKeys = JSON.parse(
            (await fs.readFile("migrations/devnet/phantom-key.json")).toString()
        );
    } catch (e) {
        console.log(e);
    }

    let tree: MerkleTree = buildMerkleTree();

    // const gameAccount = Keypair.fromSecretKey(
    //   Uint8Array.from(keys.gameAccountSK)
    // );

    //To get proper format of phantom private key, export mnemonic and then use this below
    //solana-keygen recover 'prompt://?key=0/0' -o phanton-priv.json

    const phantomAccount = Keypair.fromSecretKey(Uint8Array.from(phantomKeys.SK));

    const connection = new Connection(
        "https://api.devnet.solana.com",
        "confirmed"
    );

    const provider = new anchor.Provider(
        connection,
        new NodeWallet(phantomAccount),
        {}
    );

    const program: Program<Laddercast> = new anchor.Program<Laddercast>(
        idl,
        idl.metadata.address,
        provider
    );

    //My player is index 0 "3yo7Vdzm9xPpT1nQmSMuRoD86ZwJHpWvBeLiWTxhapoD"
    //My item is index 0 "DJmuo2XcLrALphonCrcDDXQL1CDHEoScoaZRrJGgcvJu"

    const nftMintKeys = Keypair.generate();

    const [nftMetadata, bump] = await anchor.web3.PublicKey.findProgramAddress(
        [
            Buffer.from(anchor.utils.bytes.utf8.encode("metadata")),
            nftMintKeys.publicKey.toBuffer(),
        ],
        program.programId
    );

    const nftToken = await Token.getAssociatedTokenAddress(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        nftMintKeys.publicKey,
        phantomAccount.publicKey
    );

    const metaplexMetadataAccount = await Metadata.getPDA(nftMintKeys.publicKey);

    //Merkle proof part
    const leaf = keccak256("chest:1:http://google2.com");
    // noinspection TypeScriptValidateTypes
    const proof = tree.getProof(leaf);
    const validProof: Buffer[] = proof.map((p) => p.data);

    await connection.confirmTransaction(
        await program.rpc.mintItem(validProof, {
            accounts: {
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: anchor.web3.SystemProgram.programId,
                rent: "SysvarRent111111111111111111111111111111111",

                authority: phantomAccount.publicKey,

                game: "E764XSjoMuDDKxV1QyQkiDMRqGxdB2TbjMA3vRaUsKHG",
                player: "3yo7Vdzm9xPpT1nQmSMuRoD86ZwJHpWvBeLiWTxhapoD",
                item: "DJmuo2XcLrALphonCrcDDXQL1CDHEoScoaZRrJGgcvJu",

                metaplexMetadataAccount: metaplexMetadataAccount,
                metaplexTokenMetadataProgram: MetadataProgram.PUBKEY,

                nftMint: nftMintKeys.publicKey,
                nftToken: nftToken,
                nftMetadata: nftMetadata,
            },
            signers: [phantomAccount, nftMintKeys],
        })
    );
}

mintItem();

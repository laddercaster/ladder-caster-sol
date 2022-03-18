const nReadlines = require("n-readlines");
const Arweave = require("arweave");
const { default: deepHash } = require("arweave/node/lib/deepHash");
const { default: ArweaveBundles } = require("arweave-bundles");
const fs = require("fs");
const path = require("path");

const TEST_CONFIG_ARWEAVE = {
  host: "127.0.0.1",
  port: 1984,
  protocol: "http",
  timeout: 20000,
  logging: false,
};

const PROD_CONFIG_ARWEAVE = {
  host: "arweave.net",
  port: 443,
  protocol: "https",
  timeout: 20000, // Network request timeouts in milliseconds
  logging: false, // Disable network request logging
};

//Initialize Arweave
const deps = {
  utils: Arweave.utils,
  crypto: Arweave.crypto,
  deepHash: deepHash,
};

const arweave = Arweave.init(PROD_CONFIG_ARWEAVE);

const arweaveBundle = ArweaveBundles(deps);

//Uploading the images on Arweave
async function uploadImagesData(arweaveWallet) {
  let files = fs.readdirSync(path.resolve(__dirname, "images"));

  let transactionInformation = [];

  for (let i = 0; i < files.length; i++) {
    let imgData = fs.readFileSync(path.resolve(__dirname, "images", files[i]));

    const transaction = await arweave.createTransaction(
      {
        data: imgData,
      },
      arweaveWallet
    );
    transaction.addTag("Content-Type", "application/png");

    await arweave.transactions.sign(transaction, arweaveWallet);

    let uploader = await arweave.transactions.getUploader(transaction);

    while (!uploader.isComplete) {
      await uploader.uploadChunk();
      console.log(
        `${uploader.pctComplete}% complete, ${uploader.uploadedChunks}/${uploader.totalChunks}`
      );
    }

    console.log(`Transaction ID for ${files[i]} is ${transaction.id}`);

    transactionInformation.push({ txId: transaction.id, fileName: files[i] });
  }
}

const initUploadImages = async function () {
  try {
    const arweaveWallet = JSON.parse(
      fs.readFileSync("arweave/arweave-key.json").toString()
    );

    await uploadImagesData(arweaveWallet);
  } catch (err) {
    console.error(err);
  }
};

initUploadImages()
  .then((resolve) => {
    return resolve();
  })
  .catch((error) => {
    console.log(error);
  });

///////////////////////////////////
///////////////////////////////////
///////////////////////////////////
///////////////////////////////////
///////////////////////////////////
// async function createDataItem(arweaveWallet, data) {
//     const myTags = [
//         {name: "App-Name", value: "LadderCaster"},
//         {name: "App-Season", value: "0"},
//     ];
//
//     let item = await arweaveBundle.createData(
//         {data, tags: myTags},
//         arweaveWallet
//     );
//
//     // Sign the data, ready to be added to a bundle
//     return await arweaveBundle.sign(item, arweaveWallet);
// }
//
// async function uploadBundleData(arweaveWallet) {
//
//     const tx2 = await arweave.transactions.getData("-3xH0qZFA4-ra51pEqmIXsGLs6c9iTURshh8Ml2X6Ag");
//     console.log(tx2);
//     const tx = await arweave.transactions.get("-3xH0qZFA4-ra51pEqmIXsGLs6c9iTURshh8Ml2X6Ag");
//
//     console.log(tx);
//
//     const data = arweaveBundle.unbundleData(tx)
//
//     console.log(data);
//
//     // const nftJSONDataLines = new nReadlines(
//     //     path.resolve(__dirname, "nft_combination_export")
//     // );
//     //
//     // let line;
//     //
//     // let items = [];
//     //
//     // while ((line = nftJSONDataLines.next())) {
//     //     items.push(await createDataItem(arweaveWallet, line.toString()));
//     // }
//     //
//     // const myBundle = await arweaveBundle.bundleData(items);
//     //
//     // const transaction = await arweave.createTransaction(
//     //     {data: myBundle},
//     //     arweaveWallet
//     // );
//     //
//     // transaction.addTag("Bundle-Format", "json");
//     // transaction.addTag("Bundle-Version", "1.0.0");
//     // transaction.addTag("Content-Type", "application/json");
//     //
//     // await arweave.transactions.sign(transaction, arweaveWallet);
//     // await arweave.transactions.post(transaction);
//
//     console.log(`TX ID for upload bundle: ${transaction.id}`);
// }
//
// const initUploadData = async function () {
//     try {
//         const arweaveWallet = JSON.parse(
//             fs.readFileSync("arweave/arweave-key.json").toString()
//         );
//
//         await uploadBundleData(arweaveWallet);
//     } catch (err) {
//         console.error(err);
//     }
// };
//
// initUploadData()
//     .then((resolve) => {
//         return resolve();
//     })
//     .catch((error) => {
//         console.log(error);
//     });

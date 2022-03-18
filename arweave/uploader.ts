import Arweave from "arweave";
import fs from "fs";
import glob from "glob";
import * as path from "path";
import { chunk, has } from "lodash";

const PROD_CONFIG_ARWEAVE = {
  host: "arweave.net",
  port: 443,
  protocol: "https",
  timeout: 20000, // Network request timeouts in milliseconds
  logging: false, // Disable network request logging
};

const arweave = Arweave.init(PROD_CONFIG_ARWEAVE);

//Taken from stackoverflow
function setObjectProp(obj: Object, path: string, value: string) {
  let schema = obj; // a moving reference to internal objects within obj
  let pList = path.split(".");
  let len = pList.length;

  for (let i = 0; i < len - 1; i++) {
    let elem = pList[i];
    if (!schema[elem]) schema[elem] = {};
    schema = schema[elem];
  }

  schema[pList[len - 1]] = value;
}

const getValues = (file: string, def: any) => {
  if (fs.existsSync(file)) {
    const data = fs.readFileSync(file).toString();
    return JSON.parse(data);
  }
  return def;
};

async function uploadJSON(jsonString: string) {
  const arweaveWallet = JSON.parse(
    fs.readFileSync("arweave/arweave-key.json").toString()
  );

  const transaction = await arweave.createTransaction(
    {
      data: jsonString,
    },
    arweaveWallet
  );
  transaction.addTag("Content-Type", "application/json");

  await arweave.transactions.sign(transaction, arweaveWallet);
  await arweave.transactions.post(transaction);

  let uploader = await arweave.transactions.getUploader(transaction);

  while (!uploader.isComplete) {
    await uploader.uploadChunk();
    console.log(
      `${uploader.pctComplete}% complete, ${uploader.uploadedChunks}/${uploader.totalChunks}`
    );
  }

  console.log(`Transaction ID for is ${transaction.id}`);

  return transaction.id;
}

const uploader = async () => {
  let fileStruct = getValues(
    path.resolve(__dirname, "outputs/fileStruct.json"),
    {}
  );

  let merkleLeaves = getValues(
    path.resolve(__dirname, "outputs/merkleLeaves.json"),
    []
  );

  const filePath = path.resolve(__dirname, "json_exports");

  let count = 0;
  let hasFailed = 0;
  let alreadyUploaded = 0;

  for (let fChunk of chunk(
    glob.sync(path.resolve(__dirname, "json_exports/*.json")),
    50
  )) {
    let promises = [];
    for (let file of fChunk) {
      promises.push(
        new Promise(async (resolve) => {
          try {
            let rawdata = fs.readFileSync(file).toString();
            let data = JSON.parse(rawdata);

            file = file.split(".")[0];

            file = file.split(filePath)[1].split("/")[1];

            const fileStructPath = file.split("_").join(".");

            if (has(fileStruct, fileStructPath)) {
              alreadyUploaded += 1;
              return resolve("");
            }

            const url = await uploadJSON(JSON.stringify(data));

            setObjectProp(
              fileStruct,
              fileStructPath,
              "https://arweave.net/" + url
            );

            merkleLeaves.push(
              `https://arweave.net/${url}:${file.split("_").join(":")}`
            );

            count += 1;
            console.log("Count: ", count);
            return resolve("");
          } catch (error) {
            console.log(error);
            hasFailed += 1;
            return resolve(""); //resolve we'll just retry
          }
        })
      );
    }
    await Promise.all(promises);
  }

  fs.writeFileSync(
    path.resolve(__dirname, "outputs/fileStruct.json"),
    JSON.stringify(fileStruct)
  );
  fs.writeFileSync(
    path.resolve(__dirname, "outputs/merkleLeaves.json"),
    JSON.stringify(merkleLeaves)
  );

  console.log("has failed: ", hasFailed);
  console.log("was already uploaded: ", alreadyUploaded);

  // for (let file of glob.sync(path.resolve(__dirname, "json_exports/*.json"))) {
  //   count += 1;
  //   console.log("Count: ", count);
  //   try {
  //     let rawdata = fs.readFileSync(file).toString();
  //     let data = JSON.parse(rawdata);
  //
  //     file = file.split(".")[0];
  //
  //     file = file.split(filePath)[1].split("/")[1];
  //
  //     const fileStructPath = file.split("_").join(".");
  //
  //     if (has(fileStruct, fileStructPath)) {
  //       continue;
  //     }
  //
  //     const url = await uploadJSON(JSON.stringify(data));
  //
  //     setObjectProp(fileStruct, fileStructPath, "https://arweave.net/" + url);
  //
  //     merkleLeaves.push(
  //       `https://arweave.net/${url}:${file.split("_").join(":")}`
  //     );
  //   } catch (e) {
  //     console.log(e);
  //   } finally {
  //     fs.writeFileSync(
  //       path.resolve(__dirname, "outputs/fileStruct.json"),
  //       JSON.stringify(fileStruct)
  //     );
  //     fs.writeFileSync(
  //       path.resolve(__dirname, "outputs/merkleLeaves.json"),
  //       JSON.stringify(merkleLeaves)
  //     );
  //   }
  // }
};

//Upload the 2 final JSONS
async function uploadFinalJSONs() {
  let fileStruct = path.resolve(__dirname, "outputs/fileStruct.json");
  let fileMerkleLeaves = path.resolve(__dirname, "outputs/merkleLeaves.json");

  const transactionIdFileStruct = await uploadJSON(
    fs.readFileSync(fileStruct).toString()
  );

  const transactionIdMerkleLeaves = await uploadJSON(
    fs.readFileSync(fileMerkleLeaves).toString()
  );

  console.log(`Transaction ID for file struct ${transactionIdFileStruct}`);
  console.log(`Transaction ID for merkle leaves ${transactionIdMerkleLeaves}`);
}

// let finished = true;
// while (true) {
//   if (!finished) {
//     continue;
//   }
// finished = false;
// uploader()
//   .then(() => {
//     // finished = true;
//     console.log("WE DID IT, uploading the 2 JSONS");
//     // uploadFinalJSONs().then(() => {
//     //   console.log("All done");
//     // });
//   })
//   .catch((e) => console.log("ERRR:", e));
// // }
uploadFinalJSONs().then(() => {
  console.log("All done");
});

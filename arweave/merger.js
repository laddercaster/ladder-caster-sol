const merge = require('deepmerge-json');
const fs = require("fs");
const path = require("path");

const pathToD = path.resolve("/Users/jdesorm/Downloads", "fileStructD.json");
const pathToJ = path.resolve("/Users/jdesorm/Downloads", "fileStructJ.json");
const dataD = JSON.parse(fs.readFileSync(pathToD).toString());
const dataJ = JSON.parse(fs.readFileSync(pathToJ).toString());

const merged = merge(dataD, dataJ);

fs.writeFileSync(path.resolve("/Users/jdesorm/Downloads", "fileStructCombined.json"), JSON.stringify(merged));

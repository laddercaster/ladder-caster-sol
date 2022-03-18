import * as anchor from "@project-serum/anchor";

export interface GameTurnInfo {
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

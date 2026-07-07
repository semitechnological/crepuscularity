import { spawnSync } from "node:child_process";

const proc = spawnSync("echo", ["hello"]);
console.log(proc.stdout.toString());

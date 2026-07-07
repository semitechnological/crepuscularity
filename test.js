const { spawnSync } = require('node:child_process');
const proc = spawnSync(process.env.CREPUS_BIN, ["native", "ir", "--stdin-json"], {
    input: JSON.stringify({ template: "hello", context: {} }),
    encoding: "utf8"
});
console.log(proc);

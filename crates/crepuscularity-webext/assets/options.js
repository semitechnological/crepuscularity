import init, * as runtime from "../vendor/runtime.js";

const wasmBytes = await fetch("../vendor/runtime_bg.wasm").then((response) => response.arrayBuffer());
await init({ module_or_path: wasmBytes });

if (typeof runtime.options_main !== "function") {
  throw new Error("runtime.options_main is required");
}
runtime.options_main();

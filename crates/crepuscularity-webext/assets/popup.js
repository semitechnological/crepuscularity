import init, * as runtime from "../vendor/runtime.js";

const wasmBytes = await fetch("../vendor/runtime_bg.wasm").then((response) => response.arrayBuffer());
await init({ module_or_path: wasmBytes });

if (typeof runtime.popup_main !== "function") {
  throw new Error("runtime.popup_main is required");
}
runtime.popup_main();

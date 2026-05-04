const api = globalThis.browser ?? globalThis.chrome;
const runtimeModule = await import("../vendor/runtime.js");
const wasmBytes = await fetch("../vendor/runtime_bg.wasm").then((response) => response.arrayBuffer());
await runtimeModule.default({ module_or_path: wasmBytes });

api.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  runtimeModule.handle_background_message(message)
    .then((response) => sendResponse(response))
    .catch((error) => sendResponse({ ok: false, error: String(error) }));
  return true;
});

runtimeModule.settings_seed().catch(() => {});

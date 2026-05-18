const api = globalThis.browser ?? globalThis.chrome;
const runtimeModule = await import("../vendor/runtime.js");
const wasmBytes = await fetch("../vendor/runtime_bg.wasm").then((response) => response.arrayBuffer());
await runtimeModule.default({ module_or_path: wasmBytes });

api.runtime?.onMessage?.addListener((message, _sender, sendResponse) => {
  if (typeof runtimeModule.handle_background_message !== "function") return false;
  runtimeModule.handle_background_message(message)
    .then((response) => sendResponse(response))
    .catch((error) => sendResponse({ ok: false, error: String(error) }));
  return true;
});

api.runtime?.onInstalled?.addListener(() => {
  runtimeModule.settings_seed?.().catch(() => {});
});

api.storage?.onChanged?.addListener((_changes, area) => {
  if (area === "sync") runtimeModule.settings_seed?.().catch(() => {});
});

runtimeModule.background_main?.();

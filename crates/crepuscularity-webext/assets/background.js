import init, * as runtimeModule from "../vendor/runtime.js";

const api = globalThis.browser ?? globalThis.chrome;
const runtimeUrl = api.runtime.getURL("vendor/runtime_bg.wasm");
const runtimeReady = fetch(runtimeUrl)
  .then((response) => response.arrayBuffer())
  .then((wasmBytes) => init({ module_or_path: wasmBytes }))
  .then(() => {
    runtimeModule.background_main?.();
    return runtimeModule;
  });

api.runtime?.onMessage?.addListener((message, _sender, sendResponse) => {
  if (typeof runtimeModule.handle_background_message !== "function") return false;
  runtimeReady
    .then(() => runtimeModule.handle_background_message(message))
    .then((response) => sendResponse(response))
    .catch((error) => sendResponse({ ok: false, error: String(error) }));
  return true;
});

api.runtime?.onInstalled?.addListener(() => {
  runtimeReady.then(() => runtimeModule.settings_seed?.()).catch(() => {});
});

api.storage?.onChanged?.addListener((_changes, area) => {
  if (area === "sync") runtimeReady.then(() => runtimeModule.settings_seed?.()).catch(() => {});
});

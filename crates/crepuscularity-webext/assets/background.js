import init, * as runtimeModule from "../vendor/runtime.js";

const api = globalThis.browser ?? globalThis.chrome;
if (!api?.runtime) {
  throw new Error("browser runtime API is unavailable");
}
const runtimeUrl = api.runtime.getURL("vendor/runtime_bg.wasm");
const runtimeReady = fetch(runtimeUrl)
  .then((response) => response.arrayBuffer())
  .then((wasmBytes) => init({ module_or_path: wasmBytes }))
  .then(() => {
    if (typeof runtimeModule.background_main !== "function") {
      throw new Error("runtime.background_main is required");
    }
    if (typeof runtimeModule.handle_background_message !== "function") {
      throw new Error("runtime.handle_background_message is required");
    }
    if (typeof runtimeModule.settings_seed !== "function") {
      throw new Error("runtime.settings_seed is required");
    }
    runtimeModule.background_main();
    return runtimeModule;
  });

api.runtime?.onMessage?.addListener((message, _sender, sendResponse) => {
  runtimeReady
    .then(() => runtimeModule.handle_background_message(message))
    .then((response) => sendResponse(response))
    .catch((error) => sendResponse({ ok: false, error: String(error) }));
  return true;
});

api.runtime?.onInstalled?.addListener(() => {
  runtimeReady.then(() => runtimeModule.settings_seed()).catch((error) => {
    console.error("settings_seed failed on install", error);
  });
});

api.storage?.onChanged?.addListener((_changes, area) => {
  if (area === "sync") {
    runtimeReady.then(() => runtimeModule.settings_seed()).catch((error) => {
      console.error("settings_seed failed on storage change", error);
    });
  }
});

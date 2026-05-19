import { browserApi } from "./browser-shim.js";
import init, * as runtimeModule from "../vendor/runtime.js";

const DEFAULT_SETTINGS = {
  enabled: true,
  autoRender: false
};

const api = globalThis.browser ?? globalThis.chrome;
if (!api?.runtime) {
  throw new Error("browser runtime API is unavailable");
}

const runtimeReady = fetch("../vendor/runtime_bg.wasm")
  .then((response) => response.arrayBuffer())
  .then((wasmBytes) => init({ module_or_path: wasmBytes }))
  .then(() => runtimeModule)
  .catch((error) => {
    console.error("background WASM init failed", error);
    return null;
  });

function seedSettings() {
  return browserApi.storage.sync
    .get(DEFAULT_SETTINGS)
    .then((settings) => browserApi.storage.sync.set({ ...DEFAULT_SETTINGS, ...settings }))
    .catch((error) => {
      console.error("Failed to seed extension settings", error);
    });
}

api.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type === "settings:get") {
    browserApi.storage.sync
      .get(DEFAULT_SETTINGS)
      .then((settings) => sendResponse({
        ok: true,
        settings: { ...DEFAULT_SETTINGS, ...settings }
      }))
      .catch((error) => sendResponse({ ok: false, error: error.message }));
    return true;
  }

  if (message?.type === "settings:set") {
    const nextSettings = { ...DEFAULT_SETTINGS, ...message.settings };
    browserApi.storage.sync
      .set(nextSettings)
      .then(() => sendResponse({ ok: true, settings: nextSettings }))
      .catch((error) => sendResponse({ ok: false, error: error.message }));
    return true;
  }

  runtimeReady
    .then((runtime) => {
      if (runtime && typeof runtime.handle_background_message === "function") {
        return runtime.handle_background_message(message);
      }
      return { ok: false, error: "unknown message type" };
    })
    .then((response) => sendResponse(response))
    .catch((error) => sendResponse({ ok: false, error: String(error) }));
  return true;
});

api.runtime.onInstalled.addListener(() => {
  seedSettings();
  runtimeReady
    .then((runtime) => {
      if (runtime && typeof runtime.settings_seed === "function") {
        return runtime.settings_seed();
      }
    })
    .catch((error) => {
      console.error("settings_seed failed on install", error);
    });
});

api.storage.onChanged.addListener((_changes, area) => {
  if (area !== "sync") return;
  runtimeReady
    .then((runtime) => {
      if (runtime && typeof runtime.settings_seed === "function") {
        return runtime.settings_seed();
      }
    })
    .catch((error) => {
      console.error("settings_seed failed on storage change", error);
    });
});

seedSettings();

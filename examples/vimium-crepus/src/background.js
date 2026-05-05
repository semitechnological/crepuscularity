const api = globalThis.browser ?? globalThis.chrome;
const runtimeModule = await import("../vendor/runtime.js");
const wasmBytes = await fetch("../vendor/runtime_bg.wasm").then((response) => response.arrayBuffer());
await runtimeModule.default({ module_or_path: wasmBytes });

let currentVersion = runtimeModule.runtime_version();

api.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  runtimeModule.handle_background_message(message)
    .then((response) => sendResponse(response))
    .catch((error) => sendResponse({ ok: false, error: String(error) }));
  return true;
});

api.runtime.onInstalled.addListener(({ reason, previousVersion }) => {
  runtimeModule.settings_seed().catch(() => {});

  if (reason === "install") {
    injectContentScripts();
  }

  if (reason === "update" && previousVersion !== currentVersion) {
    if (api.notifications) {
      api.notifications.create("vimium-crepus-upgrade", {
        type: "basic",
        iconUrl: api.runtime.getURL("icons/icon48.png"),
        title: "vimium-crepus upgraded",
        message: `Updated to version ${currentVersion}. Press <Alt+Shift+V> for help.`
      });
    }
    injectContentScripts();
  }
});

function injectContentScripts() {
  api.tabs.query({}, (tabs) => {
    for (const tab of tabs) {
      if (!tab.url || !tab.url.startsWith("http")) continue;
      api.scripting.executeScript({
        target: { tabId: tab.id, allFrames: true },
        files: ["src/content.js"]
      }).catch(() => {});
      api.scripting.insertCSS({
        target: { tabId: tab.id, allFrames: true },
        files: ["src/content.css"]
      }).catch(() => {});
    }
  });
}

api.webNavigation?.onCommitted?.addListener(({ tabId, url, frameId }) => {
  if (frameId !== 0) return;
  updateActionIcon(tabId, url);
});

function updateActionIcon(tabId, url) {
  const isExcluded = isUrlExcluded(url);
  const path = isExcluded ? "icons/action_disabled_32.png" : "icons/action_enabled_32.png";
  try {
    api.action?.setIcon({ tabId, path: { "32": path } });
  } catch (_) {}
}

function isUrlExcluded(_url) {
  return false;
}

api.storage?.onChanged?.addListener((changes, area) => {
  if (area === "sync") {
    runtimeModule.settings_seed().catch(() => {});
  }
});
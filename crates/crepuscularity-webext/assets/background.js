import { browserApi } from "./browser-shim.js";

const DEFAULT_SETTINGS = {
  enabled: true,
  autoRender: false
};

browserApi.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type === "settings:get") {
    browserApi.storage.sync
      .get(DEFAULT_SETTINGS)
      .then((settings) => sendResponse({ ok: true, settings: { ...DEFAULT_SETTINGS, ...settings } }))
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

  return false;
});

browserApi.storage.sync
  .get(DEFAULT_SETTINGS)
  .then((settings) => browserApi.storage.sync.set({ ...DEFAULT_SETTINGS, ...settings }))
  .catch((error) => {
    console.error("Failed to seed extension settings", error);
  });

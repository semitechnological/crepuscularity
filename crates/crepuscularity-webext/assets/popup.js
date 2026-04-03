import { browserApi } from "./browser-shim.js";

// popup.html is pre-rendered at build time by `crepus webext build`.
// This file only handles two concerns:
//   1. Reading storage and patching stateful elements (checkboxes).
//   2. Routing click/change actions to storage writes or view toggles.

const viewMain = document.getElementById("view-main");
const viewHelp = document.getElementById("view-help");

const DEFAULTS = { enabled: true, autoRender: false };

async function syncState() {
  const data = await browserApi.storage.sync
    .get(DEFAULTS)
    .catch(() => DEFAULTS);
  const settings = { ...DEFAULTS, ...data };

  const enabledCb = document.getElementById("enabled");
  const autoRenderCb = document.getElementById("autoRender");
  if (enabledCb) enabledCb.checked = settings.enabled;
  if (autoRenderCb) autoRenderCb.checked = settings.autoRender;
}

syncState();

document.addEventListener("click", (e) => {
  const action = e.target.closest("[data-action]")?.dataset.action;
  if (!action) return;

  if (action === "show-help") {
    if (viewMain) viewMain.hidden = true;
    if (viewHelp) viewHelp.hidden = false;
  } else if (action === "hide-help") {
    if (viewMain) viewMain.hidden = false;
    if (viewHelp) viewHelp.hidden = true;
    syncState();
  }
});

document.addEventListener("change", async (e) => {
  const actionEl = e.target.closest("input[data-action]");
  if (!actionEl) return;

  const action = actionEl.dataset.action;
  if (action === "set-enabled") {
    await browserApi.storage.sync.set({ enabled: actionEl.checked }).catch(() => {});
  } else if (action === "set-auto-render") {
    await browserApi.storage.sync.set({ autoRender: actionEl.checked }).catch(() => {});
  }
});

import { browserApi } from "./browser-shim.js";
import initWasm, * as wasmModule from "../vendor/runtime.js";

// ---------------------------------------------------------------------------
// Storage operation helper — used by WASM-driven popups.
// Apps return { storage_op: { type, key, ... } } from handle_popup_action().
// ---------------------------------------------------------------------------
async function applyStorageOp(op) {
  if (!op?.type) return;
  if (op.type === "push") {
    const current = await browserApi.storage.local.get({ [op.key]: [] });
    const arr = Array.isArray(current[op.key]) ? [...current[op.key]] : [];
    const id = Date.now().toString(36) + Math.random().toString(36).slice(2);
    arr.push({ id, ...op.item });
    await browserApi.storage.local.set({ [op.key]: arr });
  } else if (op.type === "remove") {
    const current = await browserApi.storage.local.get({ [op.key]: [] });
    const arr = (Array.isArray(current[op.key]) ? current[op.key] : [])
      .filter((item) => item.id !== op.id);
    await browserApi.storage.local.set({ [op.key]: arr });
  } else if (op.type === "set") {
    await browserApi.storage.local.set({ [op.key]: op.value });
  }
}

// ---------------------------------------------------------------------------
// WASM-driven popup: if the app exports render_popup(state) -> { html, css? }
// we hand everything over to it — no popup.html structure is assumed.
// ---------------------------------------------------------------------------
async function runWasmDrivenPopup() {
  const root = document.getElementById("root") ?? document.body;

  const fetchState = () =>
    browserApi.storage.local.get(null).catch(() => ({}));

  let injectedStyle = null;

  const render = async () => {
    const state = await fetchState();
    let result;
    try {
      result = wasmModule.render_popup(state);
    } catch (err) {
      console.error("crepuscularity render_popup failed", err);
      return;
    }

    // Inject / update app-supplied CSS on first render.
    if (result.css && !injectedStyle) {
      injectedStyle = document.createElement("style");
      injectedStyle.id = "__crepus-app-styles";
      document.head.append(injectedStyle);
    }
    if (injectedStyle && result.css) {
      injectedStyle.textContent = result.css;
    }

    root.innerHTML = result?.html ?? "";
  };

  await render();

  root.addEventListener("submit", (e) => e.preventDefault());

  root.addEventListener("click", async (e) => {
    const actionEl = e.target.closest("[data-action]");
    if (!actionEl || typeof wasmModule.handle_popup_action !== "function") return;
    e.preventDefault();

    const action = actionEl.dataset.action;
    const data = Object.assign({}, actionEl.dataset);
    delete data.action;

    const form = actionEl.closest("form");
    if (form) {
      new FormData(form).forEach((v, k) => { data[k] = String(v); });
    }

    let opResult;
    try {
      opResult = wasmModule.handle_popup_action(action, data);
    } catch (err) {
      console.error("crepuscularity handle_popup_action failed", err);
      return;
    }

    if (opResult?.storage_op) {
      await applyStorageOp(opResult.storage_op);
    }
    await render();
  });
}

// ---------------------------------------------------------------------------
// Default settings UI: rendered in JS so popup.html can stay a neutral shell.
// ---------------------------------------------------------------------------
async function runSettingsPopup() {
  const root = document.getElementById("root") ?? document.body;

  root.innerHTML = `
<main class="popup">
  <div class="popup__eyebrow">crepuscularity</div>
  <h1 class="popup__title" id="appName">Extension</h1>
  <p class="popup__copy" id="appDescription">Rust and crepus-powered extension.</p>
  <label class="popup__toggle">
    <input id="enabled" type="checkbox">
    <span>Enable scanner</span>
  </label>
  <label class="popup__toggle">
    <input id="autoRender" type="checkbox">
    <span>Auto-render widgets</span>
  </label>
</main>`;

  const enabled = root.querySelector("#enabled");
  const autoRender = root.querySelector("#autoRender");
  const appName = root.querySelector("#appName");
  const appDescription = root.querySelector("#appDescription");

  if (typeof wasmModule.app_manifest === "function") {
    const manifest = await wasmModule.app_manifest().catch(() => null);
    if (manifest?.manifest?.name && appName) {
      appName.textContent = manifest.manifest.name;
      document.title = manifest.manifest.name;
    }
    if (manifest?.manifest?.description && appDescription) {
      appDescription.textContent = manifest.manifest.description;
    }
  }

  const result = await browserApi.runtime
    .sendMessage({ type: "settings:get" })
    .catch(() => null);
  const settings = result?.settings ?? { enabled: true, autoRender: false };

  if (enabled) enabled.checked = settings.enabled;
  if (autoRender) autoRender.checked = settings.autoRender;

  const save = () =>
    browserApi.runtime
      .sendMessage({
        type: "settings:set",
        settings: {
          enabled: enabled?.checked ?? true,
          autoRender: autoRender?.checked ?? false,
        },
      })
      .catch((err) => console.error("settings:set failed", err));

  enabled?.addEventListener("change", save);
  autoRender?.addEventListener("change", save);
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------
async function main() {
  await initWasm({ module_or_path: chrome.runtime.getURL("vendor/runtime_bg.wasm") });

  if (typeof wasmModule.render_popup === "function") {
    await runWasmDrivenPopup();
  } else {
    await runSettingsPopup();
  }
}

main().catch((error) => console.error("popup init failed", error));

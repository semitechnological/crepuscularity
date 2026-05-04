import init, * as runtime from "../vendor/runtime.js";

const root = document.getElementById("root");
const wasmBytes = await fetch("../vendor/runtime_bg.wasm").then((response) => response.arrayBuffer());
await init({ module_or_path: wasmBytes });

async function render() {
  const state = await runtime.settings_get().then((result) => result.settings).catch(() => ({ enabled: true }));
  const output = runtime.render_popup(state);
  root.innerHTML = output.html;
  if (output.css && !document.getElementById("vc-popup-css")) {
    const style = document.createElement("style");
    style.id = "vc-popup-css";
    style.textContent = output.css;
    document.head.append(style);
  }
}

render();

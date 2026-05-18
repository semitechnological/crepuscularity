import init, * as runtime from "../vendor/runtime.js";

const wasmBytes = await fetch("../vendor/runtime_bg.wasm").then((response) => response.arrayBuffer());
await init({ module_or_path: wasmBytes });

if (typeof runtime.popup_main === "function") {
  runtime.popup_main();
} else if (typeof runtime.render_popup === "function") {
  const output = runtime.render_popup({});
  if (output.css) {
    const style = document.createElement("style");
    style.textContent = output.css;
    document.head.append(style);
  }
  document.getElementById("root").innerHTML = output.html ?? "";
}

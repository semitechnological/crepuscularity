(async () => {
  const runtimeApi = globalThis.browser?.runtime ?? globalThis.chrome?.runtime;
  const runtime = await import(runtimeApi.getURL("vendor/runtime.js"));
  const wasmBytes = await fetch(runtimeApi.getURL("vendor/runtime_bg.wasm")).then((response) => response.arrayBuffer());
  await runtime.default({ module_or_path: wasmBytes });

  let enabled = true;
  let state = { mode: "normal", sequence: "", countText: "" };
  let hints = [];
  let hintInput = "";
  let lastFind = "";

  runtime.send_runtime_message({ type: "settings:get" })
    .then((response) => {
      enabled = response?.settings?.enabled !== false;
    })
    .catch(() => {});

  function send(command, extra = {}) {
    runtime.send_runtime_message({ type: "vimium-crepus", command, ...extra }).catch(() => {});
  }

  function isEditable(target) {
    if (!target) return false;
    const tag = target.tagName?.toLowerCase();
    return target.isContentEditable || ["input", "textarea", "select"].includes(tag);
  }

  function isVisible(element) {
    const rect = element.getBoundingClientRect();
    const style = getComputedStyle(element);
    return rect.width > 0 && rect.height > 0 && style.visibility !== "hidden" && style.display !== "none" && rect.bottom >= 0 && rect.right >= 0 && rect.top <= innerHeight && rect.left <= innerWidth;
  }

  function clearHints() {
    for (const hint of hints) hint.marker.remove();
    hints = [];
    hintInput = "";
    if (state.mode === "hints") state = { ...state, mode: "normal" };
  }

  function clearOverlays() {
    clearHints();
    document.querySelectorAll(".vc-overlay,.vc-find").forEach((node) => node.remove());
  }

  function activateHints(newTab) {
    clearOverlays();
    state = { ...state, mode: "hints" };
    const nodes = [...document.querySelectorAll("a[href],button,input,textarea,select,summary,[role='button'],[onclick]")]
      .filter((element) => element instanceof HTMLElement && isVisible(element))
      .slice(0, 400);
    hints = nodes.map((element, index) => {
      const rect = element.getBoundingClientRect();
      const marker = document.createElement("span");
      marker.className = "vc-hint";
      marker.textContent = runtime.hint_label(index);
      marker.style.left = `${Math.max(0, rect.left + scrollX)}px`;
      marker.style.top = `${Math.max(0, rect.top + scrollY)}px`;
      document.documentElement.append(marker);
      return { element, marker, label: marker.textContent, newTab };
    });
  }

  function updateHints(key) {
    const labels = hints.map((hint) => hint.label);
    const next = runtime.update_hint_state(labels, hintInput, key);
    hintInput = next.input;
    hints.forEach((hint, index) => hint.marker.classList.toggle("vc-hint-dim", next.dim[index]));
    if (Number.isInteger(next.selected)) openHint(hints[next.selected]);
  }

  function openHint(hint) {
    const href = hint.element.href;
    clearHints();
    if (hint.newTab && href) send("open-url", { url: href });
    else hint.element.click();
  }

  function showHelp() {
    clearOverlays();
    const overlay = document.createElement("section");
    overlay.className = "vc-overlay";
    overlay.innerHTML = runtime.render_help_overlay();
    overlay.querySelector("button")?.addEventListener("click", clearOverlays);
    document.documentElement.append(overlay);
  }

  function showFind() {
    clearOverlays();
    const form = document.createElement("form");
    form.className = "vc-find";
    const input = document.createElement("input");
    input.type = "search";
    input.autocomplete = "off";
    input.value = lastFind;
    const button = document.createElement("button");
    button.type = "submit";
    button.textContent = "Find";
    form.append(input, button);
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      lastFind = input.value;
      if (lastFind) window.find(lastFind);
    });
    document.documentElement.append(form);
    input.focus();
  }

  async function openClipboard(newTab) {
    try {
      const text = await navigator.clipboard.readText();
      const url = new URL(text.trim());
      if (newTab) send("open-url", { url: url.toString() });
      else location.href = url.toString();
    } catch (_error) {}
  }

  function focusInput() {
    const input = [...document.querySelectorAll("input:not([type='hidden']),textarea,[contenteditable='true']")]
      .find((element) => element instanceof HTMLElement && isVisible(element));
    if (input) {
      input.focus();
      state = { ...state, mode: "insert" };
    }
  }

  function applyEffect(effect) {
    if (!effect) return;
    if (effect.kind === "scroll") window.scrollBy({ left: effect.x, top: effect.y, behavior: "smooth" });
    else if (effect.kind === "half-scroll") window.scrollBy({ left: 0, top: innerHeight * 0.55 * effect.direction * effect.count, behavior: "smooth" });
    else if (effect.kind === "scroll-top") scrollTo({ top: 0, behavior: "smooth" });
    else if (effect.kind === "scroll-bottom") scrollTo({ top: document.documentElement.scrollHeight, behavior: "smooth" });
    else if (effect.kind === "reload") location.reload();
    else if (effect.kind === "history-back") history.back();
    else if (effect.kind === "history-forward") history.forward();
    else if (effect.kind === "clear-overlays") clearOverlays();
    else if (effect.kind === "help") showHelp();
    else if (effect.kind === "hints") activateHints(effect.newTab);
    else if (effect.kind === "find") showFind();
    else if (effect.kind === "find-next" && lastFind) window.find(lastFind, false, effect.reverse);
    else if (effect.kind === "focus-input") focusInput();
    else if (effect.kind === "background") send(effect.command);
    else if (effect.kind === "copy-url") navigator.clipboard.writeText(location.href).catch(() => {});
    else if (effect.kind === "open-clipboard") openClipboard(effect.newTab);
  }

  function keyName(event) {
    if (event.key.length === 1) return event.key;
    if (event.key === "Escape") return "Esc";
    return event.key;
  }

  document.addEventListener("keydown", (event) => {
    if (!enabled) return;
    const key = keyName(event);
    if (state.mode === "hints" && key !== "Esc") {
      event.preventDefault();
      event.stopPropagation();
      if (key.length === 1) updateHints(key);
      return;
    }

    const result = runtime.content_key(state, key, isEditable(event.target));
    state = result.state ?? state;
    if (result.prevent) {
      event.preventDefault();
      event.stopPropagation();
    }
    applyEffect(result.effect);
  }, true);
})();

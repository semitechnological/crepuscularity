(async () => {
  const runtimeApi = globalThis.browser?.runtime ?? globalThis.chrome?.runtime;
  const runtime = await import(runtimeApi.getURL("vendor/runtime.js"));
  const wasmBytes = await fetch(runtimeApi.getURL("vendor/runtime_bg.wasm")).then((r) => r.arrayBuffer());
  await runtime.default({ module_or_path: wasmBytes });

  let enabled = true;
  let state = { mode: "normal", sequence: "", countText: "" };
  let hints = [];
  let hintInput = "";
  let lastFind = "";
  let visualMode = null;
  let settings = null;
  let lastUsedTabId = null;
  let vomnibarDebounceTimer = null;

  async function loadSettings() {
    try {
      const resp = await runtime.send_runtime_message({ type: "settings:get" });
      settings = resp?.settings ?? {};
    } catch (_) { settings = {}; }
  }

  runtime.settings_seed().then(async () => {
    await loadSettings();
    checkExclusion();
  }).catch(() => {});

  function setting(key, fallback) {
    if (settings && settings[key] !== undefined) return settings[key];
    return fallback;
  }

  function checkExclusion() {
    if (!settings) return;
    const rules = settings.exclusionRules || [];
    for (const rule of rules) {
      try {
        const re = new RegExp(
          "^" + rule.pattern.replace(/\*/g, ".*").replace(/\?/g, ".") + "$", "i"
        );
        if (re.test(location.href)) {
          enabled = false;
          return;
        }
      } catch (_) {}
    }
    enabled = true;
  }

  if (setting("grabBackFocus", false)) {
    document.addEventListener("focusin", (ev) => {
      if (enabled && ev.target && isEditable(ev.target) && state.mode !== "insert") {
        ev.target.blur();
      }
    }, true);
  }

  function send(command, extra = {}) {
    runtime.send_runtime_message({ type: "vimium-crepus", command, ...extra }).catch(() => {});
  }

  function isEditable(target) {
    if (!target) return false;
    const tag = target.tagName?.toLowerCase();
    return target.isContentEditable || ["input", "textarea", "select"].includes(tag);
  }

  function isVisible(el) {
    const r = el.getBoundingClientRect();
    const s = getComputedStyle(el);
    return r.width > 0 && r.height > 0 && s.visibility !== "hidden" &&
      s.display !== "none" && r.bottom >= 0 && r.right >= 0 &&
      r.top <= innerHeight && r.left <= innerWidth;
  }

  function clearHints() {
    for (const h of hints) h.marker.remove();
    hints = [];
    hintInput = "";
    if (state.mode === "hints") state = { ...state, mode: "normal" };
  }

  function clearOverlays() {
    clearHints();
    clearVomnibar();
    clearVisualMode();
    clearHUD();
    document.querySelectorAll(".vc-overlay,.vc-find,.vc-hint").forEach((n) => n.remove());
  }

  function showHUD(text, duration = 1500) {
    clearHUD();
    const hud = document.createElement("div");
    hud.className = "vc-hud";
    hud.textContent = text;
    document.documentElement.append(hud);
    setTimeout(() => hud.remove(), duration);
  }

  function clearHUD() {
    document.querySelectorAll(".vc-hud").forEach((n) => n.remove());
  }

  function activateHints(newTab, kind = "click") {
    clearOverlays();
    state = { ...state, mode: "hints" };
    const chars = setting("linkHintCharacters", "sadfjklewcmpgh");
    const filterHints = setting("filterLinkHints", false);

    const selectors = [
      "a[href]", "button", "input:not([type='hidden'])", "textarea", "select",
      "summary", "[role='button']", "[onclick]", "[contenteditable='true']",
      "[tabindex]:not([tabindex='-1'])"
    ];

    const nodes = [...document.querySelectorAll(selectors.join(","))]
      .filter((el) => el instanceof HTMLElement && isVisible(el))
      .slice(0, 600);

    hints = nodes.map((el, i) => {
      const r = el.getBoundingClientRect();
      const m = document.createElement("span");
      m.className = "vc-hint";
      m.textContent = runtime.hint_label(i);
      m.style.left = `${Math.max(2, r.left + scrollX)}px`;
      m.style.top = `${Math.max(2, r.top + scrollY)}px`;
      document.documentElement.append(m);
      return { element: el, marker: m, label: m.textContent, kind };
    });
  }

  function updateHints(key) {
    const labels = hints.map((h) => h.label);
    const next = runtime.update_hint_state(labels, hintInput, key);
    hintInput = next.input;
    hints.forEach((h, i) => h.marker.classList.toggle("vc-hint-dim", next.dim[i]));
    if (Number.isInteger(next.selected)) openHint(hints[next.selected]);
  }

  function openHint(hint) {
    const el = hint.element;
    const href = el.href;
    clearHints();

    if (hint.kind === "copy-url" && href) {
      navigator.clipboard.writeText(href).catch(() => {});
      showHUD(`Copied: ${href}`);
      return;
    }
    if (hint.kind === "download" && href) {
      const a = document.createElement("a");
      a.href = href; a.download = ""; a.click();
      return;
    }
    if (hint.kind === "hover") {
      el.dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
      return;
    }
    if (hint.kind === "focus") {
      el.focus();
      return;
    }
    el.click();
  }

  function showHelp() {
    clearOverlays();
    const showAdv = setting("helpDialog_showAdvancedCommands", false);
    const overlay = document.createElement("section");
    overlay.className = "vc-overlay";
    overlay.innerHTML = runtime.render_help_overlay(showAdv);
    overlay.querySelector(".vc-overlay-close")?.addEventListener("click", clearOverlays);
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
    input.className = "vc-find-input";
    const btn = document.createElement("button");
    btn.type = "submit";
    btn.textContent = "Find";
    btn.className = "vc-find-btn";
    form.append(input, btn);
    form.addEventListener("submit", (ev) => {
      ev.preventDefault();
      lastFind = input.value;
      if (lastFind) window.find(lastFind, false, setting("regexFindMode", false));
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
    } catch (_) {}
  }

  function focusInput() {
    const sel = [...document.querySelectorAll(
      "input:not([type='hidden']):not([type='submit']):not([type='button'])," +
      "textarea,[contenteditable='true']"
    )].find((el) => el instanceof HTMLElement && isVisible(el));
    if (sel) { sel.focus(); state = { ...state, mode: "insert" }; }
  }

  function goUpHierarchy() {
    const parts = location.pathname.split("/").filter(Boolean);
    parts.pop();
    location.pathname = "/" + parts.join("/") + (location.search || "");
  }

  function goToRoot() {
    location.pathname = "/";
  }

  function toggleViewSource() {
    location.href = "view-source:" + location.href;
  }

  function enterVisualMode(submode) {
    state = { ...state, mode: submode };
    showHUD(`-- ${submode === "visual-line" ? "VISUAL LINE" : "VISUAL"} --`);
  }

  function clearVisualMode() {
    if (visualMode && state.mode.includes("visual")) {
      visualMode = null;
      state = { ...state, mode: "normal" };
    }
  }

  function copySelection() {
    try {
      const sel = getSelection().toString();
      if (sel) navigator.clipboard.writeText(sel).catch(() => {});
    } catch (_) {}
  }

  function followPattern(kind) {
    const patterns = (setting(kind === "previous" ? "previousPatterns" : "nextPatterns", "") || "")
      .split(",").map((s) => s.trim());
    const links = [...document.querySelectorAll("a")];
    for (const pat of patterns) {
      for (const a of links) {
        const text = (a.textContent || "").trim().toLowerCase();
        if (text === pat || text.includes(pat)) {
          a.click();
          return;
        }
      }
    }
  }

  function cycleFrame(dir) {
    const frames = [...document.querySelectorAll("frame,iframe")];
    if (!frames.length) return;
    let currentIdx = frames.findIndex((f) => f.contentWindow === document.activeElement?.contentWindow);
    if (currentIdx < 0) currentIdx = -1;
    const next = (currentIdx + dir + frames.length) % frames.length;
    frames[next]?.focus();
  }

  function focusMainFrame() {
    if (window.top !== window) window.top.focus();
  }

  let markMode = null;
  let markEntry = null;

  function activateCreateMark() {
    markMode = "create";
    showHUD("Create mark — press a key");
    document.addEventListener("keydown", markKeyHandler, { once: true });
  }

  function activateGotoMark() {
    markMode = "goto";
    showHUD("Go to mark — press a key");
    document.addEventListener("keydown", markKeyHandler, { once: true });
  }

  function markKeyHandler(ev) {
    const key = ev.key;
    if (ev.shiftKey) {
      markEntry = "global:" + key;
      if (markMode === "create") {
        chrome.storage.local.set({
          ["vimiumGlobalMark|" + key]: JSON.stringify({ scrollX, scrollY, url: location.href })
        });
        showHUD("Created global mark: " + key);
      } else {
        chrome.storage.local.get("vimiumGlobalMark|" + key, (items) => {
          const data = items["vimiumGlobalMark|" + key];
          if (data) {
            const pos = JSON.parse(data);
            if (pos.url && pos.url !== location.href) location.href = pos.url;
            else scrollTo(pos.scrollX || 0, pos.scrollY || 0);
            showHUD("Jumped to global mark: " + key);
          } else {
            showHUD("Global mark not set: " + key);
          }
        });
      }
    } else {
      const mkKey = "vimiumMark|" + location.href.split("#")[0] + "|" + key;
      if (markMode === "create") {
        localStorage[mkKey] = JSON.stringify({ scrollX, scrollY, hash: location.hash });
        showHUD("Created local mark: " + key);
      } else {
        const data = localStorage[mkKey];
        if (data) {
          const pos = JSON.parse(data);
          if (pos.hash) location.hash = pos.hash;
          else scrollTo(pos.scrollX || 0, pos.scrollY || 0);
          showHUD("Jumped to local mark: " + key);
        } else {
          showHUD("Local mark not set: " + key);
        }
      }
    }
    markMode = null;
    ev.preventDefault();
    ev.stopPropagation();
  }

  let vomnibarEl = null;
  let vomnibarInput = null;
  let vomnibarMode = null;

  function showVomnibar(opts = {}) {
    clearOverlays();
    vomnibarMode = opts;

    const bar = document.createElement("div");
    bar.className = "vc-vomnibar";
    bar.innerHTML = `<div class="vc-vomnibar-box">
      <input class="vc-vomnibar-input" type="text" autocomplete="off" placeholder="${opts.placeholder || "Search bookmarks, history, and tabs"}">
      <ul class="vc-vomnibar-list"></ul>
    </div>`;
    document.documentElement.append(bar);

    vomnibarEl = bar;
    vomnibarInput = bar.querySelector("input");

    if (opts.initial) vomnibarInput.value = opts.initial;
    vomnibarInput.focus();

    if (opts.urlOnly) {
      vomnibarInput.value = location.href;
      vomnibarInput.select();
    }

    vomnibarInput.addEventListener("input", updateVomnibar);
    vomnibarInput.addEventListener("keydown", vomnibarKeydown);
    bar.addEventListener("click", (ev) => { if (ev.target === bar) clearVomnibar(); });
  }

  function clearVomnibar() {
    if (vomnibarEl) { vomnibarEl.remove(); vomnibarEl = null; }
    vomnibarInput = null;
    vomnibarMode = null;
  }

  let vomnibarSelectedIdx = -1;
  let vomnibarResults = [];

  async function updateVomnibar() {
    const query = vomnibarInput.value.trim();
    if (!query) { vomnibarResults = []; renderVomnibarList(); return; }

    if (vomnibarDebounceTimer) clearTimeout(vomnibarDebounceTimer);
    vomnibarDebounceTimer = setTimeout(async () => {
      const currentQuery = vomnibarInput?.value?.trim();
      if (!currentQuery || currentQuery !== query) return;

      const resolvable = runtime.resolve_navigable(currentQuery);
      let items = [];

      if (resolvable.kind === "url") {
        items.push({ title: resolvable.display || resolvable.url, url: resolvable.url, kind: "navigate" });
      }

      try {
        const mode = (vomnibarMode || {}).bookmarksOnly ? "bookmarks" :
                     (vomnibarMode || {}).tabsOnly ? "tabs" : "full";
        const result = await runtime.query_vomnibar(currentQuery, mode);
        for (const item of (result.items || [])) {
          items.push(item);
        }
      } catch (_) {}

      if (vomnibarInput?.value?.trim() !== currentQuery) return;
      vomnibarResults = items;
      vomnibarSelectedIdx = -1;
      renderVomnibarList();
      vomnibarDebounceTimer = null;
    }, 150);
  }

  function renderVomnibarList() {
    const list = vomnibarEl?.querySelector(".vc-vomnibar-list");
    if (!list) return;
    list.innerHTML = vomnibarResults.map((item, i) => {
      const sel = i === vomnibarSelectedIdx ? " vc-vomnibar-selected" : "";
      const kindLabel = item.kind === "bookmark" ? "[bookmark]" : item.kind === "history" ? "[history]" : item.kind === "tab" ? "[tab]" : "";
      return `<li class="vc-vomnibar-item${sel}" data-idx="${i}">
        <span class="vc-vomnibar-title">${esc(item.title || item.url)}</span>
        <span class="vc-vomnibar-url">${kindLabel} ${esc(item.url)}</span>
      </li>`;
    }).join("");

    list.querySelectorAll(".vc-vomnibar-item").forEach((li) => {
      li.addEventListener("click", () => {
        const idx = parseInt(li.dataset.idx);
        if (idx >= 0 && idx < vomnibarResults.length) commitVomnibar(vomnibarResults[idx]);
      });
    });
  }

  function vomnibarKeydown(ev) {
    if (ev.key === "Escape") { clearVomnibar(); return; }
    if (ev.key === "ArrowDown") {
      ev.preventDefault();
      vomnibarSelectedIdx = Math.min(vomnibarSelectedIdx + 1, vomnibarResults.length - 1);
      renderVomnibarList();
      return;
    }
    if (ev.key === "ArrowUp") {
      ev.preventDefault();
      vomnibarSelectedIdx = Math.max(vomnibarSelectedIdx - 1, -1);
      renderVomnibarList();
      return;
    }
    if (ev.key === "Enter") {
      ev.preventDefault();
      if (vomnibarSelectedIdx >= 0 && vomnibarSelectedIdx < vomnibarResults.length) {
        commitVomnibar(vomnibarResults[vomnibarSelectedIdx]);
      } else {
        const query = vomnibarInput.value.trim();
        if (query) {
          const nav = runtime.resolve_navigable(query);
          commitVomnibar({ url: nav.url, kind: "navigate", title: query });
        }
      }
    }
  }

  function commitVomnibar(item) {
    if (!item || !item.url) { clearVomnibar(); return; }
    const newTab = vomnibarMode?.newTab ?? false;
    clearVomnibar();
    if (newTab) send("open-url", { url: item.url, active: true });
    else location.href = item.url;
  }

  function esc(s) {
    const div = document.createElement("div");
    div.textContent = s;
    return div.innerHTML;
  }

  function applyEffect(effect) {
    if (!effect) return;
    const k = effect.kind;
    const step = setting("scrollStepSize", 60);
    const smooth = setting("smoothScroll", true) ? "smooth" : "auto";

    if (k === "scroll") scrollBy({ left: effect.x, top: effect.y, behavior: smooth });
    else if (k === "half-scroll") scrollBy({ left: 0, top: innerHeight * 0.55 * effect.direction * effect.count, behavior: smooth });
    else if (k === "full-scroll") scrollBy({ left: 0, top: innerHeight * 0.9 * effect.direction * effect.count, behavior: smooth });
    else if (k === "scroll-top") scrollTo({ top: 0, behavior: smooth });
    else if (k === "scroll-bottom") scrollTo({ top: document.documentElement.scrollHeight, behavior: smooth });
    else if (k === "scroll-left") scrollTo({ left: 0, behavior: smooth });
    else if (k === "scroll-right") scrollTo({ left: document.documentElement.scrollWidth, behavior: smooth });
    else if (k === "reload") location.reload();
    else if (k === "history-back") history.back();
    else if (k === "history-forward") history.forward();
    else if (k === "clear-overlays") clearOverlays();
    else if (k === "help") showHelp();
    else if (k === "hints") activateHints(effect.newTab, "click");
    else if (k === "hints-general") activateHints(effect.newTab || false, "click");
    else if (k === "hints-queue") activateHints(false, "queue");
    else if (k === "hints-download") activateHints(false, "download");
    else if (k === "hints-incognito") activateHints(false, "incognito");
    else if (k === "hints-copy-url") activateHints(false, "copy-url");
    else if (k === "find") showFind();
    else if (k === "find-next" && lastFind) window.find(lastFind, false, effect.reverse, false, true, false);
    else if (k === "find-selected") {
      const sel = getSelection().toString();
      if (sel) { lastFind = sel; window.find(sel, false, effect.reverse); }
    }
    else if (k === "focus-input") focusInput();
    else if (k === "background") send(effect.command);
    else if (k === "copy-url") navigator.clipboard.writeText(location.href).catch(() => {});
    else if (k === "open-clipboard") openClipboard(effect.newTab);
    else if (k === "enter-visual") enterVisualMode(effect.mode === "visual-line" ? "visual-line" : "visual");
    else if (k === "create-mark") activateCreateMark();
    else if (k === "goto-mark") activateGotoMark();
    else if (k === "vomnibar") showVomnibar({ newTab: effect.newTab, placeholder: "Search bookmarks, history, and tabs" });
    else if (k === "vomnibar-bookmarks") showVomnibar({ newTab: effect.newTab, placeholder: "Search bookmarks", bookmarksOnly: true });
    else if (k === "vomnibar-tabs") showVomnibar({ newTab: false, placeholder: "Search open tabs", tabsOnly: true });
    else if (k === "vomnibar-edit-url") showVomnibar({ newTab: effect.newTab, urlOnly: true });
    else if (k === "go-up") goUpHierarchy();
    else if (k === "go-root") goToRoot();
    else if (k === "view-source") toggleViewSource();
    else if (k === "follow-pattern") followPattern(effect.pattern);
    else if (k === "cycle-frame") cycleFrame(effect.direction);
    else if (k === "focus-main-frame") focusMainFrame();
    else if (k === "pass-next-key") {
      state = { ...state, mode: "pass-next" };
      showHUD("Pass next key...");
    }
  }

  document.addEventListener("keydown", (ev) => {
    if (!enabled) return;
    const key = runtime.key_name(ev.key);

    if (state.mode === "pass-next") {
      state = { ...state, mode: "normal" };
      return;
    }

    if (vomnibarEl) return;

    if (state.mode === "hints" && key !== "Esc") {
      ev.preventDefault();
      ev.stopPropagation();
      if (key.length === 1) updateHints(key);
      return;
    }

    if (markMode && key !== "Esc") return;

    const result = runtime.content_key(state, key, isEditable(ev.target));
    state = result.state ?? state;
    if (result.prevent) {
      ev.preventDefault();
      ev.stopPropagation();
    }
    applyEffect(result.effect);
  }, true);
})();
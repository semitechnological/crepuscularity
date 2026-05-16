import init, * as runtime from "./pkg/runtime.js";

const delegatedEvents = [
  "click",
  "input",
  "change",
  "keydown",
  "keyup",
  "mousedown",
  "mouseup",
  "mousemove",
  "scroll",
];

let currentBundle = null;
let currentRoot = null;
let currentIslandManifest = { islands: {} };

function setRootHtml(root, html) {
  const doc = new DOMParser().parseFromString(html, "text/html");
  root.replaceChildren(...doc.body.childNodes);
}

/**
 * Optional client island: element with data-slot-words="a|b|c" (pipe-separated phrases).
 * Slot-machine style blur + slide between phrases — same page can add more islands later
 * (signals, fetch, etc.) without changing the .crepus → WASM first paint.
 */
function parseSlotWords(raw) {
  const t = (raw || "").trim();
  if (t.startsWith("[")) {
    try {
      const w = JSON.parse(t);
      return Array.isArray(w) ? w.map(String).filter(Boolean) : [];
    } catch {
      return [];
    }
  }
  return t
    .split("|")
    .map((s) => s.trim())
    .filter(Boolean);
}

function initSlotRotate(root) {
  const el = root.querySelector("[data-slot-words]");
  if (!el) return;

  const raw = el.getAttribute("data-slot-words") || "";
  const words = parseSlotWords(raw);
  if (words.length < 2) return;

  el.classList.add("crepus-slot");
  el.setAttribute("aria-live", "polite");
  el.innerHTML = "";

  const inner = document.createElement("span");
  inner.className = "crepus-slot__inner";
  inner.textContent = words[0];
  el.appendChild(inner);

  el.style.display = 'inline-block';
  el.style.width = el.offsetWidth + 'px';
  el.style.transition = 'width 0.3s ease-in-out';

  let i = 0;
  const intervalAttr = el.getAttribute("data-slot-interval");
  const duration = intervalAttr ? Number(intervalAttr) || 3200 : 3200;

  const tick = () => {
    i = (i + 1) % words.length;
    inner.style.transition =
      "opacity 0.32s ease, filter 0.32s ease";
    inner.style.opacity = "0";
    inner.style.filter = "blur(6px)";

    window.setTimeout(() => {
      inner.textContent = words[i];
      el.style.width = el.offsetWidth + 'px';
      requestAnimationFrame(() => {
        inner.style.opacity = "1";
        inner.style.filter = "blur(0)";
      });
    }, 280);
  };

  window.setInterval(tick, duration);
}

function initInteractive(root) {
  initSlotRotate(root);
  initWebIslands(root);
}

function parseIslandProps(el) {
  const raw = el.getAttribute("data-crepus-island-props") || "{}";
  try {
    const value = JSON.parse(raw);
    return value && typeof value === "object" ? value : {};
  } catch {
    return {};
  }
}

async function loadIslandManifest() {
  try {
    const res = await fetch("./crepus-islands.json", { cache: "no-store" });
    if (!res.ok) return { islands: {} };
    const manifest = await res.json();
    return manifest && typeof manifest === "object" ? manifest : { islands: {} };
  } catch {
    return { islands: {} };
  }
}

function resolveIslandModule(src) {
  const entry = currentIslandManifest?.islands?.[src];
  return entry?.module || src;
}

function initWebIslands(root) {
  const islands = root.querySelectorAll("[data-crepus-island]");
  for (const el of islands) {
    if (el.dataset.crepusIslandMounted === "true") continue;
    el.dataset.crepusIslandMounted = "true";

    const src = el.getAttribute("data-crepus-island-src");
    if (!src) continue;

    const props = parseIslandProps(el);
    const modulePath = resolveIslandModule(src);
    import(modulePath)
      .then((mod) => {
        const mount = typeof mod.mount === "function" ? mod.mount : mod.default;
        if (typeof mount === "function") {
          return mount(el, props, { runtime, rerender });
        }
        if (mount && typeof mount.mount === "function") {
          return mount.mount(el, props, { runtime, rerender });
        }
        throw new Error(`Missing island mount export: ${src}`);
      })
      .catch((error) => {
        console.error(`crepus island failed: ${src}`, error);
        el.textContent = "";
        const pre = document.createElement("pre");
        pre.style.cssText = "white-space:pre-wrap;color:#ef4444;font-family:monospace";
        pre.textContent = String(error);
        el.appendChild(pre);
      });
  }
}

function findHandlerTarget(root, eventName, startNode) {
  const attr = `data-on${eventName}`;
  let node = startNode instanceof Element ? startNode : startNode?.parentElement;
  while (node && node !== root) {
    if (node.hasAttribute?.(attr)) {
      return node;
    }
    node = node.parentElement;
  }
  return root?.hasAttribute?.(attr) ? root : null;
}

function initDelegatedEvents(root) {
  if (!root || root.dataset.crepusEventsBound === "true") return;
  root.dataset.crepusEventsBound = "true";

  for (const eventName of delegatedEvents) {
    root.addEventListener(eventName, (event) => {
      const target = findHandlerTarget(root, eventName, event.target);
      if (!target) return;

      const handlerName = target.getAttribute(`data-on${eventName}`);
      const handler = handlerName ? runtime[handlerName] : null;
      if (typeof handler !== "function") {
        console.warn(`Missing WASM handler export: ${handlerName}`);
        return;
      }

      Promise.resolve()
        .then(() => handler())
        .then(() => rerender())
        .catch((error) => {
          console.error(`crepus event handler failed: ${handlerName}`, error);
        });
    });
  }
}

function rerender() {
  if (!currentRoot || !currentBundle) return;
  setRootHtml(currentRoot, runtime.crepus_render(JSON.stringify(currentBundle)));
  initDelegatedEvents(currentRoot);
  initInteractive(currentRoot);
}

async function main() {
  await init();
  const res = await fetch("./crepus-bundle.json", { cache: "no-store" });
  if (!res.ok) {
    throw new Error(`crepus-bundle.json: HTTP ${res.status}`);
  }
  const bundle = await res.json();
  currentBundle = bundle;
  currentIslandManifest = await loadIslandManifest();
  const root = document.getElementById("crepus-root");
  if (root) {
    currentRoot = root;
    setRootHtml(root, runtime.crepus_render(JSON.stringify(bundle)));
    initDelegatedEvents(root);
    initInteractive(root);
  }
}

main().catch((e) => {
  console.error(e);
  const root = document.getElementById("crepus-root");
  if (root) {
    root.textContent = "";
    const wrap = document.createElement("div");
    wrap.style.cssText = "padding:2rem;font-family:monospace;color:#ef4444";
    const strong = document.createElement("strong");
    strong.textContent = "WASM boot failed";
    const pre = document.createElement("pre");
    pre.style.whiteSpace = "pre-wrap";
    pre.textContent = String(e);
    wrap.appendChild(strong);
    wrap.appendChild(pre);
    root.appendChild(wrap);
  }
});

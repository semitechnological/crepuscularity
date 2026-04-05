import init, { crepus_render } from "./pkg/runtime.js";

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

  let i = 0;
  const intervalAttr = el.getAttribute("data-slot-interval");
  const duration = intervalAttr ? Number(intervalAttr) || 3200 : 3200;

  const tick = () => {
    i = (i + 1) % words.length;
    inner.style.transition =
      "transform 0.38s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.32s ease, filter 0.32s ease";
    inner.style.transform = "translateY(0.35em)";
    inner.style.opacity = "0";
    inner.style.filter = "blur(6px)";

    window.setTimeout(() => {
      inner.textContent = words[i];
      inner.style.transform = "translateY(-0.35em)";
      inner.style.opacity = "0";
      requestAnimationFrame(() => {
        inner.style.transform = "translateY(0)";
        inner.style.opacity = "1";
        inner.style.filter = "blur(0)";
      });
    }, 280);
  };

  window.setInterval(tick, duration);
}

function initInteractive(root) {
  initSlotRotate(root);
}

async function main() {
  await init();
  const res = await fetch("./crepus-bundle.json", { cache: "no-store" });
  if (!res.ok) {
    throw new Error(`crepus-bundle.json: HTTP ${res.status}`);
  }
  const bundle = await res.json();
  const html = crepus_render(JSON.stringify(bundle));
  const root = document.getElementById("crepus-root");
  if (root) {
    setRootHtml(root, html);
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

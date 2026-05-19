(async () => {
  const runtimeApi = globalThis.browser?.runtime ?? globalThis.chrome?.runtime;
  if (!runtimeApi) {
    throw new Error("browser runtime API is unavailable");
  }

  const topFrame = (() => {
    try {
      return globalThis.top === globalThis;
    } catch (_) {
      return false;
    }
  })();

  const cacheKey = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const runtimeUrl = `${runtimeApi.getURL("vendor/runtime.js")}?v=${cacheKey}`;
  const wasmUrl = `${runtimeApi.getURL("vendor/runtime_bg.wasm")}?v=${cacheKey}`;

  let browserApi;
  let wasmModule;

  try {
    [{ browserApi }, wasmModule] = await Promise.all([
      import(`${runtimeApi.getURL("src/browser-shim.js")}?v=${cacheKey}`),
      import(runtimeUrl)
    ]);

    const wasmBytes = await fetch(wasmUrl, { cache: "no-store" }).then((response) => response.arrayBuffer());
    await wasmModule.default({ module_or_path: wasmBytes });
  } catch (error) {
    if (topFrame) throw error;
    if (!(error instanceof WebAssembly.CompileError)) throw error;
    return;
  }

  const settingsResponse = await browserApi.runtime.sendMessage({ type: "settings:get" });
  const settings = settingsResponse?.settings ?? { enabled: true, autoRender: false };
  if (!settings.enabled) {
    return;
  }

  const unocssSource = await fetch(`${runtimeApi.getURL("vendor/unocss.js")}?v=${cacheKey}`)
    .then((response) => response.text())
    .catch(() => "");

  if (typeof wasmModule.browser_program_data === "function") {
    runBrowserProgramData(browserApi, JSON.parse(wasmModule.browser_program_data()))
      .catch((error) => console.error("crepuscularity browser program failed", error));
  }

  const mounted = new WeakSet();
  const pending = new Set();
  let flushScheduled = false;

  function normalizeWidgetText(text) {
    return text
      .replace(/&lt;/gi, "<")
      .replace(/&gt;/gi, ">")
      .replace(/&amp;/gi, "&");
  }

  function hasAnywhereContent(node) {
    const text = normalizeWidgetText(node.textContent || "");
    const html = node.innerHTML || "";
    return (
      text.includes("```") ||
      text.includes("<ai-anywhere") ||
      html.includes("<ai-anywhere") ||
      html.includes("&lt;ai-anywhere")
    );
  }

  function widgetTextFromPre(pre) {
    const code = pre.querySelector("code");
    return normalizeWidgetText((code ?? pre).textContent || "");
  }

  function collectWidgetPres() {
    const pres = [];
    document.querySelectorAll("pre").forEach((node) => {
      if (node instanceof HTMLElement && hasAnywhereContent(node)) {
        pres.push(node);
      }
    });
    return pres.filter(
      (pre) => !pres.some((other) => other !== pre && other.contains(pre))
    );
  }

  function dedupeNodes(nodes) {
    const list = [...nodes];
    return list.filter((node) => !list.some((other) => other !== node && other.contains(node)));
  }

  function wasmField(value, key) {
    if (value == null) return undefined;
    if (value instanceof Map) return value.get(key);
    if (typeof value === "object") return value[key];
    return undefined;
  }

  function extractRenderedHtml(rendered, label) {
    const html = wasmField(rendered, "html");
    if (typeof html !== "string") {
      console.error(`crepuscularity: ${label} returned no html`, rendered);
      return null;
    }
    return html;
  }

  async function runBrowserProgramData(api, program) {
    const vars = {};

    function resolveExpr(expr) {
      if (expr !== null && typeof expr === "object" && "$var" in expr) return vars[expr.$var];
      return expr;
    }
    function resolveObj(obj) {
      if (typeof obj !== "object" || obj === null) return obj;
      return Object.fromEntries(Object.entries(obj).map(([k, v]) => [k, resolveExpr(v)]));
    }

    for (const b of (program.bindings ?? [])) {
      if (b.type === "storage_get") {
        const res = await api.storage[b.area].get({ [b.key]: undefined }).catch(() => ({}));
        vars[b.name] = res[b.key];
      } else if (b.type === "runtime_message") {
        vars[b.name] = await api.runtime.sendMessage(resolveObj(b.payload)).catch(() => null);
      }
    }

    for (const s of (program.statements ?? [])) {
      if (s.type === "storage_set") {
        await api.storage[s.area].set({ [s.key]: resolveExpr(s.value) }).catch(() => {});
      } else if (s.type === "runtime_send") {
        await api.runtime.sendMessage(resolveObj(s.payload)).catch(() => {});
      } else if (s.type === "console_log") {
        console.log(...(s.args ?? []).map(resolveExpr));
      }
    }
  }

  function codeBlockReplaceTarget(pre) {
    return pre;
  }

  function mountWidgetShell(pre, shell) {
    if (!(pre instanceof HTMLElement) || mounted.has(pre)) {
      return;
    }
    mounted.add(pre);

    const wrapper = document.createElement("div");
    wrapper.className = "aa-mount";
    wrapper.append(shell);

    const replaceTarget = codeBlockReplaceTarget(pre);
    if (replaceTarget?.isConnected) {
      replaceTarget.replaceWith(wrapper);
      return;
    }
    console.warn("crepuscularity: code block not connected, skip mount");
  }

  function createShell(spec) {
    let rendered;
    try {
      rendered = wasmModule.render_frontend({
        entry: "views/ui.crepus#Panel",
        props: {
          title: spec.title,
          format: spec.format,
          source: spec.source,
          warning: "Runs inside a sandboxed iframe. Review AI code before execution.",
          payload: JSON.stringify(spec, null, 2),
          show_source: true,
          badges: [
            { label: spec.format, tone: "accent" },
            { label: spec.source, tone: "neutral" }
          ],
          actions: [
            { label: "Render widget", action: "render", kind: "primary" },
            { label: "Show source", action: "toggle-source", kind: "secondary" }
          ]
        }
      });
    } catch (error) {
      console.error("crepuscularity: failed to render widget shell", error);
      return null;
    }

    const html = extractRenderedHtml(rendered, "render_frontend(Panel)");
    if (!html) return null;

    const mount = document.createElement("section");
    mount.className = "aa-shell";
    mount.innerHTML = html;

    const runButton = mount.querySelector("[data-action='render']");
    const sourceButton = mount.querySelector("[data-action='toggle-source']");
    const details = mount.querySelector("[data-role='payload']");
    const frameHost = mount.querySelector("[data-role='frame-host']");

    sourceButton?.addEventListener("click", () => {
      if (details) {
        details.hidden = !details.hidden;
      }
    });

    runButton?.addEventListener("click", () => {
      if (!frameHost || frameHost.querySelector("iframe")) {
        return;
      }
      let frameDoc;
      try {
        frameDoc = wasmModule.render_frame_doc({ ...spec, unocss: unocssSource });
      } catch (error) {
        console.error("crepuscularity: failed to render widget frame", error);
        return;
      }
      const srcdoc = wasmField(frameDoc, "srcdoc");
      if (typeof srcdoc !== "string") {
        console.error("crepuscularity: render_frame_doc returned no srcdoc", frameDoc);
        return;
      }
      const frame = document.createElement("iframe");
      frame.className = "aa-widget-frame";
      frame.sandbox = "allow-scripts";
      frame.srcdoc = srcdoc;
      frameHost.append(frame);
      runButton.setAttribute("disabled", "disabled");
      runButton.textContent = "Rendered";
    });

    if (settings.autoRender) {
      queueMicrotask(() => runButton?.click());
    }

    return mount;
  }

  function createAnywhereShell(widget) {
    const title = widget.title || widget.widget_type || "Widget";
    const widgetType = widget.widget_type || "widget";

    let rendered;
    try {
      rendered = wasmModule.render_frontend({
        entry: "views/ui.crepus#AnywhereShell",
        props: { title, widget_type: widgetType }
      });
    } catch (error) {
      console.error("crepuscularity: failed to render anywhere shell", error);
      return null;
    }

    const html = extractRenderedHtml(rendered, "render_frontend(AnywhereShell)");
    if (!html) return null;

    const mount = document.createElement("section");
    mount.className = "aa-shell";
    mount.innerHTML = html;

    const runButton = mount.querySelector("[data-action='render']");
    const frameHost = mount.querySelector("[data-role='frame-host']");

    runButton?.addEventListener("click", () => {
      if (!frameHost || frameHost.querySelector("iframe")) {
        return;
      }
      let frameDoc;
      try {
        frameDoc = wasmModule.render_anywhere_frame_doc({ ...widget, unocss: unocssSource });
      } catch (error) {
        console.error("crepuscularity: failed to render anywhere frame", error);
        return;
      }
      const srcdoc = wasmField(frameDoc, "srcdoc");
      if (typeof srcdoc !== "string") {
        console.error("crepuscularity: render_anywhere_frame_doc returned no srcdoc", frameDoc);
        return;
      }
      const frame = document.createElement("iframe");
      frame.className = "aa-widget-frame";
      frame.sandbox = "allow-scripts";
      frame.srcdoc = srcdoc;
      frameHost.append(frame);
      runButton.setAttribute("disabled", "disabled");
      runButton.textContent = "Rendered";
    });

    if (settings.autoRender) {
      queueMicrotask(() => runButton?.click());
    }

    return mount;
  }

  function shellForPreText(text) {
    if (text.includes("<ai-anywhere")) {
      let widgets;
      try {
        widgets = wasmModule.extract_widgets(text);
      } catch (error) {
        console.error("crepuscularity: failed to parse <ai-anywhere> tags", error);
        return null;
      }
      if (Array.isArray(widgets) && widgets.length > 0) {
        return createAnywhereShell(widgets[0]);
      }
    }

    if (text.includes("```")) {
      let specs;
      try {
        specs = wasmModule.extract_specs(text);
      } catch (error) {
        console.error("crepuscularity: failed to parse widget blocks", error);
        return null;
      }
      if (Array.isArray(specs) && specs.length > 0) {
        return createShell(specs[0]);
      }
    }

    return null;
  }

  function enhanceCodeBlock(pre) {
    if (!(pre instanceof HTMLElement) || mounted.has(pre)) {
      return;
    }

    const shell = shellForPreText(widgetTextFromPre(pre));
    if (!shell) {
      return;
    }

    mountWidgetShell(pre, shell);
  }

  function queueNode(node) {
    if (!(node instanceof HTMLElement)) {
      return;
    }
    if (node.matches("pre") && hasAnywhereContent(node)) {
      pending.add(node);
      return;
    }
    node.querySelectorAll("pre").forEach((pre) => {
      if (pre instanceof HTMLElement && hasAnywhereContent(pre)) {
        pending.add(pre);
      }
    });
  }

  function flushPending() {
    flushScheduled = false;
    const nodes = dedupeNodes([...pending]);
    pending.clear();
    for (const node of nodes) {
      enhanceCodeBlock(node);
    }
  }

  function scheduleFlush() {
    if (flushScheduled) {
      return;
    }
    flushScheduled = true;
    requestAnimationFrame(flushPending);
  }

  for (const pre of collectWidgetPres()) {
    pending.add(pre);
  }
  scheduleFlush();

  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      mutation.addedNodes.forEach((node) => queueNode(node));
    }
    if (pending.size > 0) {
      scheduleFlush();
    }
  });
  observer.observe(document.documentElement, { subtree: true, childList: true });

  if (typeof wasmModule.content_main === "function") {
    wasmModule.content_main();
  }
})();

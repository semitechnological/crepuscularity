(async () => {
  const [{ browserApi }, wasmModule] = await Promise.all([
    import(chrome.runtime.getURL("src/browser-shim.js")),
    import(chrome.runtime.getURL("vendor/runtime.js"))
  ]);

  await wasmModule.default({ module_or_path: chrome.runtime.getURL("vendor/runtime_bg.wasm") });

  const settingsResponse = await browserApi.runtime.sendMessage({ type: "settings:get" });
  const settings = settingsResponse?.settings ?? { enabled: true, autoRender: false };
  if (!settings.enabled) {
    return;
  }

  const unocssSource = await fetch(chrome.runtime.getURL('vendor/unocss.js')).then(r => r.text()).catch(() => '');

  // Run the app's browser program using the data-driven API (CSP-safe — no blob URL import).
  if (typeof wasmModule.browser_program_data === "function") {
    runBrowserProgramData(browserApi, JSON.parse(wasmModule.browser_program_data()))
      .catch((error) => console.error("crepuscularity browser program failed", error));
  }

  const messageSelectors = [
    "[data-message-author-role='assistant']",
    ".font-claude-message",
    ".model-response",
    "main [data-testid*='message']",
    "main article"
  ];

  const seen = new WeakSet();
  const pending = new Set();
  let flushScheduled = false;

  function hasAnywhereContent(node) {
    const text = node.textContent || "";
    return text.includes("```") || text.includes("<ai-anywhere");
  }

  function findMessages() {
    const nodes = new Set();
    for (const selector of messageSelectors) {
      document.querySelectorAll(selector).forEach((node) => {
        if (node instanceof HTMLElement && hasAnywhereContent(node)) {
          nodes.add(node);
        }
      });
    }
    return [...nodes];
  }

  // Interprets a browser program data structure produced by BrowserProgram::emit_data().
  // Resolves { "$var": "name" } references against the `vars` map built from bindings.
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

  function createShell(spec) {
    const rendered = wasmModule.render_frontend({
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

    const mount = document.createElement("section");
    mount.className = "aa-shell";
    mount.innerHTML = rendered.html;

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
      const frame = document.createElement("iframe");
      frame.className = "aa-widget-frame";
      frame.sandbox = "allow-scripts";
      frame.srcdoc = wasmModule.render_frame_doc({ ...spec, unocss: unocssSource }).srcdoc;
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

    const rendered = wasmModule.render_frontend({
      entry: "views/ui.crepus#AnywhereShell",
      props: { title, widget_type: widgetType }
    });

    const mount = document.createElement("section");
    mount.className = "aa-shell";
    mount.innerHTML = rendered.html;

    const runButton = mount.querySelector("[data-action='render']");
    const frameHost = mount.querySelector("[data-role='frame-host']");

    runButton?.addEventListener("click", () => {
      if (!frameHost || frameHost.querySelector("iframe")) {
        return;
      }
      const frame = document.createElement("iframe");
      frame.className = "aa-widget-frame";
      frame.sandbox = "allow-scripts";
      frame.srcdoc = wasmModule.render_anywhere_frame_doc({ ...widget, unocss: unocssSource }).srcdoc;
      frameHost.append(frame);
      runButton.setAttribute("disabled", "disabled");
      runButton.textContent = "Rendered";
    });

    if (settings.autoRender) {
      queueMicrotask(() => runButton?.click());
    }

    return mount;
  }

  function enhanceMessage(messageNode) {
    if (seen.has(messageNode)) {
      return;
    }

    const text = messageNode.textContent || "";
    let hasContent = false;
    const host = document.createElement("div");

    if (text.includes("<ai-anywhere")) {
      let widgets;
      try {
        widgets = wasmModule.extract_widgets(text);
      } catch (error) {
        console.error("crepuscularity: failed to parse <ai-anywhere> tags", error);
      }
      if (Array.isArray(widgets) && widgets.length > 0) {
        hasContent = true;
        for (const widget of widgets) {
          host.append(createAnywhereShell(widget));
        }
      }
    }

    if (text.includes("```")) {
      let specs;
      try {
        specs = wasmModule.extract_specs(text);
      } catch (error) {
        console.error("crepuscularity: failed to parse widget blocks", error);
      }
      if (Array.isArray(specs) && specs.length > 0) {
        hasContent = true;
        for (const spec of specs) {
          host.append(createShell(spec));
        }
      }
    }

    if (!hasContent) {
      return;
    }

    seen.add(messageNode);
    messageNode.append(host);
  }

  function queueNode(node) {
    if (!(node instanceof HTMLElement)) {
      return;
    }
    if (hasAnywhereContent(node)) {
      pending.add(node);
    }
    for (const selector of messageSelectors) {
      if (node.matches?.(selector)) {
        pending.add(node);
      }
      node.querySelectorAll?.(selector).forEach((match) => {
        if (match instanceof HTMLElement && hasAnywhereContent(match)) {
          pending.add(match);
        }
      });
    }
  }

  function flushPending() {
    flushScheduled = false;
    const nodes = [...pending];
    pending.clear();
    for (const node of nodes) {
      enhanceMessage(node);
    }
  }

  function scheduleFlush() {
    if (flushScheduled) {
      return;
    }
    flushScheduled = true;
    requestAnimationFrame(flushPending);
  }

  for (const node of findMessages()) {
    pending.add(node);
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
})();

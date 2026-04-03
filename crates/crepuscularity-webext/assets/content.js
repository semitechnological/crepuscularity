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

  const templateFiles = await loadTemplateFiles();
  const browserProgramSource = wasmModule.browser_program();
  const browserProgramUrl = URL.createObjectURL(new Blob([browserProgramSource], { type: "text/javascript" }));
  const { runBrowserProgram } = await import(browserProgramUrl);
  URL.revokeObjectURL(browserProgramUrl);
  runBrowserProgram(browserApi).catch((error) => console.error("anywhere browser program failed", error));

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

  async function loadTemplateFiles() {
    const paths = ["views/ui.crepus"];
    const entries = await Promise.all(
      paths.map(async (path) => {
        const response = await fetch(chrome.runtime.getURL(path));
        return [path, await response.text()];
      })
    );
    return Object.fromEntries(entries);
  }

  function buildFrameDoc(spec) {
    return `<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>html,body{margin:0;padding:0;background:#fffdf8;color:#111;font-family:"IBM Plex Sans",sans-serif;}body{padding:16px;}${spec.css || ""}</style>
  <script>${unocssSource}<\/script>
</head>
<body>
  ${spec.html || "<p>Widget had no HTML payload.</p>"}
  <script type="module">${spec.js || ""}<\/script>
</body>
</html>`;
  }

  function createShell(spec) {
    const request = {
      entry: "views/ui.crepus#Panel",
      files: templateFiles,
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
    };

    const rendered = wasmModule.render_frontend(request);
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
      frame.srcdoc = buildFrameDoc(spec);
      frameHost.append(frame);
      runButton.setAttribute("disabled", "disabled");
      runButton.textContent = "Rendered";
    });

    if (settings.autoRender) {
      queueMicrotask(() => runButton?.click());
    }

    return mount;
  }

  function buildAnywhereFrameDoc(widget) {
    const data = widget.data ? (() => { try { return JSON.parse(widget.data); } catch { return {}; } })() : {};
    let html = "";
    let css = "";
    let js = "";

    if (widget.ui) {
      if (widget.ui.lang === "html") {
        html = widget.ui.source;
      } else if (widget.ui.lang === "crepus") {
        try {
          const rendered = wasmModule.render_frontend({
            entry: "__ai_widget__#Widget",
            files: { "__ai_widget__": widget.ui.source },
            props: data
          });
          html = rendered.html || "";
          css = rendered.css || "";
        } catch (err) {
          html = `<pre style="color:red">Crepus render error: ${String(err)}</pre>`;
        }
      }
    }

    if (widget.script) {
      if (widget.script.lang === "mermaid") {
        html = `<div class="mermaid">${widget.script.source}</div>`;
        js = `import('https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.esm.min.mjs').then(m=>m.default.initialize({startOnLoad:true}));`;
      } else if (widget.script.lang === "latex") {
        html = `<div class="latex">${widget.script.source}</div>`;
        js = `import('https://cdn.jsdelivr.net/npm/katex/dist/katex.mjs').then(m=>{document.querySelectorAll('.latex').forEach(el=>m.default.render(el.textContent,el,{throwOnError:false}));});`;
      } else {
        js = widget.script.source;
      }
    }

    return `<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>html,body{margin:0;padding:0;background:#fffdf8;color:#111;font-family:"IBM Plex Sans",sans-serif;}body{padding:16px;}${css}</style>
  <script>${unocssSource}<\/script>
</head>
<body>
  ${html || "<p>Widget had no content.</p>"}
  <script type="module">${js}<\/script>
</body>
</html>`;
  }

  function createAnywhereShell(widget) {
    const title = widget.title || widget.widget_type || "Widget";
    const widgetType = widget.widget_type || "widget";

    const request = {
      entry: "views/ui.crepus#AnywhereShell",
      files: templateFiles,
      props: { title, widget_type: widgetType }
    };

    const rendered = wasmModule.render_frontend(request);
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
      frame.srcdoc = buildAnywhereFrameDoc(widget);
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
        console.error("anywhere failed to parse <ai-anywhere> tags", error);
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
        console.error("anywhere failed to parse widget blocks", error);
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

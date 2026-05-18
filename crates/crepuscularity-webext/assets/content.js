(async () => {
  const runtimeApi = globalThis.browser?.runtime ?? globalThis.chrome?.runtime;
  if (!runtimeApi) {
    throw new Error("browser runtime API is unavailable");
  }
  const cacheKey = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const topFrame = (() => {
    try {
      return globalThis.top === globalThis;
    } catch (_) {
      return false;
    }
  })();
  try {
    const runtime = await import(`${runtimeApi.getURL("vendor/runtime.js")}?v=${cacheKey}`);
    const wasmBytes = await fetch(`${runtimeApi.getURL("vendor/runtime_bg.wasm")}?v=${cacheKey}`, { cache: "no-store" }).then((response) => response.arrayBuffer());
    await runtime.default({ module_or_path: wasmBytes });
    if (typeof runtime.content_main !== "function") {
      throw new Error("runtime.content_main is required");
    }
    runtime.content_main();
  } catch (error) {
    if (topFrame) throw error;
    if (!(error instanceof WebAssembly.CompileError)) throw error;
  }
})();

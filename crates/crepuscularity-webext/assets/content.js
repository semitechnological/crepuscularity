(async () => {
  const runtimeApi = globalThis.browser?.runtime ?? globalThis.chrome?.runtime;
  const cacheKey = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const runtime = await import(`${runtimeApi.getURL("vendor/runtime.js")}?v=${cacheKey}`);
  const wasmBytes = await fetch(`${runtimeApi.getURL("vendor/runtime_bg.wasm")}?v=${cacheKey}`, { cache: "no-store" }).then((response) => response.arrayBuffer());
  await runtime.default({ module_or_path: wasmBytes });
  if (typeof runtime.content_main === "function") runtime.content_main();
})();

(async () => {
  const runtimeApi = globalThis.browser?.runtime ?? globalThis.chrome?.runtime;
  const runtime = await import(runtimeApi.getURL("vendor/runtime.js"));
  const wasmBytes = await fetch(runtimeApi.getURL("vendor/runtime_bg.wasm")).then((response) => response.arrayBuffer());
  await runtime.default({ module_or_path: wasmBytes });
  if (typeof runtime.content_main === "function") runtime.content_main();
})();

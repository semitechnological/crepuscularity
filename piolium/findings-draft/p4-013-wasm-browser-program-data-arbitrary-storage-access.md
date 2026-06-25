---
id: p4-013
phase: L3
slug: wasm-browser-program-data-storage-access
severity: medium
category: unauthorized-storage-access
cwe: CWE-200
status: rejected-fp
rejection_reason: WASM module is bundled with extension; no attacker control path for storage key selection
---

# WASM `browser_program_data` Grants Arbitrary Browser Storage Access

## Summary

The WASM module can define a `browser_program_data()` function that returns a program descriptor with `storage_get` and `storage_set` bindings. The content script executes these bindings against the browser extension's storage API without any access control, allowing the WASM module to read or write ANY key in the extension's `local` or `sync` storage areas.

## Vulnerable Code

`crates/crepuscularity-webext/assets/content.js:126-147`

```javascript
async function runBrowserProgramData(api, program) {
    const vars = {};
    // ...
    for (const b of (program.bindings ?? [])) {
        if (b.type === "storage_get") {
            const res = await api.storage[b.area].get({ [b.key]: undefined });
            vars[b.name] = res[b.key];
        }
        // ...
    }

    for (const s of (program.statements ?? [])) {
        if (s.type === "storage_set") {
            await api.storage[s.area].set({ [s.key]: resolveExpr(s.value) });
        }
        // ...
    }
}
```

The `b.area` and `b.key` values come directly from the WASM module's `browser_program_data()` function, which is loaded from the extension's vendor directory. These values are NOT validate against an allowlist of permitted keys.

## Impact

1. Extension settings can be read or modified by the WASM module
2. Storage keys that control extension behavior (e.g., `enabled`, `autoRender`, `serverUrl`) can be tampered with
3. No key allowlist means the WASM module has unrestricted storage access

## Attacker Control

The WASM module binary (`vendor/runtime_bg.wasm`) is bundled with the extension. An attacker must modify this file through an extension update, file system access, or supply chain compromise.

## Recommended Fix

1. Define an allowlist of storage keys the WASM module is permitted to read/write
2. Validate storage keys against the allowlist before performing operations
3. Reject operations on keys not in the allowlist

```javascript
const ALLOWED_STORAGE_KEYS = new Set([
    "theme", "fontSize", "language", "recentFiles"
]);
if (!ALLOWED_STORAGE_KEYS.has(b.key)) {
    console.warn(`Blocked storage access to unauthorized key: ${b.key}`);
    continue;
}
```

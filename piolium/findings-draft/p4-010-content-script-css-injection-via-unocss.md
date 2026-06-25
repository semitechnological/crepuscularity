---
id: p4-010
phase: L3
slug: content-script-css-injection-via-unocss
severity: medium
category: css-injection
cwe: CWE-79
status: rejected-fp
rejection_reason: requires extension directory write access, which is above normal attacker position; not remotely exploitable
---

# Content Script CSS Injection via UnoCSS Source Fetch

## Summary

The browser extension content script fetches the UnoCSS runtime from a `chrome-extension://` URL via `fetch()` and evaluates it as a script. While the URL is constructed from a fixed path (`vendor/unocss.js`), the `cacheKey` appended as a query parameter uses `Math.random()`. Any path traversal or MITM on the extension's own origin could allow CSS injection via the UnoCSS parsing pipeline.

## Vulnerable Code

`crates/crepuscularity-webext/assets/content.js:42-44`

```javascript
const unocssSource = await fetch(`${runtimeApi.getURL("vendor/unocss.js")}?v=${cacheKey}`)
    .then((response) => response.text())
    .catch(() => "");
```

The `unocssSource` is later passed to WASM functions and used in shadow DOM rendering:

```javascript
wasmModule.render_anywhere_parts({ widget, unocss: unocssSource })
```

and in inline mounts (content.js around line 200):

```javascript
wasmModule.render_anywhere_frame_doc({ widget, unocss: unocssSource })
```

## Impact

1. The UnoCSS source code is injected into the shadow DOM rendering pipeline
2. If an attacker can modify the `vendor/unocss.js` file on disk (e.g., via malicious extension update), the CSS injection becomes arbitrary code execution in all content script mounts
3. The `cacheKey` does not provide integrity validation — it only busts the browser cache

## Attacker Control

The `unocssSource` content is fetched from the extension's own resource directory. An attacker needs write access to the extension directory (physical access or malicious extension update) to exploit this.

## Recommended Fix

1. Add Subresource Integrity (SRI) hash validation on fetched resources
2. Use `crypto.subtle.digest('SHA-256', ...)` to verify the content matches an expected hash
3. Bundle UnoCSS as a static import rather than dynamically fetching it

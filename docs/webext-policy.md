# MV3 Browser-Policy Validation

> Checklist for Chrome Web Store and Firefox Add-ons submission readiness.

## Manifest V3 Compliance

- [ ] `manifest_version: 3` in generated `manifest.json`
- [ ] Service worker background (no persistent background pages)
- [ ] `host_permissions` declared explicitly (no wildcard `*://*/*` by default)
- [ ] `content_scripts` use `matches` patterns, not `include_globs`
- [ ] Action API used for toolbar button (no `browser_action` or `page_action`)

## Content Security Policy (CSP)

Crepuscularity extensions require `wasm-unsafe-eval` for WebAssembly. Verify:

- [ ] `content_security_policy.extension_pages` includes `'wasm-unsafe-eval'` (Chrome)
- [ ] `content_security_policy` includes `'wasm-unsafe-eval'` (Firefox)
- [ ] No `'unsafe-eval'` in CSP (only `wasm-unsafe-eval`)
- [ ] No remote script origins (`https://cdn.example.com`) in CSP
- [ ] No inline script allowances (`'unsafe-inline'` for scripts)
- [ ] Dev-mode content script (`dev.js`) uses `fetch` + `location.reload()`, not `eval`
- [ ] Generated page HTML has no inline `<script>` with executable content
- [ ] Hydration payload uses `type="application/json"` (not executable JS)

Verify generated manifest CSP:

```bash
crepus webext build
cat dist/unpacked/manifest.json | jq '.content_security_policy'
```

## Web Accessible Resources

- [ ] `web_accessible_resources` lists only `vendor/runtime_bg.wasm` and `vendor/runtime.js`
- [ ] No source `.rs` or `.crepus` files exposed as web-accessible
- [ ] `use_dynamic_url` enabled for WASM resources (Chrome)

## Permissions

- [ ] Only permissions matching declared capabilities in `crepus.toml`
- [ ] `storage` permission present if using `chrome.storage` APIs
- [ ] `activeTab` preferred over broad host permissions where possible
- [ ] No `tabs` permission unless using `chrome.tabs.query` or similar

## Host Permissions

- [ ] `host_permissions` array present (MV3 requirement)
- [ ] Patterns are as narrow as possible (e.g., `https://example.com/*` not `<all_urls>`)
- [ ] Empty `host_permissions: []` if no cross-origin access needed (default for crepus webext)
- [ ] Content scripts have their own `matches` patterns, separate from host permissions

## Content Scripts

- [ ] `run_at: "document_idle"` or `"document_end"` specified
- [ ] `all_frames: false` unless iframe injection is intended
- [ ] `match_about_blank: false` unless about:blank pages need injection
- [ ] WASM compilation wrapped in try/catch (handles child frame failures gracefully)

## Icons

- [ ] At minimum: 16x16, 48x48, 128x128 PNG icons
- [ ] Icons are square and recognizable at small sizes
- [ ] No transparent-only icons (render poorly on dark/light theme switches)

## Privacy & Data

- [ ] No secrets in templates, `crepus.toml`, `manifest.json`, or bundled assets
- [ ] `dist/` directory contains only public assets
- [ ] Privacy policy URL in manifest (required for Chrome Web Store)
- [ ] `declarative_net_request` used instead of `webRequest` where possible (MV3)

## Store Submission Checklist

### Chrome Web Store

- [ ] Extension name ≤ 75 characters
- [ ] Description ≤ 132 characters (short) + full description
- [ ] Screenshots (1280×800 or 640×400)
- [ ] Small tile image (128×128)
- [ ] Privacy practices disclosed
- [ ] Single purpose policy compliant
- [ ] No obfuscated code (WASM is acceptable)

### Firefox Add-ons

- [ ] Manifest includes `browser_specific_settings.gecko.id`
- [ ] `browser_specific_settings.gecko.strict_min_version` set
- [ ] CSP uses Firefox syntax (no `extension_pages` wrapper)
- [ ] Source code submission ready (if required)

## Automated Checks

Run capability scan before submission:

```bash
crepus webext build
```

The build automatically runs `check_project_capabilities` and warns about missing capability declarations. Fix all warnings before submitting.

## Testing Matrix

- [ ] Chrome stable (latest)
- [ ] Chrome Canary
- [ ] Firefox stable (latest)
- [ ] Edge (Chromium)
- [ ] Service worker wakes and handles events correctly
- [ ] Extension works after browser restart
- [ ] Extension works in incognito/private mode (if enabled)
- [ ] Popup renders fully before WASM loads (pre-rendered popup.html)
- [ ] Hot reload works in dev mode (`.reload-id` polling)

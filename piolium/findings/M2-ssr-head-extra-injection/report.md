# M2 — SSR `head_extra` Insufficient Sanitization

**Severity:** MEDIUM  
**Category:** Improper Neutralization of HTML (CWE-80)  
**Affected Components:** `crates/crepuscularity-web/src/ssr.rs`  
**Status:** Validated

## Summary

`head_extra` content passes through `ammonia::clean()` with default config, which allows dangerous `<head>` tags including `<base>`, `<style>`, `<meta http-equiv="refresh">`, and `<link>`.

## Attack Vector

In multi-tenant/template-upload scenarios, a user-supplied `.crepus` template can set `head_extra` to:
```html
<base href="https://attacker.com/">
<meta http-equiv="refresh" content="0;url=https://attacker.com/phish">
```

The `<base>` tag hijacks all relative URLs. CSS injection via `<style>` can exfiltrate data via attribute selectors.

## Impact

Phishing via base URL hijacking, CSS data exfiltration, meta refresh redirects in rendered SSR pages.

## Root Cause

`crates/crepuscularity-web/src/ssr.rs:191-192` — default ammonia config allows `<base>`, `<style>`, `<meta>`.

## Recommended Fix

Use a custom ammonia configuration that blocks `<base>`, `<meta>`, `<link>`, and `<style>` in `head_extra`. Or strip non-content tags from `head_extra` entirely.

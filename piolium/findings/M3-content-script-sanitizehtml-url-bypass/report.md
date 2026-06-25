# M3 — Content Script URL Scheme Bypass via Protocol-Relative URLs

**Severity:** MEDIUM  
**Category:** Improper Input Validation (CWE-20)  
**Affected Components:** `crates/crepuscularity-webext/assets/content.js`  
**Status:** Validated

## Summary

`sanitizeHTML()` validates URL schemes with string prefix matching. `value.startsWith("/")` catches path-absolute URLs but also allows protocol-relative `//evil.com` URLs, which browsers resolve to the page's protocol.

## Attack Vector

```html
<a href="//evil.com/phish">Click here</a>
```

The sanitizer keeps the link because `//evil.com/phish` starts with `/`. Browser resolves to `https://evil.com/phish`.

## Impact

Phishing links in content-script-rendered widgets navigate to attacker domains.

## Root Cause

`content.js:277-287` — `value.startsWith("/")` doesn't distinguish between path-absolute `/foo` and protocol-relative `//evil.com`.

## Recommended Fix

Check for `//` prefix explicitly: `value.startsWith("/") && !value.startsWith("//")`

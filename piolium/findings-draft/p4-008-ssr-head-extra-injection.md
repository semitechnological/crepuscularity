---
id: p4-008
phase: L3
slug: ssr-head-extra-injection
severity: medium
category: html-injection
cwe: CWE-80
status: rejected-fp
rejection_reason: superseded by p8-005 (consolidated version with additional evidence)
---

# SSR `head_extra` Sanitized but Allows Dangerous HTML Tags

## Summary

The `SsrDocument.head_extra` field allows injecting raw HTML into the `<head>` section of SSR-rendered pages. While it passes through `ammonia::clean` for sanitization, the default ammonia configuration allows many HTML tags that may be abused for CSS injection, clickjacking, or information disclosure.

## Vulnerable Code

`crates/crepuscularity-web/src/ssr.rs:191-192`

```rust
let head_safe = ammonia::clean(doc.head_extra);
```

The default `ammonia::clean()` allows `link`, `meta`, `style`, `base`, and other head-specific tags with their attributes.

## Impact

1. **CSS injection**: `<style>` tags are allowed, enabling data exfiltration via CSS selectors
2. **Base URL manipulation**: `<base href="https://attacker.com/">` redirects all relative URLs
3. **Resource injection**: `<link rel="import" href="...">` or `<meta http-equiv="refresh" content="0;url=...">`
4. While XSS is prevented (scripts stripped), the damage from CSS injection + base manipulation is significant

## Attacker Control

Template expressions that flow into `head_extra` (set via template context variables used in page rendering configuration).

## Evidence

The ammonia crate documentation (v4.1.2) shows the default tags allowed include: `head`, `link`, `meta`, `style`, `title`, `base`, etc. The allowed URL schemes include `http`, `https`, `ftp`, `mailto` — but no restriction on `<base>` tag which can hijack all relative URLs on the page.

## Recommended Fix

Use a more restrictive sanitizer configuration for `head_extra` that:
1. Disallows `<base>` tag  
2. Disallows `<style>` tags (or restricts to inline styles only)
3. Disallows `<meta http-equiv="refresh">`
4. Disallows `<link>` with `href` pointing to external domains

```rust
use ammonia::Builder;
let sanitized = Builder::new()
    .tags(HashSet::from_iter(["meta", "link", "title"]))
    .clean(doc.head_extra);
```

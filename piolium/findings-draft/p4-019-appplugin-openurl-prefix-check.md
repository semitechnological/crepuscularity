---
id: p4-019
phase: L3
slug: appplugin-openurl-prefix-check
severity: low
category: url-validation
cwe: CWE-601
status: rejected-fp
rejection_reason: LOW severity per policy — only Medium+ findings survive; prefix check correctly validates scheme for http/https
---

# `AppPlugin::openUrl` Uses Prefix Check Instead of Full URL Parsing

## Summary

The `AppPlugin::openUrl` method validates URLs by checking if they start with `"https://"` or `"http://"`. This prefix-based validation can be bypassed using URL schemes that browsers interpret differently than a simple string comparison, or by using URL obfuscation techniques.

## Vulnerable Code

`crates/crepuscularity-lite/src/plugins.rs:103-107`

```rust
"openUrl" => {
    let url = payload.get("url").and_then(|v| v.as_str()).ok_or_else(|| { ... })?;
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(BridgeError::new(
            "invalid_url",
            "only http(s): URLs are allowed",
        ));
    }
    open::that(url).map_err(|e| BridgeError::new("open_failed", format!("{e}")))?;
```

## Issues

1. **`open::that(url)` calls the OS-level opener** (`xdg-open`, `open`, `start`) which may handle URLs differently than a browser
2. **No check for URL with auth user**: `http://attacker.com:password@evil.com/` may resolve differently by the OS
3. **No URL normalization**: `http://evil.com` passes, but so does `http://evil.com\@trusted.com` (newline injection)
4. **No hostname validation**: Any domain is allowed

## Attacker Control

The V8 guest JavaScript can call `Crepus.invoke("app", "openUrl", { url: "http://malicious.com" })`. The URL is attacker-controlled.

## Recommended Fix

Use proper URL parsing:

```rust
use url::Url;

"openUrl" => {
    let url_str = payload.get("url").and_then(|v| v.as_str()).ok_or_else(|| { ... })?;
    let parsed = Url::parse(url_str).map_err(|_| BridgeError::new("invalid_url", "malformed URL"))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(BridgeError::new("invalid_url", "only http(s): URLs are allowed"));
    }
    // Additional: check hostname against blocklist
    open::that(url_str).map_err(|e| BridgeError::new("open_failed", format!("{e}")))?;
```

Also consider adding a confirmation dialog before opening external URLs.

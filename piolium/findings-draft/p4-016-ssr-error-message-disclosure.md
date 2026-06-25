---
id: p4-016
phase: L3
slug: ssr-error-message-disclosure
severity: low
category: information-disclosure
cwe: CWE-209
status: rejected-fp
rejection_reason: LOW severity per policy — only Medium+ findings survive
---

# SSR Server Error Messages Disclosed in HTML Response

## Summary

The SSR router renders error messages directly into the HTML response using `<pre style='color:red'>` tags. Error messages from template parsing, rendering failures, and even `tokio::task::spawn_blocking` panics are included in the HTML response sent to the browser. These messages may leak internal file paths, template structure, and implementation details.

## Vulnerable Code

`crates/crepuscularity-ssr/src/router.rs:84-94`

```rust
match result {
    Ok(Ok(h)) => Html(h),
    Ok(Err(e)) => Html(format!(
        "<pre style='color:red'>{}</pre>",
        escape_html_error(&e.to_string())
    )),
    Err(e) => Html(format!(
        "<pre style='color:red'>render task panicked: {}</pre>",
        escape_html_error(&e.to_string())
    )),
}
```

## Impact

1. **Path disclosure**: Template file paths and directory structure are revealed in error messages
2. **Template structure**: Parsing errors reveal template syntax and structure
3. **Internal architecture**: Panic messages reveal thread/task information
4. While the output is HTML-escaped, the information content is still leaked

## Attacker Control

An attacker sends requests to the SSR server with crafted URL paths that trigger template errors (e.g., requesting a non-existent template, or providing malformed query parameters that affect template rendering).

## Recommended Fix

1. In production mode (non-debug), return a generic "500 Internal Server Error" for all errors
2. Only include detailed error information when `RUST_LOG=debug` or a debug mode flag is set
3. Log errors server-side instead of leaking them to clients

```rust
if cfg!(debug_assertions) {
    Html(format!("<pre style='color:red'>{}</pre>", escape_html_error(&e)))
} else {
    Html("<h1>500 Internal Server Error</h1>".to_string())
}
```

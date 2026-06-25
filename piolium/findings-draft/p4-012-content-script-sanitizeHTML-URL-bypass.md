---
id: p4-012
phase: L3
slug: content-script-sanitizehtml-url-bypass
severity: medium
category: html-injection
cwe: CWE-79
status: rejected-fp
rejection_reason: superseded by p8-006 (consolidated version with additional evidence)
---

# Content Script `sanitizeHTML()` URL Scheme Validation Can Be Bypassed

## Summary

The `sanitizeHTML()` function in the content script validates URL schemes for `href` and `src` attributes using string prefix matching. The validation can be bypassed by:

1. Using uppercase schemes (`HTTP://` instead of `http://`)
2. Using tab/newline characters to prefix the scheme
3. Using protocol-relative URLs starting with `//` which are NOT checked

## Vulnerable Code

`crates/crepuscularity-webext/assets/content.js:277-287`

```javascript
if (name === "href" || name === "src") {
    const value = attr.value.trim().toLowerCase();
    const isSafeUrl = value.startsWith("http://") ||
                      value.startsWith("https://") ||
                      value.startsWith("mailto:") ||
                      value.startsWith("#") ||
                      value.startsWith("/");
    if (!isSafeUrl) {
        el.removeAttribute(attr.name);
    }
}
```

## Issues

1. **Protocol-relative URLs NOT blocked**: `//evil.com/phish` starts with `/` and passes the check, but browsers resolve it to the current page's protocol (which is `https://` for extension pages, making it work)
2. **Double-slash confusion**: `//evil.com` starts with `/` which is allowed
3. **Inline JavaScript in `href`**: `javascript:alert(1)` is blocked correctly, but ` JaVaScript:...` (with space before j) passes `.trim()` but browsers parse `javascript:` case-insensitively

## Impact

1. `<a href="//evil.com">` passes sanitization and navigates to attacker site
2. In the context of a third-party page, clicking such a link navigates the user away
3. `<a href="/../../secrets">` passes the `/` check and could allow path-relative navigation

## Example Attack

Third-party page contains:
```html
<pre><code>```html
<a href="//malicious.com/phish">Click here</a>
```</code></pre>
```

The content script renders this through `sanitizeHTML()` which allows the link because `//` starts with `/`.

## Recommended Fix

1. Use a URL parser (e.g., `new URL(value, window.location.href)`) for scheme validation
2. Reject protocol-relative URLs (`//` at the start after trim)
3. Normalize the URL before checking the scheme
4. Add `javascript:` and `data:` to the explicit deny list

---
Phase: 8
Sequence: 006
Slug: content-script-sanitizehtml-url-bypass
Verdict: VALID
Rationale: Protocol-relative URLs starting with // bypass the sanitizeHTML URL check because startsWith("/") matches them, enabling phishing links in content-script-rendered widgets.
Severity-Original: medium
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - crates/crepuscularity-webext/assets/content.js:277-287
status: valid
---

# Content Script `sanitizeHTML()` URL Scheme Bypass via Protocol-Relative URLs

## Summary

The `sanitizeHTML()` function in the browser extension content script validates URL schemes for `href` and `src` attributes using string prefix matching. The check `value.startsWith("/")` allows protocol-relative URLs starting with `//`, which browsers resolve to the current page's protocol. This allows an attacker on a third-party page to inject phishing links into content-script-rendered widgets.

## Location

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

## Attacker Control

The attacker controls the HTML content of `<pre>` elements on a third-party web page. When the content script processes this content through `sanitizeHTML()`, the attacker's links are sanitized with the flawed URL check.

### Attack Input

```html
<a href="//malicious.com/phish">Click here for free stuff</a>
```

The sanitizer keeps this link because `//malicious.com/phish` starts with `/`. The browser resolves `//malicious.com/phish` to `https://malicious.com/phish` (or `http://` depending on the page protocol).

## Trust Boundary Crossed

Web page → Content script (TB-5). Third-party page content flows through the extension's sanitization pipeline and is rendered into the page's shadow DOM.

## Impact

**MEDIUM** — An attacker can inject clickable links that navigate to attacker-controlled domains:

1. **Phishing**: `<a href="//evil.com/fake-login">Login</a>` renders in the widget and navigates to the attacker's site
2. **Path-relative traversal**: `<a href="/../../secrets">` passes the `/` check and could traverse on the same origin
3. **Social engineering**: Malicious links in rendered widget content appear to be from the extension but lead to attacker sites

## Evidence

The URL validation at `content.js:277-287` uses `startsWith("/")` which unintentionally allows protocol-relative URLs:

```javascript
const isSafeUrl = value.startsWith("http://") ||
                  value.startsWith("https://") ||
                  value.startsWith("mailto:") ||
                  value.startsWith("#") ||
                  value.startsWith("/");
// "//evil.com" matches startsWith("/") — bypass!
```

The `createInlineAnywhereMount` function at line 355 renders the sanitized HTML:
```javascript
root.innerHTML = sanitizeHTML(html);
```

## Existing Mitigations

- Tag allowlist restricts which elements can have `href`/`src` (only `<a>` for `href`)
- `sanitizeHTML` strips `on*` event handler attributes
- The content runs in a shadow DOM, limiting CSS leakage from the parent page

## Reproduction Steps

1. Visit a third-party page containing: `<pre><code>```html\n<a href="//evil.com/phish">Click</a>\n```</code></pre>`
2. The extension content script processes the pre element
3. The rendered widget contains a link to `//evil.com/phish`
4. Clicking the link navigates to the attacker's domain

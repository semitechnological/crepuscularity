---
id: l4-009
phase: L4
slug: content-script-sanitizehtml-mxxs-surface
severity: MEDIUM
title: Browser Extension Content Script — Mutation XSS Surface via DOMParser/InnerHTML Dual Parse
status: rejected-fp
rejection_reason: merged into consolidated p8-007
---

## Summary

The `sanitizeHTML()` function in `content.js` uses `DOMParser.parseFromString()` to parse and sanitize HTML before assigning it via `innerHTML`. This triggers two HTML parses: one by DOMParser (for sanitization) and one by the browser's HTML parser (for rendering). If the two parsers disagree on the DOM representation, malicious content can survive sanitization. While the restrictive tag allowlist reduces risk, mutation XSS (mXSS) is a well-known class of sanitizer bypasses.

## Vulnerable Code

**File:** `crates/crepuscularity-webext/assets/content.js:285-324` (sanitizeHTML)
```javascript
function sanitizeHTML(html) {
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    const elements = doc.body.querySelectorAll("*");
    const ALLOWED_TAGS = new Set([...]);
    for (const el of elements) {
        const nodeName = el.nodeName.toUpperCase();
        if (!ALLOWED_TAGS.has(nodeName)) { el.remove(); continue; }
        for (const attr of [...el.attributes]) {
            // ... remove on* handlers, restrict href/src schemes ...
        }
    }
    return doc.body.innerHTML;
}
```

**Usage at `content.js:355`:**
```javascript
root.innerHTML = sanitizeHTML(html);
```

## Attack Classes

Known mXSS vectors that could apply despite the tag allowlist:

1. **TABLE + nested element confusion:** `<table><form><input></form></table>` — DOMParser may parse `<form>` as a child of `<table>` (and strip it since FORM is not allowed), but the browser's innerHTML parser may interpret the structure differently. Since FORM is not in the allowlist, it gets removed — but the cleanup could leave behind partial elements.

2. **Noscript/foreign content bypass:** `<noscript>` is not in the allowlist, but DOMParser may handle namespaced elements (SVG, MATH) differently than the browser. While SVG/MATH are not allowed, nested TABLE inside foreign content can create parsing discrepancies.

3. **Deeply nested HTML with event handlers inside allowed tags:** The `ALLOWED_TAGS` includes `<A>` with `href` — while the function validates URL schemes, it only checks `href` and `src` attributes. The `A` tag allows `href` — but if the value starts with `javascript:` it would be rejected. However, other protocols (like `data:` on `<a>` tags) could be used for social engineering.

## Root Cause

The two-parse pattern (DOMParser → serialize → innerHTML) is inherently vulnerable to parser differentials. The DOMParser's output (`doc.body.innerHTML`) may not round-trip safely to the browser's HTML parser.

## Evidence

- `content.js:285-324` — `sanitizeHTML` uses DOMParser then `doc.body.innerHTML`
- `content.js:355` — sanitized HTML assigned to `root.innerHTML` (second parse)
- Known CWE-116 (Improper Output Neutralization for Logs) variant — specifically mXSS (CWE-79 variant)

## Existing Mitigations

- Restrictive tag allowlist (no SCRIPT, STYLE, SVG, MATH, NOSCRIPT, OBJECT, EMBED, FORM)
- Event handler removal (`name.startsWith("on")`)
- URL scheme validation for `href`/`src`

## Priority

**MEDIUM** — Mitigated by restrictive allowlist, but mXSS is a known class with frequent new CVEs in DOMParser-based sanitizers.

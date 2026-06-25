---
Phase: 8
Sequence: 007
Slug: content-script-mutation-xss
Verdict: VALID
Rationale: DOMParser-to-innerHTML dual-parse pattern is inherently vulnerable to parser differentials (mXSS); restrictive allowlist reduces but does not eliminate risk, particularly for new mXSS vectors discovered in DOMParser implementations.
Severity-Original: medium
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - crates/crepuscularity-webext/assets/content.js:285-324
  - crates/crepuscularity-webext/assets/content.js:355
status: valid
---

# Content Script Mutation XSS via DOMParser/InnerHTML Dual Parse

## Summary

The `sanitizeHTML()` function parses attacker-controlled HTML with `DOMParser.parseFromString()` for sanitization, then assigns the sanitized output via `innerHTML`. This triggers two HTML parses: one by DOMParser (sanitization) and one by the browser's HTML parser (rendering). If the two parsers disagree on the DOM representation, malicious content can survive sanitization — a known class of mutation XSS (mXSS). While the restrictive tag allowlist reduces the attack surface, mXSS is an active research area with new bypasses discovered regularly.

## Location

`crates/crepuscularity-webext/assets/content.js:285-324` (sanitizeHTML) and `content.js:355` (usage):

```javascript
function sanitizeHTML(html) {
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    // ... strip non-allowed tags and dangerous attributes ...
    return doc.body.innerHTML;
}
// ...
root.innerHTML = sanitizeHTML(html);  // second parse
```

## Attacker Control

The attacker controls the HTML content of `<pre>` elements on a third-party web page. This HTML is passed to `sanitizeHTML()` and then into `innerHTML`.

## Trust Boundary Crossed

Web page → Content script (TB-5). Third-party page content crosses into the extension's sanitization pipeline.

## Impact

**MEDIUM** — If an mXSS bypass is found, the attacker can execute arbitrary JavaScript in the extension's context:

1. **Extension-level permissions**: The script runs in the extension's isolated world, not the page's context
2. **Storage access**: Extension `storage.*` API access
3. **Chrome-extension origin**: Access to extension resources and settings
4. **Parent page manipulation**: Depending on the shadow DOM mode, the injected script may affect the parent page

### Attack Surface

Known mXSS vectors that could apply despite the restrictive allowlist:

1. **Foreign content namespace confusion**: Elements inside `<table>` or SVG/MATH (if allowed by future allowlist changes) can confuse DOMParser vs. browser parser
2. **Deep nesting with partial tags**: `<table><b><table>` type constructs that one parser flattens but the other nests
3. **Custom element parsing**: Modern browser custom element lifecycle hooks may cause parsing differences

## Evidence

The dual-parse pattern is visible at `content.js:285-324` and `content.js:355`:

1. DOMParser parses the raw HTML → `doc.body.innerHTML` serializes the sanitized result
2. `root.innerHTML = sanitizeHTML(html)` — the browser's HTML parser re-parses the serialized output

This round-trip is the classic mXSS pattern (CWE-79 variant). Known CVEs in DOMParser-based sanitizers include CVE-2021-21225, CVE-2020-6511, CVE-2019-11357.

The tag allowlist at `content.js:296-306` is restrictive:
```javascript
const ALLOWED_TAGS = new Set([
    "DIV", "SPAN", "P", "B", "I", "EM", "STRONG", "A", "UL", "OL", "LI",
    "H1", "H2", "H3", "H4", "H5", "H6", "BR", "HR", "TABLE", "THEAD",
    "TBODY", "TR", "TH", "TD", "BLOCKQUOTE", "PRE", "CODE"
]);
```

Notably excluded: `SCRIPT`, `STYLE`, `SVG`, `MATH`, `NOSCRIPT`, `OBJECT`, `EMBED`, `FORM`, `IFRAME`.

## Existing Mitigations

- Restrictive tag allowlist (no dangerous elements)
- Event handler removal (`startsWith("on")`)
- URL scheme validation for `href`/`src`

## Reproduction Steps (Theoretical)

1. Create a test page containing a `<pre>` with crafted HTML that triggers a DOMParser rendering differential
2. Load the extension on this page
3. If the differential causes a safe-looking sanitized output but the browser re-parses it unsafely, XSS in the extension context is achieved

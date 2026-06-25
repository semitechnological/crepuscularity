# Balanced Chamber Summary

**Generated:** 2026-06-25T05:45:00Z
**Phase:** L5 — Single Review Chamber + FP Check (balanced mode)
**Status:** CLOSED

## Reviewer Notes

Phase L5 evaluated all 30 draft findings (20 p4 from Phase L3 SAST + 10 l4 from Phase L4 Deep Probe) through a three-role review process (Ideator challenge, Devil's-Advocate rejection, Synthesizer verdict). Overlapping findings were consolidated to remove redundancy. The cap of 12 surviving findings was respected (11 survivors).

## Verdict Summary

| Draft ID | Original Severity | Verdict | Disposition | P8 Reference |
|----------|-------------------|---------|-------------|-------------|
| p4-001 | CRITICAL | MERGED | Path traversal (CWE-22), not command injection (CWE-78) for non-shell plugins; shell injection separated to p8-001 | → p8-002 |
| p4-002 | HIGH | MERGED | baseDir risk subsumed by path traversal | → p8-002 |
| p4-003 | HIGH | MERGED | Go mustRead panic merged into consolidated file read | → p8-002 |
| p4-004 | HIGH | FALSE POSITIVE | escapeshellarg() effectively prevents shell injection on Unix | — |
| p4-005 | HIGH | MERGED | Ruby File.read merged into consolidated file read | → p8-002 |
| p4-006 | HIGH | MERGED | CREPUS_BIN env var merged into consolidated finding | → p8-003 |
| p4-007 | MEDIUM | SUPERSEDED | Copied to p8-004 with additional evidence | → p8-004 |
| p4-008 | MEDIUM | SUPERSEDED | Copied to p8-005 with additional evidence | → p8-005 |
| p4-009 | MEDIUM | FALSE POSITIVE | Math.random() has no exploitable security impact | — |
| p4-010 | MEDIUM | FALSE POSITIVE | Requires extension directory write access | — |
| p4-011 | MEDIUM | SUPERSEDED | Copied to p8-011 with additional evidence | → p8-011 |
| p4-012 | MEDIUM | SUPERSEDED | Copied to p8-006 with additional evidence | → p8-006 |
| p4-013 | MEDIUM | FALSE POSITIVE | WASM module is bundled with extension; no attacker control path | — |
| p4-014 | MEDIUM | FALSE POSITIVE | Example code, not production | — |
| p4-015 | MEDIUM | SUPERSEDED | Copied to p8-009 with additional evidence | → p8-009 |
| p4-016 | LOW | DROPPED | Low severity per policy | — |
| p4-017 | MEDIUM | SUPERSEDED | Copied to p8-010 with additional evidence | → p8-010 |
| p4-018 | MEDIUM | FALSE POSITIVE | DOM clobbering cannot affect content script local variables | — |
| p4-019 | LOW | DROPPED | Low severity per policy | — |
| p4-020 | LOW | DROPPED | Low severity per policy | — |
| l4-001 | CRITICAL | MERGED | C popen injection merged into consolidated shell injection | → p8-001 |
| l4-002 | CRITICAL | MERGED | C++ popen injection merged into consolidated shell injection | → p8-001 |
| l4-003 | CRITICAL | MERGED | Zig sh -c injection merged into consolidated shell injection | → p8-001 |
| l4-004 | HIGH | MERGED | All plugins file read merged into consolidated file read | → p8-002 |
| l4-005 | HIGH | MERGED | CLI path traversal merged into consolidated file read | → p8-002 |
| l4-006 | MEDIUM | MERGED | Iframe unsanitized content merged into consolidated finding | → p8-008 |
| l4-007 | MEDIUM | FALSE POSITIVE | os.quoted_path() is a V builtin for shell-safe quoting | — |
| l4-008 | MEDIUM | FALSE POSITIVE | escapeshellarg() provides adequate protection on Unix | — |
| l4-009 | MEDIUM | MERGED | mXSS surface merged into consolidated finding | → p8-007 |
| l4-010 | MEDIUM | MERGED | CREPUS_BIN duplicate merged into consolidated finding | → p8-003 |

## Final Finding Set (p8-*)

| # | File | Title | Severity | Category | Source Drafts |
|---|------|-------|----------|----------|-------------|
| 001 | p8-001-shell-command-injection-c-cpp-zig-plugins.md | Shell Command Injection via C/C++/Zig Plugin Bindings | CRITICAL | Command Injection (CWE-78) | l4-001, l4-002, l4-003 |
| 002 | p8-002-arbitrary-file-read-via-plugin-path.md | Arbitrary File Read via Unvalidated Plugin Path Parameter | HIGH | Path Traversal (CWE-22) | p4-001, p4-002, p4-003, p4-005, l4-004, l4-005 |
| 003 | p8-003-crepus-bin-env-var-hijacking.md | CREPUS_BIN Environment Variable Hijacking | HIGH | Insecure Defaults (CWE-73) | p4-006, l4-010 |
| 004 | p8-004-ssr-hydration-payload-integrity.md | SSR Hydration Payload Integrity Missing | MEDIUM | Integrity (CWE-345) | p4-007 |
| 005 | p8-005-ssr-head-extra-injection.md | SSR `head_extra` Insufficient Sanitization | MEDIUM | HTML Injection (CWE-80) | p4-008 |
| 006 | p8-006-content-script-sanitizehtml-url-bypass.md | Content Script `sanitizeHTML()` URL Scheme Bypass | MEDIUM | HTML Injection (CWE-79) | p4-012 |
| 007 | p8-007-content-script-mutation-xss.md | Content Script Mutation XSS Surface | MEDIUM | XSS (CWE-79) | l4-009 |
| 008 | p8-008-extension-iframe-unsanitized-content.md | Extension Iframe Content Not Independently Sanitized | MEDIUM | XSS (CWE-79) | l4-006 |
| 009 | p8-009-plugin-bind-context-overwrite.md | Plugin `bind:` Event Handler Context Overwrite | MEDIUM | Hidden Control Channel (CWE-233) | p4-015 |
| 010 | p8-010-no-input-size-limits.md | No Input Size Limits on Template Source | MEDIUM | DoS (CWE-770) | p4-017 |
| 011 | p8-011-v8-bridge-no-rate-limiting.md | V8 Native Bridge No Rate Limiting or Payload Limits | MEDIUM | DoS (CWE-770) | p4-011 |

## Pre-FP Gate Results

| Finding | Check 1 (Attacker Control) | Check 2 (Protection Searched) | Check 3 (Boundary) | Check 4 (Normal Position) | Check 5 (Production Code) | Status |
|---------|---------------------------|-------------------------------|-------------------|--------------------------|--------------------------|--------|
| p8-001 | ✓ argv[1] is attacker-controlled | ✓ No protection found | ✓ TB-3 crossing | ✓ Plugin caller | ✓ All plugins ship | PASS |
| p8-002 | ✓ path is attacker-controlled | ✓ No protection found | ✓ TB-3 crossing | ✓ Plugin caller | ✓ All plugins + CLI ship | PASS |
| p8-003 | ✓ Env var is attacker-influenceable | ✓ No validation found | ✓ Hidden control channel | ✓ Multi-tenant/CI/CD | ✓ All plugins ship | PASS |
| p8-004 | ✓ Stored-XSS/MITM attacker | ✓ No integrity protection | ✓ TB-1 crossing | ✓ Remote web attacker | ✓ SSR crate ships | PASS |
| p8-005 | ✓ Template context (if user-uploaded) | ✓ ammonia defaults allow dangerous tags | ✓ Template→HTML head | ✓ Multi-tenant scenario | ✓ SSR crate ships | PASS |
| p8-006 | ✓ Third-party page content | ✓ URL validation has bypass | ✓ TB-5 crossing | ✓ Any web page | ✓ Extension ships | PASS |
| p8-007 | ✓ Third-party page content | ✓ DOMParser allowlist is restrictive but mXSS is a known class | ✓ TB-5 crossing | ✓ Any web page | ✓ Extension ships | PASS |
| p8-008 | ✓ Page→WASM→iframe content | ✓ No sanitization on iframe path | ✓ TB-5 crossing | ✓ Any web page | ✓ Extension ships | PASS |
| p8-009 | ✓ Event dispatcher controls handler | ✓ No allowlist | ✓ Within plugin process | ✓ Plugin API caller | ✓ All plugins ship | PASS |
| p8-010 | ✓ stdin JSON template field | ✓ No size limits | ✓ TB-3 crossing | ✓ Plugin caller | ✓ CLI and SSR ship | PASS |
| p8-011 | ✓ V8 guest JS | ✓ No rate limits | ✓ TB-4 crossing | ✓ JS developer/guest | ✓ lite crate ships | PASS |

## Rejection Analysis

### FALSE POSITIVE findings (7)

1. **p4-004 / l4-008 (PHP exec command injection)**: `escapeshellarg()` wraps values in single quotes and escapes only single quote characters — this is proven safe on Unix for preventing shell injection. The dual `exec` + `proc_open` paths are a code quality concern but not independently exploitable.

2. **p4-009 (Weak random)**: `Math.random()` for cache keys and element IDs has no exploitable security impact. Cache keys are used for cache-busting of extension resources (the content is static). Element IDs are used for DOM identification only.

3. **p4-010 (CSS injection via UnoCSS)**: Requires extension directory write access, which is above normal attacker position. Not remotely exploitable.

4. **p4-013 (WASM storage access)**: The WASM module is bundled with the extension. An attacker would need to modify the extension binary, which requires either a supply chain compromise or file system access — both above normal attacker position.

5. **p4-014 (TAURI_DEV_HOST)**: Example/benchmark code (`examples/benchmarks/apps/tauri/vite.config.ts`), not shipping to production. Fails Pre-FP Gate Check 5.

6. **p4-018 (DOM clobbering)**: DOM clobbering via `<img name="innerHTML">` creates `window.innerHTML`, not `element.innerHTML`. The content script accesses `node.textContent` and `node.innerHTML` as element properties, which cannot be clobbered by named DOM elements. The content script also uses const/let declarations in an isolated world, immune to window property shadowing.

7. **l4-007 (V plugin shell execution)**: V's `os.quoted_path()` is a standard library builtin specifically designed for shell-safe path quoting. The risk from compiler implementation differences is speculative and not backed by evidence.

### DROPPED findings (3)

- **p4-016 (Error message disclosure)**: LOW severity
- **p4-019 (openUrl prefix check)**: LOW severity  
- **p4-020 (Catch-all route fallback)**: LOW severity

## Attack Pattern Registry

New attack patterns registered:

| Pattern | Detection Signature | Discovered In | Severity |
|---------|-------------------|--------------|----------|
| subprocess-shell-injection | `popen`, `sh -c`, `os.execute` with string-formatted command containing unsanitized path | p8-001 | CRITICAL |
| subprocess-path-traversal | `file_get_contents`, `read_to_string`, `File.read`, `os.ReadFile` with unvalidated user path | p8-002 | HIGH |
| env-var-binary-redirection | `getenv("CREPUS_BIN")`, `os.environ.get("CREPUS_BIN")`, `ENV.fetch("CREPUS_BIN")` without validation | p8-003 | HIGH |
| hydration-payload-integrity | Base64 JSON in HTML `<script id="__crepus_hydration__">` without HMAC/signature | p8-004 | MEDIUM |
| head-extra-ammonia-defaults | `ammonia::clean(doc.head_extra)` with default config allowing `<base>`, `<style>`, `<meta refresh>` | p8-005 | MEDIUM |
| protocol-relative-url-sanitizer-bypass | URL validation using `startsWith("/")` which allows `//evil.com` | p8-006 | MEDIUM |
| doomp parser-outerhtml-mxss | `DOMParser.parseFromString → innerHTML` dual parse pattern | p8-007 | MEDIUM |
| iframe-content-no-sanitization | WASM-generated iframe content not passed through `sanitizeHTML()` | p8-008 | MEDIUM |
| bind-handler-context-overwrite | `startswith("bind:")` → `context[parts[0]] = parts[1]` with no key allowlist | p8-009 | MEDIUM |
| missing-input-size-limits | No size validation on template source before `serde_json::from_str` or `fs::read_to_string` | p8-010 | MEDIUM |
| v8-bridge-no-rate-limits | `Crepus.invoke` with no rate limiting, payload size, or depth limits | p8-011 | MEDIUM |

## Variant Candidates

Findings with known variant patterns across the codebase:

1. **p8-002 (path traversal)**: Check all `fs::read_to_string()` callers in `crates/crepuscularity-cli/src/render.rs`, `crates/crepuscularity-web/src/ssr.rs`, and `crates/crepuscularity-lite/src/fs_paths.rs`

2. **p8-005 (head_extra)**: Check all `ammonia::clean()` calls for custom configuration vs. defaults; check `escape_html` usage in attribute vs. text contexts

3. **p8-006 (URL bypass)**: Check all `startsWith("/")`, `startsWith("http")` patterns in JS/TS code for similar protocol-relative URL bypasses

4. **p8-007 (mXSS)**: Check all `innerHTML` assignments in extension code after sanitization pass

## Counts

- **Original drafts evaluated**: 30 (20 p4 + 10 l4)
- **Surviving findings (p8)**: 11
- **Rejected (FALSE POSITIVE)**: 7
- **Rejected (DROPPED, low severity)**: 3
- **Rejected (MERGED)**: original drafts consolidated into p8 findings
- **New attack patterns registered**: 11
- **Variant candidates identified**: 4 categories

---

*End of Phase L5 Balanced Chamber Summary.*

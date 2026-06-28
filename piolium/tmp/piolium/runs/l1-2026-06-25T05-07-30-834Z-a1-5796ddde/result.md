Phase L1 (Intel) is complete. Here's a summary of what was accomplished:

## ✅ Completed: L1 Intel — `tschk/crepuscularity`

### Artifacts Written

| Artifact | Size | Target |
|----------|------|--------|
| `piolium/attack-surface/advisory-summary.md` | 19.5 KB | Full advisory intelligence + architecture + pattern analysis |
| `piolium/attack-surface/sbom.json` | 12.9 KB | 36-component inventory with security-relevant markers |

### Key Findings

**Repository**: `tschk/crepuscularity` — pre-1.0 Rust workspace (19 crates) for a cross-platform UI framework. Git history available.

**Advisories**: **0 published CVEs/GHSAs** found across NVD, OSV, and GitHub Advisory DB — the project is early-stage, crates not yet published to crates.io. However, **4 internal security fixes** identified in recent git history (June 2026):

| Bug | Component | Fix |
|-----|-----------|-----|
| XSS (CWE-79) | `crepuscularity-web` SSR RawHtml | `ammonia::clean` sanitizer |
| XSS (CWE-79) | `crepuscularity-webext` content script | DOMParser allowlist sanitizer |
| Path Traversal (CWE-22) | `crepuscularity-cli` dev server `/islands/` | Canonicalized path validation |
| Path Traversal (CWE-22) | `crepuscularity-cli` config loader | Path prefix check after canonicalization |

**Recurring bug types**: XSS (2 fixes) and Path Traversal (2 fixes in 7 days) — both flagged as structural-recurrence candidates.

**Highest-risk flows for later phases**:
1. Template expression values → HTML rendering (XSS)
2. File path handling in CLI config + dev server (path traversal)  
3. Plugin subprocess calls with caller-controlled template paths (potential command injection)
4. V8 embedded runtime in `crepuscularity-lite` (large attack surface)

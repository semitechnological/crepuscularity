Phase L3 (SAST) is complete. Here's a summary of what was produced:

---

## Phase L3 — SAST Summary

### Tooling
- **CodeQL:** Not on PATH — skipped database build, structural extraction, and built-in security suites
- **Semgrep:** Not on PATH — skipped all baseline/pro/custom passes
- **Fallback:** grep+read pattern analysis across all candidate files

### Agentic Actions Auditor
- Reviewed `.github/workflows/ci.yml`, `pages.yml`, `release.yml`
- No AI agent integrations (Claude Code Action, Gemini CLI, OpenAI Codex, etc.) found
- Standard CI/CD workflows only (Rust toolchain, caching, publishing)

### Artifacts Produced

| Artifact | Count/Status |
|----------|-------------|
| `piolium/findings-draft/p4-*` | **20 draft findings** (p4-001 through p4-020) |
| `piolium/attack-surface/source-sink-flows-all-severities.md` | Flow paths with DFD coverage map |
| `piolium/codeql-queries/README.md` | 8 query specifications for future CodeQL pass |
| `piolium/semgrep-rules/README.md` | 10 rule specifications for future Semgrep pass |
| `piolium/attack-surface/knowledge-base-report.md` | Updated with L3 sections (SAST Summary, CodeQL Structural Analysis, SAST Enrichment) |

### Findings by Severity

| Severity | Count | Key Issues |
|----------|-------|------------|
| **CRITICAL** | 1 | Plugin subprocess path injection (all 5 languages) |
| **HIGH** | 5 | Path traversal, CREPUS_BIN hijacking, arbitrary file reads |
| **MEDIUM** | 11 | Hydration integrity, sanitizer bypasses, weak crypto, DoS, hidden channels |
| **LOW** | 3 | Info disclosure, URL prefix check, 404 fallback |

### Key High-Risk Findings

1. **p4-001 (CRITICAL):** All 5 plugin languages pass unvalidated caller-controlled `path` to subprocess and `File.read()`
2. **p4-006 (HIGH):** `CREPUS_BIN` env var redirects subprocess binary with no validation
3. **p4-007 (MEDIUM):** SSR hydration payload has no integrity protection (HMAC missing)
4. **p4-012 (MEDIUM):** Content script `sanitizeHTML()` URL validation bypassed by `//evil.com` protocol-relative URLs
5. **p4-015 (MEDIUM):** Plugin `bind:` handler overwrites context variables without allowlist

### CodeQL Artifacts
The `piolium/codeql-artifacts/db/` database was not created (CodeQL unavailable). The instructions note this should be retained for Phases 5, 7, 8, and 10 — please ensure CodeQL is available in the execution environment before those phases.

**Phase L2 complete.** The knowledge base report has been written to:

**`piolium/attack-surface/knowledge-base-report.md`** (633 lines)

### What was produced:

| Section | Content |
|---------|---------|
| **Project Classification** | Framework + CLI + cross-platform UI SDK (20+ crates, polyglot plugins) |
| **Architecture Model** | Component table (20 items, each with security relevance), transport map (8 transports) |
| **Trust Boundaries** | 8 identified boundaries in priority order, with protocol/security details |
| **DFD/CFD Slices** | 3 Mermaid DFD diagrams (SSR HTTP, Plugin IPC, V8 Bridge) + 1 CFD (Include Resolution) |
| **Attack Surface** | 10 attacker-controlled input vectors mapped to trust boundaries and risk levels |
| **Key Dependencies** | Security-relevant subset of sbom.json with version/CVE notes |
| **Framework Contracts** | Complete inventory of Axum router, hydration markers, plugin protocol, V8 bridge contract, content script contracts, include contracts, hidden control channels |
| **Threat Model** | 5 threat actors, 5 asset categories, 10 threat scenarios with likelihood/priority, existing mitigations with evidence paths, 6 recommended mitigations |
| **Domain Attack Research** | 9 domains analyzed (Template SSTI, HTTP Server, Subprocess, XSS, File Path, V8 Bridge, Extension/Content Script), each with attack taxonomy table, custom SAST targets, manual review checklist |
| **Phase 4 CodeQL Targets** | 7 extraction targets with source/sink types; 5 custom CodeQL models needed |
| **Spec Gap Candidates** | HTML5, OWASP XSS Prevention, CSP, MV3, WASM spec references |
| **Coverage Gaps** | Architecture gaps, security gaps, dependency gaps |

### Top finding:
**T-01 (HIGH): Plugin subprocess path injection** — all 7+ plugin bindings pass caller-controlled `path` to `crepus native ir` without validation. This is the single highest-risk unmitigated issue in the codebase.

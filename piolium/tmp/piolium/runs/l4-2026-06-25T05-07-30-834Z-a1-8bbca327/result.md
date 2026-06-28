Probe for **Plugin Subprocess + Browser Extension Content Script** complete.

**Loops:** 1 (single pass)
**Validated findings:** 10
- **CRITICAL (3):** C, C++, Zig plugin shell command injection via `popen`/`sh -c`
- **HIGH (2):** All plugins arbitrary file read via path traversal; CLI `native.rs` unvalidated path
- **MEDIUM (5):** Content script iframe script execution; V plugin `os.execute` risk; PHP dual `exec` paths; Content script mXSS surface; `CREPUS_BIN` env var hijacking

**Stop reason:** All targeted entry points covered (14 plugin bindings reviewed; content.js sanitization and iframe flows analyzed)

**Files produced:**
- `piolium/attack-surface/manual-attack-surface-inventory.md` — Entry points, attacker sources, sinks, hidden control channels
- `piolium/attack-surface/balanced-probe-summary.md` — Full probe summary with coverage table
- `piolium/findings-draft/l4-001` through `l4-010` — Verified findings with file:line evidence

**Key finding:** The C, C++, and Zig plugin bindings have **shell command injection** via unvalidated `path` arguments flowing to `popen()` (C/C++) and `/bin/sh -c` (Zig). All 14+ plugin bindings have **arbitrary file read** via the same unvalidated `path` parameter — this is the single highest-risk unmitigated issue per the Phase L2 knowledge base (T-01).

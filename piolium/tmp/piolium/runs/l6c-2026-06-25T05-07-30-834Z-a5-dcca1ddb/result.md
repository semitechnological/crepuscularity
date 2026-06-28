**Phase L6c (Attempt 5/6) — FAILED: mandatory gate violated.**

`piolium/findings/` does not exist. This is the required input for final report assembly — every finding directory must have a `report.md` >500 bytes. Since zero findings were promoted by Phase L6b (consolidation), no such directories exist.

**Root cause (unchanged):** The consolidation manifest (`balanced-consolidation-manifest.json`) uses a prefix filter of `p8-` only, ignoring 31 of 41 drafts (`l4-` and `p4-` prefixes). All matched drafts are tagged with severity `"info"` (discarded by threshold) regardless of actual severity.

**Action required before next attempt:**
1. Fix consolidation prefix filter to `["l4-", "p4-", "p8-"]`
2. Fix severity extraction to read actual severity from draft frontmatter
3. Re-run Phase L6b to populate `piolium/findings/`
4. Re-run Phase L6c

Failure note updated at `piolium/l6c-failure-note.md`. No final report written.

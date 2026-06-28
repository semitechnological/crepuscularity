**Report assembly BLOCKED.** Findings directory `piolium/findings/` does not exist — gate failure. No report written. Updated `piolium/l6c-failure-note.md` with full diagnostics.

**Summary:** Critical upstream pipeline issue in Phases L6/L6b: the consolidation manifest incorrectly assigned `"severity": "info"` to all 11 validated P8 findings (actual range: CRITICAL→HIGH→MEDIUM), causing every draft to be dropped during promotion. The `promoted` array is empty; 10 of 11 findings were dropped as "below severity threshold (low/info)"; p8-009 is missing from the manifest entirely. 31 total draft files exist in `piolium/findings-draft/` but none reached `piolium/findings/`.

**Consistency checks:** Not run — prerequisite artifact missing. Cannot validate findings/promoted-dirs cross-reference, finding completeness, or orphan detection without `piolium/findings/`.

**Resolution requires:** Re-running Phase L6b with correct severity mapping so that chamber-verified P8 findings (1 CRITICAL, 2 HIGH, 8 MEDIUM) are promoted properly, then re-invoking L6c.

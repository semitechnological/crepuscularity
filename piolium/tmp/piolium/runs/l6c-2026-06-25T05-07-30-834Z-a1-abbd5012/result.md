## L6c Failure Report

**Condition met:** `piolium/findings/` directory does not exist — hard prerequisite failure.

**No final report written.**

The audit pipeline reached Phase L6c without upstream Phase L6 (Consolidation) and L6b (Finding Triage) being executed — both were skipped. The consolidation manifest (`piolium/attack-surface/balanced-consolidation-manifest.json`) confirms zero findings were promoted from `piolium/findings-draft/`. All 11 P8 findings were dropped with reason "below severity threshold (low/info)".

**Failure note written to:** `piolium/tmp/piolium/runs/l6c-2026-06-25T05-07-30-834Z-a1-abbd5012/failure-note.md`

**Orchestrator report:** Report assembly blocked. Findings: 0 (C:0, H:0, M:0). Consistency: FAIL — `piolium/findings/` missing. Prerequisite phases L6 and L6b must be run first, or the severity threshold must be lowered to allow CRITICAL/HIGH findings to survive triage.

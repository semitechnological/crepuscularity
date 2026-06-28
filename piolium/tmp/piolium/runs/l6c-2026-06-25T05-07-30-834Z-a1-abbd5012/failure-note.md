# Phase L6c Failure Note

**Run:** l6c-2026-06-25T05-07-30-834Z-a1-abbd5012
**Agent:** report-assembler
**Timestamp:** 2026-06-25T05:28:19Z

## Failure Condition

`piolium/findings/` directory does NOT exist. This is a hard prerequisite failure per the Phase L6c specification:

> "List every directory under `piolium/findings/`. Each MUST have a `report.md` of >500 bytes — if any are missing, fail with a clear error and DO NOT write the final report."

## Root Cause

The audit pipeline skipped upstream phases:

| Phase | Status | Expected Artifact |
|-------|--------|-------------------|
| L6 (Consolidation) | **skipped** | `piolium/findings/` with promoted findings |
| L6b (Finding Triage) | **skipped** | P0/P1/P2 prioritization, deferred findings |
| L6c (Report Assembly) | **blocked** | `piolium/final-audit-report.md` |

Without L6, no findings were promoted from `piolium/findings-draft/` to `piolium/findings/`. The consolidation manifest (`piolium/attack-surface/balanced-consolidation-manifest.json`) shows all 11 P8 findings were dropped as "below severity threshold (low/info)", and the `promoted` array is empty.

## Audit State Summary

- **Phases complete:** L1, L2, L3, L4, L5
- **Draft findings:** 42 files in `piolium/findings-draft/` (10 L4 + 20 P4 + 11 P8 + 1 L6 skeleton)
- **Promoted findings:** 0
- **KB artifact:** `piolium/attack-surface/knowledge-base-report.md` (complete)
- **Chamber verdicts:** L5 balanced chamber validated 11 P8 findings via Pre-FP gate, but all were dropped by the finding-triager

## Recommendation

To recover, either:
1. Re-run L6 (consolidation) and L6b (triage) with adjusted severity thresholds so at least CRITICAL/HIGH findings survive promotion, OR
2. If this is intentional (zero findings policy), update the audit state to mark the audit as completed with zero findings

## Attachments

- `piolium/attack-surface/balanced-consolidation-manifest.json` — shows all findings dropped
- `piolium/attack-surface/balanced-chamber-summary.md` — chamber verdicts
- `piolium/findings-draft/` — all draft findings

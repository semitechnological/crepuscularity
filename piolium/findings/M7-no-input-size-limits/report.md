# M7 — No Input Size Limits on Template Source

**Severity:** MEDIUM  
**Category:** Allocation of Resources Without Limits (CWE-770)  
**Affected Components:** `crates/crepuscularity-core/src/parser.rs`, `crates/crepuscularity-cli/src/native.rs`  
**Status:** Validated

## Summary

Template source from stdin/JSON has no size limits. Multi-megabyte templates cause memory exhaustion during parsing and AST construction.

## Attack Vector

100MB template via `--stdin-json` or rogue plugin input causes OOM kill or DoS.

## Impact

Denial of service via memory exhaustion on all execution paths (dev server, SSR, CLI, plugins).

## Root Cause

No `validate_template_size()` or size cap on input read in any code path.

## Recommended Fix

Cap template input at 10MB. Cap context JSON at 1MB. Return clear error on oversized input.

# M8 — V8 Native Bridge No Rate Limiting or Payload Limits

**Severity:** MEDIUM  
**Category:** Allocation of Resources Without Limits (CWE-770)  
**Affected Components:** `crates/crepuscularity-lite/src/bridge.rs`  
**Status:** Validated

## Summary

The V8 native bridge (`Crepus.invoke()`) has no rate limiting, payload size limits, or recursion depth limits. Malicious guest JavaScript can exhaust CPU, memory, disk, or cause stack overflow.

## Attack Vector

```javascript
while (true) { Crepus.invoke("core", "echo", { data: "x".repeat(100000) }); }
```

## Impact

Resource exhaustion: CPU spin, OOM from nested JSON, disk fill via `FsPlugin.writeText`, clipboard abuse.

## Root Cause

`crates/crepuscularity-lite/src/bridge.rs` — no size/depth/rate checks on the `invoke()` method.

## Recommended Fix

Add token bucket rate limiter per session. Cap JSON payload at 1MB. Enforce max JSON depth (32). Limit FsPlugin per-call write size.

---
Phase: 8
Sequence: 011
Slug: v8-bridge-no-rate-limiting
Verdict: VALID
Rationale: V8 native bridge has no rate limiting, payload size limits, or recursion depth limits on JSON payload, enabling resource exhaustion from guest JavaScript.
Severity-Original: medium
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - crates/crepuscularity-lite/src/bridge.rs
status: valid
---

# V8 Native Bridge No Rate Limiting or Payload Limits

## Summary

The V8 native bridge (`Crepus.invoke(plugin, method, payload)`) has no rate limiting, payload size limits, or recursion depth limits on the JSON payload. A malicious JavaScript guest running in the V8 isolate can:

1. Exhaust CPU by making rapid `Crepus.invoke` calls
2. Exhaust memory by passing deeply nested JSON payloads
3. Exhaust disk space by calling `FsPlugin.writeText` repeatedly with large data
4. Cause stack overflow via deeply nested JSON deserialization

## Location

`crates/crepuscularity-lite/src/bridge.rs` — the `invoke_inner` method processes the payload with no size or depth checks:

```rust
pub fn invoke(&self, method: &str, payload: &Value) -> Result<Value, BridgeError> {
    // Payload is deserialized from JSON with no size limits
    match method {
        "readText" => self.handle_read_text(path()?),
        "writeText" => self.handle_write_text(path()?, payload),
        // ...
    }
}
```

## Attacker Control

The V8 guest JavaScript that calls `Crepus.invoke()`. In `crepuscularity-lite`, the guest JS is typically application code, but the runtime is designed to support executing user-provided JS snippets. A developer using crepuscularity-lite might load untrusted JS bundles.

## Trust Boundary Crossed

V8 Guest → Native Host (TB-4). The V8 isolate is designed to be a sandbox, but the native bridge is the escape hatch.

## Impact

**MEDIUM** — Resource exhaustion:

1. **CPU exhaustion**: `Crepus.invoke("core", "echo", {...})` in a tight loop blocks the event loop
2. **Memory exhaustion**: Deeply nested JSON objects (`[[[[...]]]]`) cause stack overflow or OOM during `serde_json::deserialize`
3. **Disk exhaustion**: `FsPlugin.writeText` with large content fills the sandbox directory
4. **Clipboard abuse**: `ClipboardPlugin.writeText` called repeatedly

## Evidence

The `invoke_inner` method processes the JSON payload with `serde_json::from_str`:
```rust
let payload: Value = serde_json::from_str(payload_str)
    .map_err(|e| BridgeError::new("invalid_json", format!("{e}")))?;
```

This uses serde_json's default recursion limit (128 levels), but:
- No custom depth limit is enforced at the bridge level
- No payload size limit (e.g., 1 MB max per invoke)
- No per-invoke rate limit using a token bucket or similar
- No per-session write limit for FsPlugin

## Existing Mitigations

- The capability system restricts which plugins are available to the guest
- Each plugin has a method allowlist
- FsPlugin paths are sandboxed via `resolve_under_sandbox`

## Reproduction Steps

1. In a crepuscularity-lite application, execute guest JavaScript:
   ```javascript
   // CPU exhaustion
   while (true) {
       Crepus.invoke("core", "echo", { data: "x".repeat(100000) });
   }
   
   // Memory exhaustion via deeply nested JSON
   let deep = [];
   for (let i = 0; i < 10000; i++) deep = [deep];
   Crepus.invoke("core", "echo", { nested: JSON.stringify(deep) });
   ```
2. Observe the host process becomes unresponsive or crashes

---
id: p4-011
phase: L3
slug: v8-bridge-no-rate-limiting
severity: medium
category: denial-of-service
cwe: CWE-770
status: rejected-fp
rejection_reason: superseded by p8-011 (consolidated version with additional evidence)
---

# V8 Native Bridge Has No Rate Limiting or Payload Size Limits

## Summary

The V8 native bridge (`Crepus.invoke(plugin, method, payload)`) has no rate limiting, payload size limits, or recursion depth limits on the JSON payload. A malicious JavaScript guest can:

1. Exhaust CPU by making rapid `Crepus.invoke` calls
2. Exhaust memory by passing deeply nested JSON payloads
3. Exhaust disk space by calling `FsPlugin.writeText` repeatedly with large data

## Vulnerable Code

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

The JSON payload parsing uses `serde_json::from_str` with no `serde_json::Deserializer` depth limit override:

```rust
let payload: Value = serde_json::from_str(payload_str)
    .map_err(|e| BridgeError::new("invalid_json", format!("{e}")))?;
```

## Impact

1. **CPU exhaustion**: `Crepus.invoke("core", "echo", {...})` in a tight loop
2. **Memory exhaustion**: Deeply nested JSON objects (`[[[[...]]]]`) cause stack overflow or OOM
3. **Disk exhaustion**: `FsPlugin.writeText` with large content fills the sandbox directory
4. **Clipboard abuse**: `ClipboardPlugin.writeText` called repeatedly

## Attacker Control

The V8 guest JavaScript that calls `Crepus.invoke()`. While the guest JS is typically application code, the crepuscularity-lite runtime is designed to support executing user-provided JS snippets.

## Recommended Fix

1. Add maximum nesting depth to JSON parsing (e.g., 128 levels):

```rust
use serde_json::Deserializer;
let mut deserializer = Deserializer::from_str(payload_str);
deserializer.disable_recursion_limit();  // no — actually set a limit
```

2. Add per-invoke rate limiting using a token bucket
3. Add payload size limit (e.g., 1 MB per invoke)
4. Add total per-session write limit for FsPlugin

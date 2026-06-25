---
id: p4-017
phase: L3
slug: ssr-template-input-size-limit
severity: medium
category: denial-of-service
cwe: CWE-770
status: rejected-fp
rejection_reason: superseded by p8-010 (consolidated version with additional evidence)
---

# No Input Size Limits on Template Source from stdin/Plugin

## Summary

The `crepus native ir --stdin-json` and `crepus native ir --stdin` modes accept template source strings with no size limits. An attacker can submit arbitrarily large template payloads, causing excessive memory allocation during parsing, AST construction, and rendering.

## Vulnerable Code

The template parser (`crates/crepuscularity-core/`) accepts the full input string without size checks:

- `--stdin-json` mode: The `template` field in the JSON envelope is read as a String with no length validation
- `--stdin` mode: The full stdin is read into memory

The `MAX_INCLUDE_DEPTH` constant (64) prevents include recursion DoS, but there is no absolute limit on:

1. Total template source size (via stdin/JSON)
2. Number of AST nodes (via extremely repetitive templates)
3. Context JSON size (via `--stdin-json` context field)
4. Number of files in virtual file map

## Impact

1. **Memory exhaustion**: A multi-megabyte template with repeated `text "..."` nodes creates a massive AST
2. **CPU exhaustion**: Parsing and rendering enormous templates blocks the thread
3. **All execution paths affected**: dev server, SSR server, CLI render, plugin invocations

## Attacker Control

The attacker controls the template source sent via:
- Plugin `--stdin-json` template field
- Direct `--stdin` input
- Crafted HTTP request that triggers SSR rendering with large template

## Recommended Fix

Add size limits before parsing:

```rust
const MAX_TEMPLATE_SIZE: usize = 10 * 1024 * 1024; // 10 MB
const MAX_CONTEXT_SIZE: usize = 1 * 1024 * 1024;    // 1 MB

fn validate_template_size(template: &str) -> Result<(), CrepusError> {
    if template.len() > MAX_TEMPLATE_SIZE {
        return Err(CrepusError::render("template source exceeds maximum size"));
    }
    Ok(())
}
```

And make `serde_json::from_str` limit its depth:

```rust
let mut deserializer = serde_json::Deserializer::from_str(payload_str);
// Set recursion limit to prevent stack overflow
deserializer.disable_recursion_limit(); // or set a custom limit
```

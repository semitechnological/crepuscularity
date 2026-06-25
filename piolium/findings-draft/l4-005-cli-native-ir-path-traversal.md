---
id: l4-005
phase: L4
slug: cli-native-ir-path-traversal
severity: HIGH
title: CLI `crepus native ir` — Path Traversal via Unvalidated Path Argument
status: rejected-fp
rejection_reason: merged into consolidated p8-002
---

## Summary

The `crepus native ir <path>` command in the CLI reads a file from the filesystem at the user-supplied `path` without any validation. This is the sink for all plugin subprocess invocations: when plugins call `crepus native ir <path>` (the no-context path), or when plugins read the file themselves and pass it via stdin (the context path), the path is used directly in `fs::read_to_string()`. No `resolve_include_path`-style protection is applied.

## Vulnerable Code

**File:** `crates/crepuscularity-cli/src/native.rs:233-237`
```rust
let path = parsed.path.ok_or_else(|| {
    "Usage: crepus native ir <file.crepus> [--component Name] [--ctx FILE] [--var k=v] [--pretty]"
        .to_string()
})?;
let content = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
```

**Contrast with `include_paths.rs:13`:**
```rust
pub fn resolve_include_path(base_dir: Option<&Path>, path: &str) -> Result<PathBuf, CrepusError> {
    let requested = Path::new(path);
    if requested.has_root()
        || requested.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
    { return Err(CrepusError::include_path(...)); }
    // ... canonicalization checks ...
}
```

## Attack

**Plugin caller provides:** `path = "../../etc/shadow"` to the C plugin (via `crepus native ir ../../etc/shadow`) or any plugin that uses the no-context path.

**CLI receives:** The `path` argument at `native.rs:236` is passed to `fs::read_to_string` with no checks.

## Root Cause

The CLI's `native.rs` `run_ir_inner` function does not apply any path validation before calling `fs::read_to_string`. While the template engine's `include` directive has robust path validation in `include_paths.rs`, the `native ir` command (which is the primary subprocess command for all plugins) does not reuse this validation.

## Code Path

```
Plugin → subprocess → crepus native ir <path>
  → native.rs:run_ir() → run_ir_inner()
    → parse_ir_args(args) @ L107 (sets parsed.path from the positional arg)
    → fs::read_to_string(parsed.path) @ L236
      → content passed to render_template_to_ir() 
      → IR JSON output to stdout
```

## Security Consequence

Path traversal leading to arbitrary file read. Combined with plugin control, any file on the system can be read and exfiltrated via the IR output.

## Evidence

- `crates/crepuscularity-cli/src/native.rs:236` — `fs::read_to_string(&path)` with unvalidated path
- No canonicalization or prefix check before the file read
- `include_paths.rs:13` has `resolve_include_path()` with these checks — not used here
- `render.rs:80` has the same pattern (`fs::read_to_string(&path)` without validation)

## Existing Mitigations

None.

## Priority

**HIGH**

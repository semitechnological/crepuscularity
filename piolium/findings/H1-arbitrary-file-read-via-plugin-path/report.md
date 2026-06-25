# H1 — Arbitrary File Read via Unvalidated Plugin Path Parameter

**Severity:** HIGH  
**Category:** Path Traversal (CWE-22)  
**Affected Components:** All 14+ plugin bindings + CLI `crepus native ir`  
**Status:** Validated

## Summary

Every plugin binding and the CLI `crepus native ir <path>` reads a caller-controlled `path` argument from the filesystem with no canonicalization, traversal check, or symlink protection. The file content is returned in the IR output or error response.

## Attack Vector

```
path = "../../etc/passwd"
path = "/etc/shadow"
path = "../../.env"
```

All plugins use direct file read APIs (`Path.read_text()`, `os.ReadFile()`, `File.readAllText()`) with no validation. The Core template engine's `resolve_include_path()` has proper canonicalization guards — but `native ir` doesn't use it.

## Impact

Arbitrary file read on the host system. Config files with secrets, SSH keys, cloud credentials, source code, and system files all exfiltratable.

## Root Cause

`crates/crepuscularity-cli/src/native.rs:236` — `fs::read_to_string(&path)` with no path validation.  
All plugin bindings — direct file read APIs with no input validation.

## Recommended Fix

Reuse `crepuscularity_core::include_paths::resolve_include_path()` for CLI path resolution. Add prefix or canonicalization check in all plugin bindings.

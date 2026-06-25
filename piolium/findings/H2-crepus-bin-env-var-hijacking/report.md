# H2 — CREPUS_BIN Environment Variable Hijacking

**Severity:** HIGH  
**Category:** External Control of File Name/Path (CWE-73)  
**Affected Components:** All 14+ plugin bindings  
**Status:** Validated

## Summary

All plugin bindings read `CREPUS_BIN` from the environment with no validation, allowlist, or integrity check. Any attacker who can control environment variables (shared hosting, CI/CD, Docker, parent process) can redirect plugins to execute arbitrary binaries.

## Attack Vector

```bash
export CREPUS_BIN="/tmp/malicious"
python -c "from crepuscularity_plugin import render_ir; render_ir('test.crepus', {})"
```

## Impact

Arbitrary code execution with the calling process's privileges. Combined with the unvalidated path parameter (H1), this enables complete subversion of the plugin pipeline.

## Root Cause

All plugins: `os.environ.get("CREPUS_BIN", "crepus")` pattern with no validation of the returned path.

## Recommended Fix

Drop `CREPUS_BIN` support in favor of PATH resolution or a hardcoded binary path configurable at build time. Or add a hash/signature check on the binary before execution.

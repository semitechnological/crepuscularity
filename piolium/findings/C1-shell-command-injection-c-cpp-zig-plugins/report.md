# C1 — Shell Command Injection via C/C++/Zig Plugin Bindings

**Severity:** CRITICAL  
**Category:** Command Injection (CWE-78)  
**Affected Components:** `plugins/c/crepuscularity_plugin.c`, `plugins/cpp/crepuscularity_plugin.cpp`, `plugins/zig/crepuscularity.zig`  
**Status:** Validated

## Summary

Three plugin bindings embed the caller-supplied `path` argument directly into a shell command string via `popen()` (C/C++) or `/bin/sh -c` (Zig) without any escaping. An attacker controlling the `path` parameter can inject arbitrary shell commands.

## Attack Vector

```
path = "\";id>/tmp/pwned;\""
```

**C plugin**: `snprintf(cmd, ..., path)` → `popen(cmd, "r")` — no escaping  
**C++ plugin**: `cmd = "\"" + bin + "\" native ir \"" + path + "\""` → `popen(cmd, "r")` — no escaping  
**Zig plugin**: `allocPrint(..., path)` → `/bin/sh -c command` — no escaping

All other plugins (Python, Go, TypeScript, Ruby) safely use array-style argv which avoids shell interpretation. The C/C++/Zig bindings remain unmitigated.

## Impact

Arbitrary command execution as the plugin user. Full system compromise potential: file exfiltration, reverse shells, persistence.

## Root Cause

`plugins/c/crepuscularity_plugin.c:11-12` — shell command string built via `snprintf` with path injected  
`plugins/cpp/crepuscularity_plugin.cpp:15-16` — string concatenation with path  
`plugins/zig/crepuscularity.zig:12-17` — `allocPrint` into `/bin/sh -c` command

## Recommended Fix

Replace `popen(shell_cmd, "r")` with `posix_spawn` or manual `fork/exec` with argv array. For Zig, pass the CLI arguments as argv directly instead of through `/bin/sh -c`.

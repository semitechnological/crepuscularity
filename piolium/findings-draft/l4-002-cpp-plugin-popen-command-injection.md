---
id: l4-002
phase: L4
slug: cpp-plugin-popen-command-injection
severity: CRITICAL
title: C++ Plugin — Shell Command Injection via popen with Unsanitized Path
status: rejected-fp
rejection_reason: merged into consolidated p8-001
---

## Summary

The C++ plugin binding (`plugins/cpp/crepuscularity_plugin.cpp`) uses `popen()` with a shell command string constructed via string concatenation. The `path` argument is embedded directly into the shell command without validation. Identical vulnerability pattern to the C plugin.

## Vulnerable Code

**File:** `plugins/cpp/crepuscularity_plugin.cpp:15-16`
```cpp
std::string cmd = "\"" + bin + "\" native ir \"" + path + "\"";
FILE* pipe = popen(cmd.c_str(), "r");
```

**Full function at lines 12-27:**
```cpp
ViewIr render_ir(const std::string& path) {
    const char* env = std::getenv("CREPUS_BIN");
    std::string bin = env == nullptr ? "crepus" : env;
    std::string cmd = "\"" + bin + "\" native ir \"" + path + "\"";
    FILE* pipe = popen(cmd.c_str(), "r");
    // ...
}
```

**Call chain:** `main(argc, argv)` → `argv[1]` → `render_ir(path)` → `popen(cmd.c_str(), "r")`

## Attack Input

```
path = "\";cat /etc/shadow > /tmp/leak;\"" 
```

## Root Cause

The `path` is inserted into a shell command string via `operator+` with no escaping. `popen()` invokes `/bin/sh` to parse the command, so shell metacharacters in `path` are interpreted as shell syntax. The surrounding double quotes are insufficient protection when the input contains `"`, `;`, `$()`, or backticks.

## Code Path

```
plugins/cpp/crepuscularity_plugin.cpp
  main() @ L47
    → render_ir(argv[1]) @ L12
      → cmd = "\"" + bin + "\" native ir \"" + path + "\"" @ L15
        → popen(cmd.c_str(), "r") @ L16
```

## Security Consequence

Arbitrary command execution as the user running the C++ plugin application.

## Evidence

- `plugins/cpp/crepuscularity_plugin.cpp:15-16` — string concatenation + `popen`
- No input validation on `path`
- No shell escaping

## Existing Mitigations

None.

## Priority

**CRITICAL** — Identical exploit mechanism to C plugin.

---
id: l4-001
phase: L4
slug: c-plugin-popen-command-injection
severity: CRITICAL
title: C Plugin — Shell Command Injection via popen with Unsanitized Path
status: rejected-fp
rejection_reason: merged into consolidated p8-001
---

## Summary

The C plugin binding (`plugins/c/crepuscularity_plugin.c`) uses `popen()` with a shell command string constructed via `snprintf`. The `path` argument from the plugin caller is embedded directly into the shell command without any validation, sanitization, or escaping. Any caller of the C plugin (e.g., a C application that uses `crepus_render_ir()`) can inject arbitrary shell commands by providing a crafted `path`.

## Vulnerable Code

**File:** `plugins/c/crepuscularity_plugin.c:11-12`
```c
char cmd[4096];
snprintf(cmd, sizeof(cmd), "\"%s\" native ir \"%s\"", bin, path);
FILE *pipe = popen(cmd, "r");
```

**Full function at lines 5-20:**
```c
static int crepus_render_ir(const char *path, char *buf, size_t cap) {
    const char *bin = getenv("CREPUS_BIN");
    if (bin == NULL) { bin = "crepus"; }
    char cmd[4096];
    snprintf(cmd, sizeof(cmd), "\"%s\" native ir \"%s\"", bin, path);
    FILE *pipe = popen(cmd, "r");
    // ...
}
```

**Call chain:** `main(argc, argv)` at line 37 → `argv[1]` → `crepus_render_ir(path, ...)` → `popen(cmd, "r")`

## Attack Input

```
path = "\";id>/tmp/pwned;\"" 
```

This produces shell command:
```sh
"crepus" native ir "";id>/tmp/pwned;""
```

## Root Cause

The `path` string passed from the plugin API caller is inserted into a shell command format string at `popen`. While the path is quoted with `\"%s\"`, shell metacharacters like `;`, `` ` ``, `$()`, `|`, `&`, `\n` break out of the quoting context. There is no input validation, path canonicalization, or shell escaping before the `popen` call.

## Code Path

```
plugins/c/crepuscularity_plugin.c
  main() @ L37
    → argv[1] → crepus_render_ir(path, buf, cap) @ L5
      → snprintf(cmd, "\"%s\" native ir \"%s\"", bin, path) @ L11
        → popen(cmd, "r") @ L12
```

## Security Consequence

Arbitrary command execution as the user running the C plugin application. The attacker can execute any shell command, read or write any file the process has access to, establish reverse shells, or pivot to other systems.

## Evidence

- `plugins/c/crepuscularity_plugin.c:11-12` — `snprintf` + `popen` with shell-injectable format
- No input validation on `path` before `popen`
- No shell escaping functions used (no `escapeshellarg` equivalent)

## Existing Mitigations

None.

## Priority

**CRITICAL** — This is a trivially exploitable command injection with no authentication barrier.

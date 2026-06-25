---
id: p4-004
phase: L3
slug: php-exec-command-injection
severity: high
category: command-injection
cwe: CWE-78
status: rejected-fp
rejection_reason: escapeshellarg() effectively prevents shell injection on Unix; dual-path pattern is a code quality concern, not independently exploitable
---

# PHP Plugin Uses `exec()` with `escapeshellarg` — Bypassable via Argument Injection

## Summary

The PHP plugin has two code paths for invoking the `crepus` binary:

1. **With context** (`proc_open` with array-style command — safe from shell injection)
2. **Without context** (`exec()` with `escapeshellcmd` + `escapeshellarg` — vulnerable to argument injection)

The `exec()` path constructs a shell command string and passes it to the shell, making it potentially exploitable via argument injection even though the path is escaped.

## Vulnerable Code

`plugins/php/CrepuscularityPlugin.php:35-40`

```php
$cmd = escapeshellcmd($bin) . ' native ir ' . escapeshellarg($path);
$out = [];
$code = 0;
exec($cmd, $out, $code);
```

## Issue

While `escapeshellarg()` prevents shell metacharacter injection in the path value, `escapeshellcmd()` is applied to the binary name. The `CREPUS_BIN` environment variable controls the binary path, which is passed through `escapeshellcmd()`. If an attacker controls the environment, they could inject additional arguments via the binary path itself.

More critically, this code path constructs a shell command string rather than using the array form of `proc_open` (like the context path does). This is a defense-in-depth concern for any future code changes that might accept user input in the path.

## Impact

- **Medium**: The `escapeshellarg()` on path prevents direct shell injection on the path argument
- **HIGH**: The `CREPUS_BIN` env var (if attacker-controlled) is only `escapeshellcmd()`-escaped, which is insufficient for argument injection protection
- **The `proc_open` path** (with context) is properly using array-form command execution, but the `exec` path is inconsistent

## Attacker Control

The attacker controls the `path` argument and potentially the `CREPUS_BIN` environment variable.

## Recommended Fix

1. Remove the `exec()` code path entirely; always use array-form `proc_open`
2. Validate `CREPUS_BIN` against an allowlist of known-safe paths
3. Apply path validation to the `path` argument before reading

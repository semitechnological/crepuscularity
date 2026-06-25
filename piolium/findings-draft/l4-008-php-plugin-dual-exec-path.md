---
id: l4-008
phase: L4
slug: php-plugin-dual-exec-path
severity: MEDIUM
title: PHP Plugin — Dual Execution Paths with Partial Path Validation
status: rejected-fp
rejection_reason: escapeshellarg() provides adequate shell injection protection on Unix; dual-path pattern is a code quality concern, not independently exploitable
---

## Summary

The PHP plugin (`plugins/php/CrepuscularityPlugin.php`) has two code paths for executing `crepus native ir`: one using `proc_open` with an args array (safe), and one using `exec()` with shell string escaping (partially safe). The `exec()` path is used when no context is provided. While `escapeshellarg()` is used, the dual-path pattern creates maintenance risk and the Windows behavior of `escapeshellarg()` differs.

## Vulnerable Code

**File:** `plugins/php/CrepuscularityPlugin.php:35-38`
```php
$cmd = escapeshellcmd($bin) . ' native ir ' . escapeshellarg($path);
$out = [];
$code = 0;
exec($cmd, $out, $code);
```

**The `proc_open` path at lines 19-33 uses args list (safe):**
```php
$proc = proc_open([$bin, 'native', 'ir', '--stdin-json'], $descriptor, $pipes);
```

## Attack Scenario

The `escapeshellarg()` function on Unix wraps the argument in single quotes and escapes only single quote characters. This is generally safe for preventing shell injection. However:
- On Windows, `escapeshellarg()` behavior differs and may not be safe
- If `escapeshellcmd()` combined with `escapeshellarg()` has edge cases (e.g., very long strings, specific character combinations)
- The `exec()` path uses shell execution (via `/bin/sh` on Unix), while `proc_open` with an args array bypasses the shell entirely

## Root Cause

The PHP plugin has two separate execution paths depending on whether context is provided. The non-context path uses `exec()` with shell escaping instead of the safer args-array approach used by the context path. The `proc_open` with args array should be used consistently.

## Evidence

- `plugins/php/CrepuscularityPlugin.php:35-38` — `exec()` with `escapeshellarg()`
- `plugins/php/CrepuscularityPlugin.php:19` — `proc_open` with args array (safe pattern)
- The safe pattern exists but is only used in the context path

## Existing Mitigations

- `escapeshellarg()` on the `$path` argument prevents basic shell injection on Unix
- No file read check (same arbitrary file read issue as other plugins)

## Priority

**MEDIUM** — `escapeshellarg()` provides reasonable protection, but the dual-path pattern and cross-platform differences are concerning.

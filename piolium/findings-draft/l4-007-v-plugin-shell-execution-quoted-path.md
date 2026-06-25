---
id: l4-007
phase: L4
slug: v-plugin-shell-execution-quoted-path
severity: MEDIUM
title: V Plugin — Shell Execution via os.execute with os.quoted_path Depends on V Compiler Implementation
status: rejected-fp
rejection_reason: os.quoted_path() is a V standard library builtin designed for shell-safe quoting; risk from compiler implementation differences is speculative
---

## Summary

The V plugin (`plugins/v/crepuscularity.v`) uses `os.execute()` with `os.quoted_path()` to construct a shell command. While V's `os.quoted_path()` is intended to safely quote paths for shell execution, the security depends on the V compiler's implementation details and edge case handling (newlines, unicode, null bytes). Additionally, `os.execute()` runs via `/bin/sh` on Unix, so any escaping gaps result in shell injection.

## Vulnerable Code

**File:** `plugins/v/crepuscularity.v:21`
```v
res := os.execute('${os.quoted_path(crepus_bin())} native ir ${os.quoted_path(path)} --ctx ${os.quoted_path(ctx_path)}')
```

## Attack Scenario

If `os.quoted_path()` has edge cases (e.g., does not escape newlines or null bytes in path), an attacker can inject shell commands via crafted path values.

## Root Cause

The V plugin uses shell execution (`os.execute`) instead of process execution with argument list. Although `os.quoted_path` is designed to prevent injection, depending on a V compiler's escaping implementation for security creates an additional maintenance risk — any change to the quoting function or OS-specific behavior could introduce a vulnerability.

## Security Consequence

Potential shell injection if `os.quoted_path()` has implementation gaps. The risk is lower than the C/C++/Zig plugins because `os.quoted_path` is a V builtin designed for this purpose, but the pattern of shell execution is inherently riskier than args-list-based process execution.

## Evidence

- `plugins/v/crepuscularity.v:21` — `os.execute()` with `os.quoted_path()`
- V plugin uses shell execution pattern, not args list

## Existing Mitigations

- `os.quoted_path()` provides shell-safe quoting (depends on V compiler implementation)

## Priority

**MEDIUM** — Lower than C/C++/Zig because `os.quoted_path` is a security-aware function, but the shell execution pattern is inherent risk.

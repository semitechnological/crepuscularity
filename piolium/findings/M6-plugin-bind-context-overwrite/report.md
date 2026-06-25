# M6 — Plugin `bind:` Event Handler — Unrestricted Context Variable Overwrite

**Severity:** MEDIUM  
**Category:** Improper Restriction of Operations (CWE-233)  
**Affected Components:** All plugin bindings (Python, Go, TypeScript, Ruby, PHP)  
**Status:** Validated

## Summary

The `bind:` event handler parses `bind:key:value` from the event `handler` field and writes `key=value` into session context with no allowlist. Any context variable can be overwritten.

## Attack Vector

Attacker dispatches event with handler `bind:baseDir:/attacker/path` or `bind:role:admin` to overwrite template rendering variables.

## Impact

Context variable manipulation affecting template rendering behavior. Application-dependent escalation potential.

## Root Cause

All plugin bindings: `self.context[parts[0]] = parts[1]` with no key allowlist or validation.

## Recommended Fix

Maintain an allowlist of context keys the `bind:` handler can set. Reject any key not on the list.

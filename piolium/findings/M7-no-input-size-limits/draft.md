---
Phase: 8
Sequence: 010
Slug: no-input-size-limits
Verdict: VALID
Rationale: Template source from stdin/JSON has no size limits, enabling memory exhaustion DoS via large payloads in plugin subprocess protocol and SSR server.
Severity-Original: medium
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - crates/crepuscularity-core/src/parser.rs (no size checks)
  - crates/crepuscularity-cli/src/native.rs (no size checks)
status: valid
---

# No Input Size Limits on Template Source

## Summary

The `crepus native ir --stdin-json` and `crepus native ir --stdin` modes accept template source strings with no size limits. The template parser performs no length validation before parsing. An attacker can submit arbitrarily large template payloads, causing excessive memory allocation during parsing, AST construction, and rendering. This affects all execution paths: dev server, SSR server, CLI render, and plugin invocations.

## Location

`crates/crepuscularity-core/src/parser.rs` — the parser accepts the full input string without size checks:

- `--stdin-json` mode: The `template` field in the JSON envelope is read as a String with no length validation
- `--stdin` mode: The full stdin is read into memory with no limit
- Context JSON size: Also unlimited in `--stdin-json` context field

## Attacker Control

The attacker controls the template source sent via:
- Plugin `--stdin-json` template field (via the plugin caller who controls the path)
- Direct `--stdin` input (command line)
- SSR server page generation (if attacker can influence template rendering)

## Trust Boundary Crossed

Plugin → CLI (TB-3) via stdin, and Browser → SSR Server (TB-1) via HTTP request.

## Impact

**MEDIUM** — Denial of service via resource exhaustion:

1. **Memory exhaustion**: A multi-megabyte template with repeated nodes creates a massive AST
2. **CPU exhaustion**: Parsing and rendering enormous templates blocks the thread (rendering runs on `spawn_blocking` threads)
3. **All execution paths affected**: dev server, SSR server, CLI render, plugin invocations

The existing `MAX_INCLUDE_DEPTH` (64) prevents include recursion DoS, but does not limit:
- Total template source size
- Number of AST nodes
- Context JSON size
- Number of files in virtual file map

## Evidence

The Knowledge Base notes T-09: "No protections against large payloads in template parsing." The template parser (`crates/crepuscularity-core/`) accepts the full input string without size validation. There is no `validate_template_size()` or similar function in the codebase.

## Existing Mitigations

- `MAX_INCLUDE_DEPTH = 64` prevents stack overflow from circular includes
- No other size limits exist

## Reproduction Steps

1. Send a 100MB template to `crepus native ir --stdin-json`:
   ```bash
   python -c "
   import json, subprocess
   payload = json.dumps({'template': 'x' * 100_000_000, 'context': {}})
   subprocess.run(['crepus', 'native', 'ir', '--stdin-json'],
                  input=payload, text=True, capture_output=True)
   "
   ```
2. Observe memory consumption spike in the crepus process
3. The process may be OOM-killed on systems with memory limits

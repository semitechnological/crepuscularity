---
id: l4-010
phase: L4
slug: crepus-bin-env-var-injection
severity: MEDIUM
title: All Plugin Bindings — CREPUS_BIN Environment Variable Controls Binary Execution Path
status: rejected-fp
rejection_reason: merged into consolidated p8-003
---

## Summary

All 14+ plugin bindings read the `CREPUS_BIN` environment variable to determine which binary to execute for template rendering. If an attacker can control the environment (e.g., in a shared hosting environment, CI/CD pipeline, or via a compromised parent process), they can redirect the plugin to execute an arbitrary binary. Combined with the unvalidated `path` parameter, this creates a full arbitrary code execution vector.

## Vulnerable Code Locations

**Python** — `plugins/python/crepuscularity_plugin.py:53`
```python
def _crepus_bin() -> str:
    return os.environ.get("CREPUS_BIN", "crepus")
```

**PHP** — `plugins/php/CrepuscularityPlugin.php:11`
```php
$bin = getenv('CREPUS_BIN') ?: 'crepus';
```

**Go** — `plugins/go/crepuscularity.go:35-38`
```go
func crepusBin() string {
    if bin := os.Getenv("CREPUS_BIN"); bin != "" { return bin; }
    return "crepus"
}
```

**TypeScript/Bun** — `plugins/typescript-bun/crepuscularity.ts:7`
```typescript
function crepusBin(): string { return process.env.CREPUS_BIN ?? "crepus" }
```

**Ruby** — `plugins/ruby/crepuscularity_plugin.rb:43`
```ruby
bin = ENV.fetch("CREPUS_BIN", "crepus")
```

**C** — `plugins/c/crepuscularity_plugin.c:7-9`
```c
const char *bin = getenv("CREPUS_BIN");
if (bin == NULL) { bin = "crepus"; }
```

**C++** — same pattern as C

**C#** — `plugins/csharp/Crepusculariy.cs:87` ... wait, the C# plugin uses `Environment.GetEnvironmentVariable("CREPUS_BIN") ?? "crepus"`

**Java** — `plugins/java/CrepuscularityPlugin.java:81`
```java
String bin = System.getenv().getOrDefault("CREPUS_BIN", "crepus");
```

**Kotlin** — same as Java pattern

**Rust** — `plugins/rust/src/lib.rs:57`
```rust
let bin = std::env::var("CREPUS_BIN").unwrap_or_else(|_| { ... "crepus".to_string() });
```

**V** — `plugins/v/crepuscularity.v:14`
```v
fn crepus_bin() string { return os.getenv_opt('CREPUS_BIN') or { 'crepus' } }
```

## Attack

If an attacker can set `CREPUS_BIN=/path/to/malicious_binary`, then when any plugin call invokes:
- Python: `subprocess.run(["/path/to/malicious", "native", "ir", path])`
- C: `popen('"/path/to/malicious" native ir "' + path + '"', "r")`

The malicious binary executes with the same arguments and stdin data.

## Root Cause

The `CREPUS_BIN` env var is used as the executable path without validation. There is no check whether the binary is the expected `crepus` binary, whether it's signed, or whether it's in an allowed directory.

## Evidence

- All 14+ plugins read `CREPUS_BIN` from the environment
- None validate the binary path
- The knowledge base report identifies this as a Hidden Control Channel

## Existing Mitigations

None. The env var is used trustingly.

## Priority

**MEDIUM** — Requires control of environment variables, which has a limited attack surface in typical deployments but is exploitable in CI/CD, shared hosting, or container environments.

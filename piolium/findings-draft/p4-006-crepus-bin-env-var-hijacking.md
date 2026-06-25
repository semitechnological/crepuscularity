---
id: p4-006
phase: L3
slug: crepusbin-env-var-hijacking
severity: high
category: insecure-defaults
cwe: CWE-73
status: rejected-fp
rejection_reason: merged into consolidated p8-003
---

# `CREPUS_BIN` Environment Variable — Binary Redirection in All Plugins

## Summary

All plugin bindings read the `CREPUS_BIN` environment variable to locate the `crepus` binary. If an attacker can control the environment (e.g., via shared hosting, Docker misconfiguration, CI/CD, or parent process inheritance), they can redirect plugin subprocess execution to a malicious binary and achieve arbitrary code execution.

## Vulnerable Code

### Python (`plugins/python/crepuscularity_plugin.py:16-18`)

```python
def _crepus_bin() -> str:
    return os.environ.get("CREPUS_BIN", "crepus")
```

### Go (`plugins/go/crepuscularity.go:25-28`)

```go
func crepusBin() string {
    if bin := os.Getenv("CREPUS_BIN"); bin != "" {
        return bin
    }
    return "crepus"
}
```

### PHP (`plugins/php/CrepuscularityPlugin.php:14`)

```php
$bin = getenv('CREPUS_BIN') ?: 'crepus';
```

### TypeScript/Bun (`plugins/typescript-bun/crepuscularity.ts:6-8`)

```typescript
function crepusBin(): string {
  return process.env.CREPUS_BIN ?? "crepus"
}
```

### Ruby (`plugins/ruby/crepuscularity_plugin.rb:43`)

```ruby
bin = ENV.fetch("CREPUS_BIN", "crepus")
```

## Impact

1. **Arbitrary code execution**: If `CREPUS_BIN=/tmp/malicious`, the plugin subprocess runs the attacker's binary
2. **No validation**: No binary path exists — no hash check, path allowlist, or signature verification
3. **All languages affected**: Every binding uses the same unchecked env var pattern

## Attacker Requirements

The attacker needs the ability to set environment variables in the context where the plugin runs. Common scenarios:
- Shared hosting environments with `.env` file override
- CI/CD with attacker-controlled environment variables  
- Docker containers where `ENV` is set in the image
- Parent process injection (e.g., via `execve` with controlled `envp`)

## Recommended Fix

1. Remove `CREPUS_BIN` env var support, or
2. Validate the resolved binary path against an allowlist of known-safe paths
3. Verify the binary's checksum or signature before execution
4. Log a warning when `CREPUS_BIN` is set to a non-default value

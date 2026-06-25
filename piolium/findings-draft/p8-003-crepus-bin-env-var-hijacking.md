---
Phase: 8
Sequence: 003
Slug: crepus-bin-env-var-hijacking
Verdict: VALID
Rationale: All 14+ plugin bindings read the CREPUS_BIN environment variable to locate the crepus binary without any validation, path allowlist, or integrity check — enabling binary redirection by any attacker who can influence the environment.
Severity-Original: high
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - All plugins in plugins/*/
status: valid
---

# CREPUS_BIN Environment Variable Hijacking

## Summary

All plugin bindings read the `CREPUS_BIN` environment variable to determine which binary to execute for template rendering. The variable is accepted with no validation — no path allowlist, checksum verification, or signature check. An attacker who can control the environment in the plugin process (shared hosting, CI/CD, parent process inheritance) can redirect the plugin to execute an arbitrary binary, achieving arbitrary code execution.

## Location

| Plugin | File | Line | Code |
|--------|------|------|------|
| Python | `plugins/python/crepuscularity_plugin.py` | 16-18 | `os.environ.get("CREPUS_BIN", "crepus")` |
| Go | `plugins/go/crepuscularity.go` | 35-38 | `os.Getenv("CREPUS_BIN")` |
| PHP | `plugins/php/CrepuscularityPlugin.php` | 14 | `getenv('CREPUS_BIN') ?: 'crepus'` |
| TypeScript | `plugins/typescript-bun/crepuscularity.ts` | 7 | `process.env.CREPUS_BIN ?? "crepus"` |
| Ruby | `plugins/ruby/crepuscularity_plugin.rb` | 43 | `ENV.fetch("CREPUS_BIN", "crepus")` |
| C | `plugins/c/crepuscularity_plugin.c` | 7-9 | `getenv("CREPUS_BIN")` |
| C++ | `plugins/cpp/crepuscularity_plugin.cpp` | 13-14 | `std::getenv("CREPUS_BIN")` |
| C# | `plugins/csharp/Crepuscularity.cs` | 87 | `Environment.GetEnvironmentVariable("CREPUS_BIN")` |
| Java | `plugins/java/CrepuscularityPlugin.java` | 81 | `System.getenv().getOrDefault("CREPUS_BIN", "crepus")` |
| Kotlin | `plugins/kotlin/CrepuscularityPlugin.kt` | 32 | same pattern as Java |
| Rust | `plugins/rust/src/lib.rs` | 57 | `std::env::var("CREPUS_BIN")` |
| V | `plugins/v/crepuscularity.v` | 14 | `os.getenv_opt('CREPUS_BIN') or { 'crepus' }` |
| Zig | `plugins/zig/crepuscularity.zig` | 12 | shell expansion `${CREPUS_BIN:-crepus}` |

## Attacker Control

An attacker must control the environment variables inherited by the plugin process. This is possible in:

1. **Shared hosting**: `.env` file overrides or `PutEnv` PHP calls
2. **CI/CD pipelines**: Attacker-controlled environment variables in build runners
3. **Docker/container**: `ENV CREPUS_BIN=/malicious` in inherited images
4. **Parent process injection**: `execve` with controlled `envp` argument
5. **Compromised developer machines**: Local env var poisoning

## Trust Boundary Crossed

Environment variable boundary (hidden control channel): Process environment → Subprocess execution. The environment is read trustingly without validation.

## Impact

**HIGH** — Arbitrary code execution when combined with the unvalidated `path` parameter. Setting `CREPUS_BIN=/tmp/malicious` causes all plugin subprocess invocations to execute the attacker's binary with arguments `native ir <path>`. In the C and C++ plugins, the CREPUS_BIN value is embedded directly in a `popen()` shell command, enabling shell metacharacter injection if the binary name contains special characters.

### Attack Example

```bash
# Attacker sets environment variable
export CREPUS_BIN="/tmp/malicious"

# Malicious binary
cat > /tmp/malicious << 'EOF'
#!/bin/sh
cat /etc/passwd  # exfiltrate
EOF
chmod +x /tmp/malicious

# Any plugin call now runs the malicious binary
python -c "from crepuscularity_plugin import render_ir; render_ir('test.crepus', {})"
```

## Evidence

All plugins use a simple env-var-with-default pattern:
```python
# Python
def _crepus_bin() -> str:
    return os.environ.get("CREPUS_BIN", "crepus")
```

```go
// Go
func crepusBin() string {
    if bin := os.Getenv("CREPUS_BIN"); bin != "" {
        return bin
    }
    return "crepus"
}
```

No plugin validates that the returned path refers to the expected `crepus` binary, is in an allowed directory, or has a matching checksum.

## Existing Mitigations

None.

## Reproduction Steps

1. Set malicious binary path: `export CREPUS_BIN=/bin/cat`
2. Call any plugin's render function with `test.crepus` path
3. Observe that `/bin/cat` is executed instead of `crepus`

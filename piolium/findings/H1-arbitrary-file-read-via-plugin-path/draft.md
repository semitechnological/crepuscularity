---
Phase: 8
Sequence: 002
Slug: arbitrary-file-read-via-plugin-path
Verdict: VALID
Rationale: All 14+ plugin bindings and the CLI accept a caller-controlled path and read it from the filesystem without any canonicalization, prefix check, or traversal rejection — confirmed by source code audit across all bindings.
Severity-Original: high
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - All plugins in plugins/*/
  - crates/crepuscularity-cli/src/native.rs:233-237
status: valid
---

# Arbitrary File Read via Unvalidated Plugin Path Parameter

## Summary

All 14+ plugin bindings (Python, PHP, Go, TypeScript/Bun, Ruby, C, C++, C#, Java, Kotlin, Swift, V, Zig, Rust) and the CLI `crepus native ir <path>` command read a caller-controlled `path` argument from the filesystem using standard file read APIs (`Path.read_text()`, `os.ReadFile()`, `File.read()`, `fs::read_to_string()`, etc.) **without any validation** for path traversal (`..`), absolute paths, symlinks, or canonicalization.

An attacker who controls the `path` parameter can read arbitrary files on the plugin caller's or CLI user's filesystem. The file content is returned in the View IR output or error response, enabling exfiltration.

## Location

| Plugin | File | Line | Sink |
|--------|------|------|------|
| Python | `plugins/python/crepuscularity_plugin.py` | 59 | `Path(path).read_text()` |
| Go | `plugins/go/crepuscularity.go` | 60 | `os.ReadFile(path)` |
| TypeScript/Bun | `plugins/typescript-bun/crepuscularity.ts` | 11 | `Bun.file(path).text()` |
| Ruby | `plugins/ruby/crepuscularity_plugin.rb` | 46 | `File.read(path)` |
| PHP | `plugins/php/CrepuscularityPlugin.php` | 13 | `file_get_contents($path)` |
| C# | `plugins/csharp/Crepuscularity.cs` | 89 | `File.ReadAllTextAsync(path)` |
| Java | `plugins/java/CrepuscularityPlugin.java` | 86 | `Files.readString(Path.of(path))` |
| Kotlin | `plugins/kotlin/CrepuscularityPlugin.kt` | 33 | `Files.readString(Path.of(path))` |
| Rust | `plugins/rust/src/lib.rs` | 70 | `std::fs::read_to_string(path)?` |
| C | `plugins/c/crepuscularity_plugin.c` | 5-20 | `popen()` (reads file via subprocess) |
| C++ | `plugins/cpp/crepuscularity_plugin.cpp` | 12-27 | `popen()` (reads file via subprocess) |
| Swift | `plugins/swift/CrepuscularityPlugin.swift` | 25-28 | `Process` (reads file via subprocess) |
| V | `plugins/v/crepuscularity.v` | 17-24 | `os.execute()` (reads file via subprocess) |
| Zig | `plugins/zig/crepuscularity.zig` | 10-25 | `/bin/sh -c` (reads file via subprocess) |
| **CLI** | `crates/crepuscularity-cli/src/native.rs` | 236 | `fs::read_to_string(&path)` |

## Attacker Control

The attacker controls the `path` argument passed to any plugin's `render_ir()`, `RenderIR()`, `renderIr()`, or the CLI's `crepus native ir <path>` command. For plugins with context support, the file is read on the plugin side BEFORE being sent to the CLI via stdin JSON. This means:

1. The file read happens in the plugin process, outside the Rust CLI's control
2. The `baseDir` field in the stdin JSON envelope is also derived from the attacker-controlled path, enabling include-path traversal on the CLI side
3. Even if the CLI later validates paths, the plugin-side file read is unguarded

### Attack Input

```
path = "../../etc/passwd"
path = "/etc/shadow"
path = "../../.env"
```

## Trust Boundary Crossed

IPC boundary (TB-3): Plugin caller → Filesystem. The attacker-supplied path is used as a direct filesystem read argument with no trust validation.

## Impact

**HIGH** — Arbitrary file read from the plugin caller's or CLI user's system. Sensitive files that can be exfiltrated include:

1. Configuration files containing secrets (`.env`, `config.yml`, `database.yml`)
2. SSH keys (`~/.ssh/id_rsa`, `~/.ssh/authorized_keys`)
3. Source code and proprietary templates
4. Cloud credentials (`~/.aws/credentials`, `~/.gcloud/`)
5. System files (`/etc/passwd`, `/etc/shadow` if readable)

## Evidence

**CLI side** (`crates/crepuscularity-cli/src/native.rs:233-237`):
```rust
let path = parsed.path.ok_or_else(|| {
    "Usage: crepus native ir <file.crepus> [--component Name] [--ctx FILE] [--var k=v] [--pretty]"
        .to_string()
})?;
let content = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
```

Contrast with the template engine's include resolver (`crates/crepuscularity-core/src/include_paths.rs:13`) which implements full path validation:

```rust
pub fn resolve_include_path(base_dir: Option<&Path>, path: &str) -> Result<PathBuf, CrepusError> {
    let requested = Path::new(path);
    if requested.has_root()
        || requested.components().any(|c| {
            matches!(c, std::path::Component::ParentDir | std::path::Component::Prefix(_))
        })
    { return Err(CrepusError::include_path(...)); }
    // ... canonicalization checks ...
}
```

The `native ir` command does NOT reuse this `resolve_include_path` protection.

**Python plugin** (`plugins/python/crepuscularity_plugin.py:59`):
```python
source = Path(path).read_text()  # No path validation before read
```

**Go plugin** (`plugins/go/crepuscularity.go:60`):
```go
func mustRead(path string) string {
    data, err := os.ReadFile(path)  // No path validation, panics on error
    // ...
}
```

## Existing Mitigations

None. The knowledge base report (T-01) identifies this as the single highest-risk unmitigated issue across the entire project.

## Reproduction Steps

1. Create a test file: `echo "secret" > /tmp/test-secret.txt`
2. Using the Python plugin:
   ```python
   from crepuscularity_plugin import render_ir
   result = render_ir("../../tmp/test-secret.txt", {"key": "value"})
   # File content appears in error or rendered output
   ```
3. Using the CLI directly: `crepus native ir ../../tmp/test-secret.txt`

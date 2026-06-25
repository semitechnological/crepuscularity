---
id: l4-004
phase: L4
slug: all-plugins-arbitrary-file-read-path-traversal
severity: HIGH
title: All Plugin Bindings — Arbitrary File Read via Unvalidated Path Parameter
status: rejected-fp
rejection_reason: merged into consolidated p8-002
---

## Summary

Every plugin binding (Python, PHP, Go, TypeScript/Bun, Ruby, C, C++, C#, Java, Kotlin, Swift, V, Zig, Rust) reads the file at the caller-supplied `path` without any validation. An attacker who controls the `path` parameter can read arbitrary files from the plugin caller's filesystem via path traversal (e.g., `../../etc/passwd`). The file content is then returned in the IR output or error message.

## Vulnerable Code Locations

### Python — `plugins/python/crepuscularity_plugin.py:59`
```python
source = Path(path).read_text()  # No path validation
```

### Go — `plugins/go/crepuscularity.go:60`
```go
func mustRead(path string) string {
    data, err := os.ReadFile(path)  // No path validation
    // ...
}
```

### TypeScript/Bun — `plugins/typescript-bun/crepuscularity.ts:11`
```typescript
const template = await Bun.file(path).text()  // No path validation
```

### Ruby — `plugins/ruby/crepuscularity_plugin.rb:46`
```ruby
"template" => File.read(path),  # No path validation
```

### PHP — `plugins/php/CrepuscularityPlugin.php:13`
```php
'template' => file_get_contents($path),  // No path validation
```

### C# — `plugins/csharp/Crepuscularity.cs:89`
```csharp
var template = await File.ReadAllTextAsync(path);  // No path validation
```

### Java — `plugins/java/CrepuscularityPlugin.java:86`
```java
Files.readString(Path.of(path));  // No path validation
```

### Kotlin — `plugins/kotlin/CrepuscularityPlugin.kt:33`
```kotlin
Files.readString(Path.of(path))  // No path validation
```

### Rust — `plugins/rust/src/lib.rs:70`
```rust
let template = std::fs::read_to_string(path)?;  // No path validation
```

### C/C++ — In no-context path, the file is read by `crepus native ir <path>` subprocess

### Swift — The file is read by `crepus native ir <path>` subprocess

## Attack

**Input:** `path = "../../etc/passwd"` (or any absolute path)

**Effect:** The plugin reads the file content, sends it as `template` in the stdin JSON envelope to `crepus native ir --stdin-json`, and the content appears in the View IR output or triggers a parse error that leaks the content.

## Root Cause

None of the plugin bindings validate the `path` before passing it to a file read API. Contrast with `crepuscularity-core/src/include_paths.rs:13` which implements `resolve_include_path()` with canonicalization and prefix checks — the plugin bindings lack equivalent protection.

## Code Path (Python example)

```
plugins/python/crepuscularity_plugin.py
  render_ir(path, context) @ L53
    → Path(path).read_text() @ L59  — reads arbitrary file
    → subprocess.run([bin, "native", "ir", "--stdin-json"], input=json) @ L63
      → content flows to stdout as View IR JSON
```

## Security Consequence

Arbitrary file read from the plugin caller's system. Sensitive files (config, credentials, source code, environment files, SSH keys) can be exfiltrated.

## Evidence

- All 14 plugin bindings perform `File.read(path)` without validation
- No `resolve_include_path()` call in any plugin
- The `context` codepath (present in 10/14 plugins) reads the file before sending to subprocess
- `plugins/python/crepuscularity_plugin.py:57-60` — explicit `Path(path).read_text()`
- `plugins/go/crepuscularity.go:59-61` — explicit `os.ReadFile(path)`

## Existing Mitigations

None. The knowledge base report at T-01 identifies this as the single highest-risk unmitigated issue.

## Priority

**HIGH**

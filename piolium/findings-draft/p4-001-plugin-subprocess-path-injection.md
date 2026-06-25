---
id: p4-001
phase: L3
slug: plugin-subprocess-path-injection
severity: critical
category: command-injection
cwe: CWE-78
status: rejected-fp
rejection_reason: merged into consolidated p8-002 (rescoped to CWE-22 file read for non-shell plugins; shell injection separated to p8-001)
---

# Plugin Subprocess Path Injection (All Languages)

## Summary

All plugin bindings (Python, PHP, Go, TypeScript/Bun, Ruby) pass a caller-controlled template `path` directly to a subprocess invocation of `crepus native ir`. The `path` argument is never validated for `..` traversal, symlinks, or absolute path containment before being read from the filesystem. An attacker controlling the `path` argument can read arbitrary files on the plugin caller's system.

## Vulnerable Code

### Python (`plugins/python/crepuscularity_plugin.py:58-71`)

```python
def render_ir(path: str | Path, context: dict[str, Any] | None = None) -> ViewIr:
    args = [_crepus_bin(), "native", "ir", str(path)]  # path unvalidated
    input_data = None
    if context is not None:
        source = Path(path).read_text()  # path read directly
        ...
    proc = subprocess.run(args, input=input_data, text=True, capture_output=True, check=False)
```

### PHP (`plugins/php/CrepuscularityPlugin.php:19-38`)

```php
$proc = proc_open([$bin, 'native', 'ir', '--stdin-json'], $descriptor, $pipes);
// path is read BEFORE calling subprocess:
file_get_contents($path)
```

### Go (`plugins/go/crepuscularity.go:43-48`)

```go
func RenderIR(path string, context map[string]any) (ViewIr, error) {
    payload, err := json.Marshal(map[string]any{
        "template": mustRead(path),  // path unvalidated, panics on error
        ...
    })
    cmd := exec.Command(crepusBin(), "native", "ir", "--stdin-json")
```

### TypeScript/Bun (`plugins/typescript-bun/crepuscularity.ts:12-18`)

```typescript
export async function renderIr(path: string, ...): Promise<ViewIr> {
  const template = await Bun.file(path).text()  // path unvalidated
  const proc = spawnSync(crepusBin(), ["native", "ir", "--stdin-json"], {
```

### Ruby (`plugins/ruby/crepuscularity_plugin.rb:38-53`)

```ruby
def self.render_ir(path, context = nil)
  if context
    payload = JSON.generate({
      "template" => File.read(path),  # path unvalidated, arbitrary file read
      ...
    })
    stdout, stderr, status = Open3.capture3(bin, "native", "ir", "--stdin-json", stdin_data: payload)
  else
    stdout, stderr, status = Open3.capture3(bin, "native", "ir", path)  # path as subprocess arg
  end
```

## Attacker Control

An attacker who can control the `path` argument (e.g., via user-supplied template path in an application using the plugin) can:

1. **Read arbitrary files**: `../../etc/passwd`, `../../.env`, `../../config/database.yml`
2. **Traverse outside allowed directories**: No prefix or canonicalization check exists
3. **The `path` is read before the subprocess runs**, so even if the subprocess itself validates, the pre-read (`File.read(path)`, `os.ReadFile(path)`, `Bun.file(path).text()`) already exfiltrates the file contents into the IR request payload

## Impact

**CRITICAL**: Arbitrary file read on the system running the plugin caller. Combined with the `baseDir` field in stdin JSON (also unvalidated in all plugins), an attacker can read and include arbitrary files into the template context.

## Evidence from DFD Slice DFD-2

The knowledge base report explicitly notes: "No path validation before subprocess invocation" and "No validation of the `path` argument before passing to subprocess."

## Recommended Fix

Add path canonicalization + prefix checking before every filesystem read, modeled after `resolve_include_path` in `crates/crepuscularity-core/src/include_paths.rs`. Specifically:

1. Reject absolute paths
2. Reject paths containing `..` 
3. Canonicalize the resolved path and verify it starts with an allowed base directory
4. Do this before calling `File.read()`, `os.ReadFile()`, `Bun.file().text()`, etc.

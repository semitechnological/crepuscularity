---
id: p4-002
phase: L3
slug: basedir-stdin-json-injection
severity: high
category: path-traversal
cwe: CWE-22
status: rejected-fp
rejection_reason: merged into consolidated p8-002; baseDir risk is subsumed by the path traversal finding
---

# baseDir in stdin JSON Envelope Not Validated

## Summary

All plugin bindings pass the caller-controlled `baseDir` field inside the stdin JSON envelope to `crepus native ir --stdin-json`. This field controls the directory from which `include` directives resolve relative paths. An attacker controlling `baseDir` can cause template `include` directives to read files from arbitrary directories.

## Vulnerable Code

### Python (`plugins/python/crepuscularity_plugin.py:64`)

```python
payload = {"template": source, "context": context, "baseDir": str(Path(path).parent)}
# baseDir is derived from path, but if context is provided, the path-derived baseDir is used
# The issue: if an attacker controls the `path` argument, they control baseDir
```

### Ruby (`plugins/ruby/crepuscularity_plugin.rb:46`)

```ruby
payload = JSON.generate({
  "template" => File.read(path),
  "context" => context,
  "baseDir" => File.dirname(path)  # caller-controlled path -> caller-controlled baseDir
})
```

### PHP (`plugins/php/CrepuscularityPlugin.php:22-24`)

```php
$payload = json_encode([
    'template' => file_get_contents($path),
    'context' => $context,
    'baseDir' => dirname($path),  # caller-controlled
], JSON_THROW_ON_ERROR);
```

## Impact

The `baseDir` is passed into the `crepus` CLI's `--stdin-json` mode, where the CLI sets it as the `ctx.base_dir` for include resolution. While the Rust-side `resolve_include_path` in `crates/crepuscularity-core/src/include_paths.rs` performs canonicalization checks, the baseDir itself is attacker-controlled, meaning:

1. An attacker can set `baseDir: "/etc"` and then include a template `include "passwd"` → file read
2. The canonicalization check only verifies the resolved path stays under the canonical base directory, but it trusts the base directory value from an unauthenticated source
3. The **pre-read** of the file (`File.read(path)` in plugins) happens OUTSIDE the Rust CLI process, so the attacker's path is read before any canonicalization occurs

## Attacker Control

The attacker controls both `path` and `context` fields in the stdin JSON. The `baseDir` is derived from `path` which is attacker-controlled. The attacker can also directly control `baseDir` if the plugin implementation is modified to accept it from external input.

## Recommended Fix

1. Validate the `path` on the plugin side before calling `File.read()`, `os.ReadFile()`, etc.
2. Reject paths with `..`, absolute paths, or symlinks
3. Apply canonicalization + prefix check matching `resolve_include_path` behavior

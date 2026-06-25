---
id: p4-005
phase: L3
slug: ruby-file-read-arbitrary-file-read
severity: high
category: path-traversal
cwe: CWE-22
status: rejected-fp
rejection_reason: merged into consolidated p8-002
---

# Ruby Plugin `File.read(path)` — Arbitrary File Read with No Validation

## Summary

The Ruby plugin reads the template file at `path` using `File.read(path)` with no path validation, canonicalization, or prefix check. An attacker controlling the `path` can read any file on the system.

## Vulnerable Code

`plugins/ruby/crepuscularity_plugin.rb:44-46`

```ruby
payload = JSON.generate({
  "template" => File.read(path),  # arbitrary file read
  "context" => context,
  "baseDir" => File.dirname(path)
})
```

And at line 53:

```ruby
stdout, stderr, status = Open3.capture3(bin, "native", "ir", path)  # path as argv
```

## Impact

1. **Arbitrary file read**: `File.read(path)` reads any file the Ruby process can access
2. The file content is sent as the `template` field in the stdin JSON payload to the `crepus` CLI
3. The `baseDir` is also derived from the attacker-controlled path, compounding the include-path traversal risk

## Attacker Control

The attacker controls the `path` argument to `CrepuscularityPlugin.render_ir(path, context)` or `ViewSession.new(path, context)`.

## Evidence

The path is accepted both through the `render_ir` method and through the `ViewSession` constructor, which stores it in `@path` and uses it in `render_ir`.

## Recommended Fix

1. Reject paths containing `..`
2. Reject absolute paths
3. Canonicalize and verify path stays under an allowed base directory
4. Apply before calling `File.read()`

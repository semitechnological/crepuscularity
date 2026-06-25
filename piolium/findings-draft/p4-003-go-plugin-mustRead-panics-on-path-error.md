---
id: p4-003
phase: L3
slug: go-plugin-mustread-panics-on-path
severity: high
category: path-traversal
cwe: CWE-22
status: rejected-fp
rejection_reason: merged into consolidated p8-002
---

# Go Plugin `mustRead` Panics on Path Error, No Validation

## Summary

The Go plugin uses a `mustRead(path)` helper that calls `os.ReadFile(path)` and **panics** on error, with no path validation before the read. If the caller passes an invalid, inaccessible, or traversed path, the plugin panics (denial of service) or reads arbitrary files.

## Vulnerable Code

`plugins/go/crepuscularity.go:46-51`

```go
func RenderIR(path string, context map[string]any) (ViewIr, error) {
    payload, err := json.Marshal(map[string]any{
        "template": mustRead(path),  // <-- panic on any read error
        "context":  context,
    })
    // ...
}

func mustRead(path string) string {
    data, err := os.ReadFile(path)
    if err != nil {
        panic(err)  // <-- panic, not error return
    }
    return string(data)
}
```

## Impact

1. **Arbitrary file read**: `path` is caller-controlled with no validation for `..`, absolute paths, or symlinks
2. **Denial of service**: A non-existent path causes a panic that crashes the caller
3. **No recovery**: Unlike the Python/TypeScript plugins which return errors, Go plugin panics

## Attacker Control

The attacker controls the `path` argument to `RenderIR()`. Since Go uses `os.ReadFile()` without canonicalization, the attacker can read any file the process has access to.

## Recommended Fix

1. Replace `mustRead` with error-propagating read
2. Add path validation: reject `..`, absolute paths, symlinks
3. Use `filepath.Clean()` + `filepath.Abs()` + prefix check with `strings.HasPrefix()`

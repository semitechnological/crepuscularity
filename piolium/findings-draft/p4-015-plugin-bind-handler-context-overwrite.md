---
id: p4-015
phase: L3
slug: plugin-bind-handler-context-overwrite
severity: medium
category: hidden-control-channel
cwe: CWE-233
status: rejected-fp
rejection_reason: superseded by p8-009 (consolidated version with additional evidence)
---

# Plugin `bind:` Event Handler — Hidden Context Variable Overwrite

## Summary

All plugin bindings implement a `bind:` event handler pattern that allows event payloads to overwrite session context variables. When a dispatched event's `handler` field starts with `bind:`, the handler parses the remainder as `key:value` and writes it into the session context. An attacker controlling event input can overwrite any context variable, including ones that affect template rendering behavior (include paths, rendering options, security-relevant configuration).

## Vulnerable Code

### Python (`plugins/python/crepuscularity_plugin.py:46-48`)

```python
if handler.startswith("bind:"):
    parts = handler.removeprefix("bind:").split(":", 1)
    if len(parts) == 2:
        self.context[parts[0]] = parts[1]
```

### Ruby (`plugins/ruby/crepuscularity_plugin.rb:35-37`)

```ruby
if handler.start_with?("bind:")
    key, value = handler.delete_prefix("bind:").split(":", 2)
    @context[key] = value unless value.nil?
```

### TypeScript (`plugins/typescript-bun/crepuscularity.ts:62-64`)

```typescript
if (parsed.handler.startsWith("bind:")) {
    const [, key, ...rest] = parsed.handler.split(":")
    this.context[key] = rest.join(":")
}
```

### Go (`plugins/go/crepuscularity.go:82-85`)

```go
if strings.HasPrefix(event.Handler, "bind:") {
    parts := strings.SplitN(strings.TrimPrefix(event.Handler, "bind:"), ":", 2)
    if len(parts) == 2 {
        session.Context[parts[0]] = parts[1]
    }
}
```

### PHP (`plugins/php/CrepuscularityPlugin.php:81-84`)

```php
if (str_starts_with($handler, 'bind:')) {
    $parts = explode(':', substr($handler, 5), 2);
    if (count($parts) === 2) {
        $this->context[$parts[0]] = $parts[1];
    }
}
```

## Impact

Context variables are used during template rendering. Overwriting them via the `bind:` handler can:

1. Override `baseDir` (or similar) affecting include resolution (if passed through)
2. Override template variables used in interpolation
3. Affect rendering logic that depends on context variable values

## Attacker Control

An attacker who can dispatch events to the plugin session (e.g., via user-triggered event payloads) can set any context key to any string value.

## Recommended Fix

1. Define an allowlist of context keys that can be set via `bind:` events
2. Validate key names (reject `__`, `baseDir`, `config`, etc.)
3. Log warnings when blocked `bind:` attempts occur

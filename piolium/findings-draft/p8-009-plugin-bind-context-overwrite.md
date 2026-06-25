---
Phase: 8
Sequence: 009
Slug: plugin-bind-context-overwrite
Verdict: VALID
Rationale: All plugin bindings implement bind: handler that allows overwriting any context variable from event payloads without an allowlist, enabling injection into template rendering variables.
Severity-Original: medium
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - plugins/python/crepuscularity_plugin.py:46-48
  - plugins/go/crepuscularity.go:82-85
  - plugins/typescript-bun/crepuscularity.ts:62-64
  - plugins/ruby/crepuscularity_plugin.rb:35-37
  - plugins/php/CrepuscularityPlugin.php:81-84
status: valid
---

# Plugin `bind:` Event Handler — Unrestricted Context Variable Overwrite

## Summary

All plugin bindings implement a `bind:` event handler pattern. When a dispatched event's `handler` field starts with `bind:`, the handler parses the remainder as `key:value` and writes it into the session context. An attacker controlling event input can overwrite ANY context variable, including ones that affect template rendering behavior — no allowlist restricts which keys can be set.

## Location

| Plugin | File | Lines | Code |
|--------|------|-------|------|
| Python | `plugins/python/crepuscularity_plugin.py` | 46-48 | `self.context[parts[0]] = parts[1]` |
| Go | `plugins/go/crepuscularity.go` | 82-85 | `session.Context[parts[0]] = parts[1]` |
| TypeScript | `plugins/typescript-bun/crepuscularity.ts` | 62-64 | `this.context[key] = rest.join(":")` |
| Ruby | `plugins/ruby/crepuscularity_plugin.rb` | 35-37 | `@context[key] = value` |
| PHP | `plugins/php/CrepuscularityPlugin.php` | 81-84 | `$this->context[$parts[0]] = $parts[1]` |

## Attacker Control

An attacker who can dispatch events to the plugin session (e.g., via user-triggered event payloads in an application using the plugin) can set any context key to any string value. The event payload's `handler` field is attacker-controlled.

## Trust Boundary Crossed

Event dispatcher → Session context (within the plugin process, but an untrusted event dispatcher can modify context variables used in template rendering).

## Impact

**MEDIUM** — Context variables are used during template rendering. Overwriting them via the `bind:` handler can:

1. Override template variables used in interpolation (e.g., `{{ user.name }}`)
2. Affect rendering logic that depends on context variable values
3. In application-specific scenarios, context variables might control template include paths, rendering options, or security-relevant configuration

### Python example

```python
session = ViewSession("template.crepus", {"role": "admin"})

# Attacker dispatches a bound event that overwrites context
session.dispatch({"handler": "bind:role:user"})

# Now session.context["role"] == "user"
```

### Go example

```go
session := NewViewSession("template.crepus", map[string]any{"role": "admin"})

// Attacker dispatches a bound event
session.Dispatch(Event{Handler: "bind:role:user"})

// Now session.Context["role"] == "user"
```

## Evidence

**Python** (`plugins/python/crepuscularity_plugin.py:46-48`):
```python
if handler.startswith("bind:"):
    parts = handler.removeprefix("bind:").split(":", 1)
    if len(parts) == 2:
        self.context[parts[0]] = parts[1]
```

**Go** (`plugins/go/crepuscularity.go:82-85`):
```go
if strings.HasPrefix(event.Handler, "bind:") {
    parts := strings.SplitN(strings.TrimPrefix(event.Handler, "bind:"), ":", 2)
    if len(parts) == 2 {
        session.Context[parts[0]] = parts[1]
    }
}
```

**TypeScript** (`plugins/typescript-bun/crepuscularity.ts:62-64`):
```typescript
if (parsed.handler.startsWith("bind:")) {
    const [, key, ...rest] = parsed.handler.split(":")
    this.context[key] = rest.join(":")
}
```

No plugin validates the key name against an allowlist. Any string key can be overwritten.

## Existing Mitigations

- None. The `bind:` handler trusts any event with a matching prefix.

## Reproduction Steps

1. Create a ViewSession with context variables
2. Dispatch an event with handler `bind:baseDir:/attacker/controlled/path`
3. Observe that the context `baseDir` is overwritten
4. Subsequent template rendering uses the modified context

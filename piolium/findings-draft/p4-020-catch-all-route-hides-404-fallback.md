---
id: p4-020
phase: L3
slug: ssr-catchall-route-fallback
severity: low
category: misconfiguration
cwe: CWE-200
status: rejected-fp
rejection_reason: LOW severity per policy — only Medium+ findings survive; documented behavior with no file system resolution
---

# SSR Catch-All Route Silently Falls Back to Root Route on 404

## Summary

The SSR router registers `"/*path"` as a catch-all route. When a requested path is not found in the route table, it silently falls back to rendering the `/` route entry. This means 404 errors are never surfaced to the user and any path on the domain renders the root template.

## Vulnerable Code

`crates/crepuscularity-ssr/src/router.rs:47-49`

```rust
let entry = match routes.get(path).or_else(|| routes.get("/")) {
    Some(e) => e,
    None => return Html("<h1>404 Not Found</h1>".to_string()),
};
```

If the root path `/` is registered (which it always is), **every unknown URL path** renders the root template. Only if `/` is also not registered does a 404 appear.

## Impact

1. **Unintended content disclosure**: Every URL path on the domain renders some content instead of a 404
2. **Web fingerprinting**: Attackers cannot distinguish between existing and non-existing routes
3. **Cache poisoning**: Different URLs serving the same content may confuse caching layers
4. **SEO issues**: Search engines may see duplicate content across many URLs

## Evidence

The knowledge base notes: "Route fallback to `/` with no file system resolution" as mitigation for T-06, but this behavior itself is documented as: "Route fallback to `/` — This could misdirect users but not security-critical."

However, this behavior does have genuine security implications for content hosting scenarios where:

1. A secret page is removed but a cached copy exists elsewhere
2. The lack of 404 allows URL enumeration without triggering 404 alerts
3. `<iframe>` or Open Graph preview of arbitrary paths renders unintended content

## Recommended Fix

1. Return a true 404 page for unregistered routes instead of falling back to `/`
2. Log 404 attempts for monitoring
3. Add an option to disable catch-all fallback behavior

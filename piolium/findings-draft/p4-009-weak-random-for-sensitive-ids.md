---
id: p4-009
phase: L3
slug: weak-random-for-sensitive-ids
severity: medium
category: weak-crypto
cwe: CWE-330
status: rejected-fp
rejection_reason: Math.random() cache keys and element IDs have no exploitable security impact; no attacker control path for cache poisoning or DOM clobbering of local variables
---

# Weak Randomness Used for Cache Keys and Element IDs

## Summary

Multiple locations use `Math.random()` for cache key generation and DOM element IDs. `Math.random()` is NOT cryptographically secure and can be predicted by an attacker with enough samples.

## Vulnerable Code Locations

### 1. Content script cache key (`crates/crepuscularity-webext/assets/content.js:15`)

```javascript
const cacheKey = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
```

This cache key is appended to WASM module URLs (`?v=${cacheKey}`) and used for cache-busting. If an attacker can predict the cache key, they may be able to serve malicious content on the extension's origin.

### 2. Island element IDs (`crates/crepuscularity-cli/assets/web/app.js:113`)

```javascript
el.id = "crepus-island-" + Math.random().toString(36).slice(2, 8);
```

Element IDs are used for DOM identification and may be referenced in CSS selectors or stored in page state. Predictable IDs can lead to DOM clobbering attacks.

## Impact

1. **Cache poisoning**: Predictable cache keys allow attackers to serve stale or malicious content on the extension's privileged `chrome-extension://` origin
2. **DOM clobbering**: Predictable element IDs increase the risk of DOM clobbering attacks on third-party pages
3. **Fingerprinting**: Weak random patterns enable long-term tracking of extension instances

## Recommended Fix

Use `crypto.getRandomValues()` (Web Crypto API) instead of `Math.random()`:

```javascript
function secureRandom() {
  const arr = new Uint32Array(1);
  crypto.getRandomValues(arr);
  return arr[0].toString(36);
}
const cacheKey = `${Date.now()}-${secureRandom()}`;
```

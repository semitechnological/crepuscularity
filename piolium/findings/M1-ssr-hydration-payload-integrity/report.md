# M1 — SSR Hydration Payload Missing Integrity Protection

**Severity:** MEDIUM  
**Category:** Insufficient Verification of Data Authenticity (CWE-345)  
**Affected Components:** `crates/crepuscularity-web/src/ssr.rs`  
**Status:** Validated

## Summary

SSR hydration payload embedded as base64 JSON in `<script id="__crepus_hydration__">` has no HMAC/signature. Any attacker with stored-XSS or MITM position can tamper with client-side template state.

## Attack Vector

Stored XSS on SSR page or MITM replaces the base64 payload with crafted JSON that modifies template context variables used by the WASM runtime.

## Impact

Manipulation of template state, binding descriptors, and context variables on the client side. Not directly RCE, but alters application behavior.

## Root Cause

`crates/crepuscularity-web/src/ssr.rs:230-237` — payload base64-encoded but unsigned.

## Recommended Fix

Add an HMAC tag computed over the JSON payload with a server-side secret, verified on the client before use.

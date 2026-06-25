---
id: p4-007
phase: L3
slug: ssr-hydration-payload-integrity
severity: medium
category: integrity
cwe: CWE-345
status: rejected-fp
rejection_reason: superseded by p8-004 (consolidated version with additional evidence)
---

# SSR Hydration Payload Has No Integrity Protection

## Summary

The SSR hydration manifest is embedded as a base64-encoded JSON payload inside a `<script>` tag with `id="__crepus_hydration__"`. The payload contains serialized context variables and binding descriptors. There is no HMAC, signature, or integrity check, so a stored XSS or MITM attacker can tamper with the hydration payload to manipulate the client-side template state.

## Vulnerable Code

`crates/crepuscularity-web/src/ssr.rs:230-237`

```rust
fn append_hydration_payload(
    html: &mut String,
    ctx: &TemplateContext,
    bind: &BindMap,
) -> Result<(), CrepusError> {
    let ctx_val = serialize_ctx_for_ssr(ctx)?;
    let raw = crate::hydration_payload_bytes(ctx_val, Value::Object(bind.clone()))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(raw);
    let script = format!(
        r#"<script type="application/json" id="__crepus_hydration__" data-crepus-encoding="base64">{b64}</script>"#
    );
```

## Impact

The hydration JSON is decoded and used by the WASM runtime for reactive state initialization. An attacker who can modify the HTML response (stored XSS, MITM, or CDN compromise) can:

1. Modify context variables used in template rendering
2. Manipulate binding descriptors to affect client-side behavior
3. Inject arbitrary data that flows into WASM state initialization

## Attacker Control

- Stored XSS in a page that includes the hydration script
- CDN/middleware compromise on the serving path
- Shared hosting with write access to HTML files

## Evidence from Knowledge Base

The knowledge base explicitly lists this as T-10: "No protection — hydration payload integrity not verified".

## Recommended Fix

1. Wrap the hydration JSON in an HMAC-SHA256 using a server-side secret key
2. Verify the HMAC on the client side before processing the payload
3. Alternatively, sign with a session-bound key embedded as a cookie (if sessions exist)

```rust
// Example fix:
use hmac::{Hmac, Mac};
use sha2::Sha256;
let mut mac = Hmac::<Sha256>::new_from_secret(secret_key)?;
mac.update(&raw);
let signature = hex::encode(mac.finalize().into_bytes());
let payload_with_sig = json!({"data": raw_b64, "sig": signature});
```

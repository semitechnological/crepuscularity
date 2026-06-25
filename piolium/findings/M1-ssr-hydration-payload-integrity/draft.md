---
Phase: 8
Sequence: 004
Slug: ssr-hydration-payload-integrity
Verdict: VALID
Rationale: Hydration payload embedded as base64 JSON in HTML has no HMAC or integrity check, allowing tampering by any attacker with stored-XSS or MITM position on the HTML response — confirmed by source code audit of the SSR hydration output.
Severity-Original: medium
PoC-Status: pending
Pre-FP-Flag: none
Debate: piolium/attack-surface/balanced-chamber-summary.md
Sources:
  - crates/crepuscularity-web/src/ssr.rs:230-237
status: valid
---

# SSR Hydration Payload Integrity Missing

## Summary

The SSR hydration manifest is embedded as a base64-encoded JSON payload inside a `<script>` tag with `id="__crepus_hydration__"`. The payload contains serialized context variables and binding descriptors. There is no HMAC, signature, or integrity check, so a stored XSS or MITM attacker can tamper with the hydration payload to manipulate the client-side template state.

## Location

`crates/crepuscularity-web/src/ssr.rs:230-237`

## Attacker Control

The attacker must have a position that can modify the HTML response:

1. **Stored XSS** on any page served by the SSR server that includes the hydration script
2. **MITM** between the SSR server and browser (if HTTPS is not enforced)
3. **CDN or proxy compromise** that modifies responses in transit

## Trust Boundary Crossed

Network boundary (TB-1): SSR Server → Browser. The hydration payload crosses the trust boundary without integrity protection.

## Impact

**MEDIUM** — The hydration JSON is decoded and used by the WASM runtime for reactive state initialization. An attacker who can modify the HTML response can:

1. Modify context variables used in template rendering
2. Manipulate binding descriptors to affect client-side behavior
3. Inject arbitrary data that flows into WASM state initialization

The impact is limited by the WASM runtime's parsing: arbitrary code execution is not directly achievable, but manipulated state can alter application behavior.

## Evidence

`crates/crepuscularity-web/src/ssr.rs:230-237`:
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

The payload is base64-encoded but not signed or HMAC-authenticated. The client-side WASM runtime decodes and trusts the payload without verifying its origin.

The Knowledge Base lists this as T-10: "No protection — hydration payload integrity not verified."

## Existing Mitigations

- None. The payload is protected only by the transport layer (HTTPS), which is insufficient if the attacker has stored XSS or proxy access.

## Reproduction Steps

1. Set up an SSR server serving a template with context variables
2. Intercept the HTML response (or use stored XSS to modify it)
3. Replace the hydration script's base64 content with a crafted payload
4. Observe the client-side WASM runtime using the modified state

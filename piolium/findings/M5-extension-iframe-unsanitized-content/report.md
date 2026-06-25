# M5 — Extension Iframe Content Not Independently Sanitized

**Severity:** MEDIUM  
**Category:** Cross-Site Scripting (CWE-79)  
**Affected Components:** `crates/crepuscularity-webext/assets/content.js`  
**Status:** Validated

## Summary

When the WASM module signals `needs_iframe: true`, widget content goes into an iframe and bypasses `sanitizeHTML()` entirely. The iframe has `sandbox="allow-scripts"` (no `allow-same-origin`), limiting but not eliminating risk.

## Attack Vector

Iframe content from WASM is not sanitized by the content script. Scripts execute inside the sandboxed iframe. Sandbox escape or message channel abuse could escalate.

## Impact

Script execution in sandboxed iframe. Message spoofing via `attachFrameResize()`. Resource exfiltration via iframe network access.

## Root Cause

`content.js:342-345` — iframe path calls `createIframeMount()` with raw WASM output; no `sanitizeHTML()` call in this code path.

## Recommended Fix

Run iframe HTML content through the same `sanitizeHTML()` pipeline before setting as `srcdoc`. Add message origin validation in resize handler.

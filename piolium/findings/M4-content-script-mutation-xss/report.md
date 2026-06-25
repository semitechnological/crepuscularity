# M4 — Content Script Mutation XSS via DOMParser/InnerHTML Dual Parse

**Severity:** MEDIUM  
**Category:** Cross-Site Scripting (CWE-79)  
**Affected Components:** `crates/crepuscularity-webext/assets/content.js`  
**Status:** Validated

## Summary

`sanitizeHTML()` parses attacker-controlled HTML with `DOMParser`, serializes back, then assigns via `innerHTML`. Two different HTML parsers -> parser differential -> mXSS risk.

## Attack Vector

DOMParser and the browser's rendering parser disagree on DOM representation of crafted HTML. Known mXSS bypasses (CVE-2021-21225, CVE-2020-6511, CVE-2019-11357) use namespace confusion, deep nesting, or foreign content to survive sanitization.

## Impact

If bypass found, arbitrary JS execution in extension context with extension API access.

## Root Cause

`content.js:285-324` + `content.js:355` — dual-parse pattern is inherently risky. Restrictive tag allowlist reduces but doesn't eliminate risk.

## Recommended Fix

Use `textContent` or `setHTML()` (Sanitizer API) instead of DOMParser+innerHTML. Or verify output matches expected DOM structure before assignment.

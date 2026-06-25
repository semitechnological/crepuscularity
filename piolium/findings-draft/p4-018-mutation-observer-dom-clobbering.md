---
id: p4-018
phase: L3
slug: content-script-mutation-observer-clobbering
severity: medium
category: dom-clobbering
cwe: CWE-79
status: rejected-fp
rejection_reason: DOM clobbering cannot affect local const/let variables in content script's isolated world; node.innerHTML is a DOM property not subject to window clobbering
---

# Content Script MutationObserver Creates Race Condition with Third-Party DOM

## Summary

The content script uses a `MutationObserver` on `document.documentElement` that processes `addedNodes` for content matching `<pre>` elements. A third-party page can exploit this by dynamically injecting malicious `<pre>` elements or by performing DOM clobbering that interferes with the content script's helper functions.

## Vulnerable Code

`crates/crepuscularity-webext/assets/content.js:332-339`

```javascript
const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
        mutation.addedNodes.forEach((node) => queueNode(node));
    }
    if (pending.size > 0) {
        scheduleFlush();
    }
});
observer.observe(document.documentElement, { subtree: true, childList: true });
```

## Issues

1. **`queueNode(node)` called on every added node**, including text nodes and non-element nodes — `hasAnywhereContent()` checks `node.textContent` which could be clobbered
2. **`hasAnywhereContent(node)` reads `node.innerHTML`** — a third-party page can define a getter on `innerHTML` that returns malicious content when read by the content script
3. **`widgetTextFromPre(pre)` accesses `pre.querySelector("code")`** — a page could inject a `<code>` element inside a `<pre>` to manipulate the extracted text

## Impact

Third-party pages can craft specific DOM structures that trigger unexpected behavior in the content script:

1. **DOM clobbering of `innerHTML`**: `<img name="innerHTML">` creates `window.innerHTML` reference
2. **Fingerprinting**: Observing which `<pre>` elements are replaced with extension widgets reveals extension state
3. **Timing attacks**: The `requestAnimationFrame` flush creates predictable timing windows

## Recommended Fix

1. Validate `node instanceof Element` before accessing `.textContent` and `.innerHTML`
2. Use `Object.prototype.hasOwnProperty.call(node, 'textContent')` instead of direct property access
3. Wrap the mutation handler in a try/catch to prevent page exceptions from breaking the extension
4. Consider using `TreeWalker` for more robust DOM traversal

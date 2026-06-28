# Hypotheses — Backward + Contradiction Reasoning

Generated: 2026-06-25T05:35:00Z

---

## Backward Reasoner Hypotheses (sink → source)

### H1: C Plugin — Shell Command Injection via popen

**Sink**: `popen(cmd, "r")` at `plugins/c/crepuscularity_plugin.c:9`
**Source**: `path` = `argv[1]` (caller-supplied argument from `main` at line 38)
**Code path**:
```
main(argc, argv) → argv[1] → crepus_render_ir(path, ...) → snprintf(cmd, ...) → popen(cmd, "r")
```
**Missing control**: No input validation, sanitization, or escaping between `argv[1]` and `popen()`. The format string `"\"%s\" native ir \"%s\""` wraps path in quotes, but shell metacharacters like `;`, `` ` ``, `$()`, `|` break out of the quoted context.
**Attack**: `path = "\";id>/tmp/crepus_pwn;\""` results in shell command `"crepus" native ir "";id>/tmp/crepus_pwn;""`
**Consequence**: Arbitrary command execution as the plugin caller's user.

### H2: C++ Plugin — Shell Command Injection via popen

**Sink**: `popen(cmd.c_str(), "r")` at `plugins/cpp/crepuscularity_plugin.cpp:17`
**Source**: `path` parameter (from `argv[1]` via `render_ir(path)`)
**Code path**:
```
main(argc, argv) → argv[1] → render_ir(path) → cmd = "\"" + bin + "\" native ir \"" + path + "\"" → popen(cmd.c_str(), "r")
```
**Missing control**: Same as H1 — no validation, string concatenation into shell command.
**Consequence**: Arbitrary command execution.

### H3: Zig Plugin — Shell Command Injection via /bin/sh -c

**Sink**: `std.process.Child.run(.{ .argv = &.{"/bin/sh", "-c", command} })` at `plugins/zig/crepuscularity.zig:13`
**Source**: `path` parameter
**Code path**:
```
renderIr(allocator, path) → fmt.allocPrint(..., "\"${CREPUS_BIN:-crepus}\" native ir \"{s}\"", .{path}) → ShellChild.run(sh, -c, command)
```
**Missing control**: `allocPrint` embeds `path` directly into a shell command string, then passes to `sh -c`. Shell metacharacters in path break the quote context.
**Consequence**: Arbitrary command execution.

### H4: All Plugin Bindings — Arbitrary File Read via Unvalidated Path

**Sink**: File read before subprocess invocation — the plugin reads the file at `path` on the caller's system.
**Source**: `path` parameter from caller
**Code path**:
```
Python:  render_ir(path) → Path(path).read_text()    (line 57)
Go:      RenderIR(path)  → os.ReadFile(path)         (line 43 → 63)
TS/Bun:  renderIr(path)  → Bun.file(path).text()     (line 11)
Ruby:    render_ir(path) → File.read(path)            (line 46)
PHP:     renderIr(path)  → file_get_contents(path)    (line 13)
C#:      RenderIrAsync(path) → File.ReadAllTextAsync(path)  (line 79)
Java:    renderIr(path)  → Files.readString(Path.of(path))  (line 86)
Kotlin:  renderIr(path)  → Files.readString(Path.of(path))  (line 33)
Rust:    render_ir(path) → std::fs::read_to_string(path)    (line 60)
```
**Missing control**: NONE of the plugins validate the `path` before reading the file. Path traversal (e.g., `../../etc/passwd`) works in every binding.
**Consequence**: Arbitrary file read from the plugin caller's filesystem. Combined with the `context` codepath (which sends file content as `template` to stdin JSON), the file content appears in the IR output.

### H5: CLI `crepus native ir` — No Path Validation

**Sink**: `fs::read_to_string(&path)` at `crates/crepuscularity-cli/src/native.rs:133`
**Source**: `path` from CLI argument or stdin JSON `template`
**Code path**:
```
plugin subprocess → crepus native ir <path> → parse_ir_args → path → fs::read_to_string(&path)
```
**Missing control**: Unlike `resolve_include_path` in `include_paths.rs`, the CLI's `native.rs` does NOT validate the path argument. It directly calls `fs::read_to_string()` on the path without canonicalization or prefix checks.
**Consequence**: Path traversal in the CLI context — attacker who controls the plugin subprocess stdin or args can read any file the `crepus` process has access to.

### H6: Content Script `sanitizeHTML()` — Mutation XSS

**Sink**: `root.innerHTML = sanitizeHTML(html)` at `content.js:355`
**Source**: Third-party web page `<pre>` element content → WASM `render_anywhere_parts` → HTML string
**Vulnerability analysis**:
- `sanitizeHTML()` at `content.js:296-324` uses `DOMParser` to parse the HTML
- It removes disallowed tags and strips dangerous attributes
- BUT: the sanitized HTML (`doc.body.innerHTML`) is then assigned to `root.innerHTML`
- This triggers a SECOND browser parse of the same HTML
- Known mXSS vectors exist where DOMParser and browser HTML parser disagree:
  - `<noscript>` elements (rendered differently depending on scripting state)
  - `<style>` element contents
  - `<svg>` and `<math>` namespaced elements
  - Form elements inside tables
- The tag allowlist (DIV, SPAN, P, B, I, EM, STRONG, A, UL, OL, LI, H1-H6, BR, HR, TABLE, THEAD, TBODY, TR, TH, TD, BLOCKQUOTE, PRE, CODE) includes TABLE/THEAD/TBODY/TR/TH/TD which are known mXSS vectors when combined with nested elements
**Consequence**: Stored cross-site scripting in the extension context. Attacker-controlled script execution in the shadow DOM, with ability to:
- Access extension storage APIs
- Make requests with extension permissions
- Perform actions on behalf of the extension

### H7: `createIframeMount` — No Content Sanitization for iframe Documents

**Sink**: `setIframeDocument(frame, srcdoc)` at `content.js:164-175` — sets `frame.src = url` (Blob URL) or `frame.srcdoc = srcdoc`
**Source**: WASM module's `render_frame_doc()` / `render_anywhere_frame_doc()` output
**Vulnerability**:
- The iframe has `sandbox="allow-scripts"` (no `allow-same-origin`)
- But the iframe content is NOT sanitized independently — the WASM module's output goes directly into the iframe
- Any `<script>` tags in the WASM output would execute inside the sandboxed iframe
- While the sandbox prevents access to parent DOM, the iframe can still:
  - Make network requests to third-party origins
  - Use storage APIs accessible from the blob URL origin (depending on browser behavior)
  - Run crypto miners or other arbitrary JS
**Consequence**: Arbitrary JavaScript execution in a sandboxed iframe. Limited impact due to `sandbox="allow-scripts"` alone, but could be combined with UX phishing attacks within the iframe.

---

## Contradiction Reasoner Hypotheses (breaking assumptions)

### H8: "Args list protects from ALL exploitation" — Contradiction via Arbitrary File Read

**Assumption**: Plugins that pass arguments as lists (Python, Go, TS, C#, Java, Kotlin, Swift, Rust) are safe from command injection — which is true for shell injection, but:
**Contradiction**: They ALL read the file at `path` BEFORE invoking the subprocess. The file content is then sent to `crepus` via stdin JSON.
- Python: `Path(path).read_text()` at line 57 → content sent as `template` in stdin JSON
- Go: `mustRead(path)` at line 43 → `os.ReadFile(path)` at line 63 → content in stdin JSON
- TS: `Bun.file(path).text()` at line 11 → content in stdin JSON
- Ruby: `File.read(path)` at line 46 → content in stdin JSON
- C#: `File.ReadAllTextAsync(path)` at line 79 → content in stdin JSON
- Java: `Files.readString(Path.of(path))` at line 86 → content in stdin JSON
- Kotlin: `Files.readString(Path.of(path))` at line 33 → content in stdin JSON
- Rust: `std::fs::read_to_string(path)` at line 60 → content in stdin JSON
**Result**: Even without shell injection, ANY plugin caller can read ARBITRARY FILES from the plugin host system via path traversal.

### H9: "PHP escapeshellarg prevents injection" — Contradiction via Alternative Code Path

**Assumption**: The PHP plugin uses `escapeshellarg()` for the path at line 38, making it safe.
**Contradiction**: PHP has TWO code paths:
1. `proc_open([$bin, 'native', 'ir', '--stdin-json'], ...)` at line 19 (uses args array — safe from shell injection)
2. `exec(escapeshellcmd($bin) . ' native ir ' . escapeshellarg($path), ...)` at line 38 (uses shell string with `escapeshellarg` — mostly safe)
- BUT: `escapeshellarg()` wraps the argument in single quotes and escapes only single quotes
- On Windows, `escapeshellarg()` behavior differs and may not prevent injection
- Additionally, the `proc_open` code path at line 19 reads the file via `file_get_contents($path)` at line 13 — arbitrary file read still works regardless of shell injection safety

### H10: "V plugin os.quoted_path prevents injection" — Contradiction via Implementation Details

**Assumption**: V's `os.quoted_path()` properly quotes the path for shell execution
**Contradiction**: The V plugin at `crepuscularity.v:24` uses `os.execute()` with a shell command string:
```v
res := os.execute('${os.quoted_path(crepus_bin())} native ir ${os.quoted_path(path)} --ctx ${os.quoted_path(ctx_path)}')
```
- V's `os.quoted_path` wraps the string in double quotes and escapes `$`, `\`, `"`, and backticks
- If there are edge cases in the escaping (e.g., newlines in path), injection may still be possible
- Additionally, `os.execute()` may be affected by the user's shell (sh, bash, zsh) which handle certain characters differently

### H11: "Content script only processes our own markup" — Contradiction

**Assumption**: The content script only processes `<pre>` elements with `ai-anywhere` or ``` markers
**Contradiction**: At `content.js:162-168`:
```javascript
function hasAnywhereContent(node) {
    const text = normalizeWidgetText(node.textContent || "");
    const html = node.innerHTML || "";
    return (
        text.includes("```") ||
        text.includes("<ai-anywhere") ||
        html.includes("<ai-anywhere") ||
        html.includes("&lt;ai-anywhere")
    );
}
```
- ANY third-party page with a `<pre>` containing ``` or `<ai-anywhere` in its text will be processed
- The page operator can easily inject these markers into any `<pre>` element
- The WASM module's `extract_specs(normalized)` and `extract_widgets(normalized)` functions parse the pre content
- If the WASM module has parsing vulnerabilities, they can be exploited by crafting the pre content
- The WASM WASM module (compiled from Rust) operates on the same data that feeds the sanitizer

### H12: "Extension writes only to shadow DOM, safe from page access" — Contradiction via CSS Injection / Data Theft

**Assumption**: Content script DOM injection goes into shadow DOM, preventing page access
**Contradiction**: While shadow DOM provides style isolation, it doesn't prevent:
1. The INLINE_HOST_CSS at line 215-222 is injected via `style.textContent` — CSS injection could leak data via attribute selectors
2. The `attachFrameResize` at line 154-162 listens for `message` events from iframes — could be spoofed by the page using `window.postMessage`
3. The content script uses `browserApi` and `wasmModule` globals that might be clobbered by the page

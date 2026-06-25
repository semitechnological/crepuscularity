# Custom CodeQL Queries — crepuscularity Audit (Phase L3)

**Note:** CodeQL was not available on PATH during this analysis. These are query specifications intended for a future CodeQL-enabled pass.

## Query Specifications

### 1. Plugin Subprocess Path Injection (CRITICAL)
- **QL file:** `plugin-subprocess-path-injection.ql`
- **Source:** `RemoteFlowSource` or `LocalUserInput` (path argument to plugin APIs)
- **Sink:** `exec.Command(bin, args...)` / `subprocess.run([...])` / `Open3.capture3(...)` / `spawnSync(...)` / `proc_open([...])` where one argument is the unvalidated `path`
- **Sanitizer:** Any check for `".."`, absolute path, or canonicalization prefix
- **Extensions needed:** Model all plugin binding entry points as sources, model `exec.Command`/`subprocess.run`/etc. as sinks

### 2. RawHtml Without ammonia::clean (HIGH)
- **QL file:** `raw-html-without-sanitizer.ql`
- **Source:** `Node::RawHtml(expr)` evaluation
- **Sink:** `format!(...)` or `write!` to HTML output
- **Sanitizer:** `ammonia::clean()` applied to the output
- **Note:** Currently the code does apply ammonia::clean; this query would detect regressions

### 3. Path Read Without Canonicalization (HIGH)
- **QL file:** `file-read-without-canonicalization.ql`
- **Source:** Function parameter named `path` / user input
- **Sink:** `std::fs::read_to_string(path)` / `File::open(path)` / `File::read(path)` (externally in plugins)
- **Sanitizer:** `resolve_include_path()` / `resolve_under_sandbox()` / canonicalization + prefix check

### 4. V8 Bridge Invoke Without Capability Check (HIGH)
- **QL file:** `bridge-invoke-without-capability.ql`
- **Source:** `Crepus.invoke(plugin, method, payload)` call from JS
- **Sink:** `NativePlugin::invoke()` on a plugin that requires capability check
- **Sanitizer:** Capability check before invoke

### 5. HTML Injection in SSR head_extra (MEDIUM)
- **QL file:** `ssr-head-extra-injection.ql`
- **Source:** Template variable flowing to `SsrDocument.head_extra`
- **Sink:** `ammonia::clean()` output → `<head>` tag
- **Note:** This would detect dangerous tag/attribute allowlist configurations

### 6. InnerHTML Assignment Without Sanitization (HIGH)
- **QL file:** `innerhtml-without-sanitization.ql`
- **Source:** Any untrusted string value
- **Sink:** `.innerHTML =` / `.insertAdjacentHTML()` / `document.write()`
- **Sanitizer:** `sanitizeHTML()` function application

### 7. Missing escape_html on Attribute Values (HIGH)
- **QL file:** `attribute-without-escape.ql`
- **Source:** Template expression in element attribute binding
- **Sink:** `format!(r#" ... {} ... "#, value)` inside attribute quotes
- **Sanitizer:** `escape_html()` / `escape_html_attr()` applied to value

### 8. Content Script URL Scheme Bypass (MEDIUM)
- **QL file:** `content-script-url-bypass.ql`
- **Source:** Third-party page `<pre>` element content
- **Sink:** `sanitizeHTML()` URL validation using `startsWith()` pattern
- **Issue:** Protocol-relative URLs `//evil.com` bypass prefix check

## Data Extension Models Needed

1. **Plugin entry points:** `RenderIR(path, context)` in Python/PHP/Go/TS/Ruby — mark `path` parameter as `RemoteFlowSource`
2. **Subprocess sinks:** `exec.Command`, `subprocess.run`, `Open3.capture3`, `proc_open`, `spawnSync` — mark as command injection sinks
3. **File read sinks:** `os.ReadFile`, `File.read`, `Bun.file().text`, `file_get_contents`, `Path.read_text` — mark as file read sinks
4. **Sanitizers:** `escape_html`, `escape_html_attr`, `ammonia::clean`, `CGI.escapeHTML`, `html.EscapeString`, `html.escape()`, `htmlspecialchars()`, `sanitizeHTML()` — mark as HTML sanitizers
5. **Path validators:** `resolve_include_path`, `resolve_under_sandbox` — mark as path validation sanitizers

# Manual Attack Surface Inventory

Generated: 2026-06-25T05:30:00Z
Target: `piolium balanced probe` — Phase L4
Slices selected: Plugin Subprocess Command Injection (T-01) + Content Script DOM Injection (T-05)

---

## Slice 1: Plugin Subprocess Command Injection / Arbitrary File Read

### Entry Points

| Entry Point | File | Direction | Trust Boundary |
|---|---|---|---|
| `crepuscularity_plugin.py::render_ir(path, context)` | `plugins/python/crepuscularity_plugin.py:53` | Plugin caller → Plugin (Python) → CLI binary | IPC (stdin/stdout) |
| `CrepuscularityPlugin::renderIr(path, context)` | `plugins/php/CrepuscularityPlugin.php:10` | Plugin caller → Plugin (PHP) → CLI binary | IPC (stdin/stdout) |
| `RenderIR(path, context)` | `plugins/go/crepuscularity.go:43` | Plugin caller → Plugin (Go) → CLI binary | IPC (stdin/stdout) |
| `renderIr(path)` | `plugins/typescript-bun/crepuscularity.ts:10` | Plugin caller → Plugin (Bun) → CLI binary | IPC (stdin/stdout) |
| `CrepuscularityPlugin.render_ir(path, context)` | `plugins/ruby/crepuscularity_plugin.rb:37` | Plugin caller → Plugin (Ruby) → CLI binary | IPC (stdin/stdout) |
| `crepus_render_ir(path, buf, cap)` | `plugins/c/crepuscularity_plugin.c:5` | Plugin caller → Plugin (C) → CLI binary | IPC (stdin/stdout) |
| `render_ir(path)` | `plugins/cpp/crepuscularity_plugin.cpp:12` | Plugin caller → Plugin (C++) → CLI binary | IPC (stdin/stdout) |
| `Crepuscularity.RenderIrAsync(path)` | `plugins/csharp/Crepuscularity.cs:76` | Plugin caller → Plugin (C#) → CLI binary | IPC (stdin/stdout) |
| `CrepuscularityPlugin.renderIr(path)` | `plugins/java/CrepuscularityPlugin.java:80` | Plugin caller → Plugin (Java) → CLI binary | IPC (stdin/stdout) |
| `CrepuscularityPlugin.renderIr(path)` | `plugins/kotlin/CrepuscularityPlugin.kt:31` | Plugin caller → Plugin (Kotlin) → CLI binary | IPC (stdin/stdout) |
| `renderIr(path:)` | `plugins/swift/CrepuscularityPlugin.swift:25` | Plugin caller → Plugin (Swift) → CLI binary | IPC (stdin/stdout) |
| `render_ir(path, context)` | `plugins/v/crepuscularity.v:17` | Plugin caller → Plugin (V) → CLI binary | IPC (stdin/stdout) |
| `renderIr(allocator, path)` | `plugins/zig/crepuscularity.zig:10` | Plugin caller → Plugin (Zig) → CLI binary | IPC (stdin/stdout) |
| `render_ir(path)` | `plugins/rust/src/lib.rs:54` | Plugin caller → Plugin (Rust) → CLI binary | IPC (stdin/stdout) |

### Attacker Sources

- **`path` parameter**: String from plugin caller. No validation or sanitization in ANY plugin binding before:
  - File read (`Path(path).read_text()`, `std::fs::read_to_string`, `File.ReadAllText`, etc.)
  - Subprocess execution (`subprocess.run()`, `popen()`, `proc_open()`, `exec.Command()`, etc.)
  - Shell execution (`popen()` in C, C++; `/bin/sh -c` in Zig)

### Sinks

| Sink | File:Line | Vulnerable? | Impact |
|---|---|---|---|
| `subprocess.run([bin, "native", "ir", str(path)], ...)` | `python/crepuscularity_plugin.py:63` | No (args list) | Arbitrary file read via `crepus native ir` |
| `proc_open([bin, 'native', 'ir', '--stdin-json'], ...)` | `php/CrepuscularityPlugin.php:19` | No (args array) | Arbitrary file read |
| `exec(escapeshellcmd(bin) . ' native ir ' . escapeshellarg(path), ...)` | `php/CrepuscularityPlugin.php:38` | Partially (escapeshellarg used) | Limited |
| `exec.Command(crepusBin(), "native", "ir", "--stdin-json")` | `go/crepuscularity.go:46` | No (args list) | Arbitrary file read via `mustRead(path)` |
| `spawnSync(crepusBin(), ["native", "ir", "--stdin-json"], ...)` | `typescript-bun/crepuscularity.ts:14` | No (args list) | Arbitrary file read via `Bun.file(path).text()` |
| `Open3.capture3(bin, "native", "ir", path, ...)` | `ruby/crepuscularity_plugin.rb:50` | No (args list) | Arbitrary file read |
| **`popen("\"%s\" native ir \"%s\"", bin, path)`** | `c/crepuscularity_plugin.c:9` | **YES — Shell injection** | **Arbitrary command execution** |
| **`popen(cmd, "r")` with shell-injectable path** | `cpp/crepuscularity_plugin.cpp:17` | **YES — Shell injection** | **Arbitrary command execution** |
| `ProcessStartInfo(bin, "native ir --stdin-json")` | `csharp/Crepuscularity.cs:86` | No (process start info) | Arbitrary file read via `File.ReadAllTextAsync(path)` |
| `ProcessBuilder(List.of(bin, "native", "ir", "--stdin-json"))` | `java/CrepuscularityPlugin.java:83` | No (args list) | Arbitrary file read via `Files.readString(Path.of(path))` |
| `ProcessBuilder(bin, "native", "ir", "--stdin-json")` | `kotlin/CrepuscularityPlugin.kt:32` | No (args list) | Arbitrary file read via `Files.readString(Path.of(path))` |
| `Process(executableURL:, arguments:)` | `swift/CrepuscularityPlugin.swift:28` | No (Process API) | Arbitrary file read via `crepus native ir path` |
| `os.execute(...)` with `os.quoted_path(path)` | `v/crepuscularity.v:24` | **Partially (quoted_path)** | Depends on V compiler `quoted_path` implementation |
| **`/bin/sh -c` with shell-injectable path** | `zig/crepuscularity.zig:13` | **YES — Shell injection** | **Arbitrary command execution** |
| `Command::new(bin).args(["native", "ir", "--stdin-json"])` | `rust/src/lib.rs:64` | No (args list) | Arbitrary file read via `std::fs::read_to_string(path)` |

### Hidden Control Channels

| Channel | File:Line | Description |
|---|---|---|
| `CREPUS_BIN` env var | All plugins | Controls which binary is invoked; if attacker controls environment, they can redirect to arbitrary executable |
| `baseDir` in stdin JSON | Python:66, Ruby:42, CLI: `native.rs:143` | Controls include resolution base; unvalidated in stdin-JSON envelope |
| `--ctx` flag | CLI `native.rs:115` | Loads arbitrary JSON/TOML files from caller-controlled path |

### Key Files

| File | Lines | Role |
|---|---|---|
| `plugins/c/crepuscularity_plugin.c` | 42 | **Command injection via popen** |
| `plugins/cpp/crepuscularity_plugin.cpp` | 51 | **Command injection via popen** |
| `plugins/zig/crepuscularity.zig` | 40 | **Command injection via /bin/sh -c** |
| `plugins/v/crepuscularity.v` | 30 | Possible injection via os.execute |
| `plugins/python/crepuscularity_plugin.py` | 112 | Arbitrary file read (no injection) |
| `plugins/php/CrepuscularityPlugin.php` | 86 | Arbitrary file read |
| `plugins/go/crepuscularity.go` | 88 | Arbitrary file read |
| `plugins/typescript-bun/crepuscularity.ts` | 80 | Arbitrary file read |
| `plugins/ruby/crepuscularity_plugin.rb` | 86 | Arbitrary file read |
| `crates/crepuscularity-cli/src/native.rs` | 1848 | CLI argument parsing — `path` unvalidated before `fs::read_to_string` at line ~133 |

---

## Slice 2: Browser Extension Content Script DOM Injection

### Entry Points

| Entry Point | File | Direction | Trust Boundary |
|---|---|---|---|
| `content.js` IIFE (line 1-359) | `crates/crepuscularity-webext/assets/content.js` | Third-party web page → Extension content script | Browser → Extension |

### Attacker Sources

- **`<pre>` element text content** on any third-party web page
- **HTML content** with `ai-anywhere` markers
- **DOM mutations** observed by MutationObserver

### Sinks

| Sink | File:Line | Vulnerable? |
|---|---|---|
| `root.innerHTML = sanitizeHTML(html)` | `content.js:355` | **Partially** — sanitizeHTML is restrictive but mutation XSS or DOMParser quirks could bypass |
| `frame.src = url` (Blob URL) | `content.js:173` | No — sandboxed iframe |
| `wrapper.replaceWith(wrapper)` (DOM insertion) | `content.js:128` | No — DOM API, not innerHTML |
| `el.innerHTML = ...` at `app.js:120` | `crates/crepuscularity-cli/assets/web/app.js:120` | Yes — but this is the dev server, not extension |

### Sanitizer Analysis

`sanitizeHTML()` at `content.js:296-324`:
- Uses DOMParser to parse HTML into a document
- Iterates all elements and removes non-ALLOWED_TAGS
- Strips all `on*` event handler attributes
- Restricts `href`/`src` URL schemes to `http://`, `https://`, `mailto:`, `#`, `/`
- Returns `doc.body.innerHTML`

**Limitations:**
1. DOMParser does not execute scripts or load resources — safe for parsing
2. But DOMParser's behavior differs from browser rendering (mXSS vectors exist for some HTML constructs)
3. The sanitized HTML is inserted via `innerHTML`, which triggers a second parse by the browser — this second parse could interpret the sanitized HTML differently than DOMParser did (mutation XSS)
4. Tag allowlist missing `<FORM>`, `<OBJECT>`, `<EMBED>`, `<IFRAME>`, `<SCRIPT>`, `<STYLE>` — but `<TABLE>`, `<SVG>`, `<MATH>` are also absent

### Key Files

| File | Lines | Role |
|---|---|---|
| `crates/crepuscularity-webext/assets/content.js` | 359 | Main content script — sanitization + DOM injection |
| `crates/crepuscularity-webext/assets/browser-shim.js` | ~50 | Browser API shim |

---

## Trust Boundary Crossings

### TB-3 (Plugin → CLI Binary)

**Attacker-controlled data**: `path` argument (String) — **unvalidated**
**Crossing type**: IPC subprocess
**Security decision**: None — no validation before `File.read(path)` or `subprocess.run(path)`

### TB-5 (Content Script → Web Page)

**Attacker-controlled data**: `<pre>` element content from third-party pages
**Crossing type**: DOM API
**Security decision**: `sanitizeHTML()` applied before `innerHTML` assignment; iframe `sandbox="allow-scripts"`

---

## Attack Scenarios

### Scenario A1: Arbitrary File Read via Plugin Path (all bindings)

1. Attacker controls the `path` parameter passed to any plugin's `render_ir()` or `render_html()` function
2. Plugin reads the file at `path` (e.g., `../../etc/passwd`) and passes its content to `crepus` CLI
3. CLI renders the template and returns IR JSON
4. Content of the file is leaked in the IR output or error message

### Scenario A2: Command Injection via C/C++/Zig Plugin (shell-injection-prone bindings)

1. Attacker controls `path` parameter passed to C/C++/Zig plugin
2. Plugin embeds `path` directly into shell command string via `popen()` or `/bin/sh -c`
3. Shell executes the injected command before or after the `crepus` binary invocation
4. Attacker gains arbitrary command execution on the system running the plugin

### Scenario B1: Content Script Mutation XSS

1. Attacker crafts a third-party page with a `<pre>` element containing malicious HTML
2. Content script extracts the text, detects widget markers, and processes via WASM
3. Rendered HTML passes through `sanitizeHTML()` which may have parsing quirks
4. Sanitized HTML is inserted into shadow DOM via `innerHTML` — browser re-parses
5. If DOMParser and browser differ in HTML parsing, attacker achieves XSS in extension context

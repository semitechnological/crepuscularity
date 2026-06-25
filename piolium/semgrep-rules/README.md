# Custom Semgrep Rules — crepuscularity Audit (Phase L3)

**Note:** Semgrep was not available on PATH during this analysis. These are rule specifications for a future Semgrep-enabled pass. Once Semgrep is available, apply with `--pro` for cross-file taint tracking.

## Rule Specifications

### 1. Plugin Subprocess Path Injection (CRITICAL)
- **Rule file:** `plugin-subprocess-path-injection.yaml`
- **Languages:** Python, PHP, Go, TypeScript, Ruby
- **Pattern:** Detect `path` variable passed to subprocess execution functions where `path` is unvalidated
- **Python rule:**
```yaml
rules:
  - id: python-plugin-path-injection
    patterns:
      - pattern: subprocess.run([..., "native", "ir", $PATH], ...)
      - metavariable-pattern:
          metavariable: $PATH
          pattern: $PATH
      - pattern-not: subprocess.run([..., "native", "ir", "--stdin-json"], ...)
    message: "Unvalidated path '$PATH' passed to subprocess in Python plugin"
    languages: [python]
    severity: ERROR
```

### 2. Go Plugin `mustRead` Pattern (HIGH)
- **Rule file:** `go-mustread-panic.yaml`
- **Language:** Go
```yaml
rules:
  - id: go-mustread-panic
    patterns:
      - pattern: |
          func mustRead(...) string {
            ...
            panic(err)
          }
    message: "mustRead() panics on read error instead of returning error; should add path validation"
    languages: [go]
    severity: ERROR
```

### 3. PHP `exec()` Shell Command Construction (HIGH)
- **Rule file:** `php-exec-shell-command.yaml`
- **Language:** PHP
```yaml
rules:
  - id: php-exec-shell-command
    patterns:
      - pattern: exec($CMD, ...)
      - metavariable-pattern:
          metavariable: $CMD
          pattern: |
            escapeshellcmd(...) . "..." . escapeshellarg(...)
    message: "PHP plugin uses exec() with shell command string; prefer array-form proc_open()"
    languages: [php]
    severity: ERROR
```

### 4. Ruby `File.read` Without Path Validation (HIGH)
- **Rule file:** `ruby-file-read-unvalidated.yaml`
- **Language:** Ruby
```yaml
rules:
  - id: ruby-file-read-unvalidated
    patterns:
      - pattern: File.read($PATH)
      - metavariable-pattern:
          metavariable: $PATH
          pattern: $PATH
      - pattern-not: File.read("...")
    message: "File.read() called with variable path; add canonicalization + prefix check"
    languages: [ruby]
    severity: WARNING
```

### 5. Content Script `innerHTML` Without `sanitizeHTML` (HIGH)
- **Rule file:** `innerhtml-without-sanitize.yaml`
- **Language:** JavaScript/TypeScript
```yaml
rules:
  - id: innerhtml-without-sanitize
    patterns:
      - pattern: $TARGET.innerHTML = $VALUE
      - metavariable-pattern:
          metavariable: $VALUE
          pattern: sanitizeHTML(...)
      - pattern-not: $TARGET.innerHTML = ""
    fix: $TARGET.innerHTML = sanitizeHTML($VALUE)
    message: "innerHTML assignment without sanitizeHTML() call"
    languages: [javascript, typescript]
    severity: WARNING
```

### 6. Weak Random `Math.random()` for Security Contexts (MEDIUM)
- **Rule file:** `weak-random-security.yaml`
- **Language:** JavaScript/TypeScript
```yaml
rules:
  - id: weak-random-security
    patterns:
      - pattern-either:
          - pattern: Math.random().toString(36).slice(...)
          - pattern: Math.random().toString(36).substring(...)
    message: "Math.random() is not cryptographically secure; use crypto.getRandomValues() instead"
    languages: [javascript, typescript]
    severity: WARNING
```

### 7. Content Script URL Scheme Prefix Check (MEDIUM)
- **Rule file:** `content-script-url-prefix-check.yaml`
- **Language:** JavaScript/TypeScript
```yaml
rules:
  - id: content-script-url-prefix-bypass
    patterns:
      - pattern: |
          if ($VALUE.startsWith("http://") || ...) { ... }
      - metavariable-pattern:
          metavariable: $VALUE
          pattern: attr.value.trim().toLowerCase()
    message: "URL scheme validation using startsWith() can be bypassed with protocol-relative URLs (//)"
    languages: [javascript, typescript]
    severity: WARNING
```

### 8. Plugin `CREPUS_BIN` Env Var Usage (MEDIUM)
- **Rule file:** `plugin-crepus-bin-env.yaml`
- **Languages:** Python, PHP, Go, TypeScript, Ruby
```yaml
rules:
  - id: plugin-crepus-bin-env
    patterns:
      - pattern-either:
          - pattern: os.environ.get("CREPUS_BIN", ...)
          - pattern: os.Getenv("CREPUS_BIN")
          - pattern: getenv('CREPUS_BIN')
          - pattern: process.env.CREPUS_BIN ?? ...
          - pattern: ENV.fetch("CREPUS_BIN", ...)
    message: "CREPUS_BIN environment variable controls binary path; validate or remove"
    languages: [python, go, php, typescript, ruby]
    severity: WARNING
```

### 9. Plugin `bind:` Handler Context Overwrite (MEDIUM)
- **Rule file:** `plugin-bind-handler-overwrite.yaml`
- **Languages:** Python, PHP, Go, TypeScript, Ruby
- **Pattern:** Look for string handler starting with "bind:" that splits on ":" and writes to context without allowlist

### 10. SSR Error Message in HTML Response (LOW)
- **Rule file:** `ssr-error-disclosure.yaml`
- **Language:** Rust
```yaml
rules:
  - id: ssr-error-disclosure
    patterns:
      - pattern: |
          Html(format!("<pre style='color:red'>{}</pre>", escape_html_error(...)))
    message: "SSR error messages disclosed in HTML response; use generic error in production"
    languages: [rust]
    severity: WARNING
```

## Batch Execution Plan

For a future Semgrep Pro pass, batch by language:

| Batch | Languages | Rules | Priority |
|-------|-----------|-------|----------|
| 1 (Pro-heavy) | Python, Go, Ruby, TS | cross-file taint (path injection) | HIGH |
| 2 (structural) | JavaScript/TypeScript | innerHTML, weak random, URL patterns | MEDIUM |
| 3 (structural) | PHP, Go | exec/panic patterns | HIGH |
| 4 (structural) | Rust | error disclosure, attribute escape | LOW |

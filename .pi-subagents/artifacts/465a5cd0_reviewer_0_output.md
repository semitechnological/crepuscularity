All diffs collected, files inspected, tests verified. Here's the full review.

---

## Review: 7 Security PRs — crepuscularity plugins

### PR #177 — Fix arbitrary file read via path traversal in Go plugin

**Verdict: ⚠️** (minor residual risk)

**What it does:** Adds `filepath.Rel` / `filepath.Abs` boundary check in `RenderIR` before `os.ReadFile`. Allows dir override via `CREPUS_ALLOWED_DIR` env var. Updates existing tests with `CREPUS_ALLOWED_DIR=..` to keep existing relative-path tests passing. Adds `TestRenderIRTraversalDenied`.

**Correct:** Path containment via `filepath.Rel` check is idiomatic Go. Defaults to cwd. Env override for testing is clean.

**Edge cases:**
- Symlink bypass: `filepath.Abs` resolves `..` and `.` but NOT symlinks. If attacker drops a symlink `allowed_dir/evil -> /etc/passwd`, `filepath.Abs` preserves it. Should use `filepath.EvalSymlinks` before the check. **Residual risk, not blocker for initial merge.**
- Existing tests pass because `CREPUS_ALLOWED_DIR=..` makes `../fixtures/hello.crepus` resolve within `../..` = project root. OK.

**Tests:** One denial test. Missing: symlink-bypass test, absolute-path-traversal test (`/etc/passwd`), Windows paths. Test coverage adequate for initial fix.

**Over-engineering:** No. `filepath.Rel` is standard Go idiom.

---

### PR #176 — Fix command execution via unvalidated CREPUS_BIN in Python

**Verdict: ✅**

**What it does:** Validates `CREPUS_BIN` env var in `_crepus_bin()`: binary name must be `crepus` or `crepus.exe`; if a path is given, must be absolute.

**Correct:**
- `os.path.basename()` extracts filename, checked against allowlist.
- Relative paths with `/` (e.g., `../bin/sh`) are rejected by the `not os.path.isabs(bin_path)` check when path != name.
- Valid examples: `crepus`, `crepus.exe`, `/usr/bin/crepus`, `C:\Program Files\crepus.exe`.

**Edge cases:**
- Empty `CREPUS_BIN` → `os.path.basename("")` = `""`, fails name check. OK (falls through to error).
- Symlink `/tmp/crepus -> /bin/sh` passes name check. Not fixable without breaking legitimate use. Acceptable tradeoff.
- Absolute path with trailing whitespace: `"/usr/bin/crepus "` → `basename` = `"crepus "` → fails name check. Strict but safe.

**Tests:** Good coverage of valid/invalid values via `unittest.mock.patch`. Missing: empty-string case. Acceptable.

**Over-engineering:** No. Clean 5-line validation.

---

### PR #175 — Fix Arbitrary File Read in PHP Plugin

**Verdict: ⚠️** (missing tests, root-dir edge case)

**What it does:** Adds `realpath()`-based path containment check in `renderIr()`. Allows optional `$allowedDirectory` override.

**Correct:**
- `realpath()` resolves symlinks unlike Go's `filepath.Abs`. Better.
- `str_starts_with($realPath, $realAllowed . DIRECTORY_SEPARATOR)` is the standard PHP containment check.

**Bug: root directory edge case.** If `$realAllowed` resolves to `/`, then the check becomes `str_starts_with($realPath, "//")` which is always false. Any file under `/` is denied. Only triggers if `getcwd()` returns `/` (incredibly rare) or someone sets `$allowedDirectory` to `/`. Low impact but incorrect.

**Tests:** None. No PHP test file exists in repo. The `session_smoke.php` is a smoke test, not a unit test.

**Also changes** `BIND_BLOCKLIST` from `private const` to `public const` — needed for `CrepusViewSession::dispatch()` to access it. This is a pre-existing issue, not directly related to the security fix.

**Over-engineering:** No. The `realpath` approach is the right one in PHP.

---

### PR #174 — Fix Command Execution via Unvalidated CREPUS_BIN (Ruby)

**Verdict: ✅**

**What it does:** Rejects `CREPUS_BIN` value containing `/` or `\` in `render_ir()`.

**Correct:** Simplest possible approach — only bare binary names allowed, no paths.

**Tradeoff:** Users cannot set `CREPUS_BIN` to an absolute path. Strictest of all plugin fixes. Acceptable for security-first posture; user can put binary on PATH.

**Edge cases:** None. `%r{[/\\]}` covers both Unix and Windows separators. Works with `crepus.exe`.

**Tests:** None in the diff. No change to test infrastructure. Ruby has no test file (only `session_smoke.rb`).

**Over-engineering:** No. One-line regex. Good.

---

### PR #173 — Fix Command Execution via Unvalidated CREPUS_BIN (PHP)

**Verdict: ❌** (over-engineered, questionable logic, no tests)

**What it does:** Adds multi-regex validation in `crepusBin()`: blacklist control chars, allow simple names or absolute paths, with a questionable third path allowing spaces minus ` -` sequences.

**Over-engineering:**
1. Character blacklist `[\x00-\x1F\x7F"\'<>|&;$]` — overboard. The same threat is addressed by simpler means in other plugins.
2. Three separate regex checks (`$isBinaryName`, `$isAbsolutePath`, `$isAbsolutePathWithSpacesValid`) for what should be a simple `basename`-style check.
3. `$isAbsolutePathWithSpacesValid` has questionable logic: rejects paths with ` -` (space-hyphen) to prevent flag injection, but this is an incomplete defense against argument injection. If the attacker controls the full env var, they can set it to `/usr/bin/crepus --help` which doesn't have ` -`. The check doesn't actually prevent the threat it claims to address.

**Bug:** The three conditions are mutually non-exclusive; a path could match both `$isAbsolutePath` AND `$isAbsolutePathWithSpacesValid` and pass either way. The logic works but is unnecessarily complex.

**Also changes** `BIND_BLOCKLIST` from `private const` to `public const` — same change as PR #175. These PRs conflict on this line.

**Tests:** None.

**Over-engineering:** Yes. A simple `basename` or separator check would suffice.

---

### PR #172 — Fix Path Traversal in Python Plugin

**Verdict: ✅**

**What it does:** Adds `allowed_dir` parameter threaded through `ViewSession` → `render_ir` → `render_html`. Uses `Path.resolve()` + `Path.is_relative_to()` for containment check.

**Correct:**
- `Path.resolve()` resolves symlinks, unlike Go's `filepath.Abs`.
- `is_relative_to()` is Python 3.9+'s canonical containment check.
- `allowed_dir` defaults to `None` → `Path.cwd().resolve()` for backward compat.
- Parameter threading is complete (unlike PHP which doesn't thread `$allowedDirectory` through `ViewSession`).

**Edge cases:**
- Root directory: `is_relative_to("/")` always true, correct.
- Symlinks: resolved by `Path.resolve()`, correct.
- Backward compatibility: existing callers without `allowed_dir` work unchanged (verify by current tests passing).

**Tests:** Updates existing tests to pass `allowed_dir`. Adds `test_path_traversal_validation` testing both absolute and relative traversal. Good.

**Over-engineering:** No.

**Conflict note:** Modifies the same files as PR #176 (`crepuscularity_plugin.py`, `test_crepuscularity_plugin.py`). The changes touch different functions and don't overlap. Should merge cleanly.

---

### PR #162 — Secure Ruby plugin path parameter

**Verdict: ✅**

**What it does:** Adds `File.realpath()`-based containment check in `render_ir()`. Ensures resolved path starts with `base_dir + File::SEPARATOR` or equals `base_dir` exactly.

**Correct:**
- `File.realpath()` resolves symlinks.
- Check handles both subdirectory files and directory-as-file (edge).
- Always uses `Dir.pwd` as boundary (no configurable override — intentional simplification).

**Tests:** Excellent. Three denial tests (arbitrary file path, traversal sequence, symlink escape) + one valid path test. The symlink-bypass test creates a real symlink in the fixtures dir — practical validation.

**Edge cases:**
- Root directory: same pathological case as PHP. `Dir.pwd` being `/` makes `base_dir + File::SEPARATOR` = `//`. Negligible in practice.
- Test creates temp symlink in `plugins/ruby/` — needs write permission there. Acceptable.

**Also adds** `plugins/ruby/fixtures/interactive.crepus` — test fixture. Reuses the same fixture from `plugins/fixtures/`. Could have used the shared one but local is fine.

**Over-engineering:** No.

**Conflict note:** Modifies same file as PR #174. They touch different parts of `render_ir` — both merge cleanly.

---

### Summary table

| PR# | Title | Verdict | Key Findings |
|-----|-------|---------|-------------|
| #177 | Fix arbitrary file read / path traversal (Go) | ⚠️ | Missing symlink bypass protection (`filepath.EvalSymlinks`). Tests cover basic traversal only. |
| #176 | Fix command execution via unvalidated CREPUS_BIN (Python) | ✅ | Correct, clean, good tests. |
| #175 | Fix arbitrary file read (PHP) | ⚠️ | Root-directory edge case bug. No tests at all. |
| #174 | Fix command execution via unvalidated CREPUS_BIN (Ruby) | ✅ | Simplest possible fix, correct. No tests. |
| #173 | Fix command execution via unvalidated CREPUS_BIN (PHP) | ❌ | Over-engineered regex logic. Questionable ` -` check. No tests. Conflicts with #175 on `BIND_BLOCKLIST` visibility. |
| #172 | Fix path traversal (Python) | ✅ | Clean, complete parameter threading, good tests. |
| #162 | Secure Ruby plugin path parameter | ✅ | Best test coverage of all path fixes. No configurable override (intentional). |

**Cross-cutting issues:**
- **Go (#177) and PHP (#175)** both lack symlink resolution before path check. Python (#172) and Ruby (#162) handle this correctly.
- **PHP has no test infrastructure at all.** Both PHP PRs (#173, #175) add no tests.
- **Ruby** PRs #162 and #174 modify the same file compatibly. Merge order: #162 then #174 (path check before bin check follows logical order).
- **Python** PRs #172 and #176 modify the same file compatibly. Merge order: either works.
- **PHP** PRs #173 and #175 both change `BIND_BLOCKLIST` from `private` to `public` — merge conflict on that line. Pick one.
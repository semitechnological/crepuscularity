# PR fleet review — crepuscularity (2026-07-07)

**Scope:** 67 open PRs (#185–#244), mostly Jules automation on `main`.  
**Method:** Theme clustering, diff vs current `main`, CI spot-check, security alignment with existing plugin policy.

## Executive summary

| Verdict | Count (themes) | Action |
|--------|----------------|--------|
| **Merge candidate** | 2–3 | Cherry-pick or merge one fix per area after CI green |
| **Close as duplicate** | ~45+ | Same diff/title family; no incremental value |
| **Do not merge** | ~15+ | Regresses `CREPUS_BIN` absolute paths; wrong CREPUS_BIN tests; noisy refactors/tests failing CI |

**Fleet hygiene:** Treat this as a **duplicate storm**, not 67 independent features. Close all but **one winner per theme** (or land fixes on `main` directly).

---

## Themes

### 1. PHP path traversal / TOCTOU (~12 PRs: #233–#235, #237–#240, #231, #234, …)

**`main` today:** `realpath()` + prefix check; stdin path still used `file_get_contents($path)` and CLI used raw `$path`.

**Best diff:** **#242** — uses `$realPath` for reads, CLI arg, `baseDir`; allows exact `$realPath === $realAllowed`; regex tweak for backslash in CREPUS_BIN check.

- **Recommend:** **Approve #242** if `build` / plugin smoke pass after rebase. **Close** #233–#241, #234, etc. as duplicates of #242.
- **Note:** Comment-only noise in #242 is fine; no security regression spotted.

### 2. Go symlink / path traversal (~8 PRs: #205, #208, #214, #222, #241, …)

**`main` today:** `filepath.Abs` + `Rel` — **does not** resolve symlinks (known gap).

**#241:** Adds `EvalSymlinks` on allowed dir + path, prefix check on cleaned paths, `mustRead(absPath)`.

- **Recommend:** **Approve #241** conceptually; **re-run** Go tests on Windows (CI failed on #241). Add/adjust symlink test if missing.
- **Close** other Go symlink PRs as duplicates.

**Risk:** `EvalSymlinks` failure falls back to non-resolved path — acceptable ponytail fallback; document if kept.

### 3. Python `CREPUS_BIN` (~10 PRs: #187, #198, #210, #218, #223, #226, #230, #244, …)

**`main` policy:** Basename must be `crepus` / `crepus.exe`; **absolute paths allowed** (see `plugins/README.md`, tests).

**#244 / #230 / #198 / …:** Reject absolute paths — **breaks documented dev workflow** (`export CREPUS_BIN="$PWD/target/debug/crepus"`).

- **Recommend:** **Do not merge** any PR that removes absolute `CREPUS_BIN`. Close entire cluster.
- If hardening desired on `main`: keep current validation; optionally reject relative paths only (already done).

### 4. Ruby / Go / PHP / TS `CREPUS_BIN` variants

- **#217 (Ruby):** Moves toward Python/Go-style abs OR bare name — **closer to `main` intent** than #244, but **main Ruby is stricter** (name-only). **Needs product decision** before merge; README implies abs paths for Rust/debug binary — Ruby name-only may be intentional.
- **#188 (typescript-bun):** Review separately; CI failing; align with TS plugin tests on `main`.
- **#224 (PHP CREPUS_BIN):** Duplicate of PHP path theme; `main` PHP already blocks paths in `crepusBin()`.

### 5. Python arbitrary file read (~6 PRs)

**`main`:** `Path.resolve()` + `is_relative_to` when `context is not None`; no context path uses raw `str(path)` in argv — separate concern.

Duplicate PRs mostly re-touch same file. **Close duplicates**; if gap remains (no-context CLI path), **one** targeted PR on `main`, not 6.

### 6. Code health / refactor (#196, #197, #204, #216, #220, #221, #229, #239, …)

- **#239:** Extract `run_dev` / `run_benchmark`, `init_tracing()` — reasonable; **build job failed** on PR; fix CI before merge.
- **#197 `build_site_wasm` breakdown:** Large surface; CI failures (Plugin Smoke, Security Audit) — **hold** until green.
- **#216 native gradle extract:** CI red — **hold**.

**Recommend:** Merge refactors **one at a time** after full green CI; close duplicate/no-op Jules titles (#229).

### 7. Perf (#219, #232)

- **#232 `index_node`:** Reuses single `path` buffer — looks **correct**; verify embedded tests + Security Audit (failed on PR — investigate flake vs real).
- **#219:** Loop offsets — review only if benchmark proves win.

### 8. Testing improvements (~25 PRs)

Many add narrow error-path or `ElementRef::text` tests (#195, #202, #212, #213, #225, …).

- **#225:** Trivial extra `ElementRef::text` cases — **low risk** if CI green; **duplicate** of #213.
- **Mass close:** Keep **one** test PR per area after rebasing on `main`, or fold tests into feature PRs.

---

## CI snapshot (spot-check)

| PR | Notable CI |
|----|------------|
| #244 | `build` fail |
| #242 | `build` fail (others pending) |
| #241 | Windows test fail |
| #239 | Tests pass, `build` fail |
| #232, #225, #216, #197, #188 | Security Audit / Plugin Smoke / Windows fails common |

**Implication:** Jules branches may be behind `main` or repo `build` job is brittle — **rebase + single merge** beats merging 67.

---

## Recommended merge order (if you want minimal churn)

1. **#242** (PHP canonical path) — security, small diff  
2. **#241** (Go symlinks) — after Windows test fix  
3. **#232** (embedded path indexing perf) — if audit failure explained  
4. **#239** (CLI dispatch extract) — after `build` fixed  

Everything else in the security/test duplicate bands: **close with comment** pointing to winner or `main` already fixed.

---

## Security regressions to block (any PR)

- Removing **absolute** `CREPUS_BIN` where README/plugins rely on it  
- Allowing **non-`crepus`** binary names without explicit product approval (Go tests allow `mycrepus` on `main` — separate issue)  
- Using **raw user path** after `realpath` validation (PHP #242 fixes this)

---

## Confidence

**Medium-high** on theme clustering (diffs sampled). **Low** on per-PR CI root cause without log dive. Full per-PR Phase 1–6 differential-review not run for all 67 — would be redundant for duplicate clusters.
Now reviewing all 5 PRs with full context.

## PR Review

### PR #149 — Optimize manifest CSS vector allocation

| | |
|---|---|
| **File** | `crates/crepuscularity-webext/src/manifest.rs` |
| **Diff** | +3/-2 = +1 net line |
| **Change** | `let mut css = cs.css.clone()` → `Vec::with_capacity(cs.css.len() + opts.content_css.len())` + `extend` |

**Analysis:** Original `cs.css.clone()` allocates exact capacity. Then loop pushes `opts.content_css` items one-by-one, potentially reallocating. New code pre-allocates enough for both, avoiding reallocation during push loop. Trivially correct — `with_capacity` is an upper bound, over-allocation is fine.

**Verdict:** ✅ Real micro-optimization. Minimal diff, no downsides.

---

### PR #151 — Avoid Vec cloning in tree indexing

| | |
|---|---|
| **File** | `crates/crepuscularity-embedded/src/document.rs` |
| **Real diff** | ~+6/-6 lines |
| **Total diff size** | 333 lines (includes committed `.orig` backup file) |
| **Change** | `index_node(..., path: Vec<usize>)` → `index_node(..., path: &mut Vec<usize>)`, push/pop instead of clone per recursion |

**Analysis:** Old code cloned the path Vec per recursive call: `let mut p = path.clone(); p.push(i);`. New code mutates in place: `path.push(i); ...; path.pop();`. Saves O(depth) Vec clone per internal node. For a large tree (e.g., 4-ary depth 5 = 1365 nodes), old code clones path ~1360+ times (all non-root nodes). New code clones zero times for recursion (only `path.clone()` for `map.insert()` which was already there).

Trivially correct: push before call, pop after — path restored for siblings.

**Accidental `.orig`:** PR diff includes `document.rs.orig` as a new file — an editor backup committed by accident. This should be removed and `.orig` added to `.gitignore`.

**Verdict:** ✅ Real optimization, trivially correct. ⚠️ `.orig` file committed by accident — needs cleanup.

---

### PR #159 — Optimize subscriber removal by using HashSet

| | |
|---|---|
| **Files** | `runtime.rs`, `memo.rs`, `signal.rs` |
| **Net** | +14/-16 = −2 lines (smaller after change!) |
| **Change** | `subscribers: Vec<NodeId>` → `subscribers: HashSet<NodeId>` |
| | `subs.retain(...)` → `subs.remove(&id)` |
| | `if !subs.contains(x) { subs.push(x) }` → `subs.insert(x)` |
| | `s.to_vec()` → `s.iter().copied().collect::<Vec<_>>()` |

**Analysis:**
- **Remove:** O(n) `retain` → O(1) `remove`. Correct.
- **Insert:** O(n) contains+push → O(1) `insert` (automatic dedup). Correct.
- **Clone:** Vec clone → HashSet iteration + Vec collect. Iteration is O(n) in both cases; HashSet iterator has overhead of walking buckets but this is a one-time cost per `mark_subscribers_dirty` call.

**Trade-offs:**
- HashSet has higher memory overhead per entry (load factor ~0.7, hash bytes)
- HashSet iteration is non-deterministic — fine for reactive graph (order doesn't affect correctness)
- For signals with 1–3 subscribers (common case), Vec linear scan is faster than HashSet hash+lookup for contains/insert. The 1,000,000× benchmark number cited in PR body ("49ms → 49ns") is for 1000 subscribers — an extreme case, not representative.
- PR body acknowledges 700ns → 814ns regression on single-signal bench (≈16% slowdown for 1-subscriber case)

**Verdict:** ✅ Real optimization for large subscriber counts. Acceptable trade-off (16% regression on tiny case vs 1,000,000× gain on extreme case). Net code reduction.

---

### PR #161 — perf optimization for subscriber removal

| | |
|---|---|
| **File** | `runtime.rs` |
| **Net** | +35/-6 = +29 lines |
| **Change** | Replaces O(N) full-graph scan in `remove_node` with O(E) targeted walks over the node's sources/subscribers. Also removes `id` from subscriber's **sources** lists (previously missing). |

**Analysis:**
Current `remove_node`:
1. Calls `clear_observer_sources(id)` — removes `id` from its sources' subscribers ✓
2. Scans ALL remaining nodes: `for node in nodes.values_mut() { subs.retain(|&x| x != id) }` — O(N)
3. **Missing:** never cleans `id` from dependent nodes' `sources` lists — stale references left in graph

PR #161:
1. Collects `old_sources` and `old_subscribers` before removal
2. Removes `id` from `old_sources`' subscribers ← same as `clear_observer_sources`
3. Removes `id` from `old_subscribers`' sources ← **bug fix: was missing!**
4. Removes node from map ← same as before, without the O(N) scan

The scan is now O(deg) instead of O(total_nodes). For a graph where `remove_node` is called on a node with few connections but many unrelated nodes, this is a large win. PR body reports ~4.14ms → ~274µs for a graph of 1000 unused nodes (= ~93% reduction).

**Overlap with PR #159:** Both touch same area. PR 159 changes Vec→HashSet. PR 161 as written uses `subs.retain(|&x| x != id)` — this would need to be `subs.remove(&id)` if PR 159's HashSet change is already applied. The two PRs should be rebased/merged so PR 161 uses `remove` on the new HashSet.

**Verdict:** ✅ Real optimization AND bug fix. ⚠️ Conflicts partially with PR 159 — use `remove` not `retain` if HashSet is applied.

---

### PR #163 — Refactor run_all_suites

| | |
|---|---|
| **File** | `bench_plugin.rs` |
| **Real diff** | +74/-82 = −8 net lines |
| **Total diff** | +463/-83 (includes committed `.orig`) |
| **Change** | Closure-based callback pattern → `RunnerState` struct with methods. Removes generic `F: FnMut(...)` from 6 suite function signatures. |

**Analysis:**
Before: Each suite function has `F: FnMut(&mut Vec<Value>, &str, &str, &str, u64, f64, u64)>` — the same 7-parameter generic repeated 6 times. The `run_all_suites` function has a closure `add_suite` that's passed through to each suite.

After: `RunnerState` struct bundles `suites`, `total_work_units`, `total_ms_raw`, `acc` fields. Suite functions take `&mut RunnerState`. The `add_suite` is a method on `RunnerState`.

- Eliminates 6× generic function signatures (repetition reduction)
- Net code reduction: −8 lines
- No runtime behavior change — pure refactor
- `state.acc` is accessed directly instead of going through the closure capture
- Slightly more idiomatic Rust (struct with methods > closure-tuple soup)

**Performance:** Zero perf impact — this is organization only.

**Accidental `.orig`:** PR diff includes `bench_plugin.rs.orig` — same issue as PR 151. Editor backup committed by accident.

**Verdict:** ⚠️ Clean refactor, reduces repetition. Not a perf PR despite title emoji. `.orig` file committed by accident.

---

### Final Review Table

| PR# | Title | Verdict | Findings | Net lines |
|-----|-------|---------|----------|-----------|
| #149 | Optimize manifest CSS vector allocation | ✅ | Real micro-opt. Pre-allocates Vec capacity. Correct. | +1 |
| #151 | Avoid Vec cloning in tree indexing | ✅ | Real alloc reduction. `&mut Vec` push/pop instead of clone per recursion. **⚠️ `.orig` file committed.** | ~+0 (6 changed) |
| #159 | Optimize subscriber removal by using HashSet | ✅ | Real opt for many-subscriber case. Vec→HashSet. 16% regression on 1-subscriber bench (acceptable). Net code reduction. | −2 |
| #161 | perf optimization for subscriber removal | ✅ | Replaces O(N) scan with O(E) targeted removal. **Also fixes bug** — missing source-list cleanup. **⚠️ Conflicts with #159** — uses `retain` instead of `remove`. | +29 |
| #163 | Refactor run_all_suites | ⚠️ | Clean refactor, less redundant generics. **Not perf** — organization only. **⚠️ `.orig` file committed.** | −8 (real) |

### Key issues across PRs:

1. **PR #159 vs #161 overlap:** Both modify subscriber removal in `runtime.rs`. If applied together, PR 161's `subs.retain(|&x| x != id)` (line 178, 186) should use `subs.remove(&id)` since PR 159 makes subscribers a HashSet. **Recommendation:** Merge into one PR or rebase #161 on #159 and fix the retain→remove calls.

2. **`.orig` files committed:** PR #151 and #163 both include `.orig` editor backup files as new files. These should never be committed. **Recommendation:** Remove from PR branches, add `*.orig` to `.gitignore`.

3. **PR #163 scope:** Title says refactor, not perf. The emoji is 🧹 not ⚡. Accurate for what it does — just noting it's not a performance PR unlike the others.
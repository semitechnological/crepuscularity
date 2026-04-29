//! Native benchmark plugin — Rust and GPUI-simulated suites.
//!
//! Registered under plugin id `"bench"` when [`crate::bridge::Capability::Bench`] is enabled.
//! All suites run **synchronously on the calling thread** (V8 thread during a guest eval).  No
//! GPUI context is required; GPUI-style suites simulate entity/render-tree lifecycle patterns with
//! equivalent Rust data structures.
//!
//! ## Usage from guest JS
//! ```js
//! const raw = JSON.parse(Crepus.invoke("bench", "runSuites", JSON.stringify({})));
//! if (raw.ok) { const rustBench = raw.data; }
//! ```

use std::collections::HashMap;
use std::time::Instant;

use serde_json::{json, Value};

use crate::bridge::{BridgeError, Capability, NativePlugin};

/// Runs six Rust/GPUI-simulated benchmark suites on demand.
///
/// Suites (in order):
/// 1. `struct-alloc`       — Vec\<struct\> alloc + traverse + drop (parity: JS `object-alloc`)
/// 2. `hashmap-ops`        — HashMap insert / lookup / remove (parity: JS `map-set-ops`)
/// 3. `vec-math`           — Vec\<f32\> elementwise multiply + sum (parity: JS `typed-array-math`)
/// 4. `string-hash`        — FNV-1a hash on growing byte strings (parity: JS `string-ops`)
/// 5. `cx-entity-lifecycle`— GPUI-style entity alloc / evict / drop (simulated)
/// 6. `render-tree`        — GPUI-style render tree build + DFS traverse + drop (simulated)
pub struct BenchPlugin;

impl BenchPlugin {
    pub fn new() -> Self {
        BenchPlugin
    }
}

impl Default for BenchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl NativePlugin for BenchPlugin {
    fn id(&self) -> &'static str {
        "bench"
    }

    fn capability(&self) -> Capability {
        Capability::Bench
    }

    fn methods(&self) -> &'static [&'static str] {
        &["runSuites"]
    }

    fn invoke(&self, method: &str, _payload: &Value) -> Result<Value, BridgeError> {
        match method {
            "runSuites" => Ok(run_all_suites()),
            _ => unreachable!("method allowlist enforced by bridge"),
        }
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn round_ms(raw: f64) -> f64 {
    (raw.max(0.000_1) * 1000.0).round() / 1000.0
}

fn suite_score(work_units: u64, ms_raw: f64) -> u64 {
    (work_units as f64 / (ms_raw.max(0.000_1) / 1000.0)).round() as u64
}

// ── main runner ───────────────────────────────────────────────────────────────

fn run_all_suites() -> Value {
    let mut suites: Vec<Value> = Vec::new();
    let mut total_work_units: u64 = 0;
    let mut total_ms_raw: f64 = 0.0;
    let mut acc: u64 = 0;

    // ── Suite 1: struct-alloc ─────────────────────────────────────────────────
    // Parity with JS `object-alloc`: allocate and drop many small structs.
    {
        struct Item {
            a: u32,
            b: f64,
            c: u64,
        }

        const ROUNDS: usize = 100_000;
        let t0 = Instant::now();
        let mut v: Vec<Item> = Vec::with_capacity(ROUNDS);
        for i in 0..ROUNDS {
            v.push(Item {
                a: i as u32,
                b: i as f64 * 1.618,
                c: (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
            });
        }
        for item in &v {
            acc = acc.wrapping_add(item.a as u64 ^ item.c ^ item.b.to_bits());
        }
        drop(v);
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        let work = ROUNDS as u64;
        total_work_units += work;
        total_ms_raw += elapsed;
        suites.push(json!({
            "id": "struct-alloc",
            "title": "Vec<struct> alloc + traverse + drop (Rust parity: JS object-alloc)",
            "ms": round_ms(elapsed),
            "workUnits": work,
            "score": suite_score(work, elapsed),
            "acc": acc % 1_000_003,
            "track": "rust",
        }));
    }

    // ── Suite 2: hashmap-ops ──────────────────────────────────────────────────
    // Parity with JS `map-set-ops`: insert / lookup / remove on HashMap<u32, u64>.
    {
        const ROUNDS: usize = 100_000;
        let t0 = Instant::now();
        let mut map: HashMap<u32, u64> = HashMap::with_capacity(ROUNDS);
        for i in 0..ROUNDS {
            map.insert(i as u32, (i as u64).wrapping_mul(6_364_136_223_846_793_005));
        }
        for i in 0..ROUNDS {
            if let Some(&v) = map.get(&(i as u32)) {
                acc = acc.wrapping_add(v);
            }
        }
        for i in (0..ROUNDS).step_by(3) {
            map.remove(&(i as u32));
        }
        acc = acc.wrapping_add(map.len() as u64);
        drop(map);
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        let work = (ROUNDS * 3) as u64;
        total_work_units += work;
        total_ms_raw += elapsed;
        suites.push(json!({
            "id": "hashmap-ops",
            "title": "HashMap insert / lookup / remove (Rust parity: JS map-set-ops)",
            "ms": round_ms(elapsed),
            "workUnits": work,
            "score": suite_score(work, elapsed),
            "acc": acc % 1_000_003,
            "track": "rust",
        }));
    }

    // ── Suite 3: vec-math ─────────────────────────────────────────────────────
    // Parity with JS `typed-array-math`: elementwise f32 multiply + sum.
    {
        const N: usize = 64_000;
        const PASSES: usize = 20;
        let t0 = Instant::now();
        let mut data: Vec<f32> = (0..N).map(|i| i as f32 * 0.001_f32).collect();
        for _ in 0..PASSES {
            let mut s: f32 = 0.0;
            for x in &mut data {
                *x = (*x * 1.000_01_f32).min(1e9_f32);
                s += *x;
            }
            acc = acc.wrapping_add(s.to_bits() as u64);
        }
        drop(data);
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        let work = (N * PASSES * 2) as u64;
        total_work_units += work;
        total_ms_raw += elapsed;
        suites.push(json!({
            "id": "vec-math",
            "title": "Vec<f32> elementwise multiply + sum (Rust parity: JS typed-array-math)",
            "ms": round_ms(elapsed),
            "workUnits": work,
            "score": suite_score(work, elapsed),
            "acc": acc % 1_000_003,
            "track": "rust",
        }));
    }

    // ── Suite 4: string-hash ──────────────────────────────────────────────────
    // Parity with JS `string-ops`: FNV-1a rolling hash on growing byte strings.
    {
        const ROUNDS: usize = 25_000;
        const CHUNK: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

        let t0 = Instant::now();
        for i in 0..ROUNDS {
            let repeat = 4 + (i & 7);
            let len = CHUNK.len() * repeat;
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for k in 0..len {
                h ^= CHUNK[k % CHUNK.len()] as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
            acc = acc.wrapping_add(h);
        }
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        let work = (ROUNDS * 40) as u64;
        total_work_units += work;
        total_ms_raw += elapsed;
        suites.push(json!({
            "id": "string-hash",
            "title": "FNV-1a hash on growing byte strings (Rust parity: JS string-ops)",
            "ms": round_ms(elapsed),
            "workUnits": work,
            "score": suite_score(work, elapsed),
            "acc": acc % 1_000_003,
            "track": "rust",
        }));
    }

    // ── Suite 5: cx-entity-lifecycle ──────────────────────────────────────────
    // GPUI-simulated: repeated alloc / traverse / evict of entity-like heap boxes.
    // Models a context that manages a pool of live entities with periodic eviction,
    // approximating GPUI's `cx.new(…)` / entity drop lifecycle under churn.
    {
        const ROUNDS: usize = 50_000;
        let t0 = Instant::now();
        let mut next_id: u64 = 1;
        let mut entities: HashMap<u64, Box<[u8; 64]>> = HashMap::with_capacity(512);
        for _ in 0..ROUNDS {
            let id = next_id;
            next_id += 1;
            let mut data = Box::new([0u8; 64]);
            for (j, b) in data.iter_mut().enumerate() {
                *b = (id as u8).wrapping_add(j as u8);
            }
            entities.insert(id, data);
            if id.is_multiple_of(3) {
                entities.remove(&id.saturating_sub(2));
            }
        }
        acc = acc.wrapping_add(entities.values().map(|b| b[0] as u64).sum::<u64>());
        drop(entities);
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        let work = (ROUNDS * 2) as u64;
        total_work_units += work;
        total_ms_raw += elapsed;
        suites.push(json!({
            "id": "cx-entity-lifecycle",
            "title": "GPUI-style entity alloc / evict / drop lifecycle (simulated)",
            "ms": round_ms(elapsed),
            "workUnits": work,
            "score": suite_score(work, elapsed),
            "acc": acc % 1_000_003,
            "track": "gpui",
        }));
    }

    // ── Suite 6: render-tree ──────────────────────────────────────────────────
    // GPUI-simulated: build + DFS-traverse + drop a 5-deep 4-ary render tree.
    // Models one frame's worth of GPUI element tree construction and traversal.
    {
        struct Node {
            value: u32,
            children: Vec<Node>,
        }

        fn build(depth: usize, fanout: usize, seed: u32) -> Node {
            if depth == 0 {
                return Node {
                    value: seed,
                    children: Vec::new(),
                };
            }
            let children = (0..fanout)
                .map(|i| {
                    build(
                        depth - 1,
                        fanout,
                        seed.wrapping_mul(31).wrapping_add(i as u32),
                    )
                })
                .collect();
            Node {
                value: seed,
                children,
            }
        }

        fn traverse(node: &Node, acc: &mut u64) {
            *acc = acc.wrapping_add(node.value as u64);
            for child in &node.children {
                traverse(child, acc);
            }
        }

        const DEPTH: usize = 5;
        const FANOUT: usize = 4;
        const ROUNDS: usize = 200;
        // 4-ary tree of depth 5: sum_{k=0}^{5} 4^k = (4^6 - 1) / 3 = 1365 nodes.
        let node_count = (FANOUT.pow((DEPTH + 1) as u32) - 1) / (FANOUT - 1);

        let t0 = Instant::now();
        for r in 0..ROUNDS {
            let tree = build(DEPTH, FANOUT, r as u32);
            traverse(&tree, &mut acc);
            drop(tree);
        }
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        let work = (ROUNDS * node_count) as u64;
        total_work_units += work;
        total_ms_raw += elapsed;
        suites.push(json!({
            "id": "render-tree",
            "title": "GPUI-style render tree build + DFS traverse + drop (simulated)",
            "ms": round_ms(elapsed),
            "workUnits": work,
            "score": suite_score(work, elapsed),
            "acc": acc % 1_000_003,
            "track": "gpui",
        }));
    }

    let ms_total = round_ms(total_ms_raw);
    let score = suite_score(total_work_units, total_ms_raw);

    json!({
        "benchSuiteVersion": 2,
        "workUnits": total_work_units,
        "ms": ms_total,
        "score": score,
        "acc": acc % 1_000_003,
        "suites": suites,
    })
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_all_suites_shape() {
        let v = run_all_suites();
        assert_eq!(v["benchSuiteVersion"], 2);
        assert!(v["score"].as_u64().unwrap_or(0) > 0);
        assert!(v["ms"].as_f64().unwrap_or(0.0) > 0.0);
        let suites = v["suites"].as_array().unwrap();
        assert_eq!(suites.len(), 6);
        let ids: Vec<&str> = suites.iter().map(|s| s["id"].as_str().unwrap()).collect();
        assert_eq!(
            ids,
            [
                "struct-alloc",
                "hashmap-ops",
                "vec-math",
                "string-hash",
                "cx-entity-lifecycle",
                "render-tree",
            ]
        );
    }

    #[test]
    fn run_all_suites_tracks() {
        let v = run_all_suites();
        let suites = v["suites"].as_array().unwrap();
        let rust_count = suites.iter().filter(|s| s["track"] == "rust").count();
        let gpui_count = suites.iter().filter(|s| s["track"] == "gpui").count();
        assert_eq!(rust_count, 4);
        assert_eq!(gpui_count, 2);
    }

    #[test]
    fn bench_plugin_invoke_run_suites() {
        let plugin = BenchPlugin::new();
        let result = plugin.invoke("runSuites", &serde_json::json!({}));
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v["benchSuiteVersion"], 2);
        assert_eq!(v["suites"].as_array().unwrap().len(), 6);
    }

    #[test]
    fn all_suites_have_positive_score() {
        let v = run_all_suites();
        for suite in v["suites"].as_array().unwrap() {
            let score = suite["score"].as_u64().unwrap_or(0);
            assert!(score > 0, "suite {:?} has zero score", suite["id"]);
        }
    }
}

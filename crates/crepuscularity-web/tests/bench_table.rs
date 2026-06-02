//! Benchmark comparison table — measures render throughput across render modes.
//!
//! Run with: `cargo test -p crepuscularity-web --features hydration --test bench_table -- --nocapture`
//!
//! This test generates a comparison table that answers the Phase 6 acceptance
//! criterion: "benchmark table vs Phase 1 string-only baseline."

#![cfg(feature = "hydration")]

use crepuscularity_core::TemplateContext;
use crepuscularity_web::{render_template_to_html, render_template_to_html_with_hydration};
use std::time::Instant;

const ITERATIONS: u32 = 100;

const SMALL_TEMPLATE: &str =
    "div p-4\n  h1 text-xl\n    \"Hello {name}\"\n  span\n    \"Count: {count}\"";

const MEDIUM_TEMPLATE: &str = "div p-6 bg-white\n  header\n    h1 text-2xl font-bold\n      \"Dashboard\"\n  main\n    for item in {items}\n      div card p-4 border rounded\n        h3\n          \"{item.name}\"\n        span text-sm\n          \"{item.value}\"\n  footer\n    span text-xs text-gray-400\n      \"Footer\"";

const LARGE_TEMPLATE: &str = "div w-full h-full bg-zinc-950 text-white\n  header p-6 border-b border-zinc-800\n    h1 text-3xl font-bold\n      \"{title}\"\n  main p-6\n    for item in {items}\n      div card p-4 mb-4 border border-zinc-700 rounded\n        div flex justify-between\n          span font-medium\n            \"{item.label}\"\n          span text-green-400\n            \"{item.value}\"\n          span text-sm text-zinc-500\n            \"{item.updated}\"\n  footer p-4 text-center text-sm text-zinc-600\n    \"Page {page} of {total}\"";

fn make_large_ctx() -> TemplateContext {
    let mut ctx = TemplateContext::new();
    ctx.set("title", "Analytics Dashboard");
    ctx.set("page", 1i64);
    ctx.set("total", 10i64);

    let mut items = Vec::new();
    for i in 0..20 {
        let mut v = TemplateContext::new();
        v.set("label", format!("Metric {}", i));
        v.set("value", format!("{}", i * 42));
        v.set("updated", format!("{}m ago", i));
        items.push(v);
    }
    ctx.set("items", crepuscularity_core::TemplateValue::List(items));
    ctx
}

fn make_medium_ctx() -> TemplateContext {
    let mut ctx = TemplateContext::new();
    let mut items = Vec::new();
    for i in 0..5 {
        let mut item = TemplateContext::new();
        item.set("name", format!("Item {}", i));
        item.set("value", format!("{}", i * 10));
        items.push(item);
    }
    ctx.set("items", crepuscularity_core::TemplateValue::List(items));
    ctx
}

fn make_small_ctx() -> TemplateContext {
    let mut ctx = TemplateContext::new();
    ctx.set("name", "World");
    ctx.set("count", 42i64);
    ctx
}

fn bench_render(label: &str, template: &str, ctx: &TemplateContext, iterations: u32) -> f64 {
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = render_template_to_html(template, ctx).unwrap();
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    println!("  {:<30} {:>8.1} µs avg", label, avg_us);
    avg_us
}

fn bench_hydration(label: &str, template: &str, ctx: &TemplateContext, iterations: u32) -> f64 {
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = render_template_to_html_with_hydration(template, ctx).unwrap();
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    println!("  {:<30} {:>8.1} µs avg", label, avg_us);
    avg_us
}

#[test]
fn render_benchmark_table() {
    println!();
    println!("=== Crepuscularity SSR Render Benchmark ===");
    println!("Iterations per test: {ITERATIONS}");
    println!();
    println!(
        "{:<30} {:>10} {:>12} {:>10}",
        "Test", "Plain (µs)", "Hydration (µs)", "Overhead %"
    );
    println!("{:-<30} {:-<10} {:-<12} {:-<10}", "", "", "", "");

    let small_ctx = make_small_ctx();
    let medium_ctx = make_medium_ctx();
    let large_ctx = make_large_ctx();

    for (name, tpl, ctx) in [
        ("small (2 bindings)", SMALL_TEMPLATE, &small_ctx),
        ("medium (5-item for)", MEDIUM_TEMPLATE, &medium_ctx),
        ("large (20-item for)", LARGE_TEMPLATE, &large_ctx),
    ] {
        let plain = bench_render(&format!("plain {name}"), tpl, ctx, ITERATIONS);
        let hydr = bench_hydration(&format!("hydr  {name}"), tpl, ctx, ITERATIONS);
        let overhead = if plain > 0.0 {
            ((hydr - plain) / plain) * 100.0
        } else {
            0.0
        };
        println!(
            "  {:<30} {:>10.1} {:>12.1} {:>9.1}%",
            name, plain, hydr, overhead
        );
    }

    println!();
    println!("=== Notes ===");
    println!("- Plain: render_template_to_html (string output only)");
    println!("- Hydration: render_template_to_html_with_hydration (markers + payload)");
    println!("- Overhead = hydration cost / plain cost");
    println!("- Run with --release for production-approximate numbers");
}

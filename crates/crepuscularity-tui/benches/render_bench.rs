use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_core::ComponentMeta;
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::hint::black_box;

fn bench_render_component_defaults(c: &mut Criterion) {
    let mut defaults: HashMap<String, String> = HashMap::new();
    for i in 0..10 {
        defaults.insert(format!("prop{}", i), format!("val{}", i));
    }
    let meta = ComponentMeta {
        defaults,
        ..Default::default()
    };

    // Baseline: The original code using `entry(key.clone()).or_insert_with(...)`
    c.bench_function("baseline", |b| {
        b.iter(|| {
            let mut child_ctx = TemplateContext::default();
            for i in 0..10 {
                child_ctx.vars.insert(
                    format!("prop{}", i),
                    TemplateValue::Str("passed".to_string()),
                );
            }

            for key in meta.defaults.keys() {
                child_ctx
                    .vars
                    .entry(key.clone())
                    .or_insert_with(|| TemplateValue::Str("val".to_string()));
            }
            black_box(child_ctx);
        })
    });

    // Optimized: Using `if !contains_key` to avoid `.clone()` when possible
    c.bench_function("optimized", |b| {
        b.iter(|| {
            let mut child_ctx = TemplateContext::default();
            for i in 0..10 {
                child_ctx.vars.insert(
                    format!("prop{}", i),
                    TemplateValue::Str("passed".to_string()),
                );
            }

            for key in meta.defaults.keys() {
                if !child_ctx.vars.contains_key(key) {
                    child_ctx
                        .vars
                        .insert(key.clone(), TemplateValue::Str("val".to_string()));
                }
            }
            black_box(child_ctx);
        })
    });
}

criterion_group!(benches, bench_render_component_defaults);
criterion_main!(benches);

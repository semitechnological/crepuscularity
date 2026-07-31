//! Robustness of the expression evaluator against adversarial template input.

use crepuscularity_core::{eval::eval_expr, TemplateContext};

#[test]
fn escaped_multi_byte_char_in_string_literal_does_not_panic() {
    let ctx = TemplateContext::new();
    let _ = eval_expr("\"\\ñ\"", &ctx);
    let _ = eval_expr("\"\\你好\"", &ctx);
    let _ = eval_expr("\"a\\€b\"", &ctx);
    let _ = eval_expr("'\\ñ'", &ctx);
}

#[test]
fn deeply_nested_parens_do_not_overflow_the_stack() {
    let ctx = TemplateContext::new();
    let expr = format!("{}1{}", "(".repeat(100_000), ")".repeat(100_000));
    let _ = eval_expr(&expr, &ctx);
}

#[test]
fn deeply_nested_unary_operators_do_not_overflow_the_stack() {
    let ctx = TemplateContext::new();
    let _ = eval_expr(&format!("{}1", "!".repeat(100_000)), &ctx);
    let _ = eval_expr(&format!("{}1", "-".repeat(100_000)), &ctx);
}

#[test]
fn unclosed_parens_still_evaluate_within_depth_budget() {
    let ctx = TemplateContext::new();
    assert!(eval_expr("((1 + 2))", &ctx).is_ok());
    assert!(eval_expr("!!true", &ctx).is_ok());
}

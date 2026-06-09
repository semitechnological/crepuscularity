use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// A runtime variable context passed to the template renderer.
/// Variables can be strings, booleans, numbers, or lists of contexts.
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    pub vars: HashMap<String, TemplateValue>,
    /// Directory of the current `.crepus` file — used to resolve `include` paths.
    pub base_dir: Option<PathBuf>,
    /// Slot content passed from a parent `include` directive.
    /// `(nodes, parent_ctx)` — the nodes are rendered with the parent's context.
    pub slot: Option<(Vec<crate::ast::Node>, Box<TemplateContext>)>,
    /// In-memory virtual file system for WASM / no-filesystem environments.
    /// Keys are paths (e.g. `"views/ui.crepus"`). Checked before real filesystem.
    pub virtual_files: Arc<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub enum TemplateValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<TemplateContext>),
    /// Object scope for `for item in {items}` — enables `{item.field}` in templates.
    Scope(TemplateContext),
    Null,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<TemplateValue>) -> &mut Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&TemplateValue> {
        self.vars.get(key)
    }

    pub fn get_bool(&self, key: &str) -> bool {
        match self.vars.get(key) {
            Some(v) => crate::eval::is_truthy(v),
            None => false,
        }
    }

    pub fn get_str(&self, key: &str) -> String {
        value_to_str(self.vars.get(key).unwrap_or(&TemplateValue::Null))
    }

    pub fn get_list(&self, key: &str) -> Vec<TemplateContext> {
        match self.vars.get(key) {
            Some(TemplateValue::List(items)) => items.clone(),
            _ => Vec::new(),
        }
    }

    /// Evaluate a condition expression — delegates to the full expression evaluator.
    pub fn eval_condition(&self, expr: &str) -> Result<bool, crate::error::CrepusError> {
        crate::eval::eval_condition(expr, self)
    }

    /// Interpolate a string with `{expr}` placeholders.
    /// Expressions are fully evaluated (arithmetic, property access, etc.).
    pub fn interpolate(&self, template: &str) -> Result<String, crate::error::CrepusError> {
        let mut result = String::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut expr = String::new();
                let mut depth = 1usize;
                for c in chars.by_ref() {
                    match c {
                        '{' => {
                            depth += 1;
                            expr.push(c);
                        }
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            expr.push(c);
                        }
                        _ => expr.push(c),
                    }
                }
                let val = crate::eval::eval_expr(expr.trim(), self)?;
                result.push_str(&value_to_str(&val));
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Build a child context for `for` loops: shared virtual files, overlaid variables.
    pub fn child_with_vars(&self, vars: impl IntoIterator<Item = (String, TemplateValue)>) -> Self {
        let mut child = Self {
            vars: self.vars.clone(),
            base_dir: self.base_dir.clone(),
            slot: self.slot.clone(),
            virtual_files: Arc::clone(&self.virtual_files),
        };
        child.vars.extend(vars);
        child
    }
}

/// Convert a `TemplateValue` to a display string.
pub fn value_to_str(v: &TemplateValue) -> String {
    match v {
        TemplateValue::Str(s) => s.clone(),
        TemplateValue::Int(n) => n.to_string(),
        TemplateValue::Float(f) => {
            // Show as integer if it has no fractional part
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        TemplateValue::Bool(b) => b.to_string(),
        TemplateValue::Null => String::new(),
        TemplateValue::List(items) => format!("[{} items]", items.len()),
        TemplateValue::Scope(_) => String::new(),
    }
}

// ── From impls ────────────────────────────────────────────────────────────────

impl From<String> for TemplateValue {
    fn from(s: String) -> Self {
        TemplateValue::Str(s)
    }
}

impl From<&str> for TemplateValue {
    fn from(s: &str) -> Self {
        TemplateValue::Str(s.to_string())
    }
}

impl From<bool> for TemplateValue {
    fn from(b: bool) -> Self {
        TemplateValue::Bool(b)
    }
}

impl From<i64> for TemplateValue {
    fn from(n: i64) -> Self {
        TemplateValue::Int(n)
    }
}

impl From<i32> for TemplateValue {
    fn from(n: i32) -> Self {
        TemplateValue::Int(n as i64)
    }
}

impl From<f64> for TemplateValue {
    fn from(f: f64) -> Self {
        TemplateValue::Float(f)
    }
}

impl From<Vec<TemplateContext>> for TemplateValue {
    fn from(list: Vec<TemplateContext>) -> Self {
        TemplateValue::List(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_plain_text() {
        let ctx = TemplateContext::new();
        assert_eq!(ctx.interpolate("Hello world").unwrap(), "Hello world");
        assert_eq!(ctx.interpolate("").unwrap(), "");
    }

    #[test]
    fn test_interpolate_variable_replacement() {
        let mut ctx = TemplateContext::new();
        ctx.set("name", "Alice");
        ctx.set("age", 30);

        assert_eq!(ctx.interpolate("Hello {name}!").unwrap(), "Hello Alice!");
        assert_eq!(ctx.interpolate("{name} is {age} years old.").unwrap(), "Alice is 30 years old.");
    }

    #[test]
    fn test_interpolate_expression_evaluation() {
        let mut ctx = TemplateContext::new();
        ctx.set("x", 10);
        ctx.set("y", 20);

        assert_eq!(ctx.interpolate("Result: {1 + 2}").unwrap(), "Result: 3");
        assert_eq!(ctx.interpolate("x + y = {x + y}").unwrap(), "x + y = 30");
        assert_eq!(ctx.interpolate("Is x greater? {x > y}").unwrap(), "Is x greater? false");
    }

    #[test]
    fn test_interpolate_multiple_placeholders() {
        let mut ctx = TemplateContext::new();
        ctx.set("a", "alpha");
        ctx.set("b", "beta");
        ctx.set("c", "gamma");

        assert_eq!(
            ctx.interpolate("{a}-{b}-{c}").unwrap(),
            "alpha-beta-gamma"
        );
        assert_eq!(
            ctx.interpolate("Start {a} middle {b} end {c}.").unwrap(),
            "Start alpha middle beta end gamma."
        );
    }

    #[test]
    fn test_interpolate_nested_braces() {
        let ctx = TemplateContext::new();
        // Assuming your expression evaluator can handle something that evaluates to a string or number,
        // but maybe nested braces aren't valid expressions unless they are part of string literals?
        // Wait, the interpolate method supports nested brackets (e.g. `{ { "a": 1 } }` maybe? No wait, that's not expression syntax for dicts maybe?
        // Let's just check the depth counting logic: it should balance `{` and `}`.
        // E.g., `{"{"}` should evaluate to `{` since it's a string literal.
        assert_eq!(ctx.interpolate(r#"Brace: {"{"}"#).unwrap(), "Brace: {");
        assert_eq!(ctx.interpolate(r#"Braces: {"{}"}"#).unwrap(), "Braces: {}");
    }

    #[test]
    fn test_interpolate_error_cases() {
        let ctx = TemplateContext::new();

        // Invalid expression that parsing actually fails on.
        // Note: the basic parser handles `+` as just an empty parse returning Null in some cases,
        // so we use an expression with extra tokens to trigger an evaluation error.
        assert!(ctx.interpolate("Hello {1 2}").is_err());
    }
}

use std::collections::HashMap;

/// A runtime variable context passed to the template renderer.
/// Variables can be strings, booleans, numbers, or lists of contexts.
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    pub vars: HashMap<String, TemplateValue>,
}

#[derive(Debug, Clone)]
pub enum TemplateValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<TemplateContext>),
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
            Some(TemplateValue::Bool(b)) => *b,
            Some(TemplateValue::Int(n)) => *n != 0,
            Some(TemplateValue::Str(s)) => !s.is_empty(),
            Some(TemplateValue::Null) | None => false,
            _ => true,
        }
    }

    pub fn get_str(&self, key: &str) -> String {
        match self.vars.get(key) {
            Some(TemplateValue::Str(s)) => s.clone(),
            Some(TemplateValue::Int(n)) => n.to_string(),
            Some(TemplateValue::Float(f)) => format!("{:.2}", f),
            Some(TemplateValue::Bool(b)) => b.to_string(),
            Some(TemplateValue::Null) | None => String::new(),
            Some(TemplateValue::List(items)) => format!("[{} items]", items.len()),
        }
    }

    pub fn get_list(&self, key: &str) -> Vec<TemplateContext> {
        match self.vars.get(key) {
            Some(TemplateValue::List(items)) => items.clone(),
            _ => Vec::new(),
        }
    }

    /// Evaluate a simple condition expression. Supports:
    /// - `varname` — truthy check
    /// - `!varname` — falsy check
    /// - `varname == "literal"` — equality check
    /// - `varname != "literal"` — inequality check
    pub fn eval_condition(&self, expr: &str) -> bool {
        let expr = expr.trim();

        // Negation
        if let Some(rest) = expr.strip_prefix('!') {
            return !self.eval_condition(rest.trim());
        }

        // Equality: varname == "value" or varname == value
        if let Some(pos) = expr.find("==") {
            let lhs = expr[..pos].trim();
            let rhs = expr[pos + 2..].trim().trim_matches('"');
            return self.get_str(lhs) == rhs;
        }

        // Inequality
        if let Some(pos) = expr.find("!=") {
            let lhs = expr[..pos].trim();
            let rhs = expr[pos + 2..].trim().trim_matches('"');
            return self.get_str(lhs) != rhs;
        }

        // Greater than
        if let Some(pos) = expr.find('>') {
            let lhs = expr[..pos].trim();
            let rhs = expr[pos + 1..].trim();
            let lval = self.vars.get(lhs).and_then(|v| match v {
                TemplateValue::Int(n) => Some(*n as f64),
                TemplateValue::Float(f) => Some(*f),
                _ => None,
            });
            if let (Some(l), Ok(r)) = (lval, rhs.parse::<f64>()) {
                return l > r;
            }
        }

        // Less than
        if let Some(pos) = expr.find('<') {
            let lhs = expr[..pos].trim();
            let rhs = expr[pos + 1..].trim();
            let lval = self.vars.get(lhs).and_then(|v| match v {
                TemplateValue::Int(n) => Some(*n as f64),
                TemplateValue::Float(f) => Some(*f),
                _ => None,
            });
            if let (Some(l), Ok(r)) = (lval, rhs.parse::<f64>()) {
                return l < r;
            }
        }

        // Boolean literals
        if expr == "true" {
            return true;
        }
        if expr == "false" {
            return false;
        }

        // Simple variable truthy check
        self.get_bool(expr)
    }

    /// Interpolate a string with `{varname}` placeholders.
    pub fn interpolate(&self, template: &str) -> String {
        let mut result = String::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut key = String::new();
                let mut depth = 1usize;
                for c in chars.by_ref() {
                    match c {
                        '{' => {
                            depth += 1;
                            key.push(c);
                        }
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            key.push(c);
                        }
                        _ => key.push(c),
                    }
                }
                result.push_str(&self.get_str(key.trim()));
            } else {
                result.push(ch);
            }
        }

        result
    }
}

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

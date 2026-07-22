//! Structured errors for parser, evaluator, and include resolution.

use thiserror::Error;

/// Primary error type for [`crate::parser`], [`crate::eval`], and related core APIs.
#[derive(Debug, Error)]
pub enum CrepusError {
    #[error("parse error: {0}")]
    Parse(String),

    #[error("eval error in `{expr}`: {message}")]
    Eval { expr: String, message: String },

    #[error("include path error: {0}")]
    IncludePath(String),

    #[error("render error: {0}")]
    Render(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl CrepusError {
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    pub fn eval(expr: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Eval {
            expr: expr.into(),
            message: message.into(),
        }
    }

    pub fn include_path(message: impl Into<String>) -> Self {
        Self::IncludePath(message.into())
    }

    pub fn render(message: impl Into<String>) -> Self {
        Self::Render(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let parse_err = CrepusError::parse("unexpected token");
        assert_eq!(parse_err.to_string(), "parse error: unexpected token");

        let eval_err = CrepusError::eval("1 + 1", "variable not found");
        assert_eq!(
            eval_err.to_string(),
            "eval error in `1 + 1`: variable not found"
        );

        let include_err = CrepusError::include_path("file not found");
        assert_eq!(
            include_err.to_string(),
            "include path error: file not found"
        );

        let render_err = CrepusError::render("missing template");
        assert_eq!(render_err.to_string(), "render error: missing template");

        let io_err = CrepusError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file missing",
        ));
        assert_eq!(io_err.to_string(), "I/O error: file missing");
    }
}

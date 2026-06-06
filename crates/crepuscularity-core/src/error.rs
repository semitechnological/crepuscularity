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

use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CrepusCliError {
    Io {
        path: Option<PathBuf>,
        message: String,
    },
    Context(String),
    Command {
        code: Option<i32>,
        message: String,
    },
}

impl CrepusCliError {
    pub fn context(msg: impl Into<String>) -> Self {
        Self::Context(msg.into())
    }

    pub fn io(err: io::Error, path: impl Into<Option<PathBuf>>) -> Self {
        Self::Io {
            path: path.into(),
            message: err.to_string(),
        }
    }

    pub fn command(code: Option<i32>, message: impl Into<String>) -> Self {
        Self::Command {
            code,
            message: message.into(),
        }
    }
}

impl From<io::Error> for CrepusCliError {
    fn from(err: io::Error) -> Self {
        Self::Io {
            path: None,
            message: err.to_string(),
        }
    }
}

impl From<String> for CrepusCliError {
    fn from(message: String) -> Self {
        Self::Context(message)
    }
}

impl fmt::Display for CrepusCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                path: Some(path),
                message,
            } => write!(f, "{message} ({})", path.display()),
            Self::Io {
                path: None,
                message,
            } => write!(f, "{message}"),
            Self::Context(message) => write!(f, "{message}"),
            Self::Command {
                code: Some(code),
                message,
            } => write!(f, "{message} (exit code {code})"),
            Self::Command {
                code: None,
                message,
            } => write!(f, "{message}"),
        }
    }
}

//! Compiler invocation for generating C output.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::detector::{find_compiler, Language};

/// Error during compilation.
#[derive(Debug)]
pub enum CompileError {
    /// Compiler not found on system.
    CompilerNotFound { language: Language },
    /// Compilation failed.
    CompilationFailed { stderr: String, exit_code: Option<i32> },
    /// IO error.
    Io(std::io::Error),
    /// Language doesn't support C output.
    UnsupportedCOutput { language: Language },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::CompilerNotFound { language } => {
                write!(f, "Compiler for {:?} not found", language)
            }
            CompileError::CompilationFailed { stderr, exit_code } => {
                write!(f, "Compilation failed (exit {:?}): {}", exit_code, stderr)
            }
            CompileError::Io(e) => write!(f, "IO error: {}", e),
            CompileError::UnsupportedCOutput { language } => {
                write!(f, "{:?} doesn't support direct C output", language)
            }
        }
    }
}

impl std::error::Error for CompileError {}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        CompileError::Io(e)
    }
}

/// Result of a successful compilation.
#[derive(Clone, Debug)]
pub struct CompileResult {
    /// Path to the generated C file or object.
    pub output_path: PathBuf,
    /// Path to the generated header (if any).
    pub header_path: Option<PathBuf>,
    /// The language that was compiled.
    pub language: Language,
    /// Compiler output (stdout).
    pub stdout: String,
    /// Compiler warnings (stderr, if successful).
    pub stderr: String,
}

/// Compile a source file to C intermediate representation.
pub fn compile_to_c(input: &Path, output_dir: &Path) -> Result<CompileResult, CompileError> {
    let language = crate::detector::detect_language(input)
        .ok_or_else(|| CompileError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unknown source language",
        )))?;

    compile_to_c_with_lang(input, output_dir, language)
}

/// Compile a source file to C with explicit language.
pub fn compile_to_c_with_lang(
    input: &Path,
    output_dir: &Path,
    language: Language,
) -> Result<CompileResult, CompileError> {
    // Find compiler
    let info = find_compiler(language)
        .ok_or(CompileError::CompilerNotFound { language })?;

    let compiler = info.compiler.as_ref().unwrap();

    // Determine output file name
    let stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let c_output = output_dir.join(format!("{stem}.c"));
    let header_output = output_dir.join(format!("{stem}.h"));

    // Build command
    let input_str = input.to_string_lossy();
    let output_str = c_output.to_string_lossy();

    let args = language.to_c_args(&input_str, &output_str);

    let output = Command::new(compiler)
        .args(&args)
        .current_dir(input.parent().unwrap_or(Path::new(".")))
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(CompileError::CompilationFailed {
            stderr,
            exit_code: output.status.code(),
        });
    }

    // Check if header was generated (language-specific)
    let header_path = if header_output.exists() {
        Some(header_output)
    } else {
        // Try to generate header for some languages
        generate_header(input, output_dir, language).ok()
    };

    Ok(CompileResult {
        output_path: c_output,
        header_path,
        language,
        stdout,
        stderr,
    })
}

/// Generate a C header file for the compiled code.
fn generate_header(input: &Path, output_dir: &Path, language: Language) -> Result<PathBuf, CompileError> {
    let stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let header_path = output_dir.join(format!("{stem}.h"));

    match language {
        Language::Rust => {
            // Use cbindgen for Rust
            if which::which("cbindgen").is_ok() {
                let output = Command::new("cbindgen")
                    .args([
                        "--lang", "c",
                        "--output", header_path.to_string_lossy().as_ref(),
                        input.parent().unwrap_or(Path::new(".")).to_string_lossy().as_ref(),
                    ])
                    .output()?;

                if output.status.success() && header_path.exists() {
                    return Ok(header_path);
                }
            }
            Err(CompileError::UnsupportedCOutput { language })
        }
        Language::V => {
            // V generates headers automatically with -backend c
            // The header should be alongside the C file
            if header_path.exists() {
                Ok(header_path)
            } else {
                Err(CompileError::UnsupportedCOutput { language })
            }
        }
        _ => Err(CompileError::UnsupportedCOutput { language }),
    }
}

/// Compile multiple files to C.
pub fn compile_batch(
    files: &[(PathBuf, Language)],
    output_dir: &Path,
) -> Vec<Result<CompileResult, CompileError>> {
    files
        .iter()
        .map(|(path, lang)| compile_to_c_with_lang(path, output_dir, *lang))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_compile_error_display() {
        let err = CompileError::CompilerNotFound { language: Language::V };
        assert!(err.to_string().contains("V"));
    }

    #[test]
    fn test_compile_nonexistent() {
        let dir = tempdir().unwrap();
        let result = compile_to_c(Path::new("/nonexistent/file.v"), dir.path());
        assert!(result.is_err());
    }
}

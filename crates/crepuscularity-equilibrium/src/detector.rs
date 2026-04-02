//! Language detection for source files.

use std::path::Path;

/// Supported languages that can be compiled to C.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    /// V language (vlang.io)
    V,
    /// Zig language
    Zig,
    /// C (already native)
    C,
    /// C++
    Cpp,
    /// C#
    CSharp,
    /// Rust (for cbindgen)
    Rust,
}

/// Information about a detected language.
#[derive(Clone, Debug)]
pub struct LanguageInfo {
    pub language: Language,
    pub compiler: Option<String>,
    pub version: Option<String>,
}

impl Language {
    /// Get the file extensions for this language.
    pub fn extensions(&self) -> &[&str] {
        match self {
            Language::V => &["v"],
            Language::Zig => &["zig"],
            Language::C => &["c", "h"],
            Language::Cpp => &["cpp", "cxx", "cc", "hpp", "hxx"],
            Language::CSharp => &["cs"],
            Language::Rust => &["rs"],
        }
    }

    /// Get the typical compiler command for this language.
    pub fn default_compiler(&self) -> &str {
        match self {
            Language::V => "v",
            Language::Zig => "zig",
            Language::C => "clang",
            Language::Cpp => "clang++",
            Language::CSharp => "csc",
            Language::Rust => "rustc",
        }
    }

    /// Get the command to compile to C intermediate.
    pub fn to_c_args(&self, input: &str, output: &str) -> Vec<String> {
        match self {
            Language::V => vec![
                "-o".to_string(),
                output.to_string(),
                "-backend".to_string(),
                "c".to_string(),
                input.to_string(),
            ],
            Language::Zig => {
                // Zig doesn't have direct C output, but we can use translate-c for headers
                // For actual code, we emit object files
                vec![
                    "build-obj".to_string(),
                    "-femit-bin".to_string(),
                    format!("-femit-bin={output}"),
                    input.to_string(),
                ]
            }
            Language::C => {
                // C is already C, just preprocess
                vec![
                    "-E".to_string(),
                    "-o".to_string(),
                    output.to_string(),
                    input.to_string(),
                ]
            }
            Language::Cpp => {
                // Compile to object, we'll need headers separately
                vec![
                    "-c".to_string(),
                    "-o".to_string(),
                    output.to_string(),
                    input.to_string(),
                ]
            }
            Language::CSharp => {
                // C# to native requires AOT compilation
                vec![
                    "-target:library".to_string(),
                    format!("-out:{output}"),
                    input.to_string(),
                ]
            }
            Language::Rust => {
                // Rust uses cbindgen for headers + normal compilation
                vec![
                    "--crate-type=cdylib".to_string(),
                    "-o".to_string(),
                    output.to_string(),
                    input.to_string(),
                ]
            }
        }
    }
}

/// Detect the language of a source file based on extension.
pub fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    for lang in [
        Language::V,
        Language::Zig,
        Language::C,
        Language::Cpp,
        Language::CSharp,
        Language::Rust,
    ] {
        if lang.extensions().contains(&ext.as_str()) {
            return Some(lang);
        }
    }

    None
}

/// Check if a compiler is available on the system.
pub fn find_compiler(language: Language) -> Option<LanguageInfo> {
    let compiler_name = language.default_compiler();

    // Check if compiler exists
    if which::which(compiler_name).is_ok() {
        Some(LanguageInfo {
            language,
            compiler: Some(compiler_name.to_string()),
            version: get_compiler_version(compiler_name),
        })
    } else {
        None
    }
}

fn get_compiler_version(compiler: &str) -> Option<String> {
    let output = std::process::Command::new(compiler)
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Get first line
        stdout.lines().next().map(|s| s.to_string())
    } else {
        None
    }
}

/// Scan a directory and detect all source files with their languages.
pub fn scan_directory(dir: &Path) -> Vec<(std::path::PathBuf, Language)> {
    let mut results = Vec::new();

    fn visit(dir: &Path, results: &mut Vec<(std::path::PathBuf, Language)>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip common non-source directories
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !matches!(name, "target" | "node_modules" | ".git" | "build" | "dist") {
                        visit(&path, results);
                    }
                } else if let Some(lang) = detect_language(&path) {
                    results.push((path, lang));
                }
            }
        }
    }

    visit(dir, &mut results);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_v() {
        let path = Path::new("mylib.v");
        assert_eq!(detect_language(path), Some(Language::V));
    }

    #[test]
    fn test_detect_zig() {
        let path = Path::new("mylib.zig");
        assert_eq!(detect_language(path), Some(Language::Zig));
    }

    #[test]
    fn test_detect_cpp() {
        assert_eq!(detect_language(Path::new("foo.cpp")), Some(Language::Cpp));
        assert_eq!(detect_language(Path::new("foo.cxx")), Some(Language::Cpp));
        assert_eq!(detect_language(Path::new("foo.cc")), Some(Language::Cpp));
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_language(Path::new("foo.py")), None);
        assert_eq!(detect_language(Path::new("foo.js")), None);
    }
}

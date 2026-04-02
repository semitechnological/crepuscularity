//! crepuscularity-equilibrium — Automatic C FFI generation
//!
//! This crate auto-detects C-compiling languages (V, Zig, C++, C#, etc.),
//! compiles them to C intermediate representation, and generates Rust bindings
//! so you can call foreign code like native modules.
//!
//! # Supported Languages
//!
//! | Language | Compiler | C Backend |
//! |----------|----------|-----------|
//! | V (Vlang) | `v` | `v -o output.c -backend c` |
//! | Zig | `zig` | `zig build-obj -femit-asm` or C export |
//! | C/C++ | `clang`/`gcc` | Native |
//! | C# | `csc`/`mono` | P/Invoke + Native AOT |
//! | Rust | `rustc` | cbindgen |
//!
//! # Usage
//!
//! ```rust,ignore
//! use crepuscularity_equilibrium::{detect_language, compile_to_c, generate_bindings};
//!
//! let source = Path::new("mylib.v");
//! let lang = detect_language(source)?;
//! let c_output = compile_to_c(source, lang)?;
//! let bindings = generate_bindings(&c_output)?;
//! ```

mod compiler;
mod detector;
mod bindings;

pub use compiler::{compile_to_c, CompileError, CompileResult};
pub use detector::{detect_language, Language, LanguageInfo};
pub use bindings::{generate_bindings, BindingOptions, GeneratedBinding};

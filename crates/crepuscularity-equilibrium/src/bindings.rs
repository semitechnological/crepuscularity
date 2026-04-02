//! Rust binding generation from C headers.

use std::path::{Path, PathBuf};

/// Options for binding generation.
#[derive(Clone, Debug, Default)]
pub struct BindingOptions {
    /// Module name for the generated bindings.
    pub module_name: Option<String>,
    /// Additional include paths.
    pub include_paths: Vec<PathBuf>,
    /// Functions to allowlist (if empty, include all).
    pub allowlist_functions: Vec<String>,
    /// Types to allowlist (if empty, include all).
    pub allowlist_types: Vec<String>,
    /// Generate impl blocks for types.
    pub derive_debug: bool,
    /// Generate Default impl.
    pub derive_default: bool,
}

/// A generated Rust binding.
#[derive(Clone, Debug)]
pub struct GeneratedBinding {
    /// The generated Rust code.
    pub code: String,
    /// The source header file.
    pub source_header: PathBuf,
    /// Any warnings during generation.
    pub warnings: Vec<String>,
}

/// Generate Rust bindings from a C header file.
///
/// This creates a Rust module with extern "C" declarations
/// that can be used to call the compiled C code.
pub fn generate_bindings(header: &Path, options: &BindingOptions) -> Result<GeneratedBinding, String> {
    if !header.exists() {
        return Err(format!("Header file not found: {}", header.display()));
    }

    let content = std::fs::read_to_string(header)
        .map_err(|e| format!("Failed to read header: {e}"))?;

    let mut warnings = Vec::new();
    let mut code = String::new();

    // Module header
    code.push_str("//! Auto-generated bindings by crepuscularity-equilibrium\n");
    code.push_str("//!\n");
    code.push_str(&format!("//! Source: {}\n", header.display()));
    code.push_str("\n");
    code.push_str("#![allow(non_camel_case_types)]\n");
    code.push_str("#![allow(non_snake_case)]\n");
    code.push_str("#![allow(dead_code)]\n");
    code.push_str("\n");
    code.push_str("use std::os::raw::*;\n");
    code.push_str("\n");

    // Parse and generate bindings
    let parsed = parse_c_header(&content);

    // Generate type definitions
    for typedef in &parsed.typedefs {
        if should_include(&typedef.name, &options.allowlist_types) {
            code.push_str(&generate_typedef(typedef, options));
            code.push_str("\n");
        }
    }

    // Generate struct definitions
    for struct_def in &parsed.structs {
        if should_include(&struct_def.name, &options.allowlist_types) {
            code.push_str(&generate_struct(struct_def, options));
            code.push_str("\n");
        }
    }

    // Generate extern block with functions
    code.push_str("extern \"C\" {\n");
    for func in &parsed.functions {
        if should_include(&func.name, &options.allowlist_functions) {
            code.push_str(&generate_function(func));
        } else {
            warnings.push(format!("Skipped function: {}", func.name));
        }
    }
    code.push_str("}\n");

    Ok(GeneratedBinding {
        code,
        source_header: header.to_path_buf(),
        warnings,
    })
}

fn should_include(name: &str, allowlist: &[String]) -> bool {
    allowlist.is_empty() || allowlist.iter().any(|a| a == name)
}

// Simple C header parsing (for basic cases)
// In production, you'd use bindgen or a proper C parser

struct ParsedHeader {
    typedefs: Vec<TypedefDef>,
    structs: Vec<StructDef>,
    functions: Vec<FunctionDef>,
}

struct TypedefDef {
    name: String,
    target: String,
}

struct StructDef {
    name: String,
    fields: Vec<(String, String)>,
}

struct FunctionDef {
    name: String,
    return_type: String,
    params: Vec<(String, String)>,
}

fn parse_c_header(content: &str) -> ParsedHeader {
    let mut typedefs = Vec::new();
    let mut structs = Vec::new();
    let mut functions = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Simple typedef detection
        if line.starts_with("typedef") && line.ends_with(';') {
            if let Some((target, name)) = parse_typedef_line(line) {
                typedefs.push(TypedefDef { name, target });
            }
        }

        // Simple function declaration detection
        if !line.starts_with("typedef") && !line.starts_with("struct") && line.contains('(') && line.ends_with(';') {
            if let Some(func) = parse_function_line(line) {
                functions.push(func);
            }
        }

        // Note: struct parsing is more complex, skipping for basic impl
    }

    ParsedHeader {
        typedefs,
        structs,
        functions,
    }
}

fn parse_typedef_line(line: &str) -> Option<(String, String)> {
    // typedef int myint;
    let line = line.strip_prefix("typedef")?.trim();
    let line = line.strip_suffix(';')?.trim();

    let parts: Vec<&str> = line.rsplitn(2, ' ').collect();
    if parts.len() == 2 {
        Some((parts[1].to_string(), parts[0].to_string()))
    } else {
        None
    }
}

fn parse_function_line(line: &str) -> Option<FunctionDef> {
    // int foo(int x, char *y);
    let line = line.strip_suffix(';')?.trim();

    let paren_start = line.find('(')?;
    let paren_end = line.rfind(')')?;

    let signature = &line[..paren_start].trim();
    let params_str = &line[paren_start + 1..paren_end];

    // Split return type and name
    let parts: Vec<&str> = signature.rsplitn(2, ' ').collect();
    let (return_type, name) = if parts.len() == 2 {
        (parts[1].to_string(), parts[0].to_string())
    } else {
        ("void".to_string(), parts[0].to_string())
    };

    // Parse parameters
    let params: Vec<(String, String)> = if params_str.trim() == "void" || params_str.is_empty() {
        Vec::new()
    } else {
        params_str
            .split(',')
            .filter_map(|p| {
                let p = p.trim();
                let parts: Vec<&str> = p.rsplitn(2, ' ').collect();
                if parts.len() == 2 {
                    Some((parts[1].to_string(), parts[0].to_string()))
                } else {
                    None
                }
            })
            .collect()
    };

    Some(FunctionDef {
        name,
        return_type,
        params,
    })
}

fn generate_typedef(typedef: &TypedefDef, _options: &BindingOptions) -> String {
    let rust_type = c_type_to_rust(&typedef.target);
    format!("pub type {} = {};\n", typedef.name, rust_type)
}

fn generate_struct(struct_def: &StructDef, options: &BindingOptions) -> String {
    let mut code = String::new();

    // Derive attributes
    let mut derives = vec!["Copy", "Clone"];
    if options.derive_debug {
        derives.push("Debug");
    }
    if options.derive_default {
        derives.push("Default");
    }

    code.push_str(&format!("#[derive({})]\n", derives.join(", ")));
    code.push_str("#[repr(C)]\n");
    code.push_str(&format!("pub struct {} {{\n", struct_def.name));

    for (field_type, field_name) in &struct_def.fields {
        let rust_type = c_type_to_rust(field_type);
        code.push_str(&format!("    pub {}: {},\n", field_name, rust_type));
    }

    code.push_str("}\n");
    code
}

fn generate_function(func: &FunctionDef) -> String {
    let rust_return = c_type_to_rust(&func.return_type);

    let params: Vec<String> = func
        .params
        .iter()
        .map(|(typ, name)| format!("{}: {}", name, c_type_to_rust(typ)))
        .collect();

    let return_clause = if rust_return == "()" {
        String::new()
    } else {
        format!(" -> {}", rust_return)
    };

    format!(
        "    pub fn {}({}){};\n",
        func.name,
        params.join(", "),
        return_clause
    )
}

fn c_type_to_rust(c_type: &str) -> String {
    let c_type = c_type.trim();

    match c_type {
        "void" => "()".to_string(),
        "int" => "c_int".to_string(),
        "unsigned int" | "uint" => "c_uint".to_string(),
        "long" => "c_long".to_string(),
        "unsigned long" | "ulong" => "c_ulong".to_string(),
        "long long" => "c_longlong".to_string(),
        "unsigned long long" => "c_ulonglong".to_string(),
        "short" => "c_short".to_string(),
        "unsigned short" | "ushort" => "c_ushort".to_string(),
        "char" => "c_char".to_string(),
        "unsigned char" | "uchar" => "c_uchar".to_string(),
        "signed char" => "c_schar".to_string(),
        "float" => "c_float".to_string(),
        "double" => "c_double".to_string(),
        "size_t" => "usize".to_string(),
        "ssize_t" => "isize".to_string(),
        "bool" | "_Bool" => "bool".to_string(),
        s if s.ends_with('*') => {
            let inner = s.strip_suffix('*').unwrap().trim();
            if inner == "void" {
                "*mut c_void".to_string()
            } else if inner == "const void" {
                "*const c_void".to_string()
            } else if inner.starts_with("const ") {
                let inner_type = c_type_to_rust(inner.strip_prefix("const ").unwrap());
                format!("*const {}", inner_type)
            } else {
                format!("*mut {}", c_type_to_rust(inner))
            }
        }
        s if s.starts_with("const ") => c_type_to_rust(s.strip_prefix("const ").unwrap()),
        other => other.to_string(), // Custom type, pass through
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_type_to_rust() {
        assert_eq!(c_type_to_rust("int"), "c_int");
        assert_eq!(c_type_to_rust("void"), "()");
        assert_eq!(c_type_to_rust("char *"), "*mut c_char");
        assert_eq!(c_type_to_rust("const char *"), "*const c_char");
    }

    #[test]
    fn test_parse_function() {
        let func = parse_function_line("int add(int a, int b);").unwrap();
        assert_eq!(func.name, "add");
        assert_eq!(func.return_type, "int");
        assert_eq!(func.params.len(), 2);
    }

    #[test]
    fn test_parse_typedef() {
        let (target, name) = parse_typedef_line("typedef int myint;").unwrap();
        assert_eq!(name, "myint");
        assert_eq!(target, "int");
    }
}

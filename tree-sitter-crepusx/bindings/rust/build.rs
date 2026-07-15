fn main() {
    let src_dir = std::path::Path::new("src");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::Path::new(&out_dir);

    // Copy parser.c to a build-time file with a short name so the cc crate's
    // object name length is a multiple of 8, which keeps the ar archive member
    // aligned on macOS. Append a dummy symbol to force __.SYMDEF to a 0-mod-8
    // size and avoid ld "64-bit mach-o member not 8-byte aligned" errors.
    let parser_src = src_dir.join("parser.c");
    let build_src = out_dir.join("abcde.c");
    let mut parser = std::fs::read_to_string(&parser_src).expect("read parser.c");
    parser.push_str("\nconst char a = 0;\n");
    std::fs::write(&build_src, parser).expect("write build parser.c");

    let mut c_config = cc::Build::new();
    c_config.std("c11").include(src_dir);

    #[cfg(target_env = "msvc")]
    c_config.flag("-utf-8");

    c_config.file(&build_src);
    println!("cargo:rerun-if-changed={}", parser_src.to_str().unwrap());

    c_config.compile("tree-sitter-crepusx");
}

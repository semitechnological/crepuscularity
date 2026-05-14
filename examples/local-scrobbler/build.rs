use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(has_zig_scrobbler)");

    if let Ok(sdk) = std::env::var("SDKROOT").or_else(|_| {
        Command::new("xcrun")
            .arg("--show-sdk-path")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
    }) {
        if !sdk.is_empty() {
            println!("cargo:rustc-env=BINDGEN_EXTRA_CLANG_ARGS=-isysroot {sdk}");
        }
    }

    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest.join("foreign-code/scrobbler.zig");
    println!("cargo:rerun-if-changed={}", source.display());

    if equilibrium_ffi::detect_language(&source) != Some(equilibrium_ffi::Language::Zig) {
        return;
    }

    let Some(info) = equilibrium_ffi::find_compiler(equilibrium_ffi::Language::Zig) else {
        return;
    };
    let Some(zig) = info.compiler else {
        return;
    };

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let obj = out_dir.join("scrobbler_zig.o");
    let cache = out_dir.join("zig-cache");
    let status = Command::new(zig)
        .env("ZIG_LOCAL_CACHE_DIR", &cache)
        .env("ZIG_GLOBAL_CACHE_DIR", &cache)
        .args([
            "build-obj",
            "-fPIC",
            "-OReleaseFast",
            "-lc",
            &format!("-femit-bin={}", obj.display()),
            source.to_str().unwrap(),
        ])
        .status();

    if status.map(|s| s.success()).unwrap_or(false) && obj.exists() {
        cc::Build::new().object(&obj).compile("scrobbler_zig");
        println!("cargo:rustc-link-lib=c");
        println!("cargo:rustc-cfg=has_zig_scrobbler");
    }
}

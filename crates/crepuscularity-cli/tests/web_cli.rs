//! Integration tests for `crepus web`.

use std::process::Command;

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

#[test]
fn web_new_scaffolds_crepus_and_runtime() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let status = crepus()
        .current_dir(tmp.path())
        .args(["web", "new", "Acme Labs"])
        .status()
        .expect("spawn crepus web new");
    assert!(status.success());

    let site_dir = tmp.path().join("acme-labs");
    assert!(site_dir.join("index.crepus").is_file());
    assert!(site_dir.join("runtime/Cargo.toml").is_file());
    assert!(site_dir.join("runtime/src/lib.rs").is_file());
    let idx = std::fs::read_to_string(site_dir.join("index.crepus")).expect("read index");
    assert!(idx.contains(".crepus"));
}

#[test]
fn web_build_example_site_emits_wasm() {
    let out = tempfile::tempdir().expect("tempdir");
    let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/web-site")
        .canonicalize()
        .expect("canonicalize examples/web-site");

    let status = crepus()
        .args([
            "web",
            "build",
            "--site",
            example.to_str().unwrap(),
            "--out-dir",
            out.path().to_str().unwrap(),
        ])
        .status()
        .expect("spawn crepus web build");
    assert!(
        status.success(),
        "crepus web build should succeed with wasm32 + wasm-bindgen"
    );

    let has_wasm = std::fs::read_dir(out.path().join("pkg"))
        .expect("read pkg")
        .flatten()
        .any(|e| e.path().extension().map(|x| x == "wasm").unwrap_or(false));
    assert!(
        has_wasm,
        "expected pkg/*.wasm (wasm-bindgen output); install wasm-bindgen-cli and rustup target add wasm32-unknown-unknown"
    );
    let bundle = std::fs::read_to_string(out.path().join("crepus-bundle.json")).expect("bundle");
    assert!(bundle.contains("index.crepus"));
    assert!(bundle.contains("entry"));
    let html = std::fs::read_to_string(out.path().join("index.html")).expect("index.html");
    assert!(html.contains("<!DOCTYPE html>") || html.contains("<!doctype html>"));
    assert!(html.contains("crepus-root"));
}

#[test]
fn web_build_docs_site_emits_wasm() {
    let out = tempfile::tempdir().expect("tempdir");
    let docs_site = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs-site")
        .canonicalize()
        .expect("canonicalize docs-site");

    let status = crepus()
        .args([
            "web",
            "build",
            "--site",
            docs_site.to_str().unwrap(),
            "--out-dir",
            out.path().to_str().unwrap(),
        ])
        .status()
        .expect("spawn");
    assert!(status.success());

    let has_wasm = std::fs::read_dir(out.path().join("pkg"))
        .expect("pkg dir")
        .flatten()
        .any(|e| e.path().extension().map(|x| x == "wasm").unwrap_or(false));
    assert!(has_wasm);
}

#[test]
fn web_build_legacy_site_json_stdout() {
    let json = r##"{
  "businessName": "Stdin Co",
  "domain": null,
  "seo": {
    "title": "Stdin Co",
    "description": "test",
    "ogImage": null
  },
  "theme": {
    "accent": "#3b82f6",
    "accentSoft": "#60a5fa",
    "surface": "#09090b",
    "text": "#fafafa",
    "muted": "#a1a1aa",
    "border": "#27272a"
  },
  "elements": [
    {
      "type": "hero",
      "id": "hero-1",
      "props": {
        "eyebrow": "Welcome",
        "headline": "Meet Stdin Co",
        "subheadline": "Ship a polished landing page from structured data.",
        "primary": { "label": "Get started", "href": "#contact", "external": null },
        "secondary": null,
        "media": null
      }
    }
  ],
  "analytics": null,
  "turnstile": null
}"##;

    let out = crepus()
        .args(["web", "build", "--legacy-site-json", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    use std::io::Write;
    out.stdin
        .as_ref()
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();
    let out = out.wait_with_output().expect("wait");
    assert!(out.status.success());
    let html = String::from_utf8_lossy(&out.stdout);
    assert!(html.contains("Stdin Co"));
}

#[test]
fn web_new_fails_when_dir_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let _ = std::fs::create_dir_all(tmp.path().join("dup-site"));
    let status = crepus()
        .current_dir(tmp.path())
        .args(["web", "new", "dup-site"])
        .status()
        .expect("spawn");
    assert!(!status.success());
}

#[test]
fn root_help_lists_web_commands() {
    let out = crepus().args(["--help"]).output().expect("help");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "crepus --help failed with status {}. \nstdout: {}\nstderr: {}",
        out.status,
        stdout,
        stderr
    );
    assert!(
        stderr.contains("web new") && stderr.contains("web build") && stderr.contains("site-json"),
        "usage should list web commands in stderr:\n{stderr}"
    );
}

#[test]
fn web_build_legacy_missing_json_file_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ghost = tmp.path().join("nope.json");
    let status = crepus()
        .current_dir(tmp.path())
        .args(["web", "build", "--legacy-site-json", "--json"])
        .arg(&ghost)
        .status()
        .expect("spawn");
    assert!(!status.success());
}

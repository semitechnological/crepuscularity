//! Integration tests for `crepus web`.

use std::process::Command;
use std::sync::Mutex;

static DOCS_HOOK_TEST_LOCK: Mutex<()> = Mutex::new(());

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

fn wasm_build_prereqs_ok() -> bool {
    let has_target = std::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("wasm32-unknown-unknown"))
        .unwrap_or(false);
    let has_bindgen = std::process::Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    has_target && has_bindgen
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
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
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn web_build_example_site_emits_wasm() {
    if !wasm_build_prereqs_ok() {
        eprintln!("skipping: wasm32 target or wasm-bindgen not installed");
        return;
    }
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
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn web_build_docs_site_emits_wasm() {
    let _guard = DOCS_HOOK_TEST_LOCK.lock().expect("docs hook test lock");
    if !wasm_build_prereqs_ok() {
        eprintln!("skipping: wasm32 target or wasm-bindgen not installed");
        return;
    }
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
    assert!(status.success(), "docs-site web build should succeed");

    let has_wasm = std::fs::read_dir(out.path().join("pkg"))
        .expect("pkg dir")
        .flatten()
        .any(|e| e.path().extension().map(|x| x == "wasm").unwrap_or(false));
    assert!(has_wasm, "WASM pkg should be emitted");
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
// ponytail: docs-renderer binary removed; docs-target tests removed
#[test]
fn root_build_uses_crepus_toml_lvgl_target() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    std::fs::write(
        root.join("crepus.toml"),
        r##"
[[targets]]
type = "lvgl"
id = "dash"
template = "ui.crepus"
out = "dist/dash.xml"
name = "Dash"
root = "screen"

[targets.vars]
device = "STM32F411"
"##,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("ui.crepus"),
        r##"div #panel bg-[#101820]
  h1 text-white
    "{device}"
"##,
    )
    .expect("write template");

    let status = crepus()
        .current_dir(root)
        .args(["build", "--release", "--opt-level", "size"])
        .status()
        .expect("spawn crepus build");
    assert!(status.success());

    let xml = std::fs::read_to_string(root.join("dist/dash.xml")).expect("read xml");
    assert!(xml.contains(r#"<screen name="Dash">"#));
    assert!(xml.contains("STM32F411"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn root_build_type_selects_only_matching_targets() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    std::fs::write(
        root.join("crepus.toml"),
        r##"
[[targets]]
type = "lvgl"
id = "panel"
template = "panel.crepus"
out = "dist/panel.xml"
name = "Panel"
root = "screen"

[targets.vars]
label = "Panel"

[[targets]]
type = "native"
id = "shell"
template = "shell.crepus"
out = "dist/shell.json"

[targets.vars]
label = "Shell"

[[targets]]
type = "lvgl"
id = "tile"
template = "tile.crepus"
out = "dist/tile.xml"
name = "Tile"
root = "component"

[targets.vars]
label = "Tile"
"##,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("panel.crepus"),
        r##"div #panel
  h1
    "{label}"
"##,
    )
    .expect("write panel template");
    std::fs::write(
        root.join("shell.crepus"),
        r##"div #shell
  h1
    "{label}"
"##,
    )
    .expect("write shell template");
    std::fs::write(
        root.join("tile.crepus"),
        r##"div #tile
  span
    "{label}"
"##,
    )
    .expect("write tile template");

    let out = crepus()
        .current_dir(root)
        .args(["build", "lvgl"])
        .output()
        .expect("spawn crepus build lvgl");
    assert!(
        out.status.success(),
        "crepus build lvgl failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let panel = std::fs::read_to_string(root.join("dist/panel.xml")).expect("read panel xml");
    let tile = std::fs::read_to_string(root.join("dist/tile.xml")).expect("read tile xml");
    assert!(panel.contains(r#"<screen name="Panel">"#));
    assert!(panel.contains("Panel"));
    assert!(tile.contains(r#"<component name="Tile">"#));
    assert!(tile.contains("Tile"));
    assert!(
        !root.join("dist/shell.json").exists(),
        "crepus build lvgl should not build native targets"
    );
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn init_webext_scaffolds_same_shape_as_webext_new() {
    fn collect_files(dir: &std::path::Path) -> Vec<(String, String)> {
        let mut files = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(path) = stack.pop() {
            for entry in std::fs::read_dir(&path).expect("read dir") {
                let entry = entry.expect("dir entry");
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    let rel = path
                        .strip_prefix(dir)
                        .expect("strip prefix")
                        .to_string_lossy()
                        .replace('\\', "/");
                    let body = std::fs::read_to_string(&path).expect("read scaffold file");
                    files.push((rel, body));
                }
            }
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        files
    }

    let direct = tempfile::tempdir().expect("direct tempdir");
    let init = tempfile::tempdir().expect("init tempdir");

    let direct_status = crepus()
        .current_dir(direct.path())
        .args(["webext", "new", "Acme Extension"])
        .status()
        .expect("spawn crepus webext new");
    assert!(direct_status.success());

    let init_status = crepus()
        .current_dir(init.path())
        .args(["init", "webext", "Acme Extension"])
        .status()
        .expect("spawn crepus init webext");
    assert!(init_status.success());

    let direct_files = collect_files(&direct.path().join("acme-extension"));
    let init_files = collect_files(&init.path().join("acme-extension"));
    assert_eq!(direct_files, init_files);
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn web_build_multi_target_manifest_requires_target_flag() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let tom = r#"
[[targets]]
type = "web"
id = "a"
site = "a"
out = "a/dist"

[[targets]]
type = "web"
id = "b"
site = "b"
out = "b/dist"
"#;
    std::fs::write(root.join("crepus.toml"), tom).expect("write manifest");
    std::fs::create_dir_all(root.join("a")).expect("mkdir a");
    std::fs::create_dir_all(root.join("b")).expect("mkdir b");

    let man = root.join("crepus.toml");
    let out = crepus()
        .current_dir(root)
        .args(["web", "build", "--manifest", man.to_str().unwrap()])
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    let out = out.wait_with_output().expect("wait");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--target") || stderr.contains("target"),
        "stderr: {stderr}"
    );
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
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
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
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
        stderr.contains("web new") && stderr.contains("web build"),
        "usage should list web commands in stderr:\n{stderr}"
    );
}

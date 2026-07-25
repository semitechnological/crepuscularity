//! Integration tests for `crepus components`, `crepus moonshine`, and `--emit`.

use std::process::Command;

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn components_list_runs() {
    let output = crepus()
        .args(["components", "list"])
        .output()
        .expect("spawn crepus components list");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Catalog ships with the repo; if missing, command still exits 0 with a warning on stderr.
    if !stdout.is_empty() {
        assert!(
            stdout.contains("button") || stdout.contains("(no components"),
            "unexpected stdout: {stdout}"
        );
    }
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn components_themes_runs() {
    let output = crepus()
        .args(["components", "themes"])
        .output()
        .expect("spawn crepus components themes");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() && !stdout.contains("(no themes") {
        assert!(
            stdout.contains("zinc") || stdout.contains("dawn"),
            "unexpected themes stdout: {stdout}"
        );
    }
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn moonshine_dep_prints_packages() {
    let output = crepus()
        .args(["moonshine", "dep"])
        .output()
        .expect("spawn crepus moonshine dep");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("moonshine"));
    assert!(stdout.contains("@crepuscularity/moonshine"));
    assert!(stdout.contains("@crepuscularity/components"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn moonshine_new_scaffolds_app() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let status = crepus()
        .current_dir(tmp.path())
        .args(["moonshine", "new", "Demo App"])
        .status()
        .expect("spawn crepus moonshine new");
    assert!(status.success());
    let app = tmp.path().join("demo-app");
    assert!(app.join("package.json").is_file());
    assert!(app.join("index.crepus").is_file());
    assert!(app.join("src/main.ts").is_file());
    let pkg = std::fs::read_to_string(app.join("package.json")).expect("package.json");
    assert!(pkg.contains("@crepuscularity/moonshine"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn web_build_emit_moonshine_writes_stub() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let site = tmp.path().join("site");
    std::fs::create_dir_all(&site).expect("mkdir");
    std::fs::write(
        site.join("index.crepus"),
        "stack col gap-2\n text \"hi\"\n button \"Go\"\n",
    )
    .expect("write crepus");
    let out = tmp.path().join("dist");

    let status = crepus()
        .args([
            "web",
            "build",
            "--emit",
            "moonshine",
            "--site",
            site.to_str().unwrap(),
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .status()
        .expect("spawn crepus web build --emit moonshine");
    assert!(status.success(), "emit moonshine should succeed without WASM");
    assert!(out.join("crepus-emit.moonshine.ts").is_file());
    assert!(out.join("crepus-view-ir.json").is_file());
    let stub = std::fs::read_to_string(out.join("crepus-emit.moonshine.ts")).expect("stub");
    assert!(stub.contains("renderNode"));
}

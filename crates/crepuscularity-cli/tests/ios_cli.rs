//! Smoke test: `crepus ios new` lays out XcodeGen + SwiftPM files.

use std::process::Command;

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn ios_new_scaffold_writes_project_yml_and_native_shell() {
    let dir = tempfile::tempdir().expect("tempdir");
    let name = "crepus-ios-cli-test";
    let root = dir.path().join(name);

    let out = Command::new(env!("CARGO_BIN_EXE_crepus"))
        .args(["ios", "new", name])
        .current_dir(dir.path())
        .output()
        .expect("spawn crep ios new");

    assert!(
        out.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(root.join("project.yml").is_file());
    let crepus = std::fs::read_to_string(root.join("crepus.toml")).unwrap();
    assert!(crepus.contains("[ios]"));
    assert!(crepus.contains("scheme = \"CrepusIosCliTestApp\""));
    assert!(crepus.contains("xcodegen_spec"));
    assert!(root.join("NativeShell").join("Package.swift").is_file());
    assert!(root
        .join("NativeShell")
        .join("Sources")
        .join("NativeShell")
        .join("ViewIrModels.swift")
        .is_file());
    let yml = std::fs::read_to_string(root.join("project.yml")).expect("read project.yml");
    assert!(yml.contains("packages:") && yml.contains("targets:"));
    assert!(yml.contains("path: NativeShell"));

    let fixture = std::fs::read_to_string(
        root.join("NativeShell")
            .join("Sources")
            .join("NativeShell")
            .join("fixture.json"),
    )
    .unwrap();
    assert!(fixture.contains("\"version\": 3"));

    // Exercise naming edge case: NativeShell package name collision → HostApp target
    let coll = dir.path().join("native-shell");
    let out2 = Command::new(env!("CARGO_BIN_EXE_crepus"))
        .args(["ios", "new", "native-shell"])
        .current_dir(dir.path())
        .output()
        .expect("spawn");
    assert!(out2.status.success());
    let yml2 = std::fs::read_to_string(coll.join("project.yml")).unwrap();
    assert!(yml2.contains("NativeShellHostApp"));
    let tom = std::fs::read_to_string(coll.join("crepus.toml")).unwrap();
    assert!(tom.contains("scheme = \"NativeShellHostApp\""));
}

use std::process::Command;

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

fn fixture(version: u8) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("src-tauri")).unwrap();
    std::fs::create_dir_all(root.path().join("dist")).unwrap();
    let config = if version == 1 {
        r#"{"build":{"distDir":"../dist"}}"#
    } else {
        r#"{"build":{"frontendDist":"../dist"}}"#
    };
    std::fs::write(root.path().join("src-tauri/tauri.conf.json"), config).unwrap();
    std::fs::write(
        root.path().join("dist/crepus-bundle.json"),
        r#"{"entry":"index.crepus","files":{"index.crepus":"div flex flex-col\n  \"Hello\""}}"#,
    )
    .unwrap();
    root
}

#[test]
fn converts_v1_and_v2_static_crepus_bundles() {
    for version in [1, 2] {
        let source = fixture(version);
        let output = tempfile::tempdir().unwrap();
        let destination = output.path().join("native");
        let out = crepus()
            .args([
                "tauri",
                "convert",
                "--dir",
                source.path().to_str().unwrap(),
                "--out",
                destination.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(destination.join("fixture.json").is_file());
        assert!(destination
            .join("ios/Sources/NativeShell/Generated/CrepusGeneratedView.swift")
            .is_file());
        assert!(destination.join("android/app/src/main/java/dev/crepuscularity/nativeshell/generated/CrepusGeneratedView.kt").is_file());
    }
}

#[test]
fn converts_shipped_examples() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in ["tauri-v1-crepus", "tauri-v2-crepus"] {
        let output = tempfile::tempdir().unwrap();
        let destination = output.path().join("native");
        let out = crepus()
            .args([
                "tauri",
                "convert",
                "--dir",
                root.join("examples").join(name).to_str().unwrap(),
                "--out",
                destination.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(destination.join("fixture.json").is_file());
    }
}

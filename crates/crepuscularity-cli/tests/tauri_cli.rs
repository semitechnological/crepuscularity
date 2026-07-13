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
fn webview_conversion_targets_mobile_and_omits_desktop_from_all() {
    let source = fixture(2);
    std::fs::write(
        source.path().join("dist/crepus-bundle.json"),
        r#"{"entry":"index.crepus","files":{"index.crepus":"embed https://example.com adapter=\"webview\""}}"#,
    )
    .unwrap();
    for target in ["mobile", "all"] {
        let output = tempfile::tempdir().unwrap();
        let destination = output.path().join("native");
        let out = crepus()
            .args([
                "tauri",
                "convert",
                "--target",
                target,
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
        assert!(destination.join("ios").is_dir());
        assert!(destination.join("android").is_dir());
        assert!(!destination.join("desktop").exists());
        if target == "all" {
            assert!(String::from_utf8_lossy(&out.stderr).contains("omit GPUI desktop output"));
        }
    }
}

#[test]
fn webview_conversion_rejects_desktop() {
    let source = fixture(2);
    std::fs::write(
        source.path().join("dist/crepus-bundle.json"),
        r#"{"entry":"index.crepus","files":{"index.crepus":"embed https://example.com adapter=\"webview\""}}"#,
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join("native");
    let out = crepus()
        .args([
            "tauri",
            "convert",
            "--target",
            "desktop",
            "--dir",
            source.path().to_str().unwrap(),
            "--out",
            destination.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("GPUI has no WebView host"));
}

#[test]
fn converts_shipped_examples() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in ["tauri-v1-crepus", "tauri-v2-crepus", "tauri-v2-multiwindow"] {
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

#[test]
fn preserves_nested_bundle_entries_for_native_sync() {
    let source = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(source.path().join("src-tauri")).unwrap();
    std::fs::create_dir_all(source.path().join("dist")).unwrap();
    std::fs::write(
        source.path().join("src-tauri/tauri.conf.json"),
        r#"{"build":{"frontendDist":"../dist"}}"#,
    )
    .unwrap();
    std::fs::write(
        source.path().join("dist/crepus-bundle.json"),
        r#"{"entry":"app/index.crepus","files":{"app/index.crepus":"include card.crepus#Card","app/card.crepus":"--- Card\ndiv\n  \"Nested\""}}"#,
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join("native");
    let converted = crepus()
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
        converted.status.success(),
        "{}",
        String::from_utf8_lossy(&converted.stderr)
    );
    let sync = crepus()
        .args([
            "native",
            "ir",
            destination.join("views/main.crepus").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        sync.status.success(),
        "{}",
        String::from_utf8_lossy(&sync.stderr)
    );
}

#[test]
fn rejects_unsafe_bundle_without_creating_output() {
    let source = fixture(2);
    std::fs::write(
        source.path().join("dist/crepus-bundle.json"),
        r#"{"entry":"../bad.crepus","files":{"../bad.crepus":"div"}}"#,
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join("native");
    let converted = crepus()
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
    assert!(!converted.status.success());
    assert!(!destination.exists());
}

#[test]
fn audit_and_conversion_reject_unadapted_tauri_commands() {
    let source = fixture(2);
    std::fs::write(
        source.path().join("src-tauri/lib.rs"),
        "#[tauri::command]\nfn greet() {}\n",
    )
    .unwrap();
    let audit = crepus()
        .args(["tauri", "audit", "--dir", source.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!audit.status.success());
    assert!(String::from_utf8_lossy(&audit.stdout).contains("command"));
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join("native");
    let converted = crepus()
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
    assert!(!converted.status.success());
    assert!(!destination.exists());
}

#[test]
fn conversion_preserves_tauri_application_metadata() {
    let source = fixture(2);
    std::fs::write(
        source.path().join("src-tauri/tauri.conf.json"),
        r#"{"productName":"Desk","version":"2.3.4","identifier":"dev.example.desk","build":{"frontendDist":"../dist"},"app":{"windows":[{"label":"main","title":"Desk Window","width":800,"height":600},{"label":"settings","title":"Settings","width":480,"height":360}]}}"#,
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    let destination = output.path().join("native");
    let converted = crepus()
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
        converted.status.success(),
        "{}",
        String::from_utf8_lossy(&converted.stderr)
    );
    assert!(
        std::fs::read_to_string(destination.join("android/app/src/main/AndroidManifest.xml"))
            .unwrap()
            .contains("Desk Window")
    );
    assert!(destination
        .join("android/app/src/main/res/values/themes.xml")
        .is_file());
    assert!(
        std::fs::read_to_string(destination.join("android/app/build.gradle.kts"))
            .unwrap()
            .contains("applicationId = \"dev.example.desk\"")
    );
    let desktop = std::fs::read_to_string(destination.join("desktop/src/main.rs")).unwrap();
    assert_eq!(desktop.matches("cx.open_window").count(), 2);
    assert!(desktop.contains("\"settings\", \"Settings\""));
    assert!(desktop.contains("px(480.), px(360.)"));
}

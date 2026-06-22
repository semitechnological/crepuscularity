use std::io::Write;
use std::process::{Command, Stdio};

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_ir_renders_file_with_context() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl = tmp.path().join("hello.crepus");
    let ctx = tmp.path().join("context.json");
    std::fs::write(&tpl, "div\n  \"Hello {name}\"").expect("write template");
    std::fs::write(&ctx, r#"{"name":"Ada"}"#).expect("write context");

    let out = crepus()
        .args([
            "native",
            "ir",
            tpl.to_str().unwrap(),
            "--ctx",
            ctx.to_str().unwrap(),
        ])
        .output()
        .expect("spawn crepus native ir");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).expect("IR JSON");
    assert_eq!(value["version"], 3);
    assert_eq!(value["root"][0]["children"][0]["content"], "Hello Ada");
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_ir_pretty_outputs_pretty_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl = tmp.path().join("hello.crepus");
    std::fs::write(&tpl, "div\n  \"Hi\"").expect("write template");

    let out = crepus()
        .args(["native", "ir", tpl.to_str().unwrap(), "--pretty"])
        .output()
        .expect("spawn crepus native ir");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(stdout.contains("\n  \"version\""));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_ir_renders_stdin_template() {
    let mut child = crepus()
        .args(["native", "ir", "--stdin", "--base-dir", "."])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn crepus native ir");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"div\n  \"stdin\"")
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).expect("IR JSON");
    assert_eq!(value["root"][0]["children"][0]["content"], "stdin");
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_ir_renders_stdin_json_virtual_files() {
    let payload = serde_json::json!({
        "entry": "main.crepus",
        "files": {
            "main.crepus": "include card.crepus#Card title={name}",
            "card.crepus": "--- Card\ndiv\n  \"{title}\""
        },
        "context": { "name": "Ada" }
    });
    let mut child = crepus()
        .args(["native", "ir", "--stdin-json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn crepus native ir");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(payload.to_string().as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).expect("IR JSON");
    assert_eq!(value["root"][0]["children"][0]["content"], "Ada");
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_ir_renders_component() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl = tmp.path().join("ui.crepus");
    std::fs::write(&tpl, "--- Card\ndiv\n  \"Card\"").expect("write template");

    let out = crepus()
        .args(["native", "ir", tpl.to_str().unwrap(), "--component", "Card"])
        .output()
        .expect("spawn crepus native ir");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).expect("IR JSON");
    assert_eq!(value["root"][0]["children"][0]["content"], "Card");
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_ir_rejects_nested_context_object() {
    let payload = serde_json::json!({
        "template": "div\n  \"bad\"",
        "context": { "bad": { "nested": true } }
    });
    let mut child = crepus()
        .args(["native", "ir", "--stdin-json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn crepus native ir");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(payload.to_string().as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");
    assert!(!out.status.success());
    let err: serde_json::Value = serde_json::from_slice(&out.stderr).expect("error JSON");
    assert!(err["error"].as_str().unwrap().contains("object values"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_codegen_writes_swiftui_source() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl = tmp.path().join("screen.crepus");
    let out_dir = tmp.path().join("Generated");
    std::fs::write(
        &tpl,
        "div flex flex-col gap-4 p-4\n  span text-lg font-bold\n    \"Hello {name}\"\n  button @click=tap\n    \"Tap\"",
    )
    .expect("write template");

    let out = crepus()
        .args([
            "native",
            "codegen",
            tpl.to_str().unwrap(),
            "--platform",
            "swiftui",
            "--out",
            out_dir.to_str().unwrap(),
            "--view-name",
            "GreetingScreen",
            "--var",
            "name=Ada",
        ])
        .output()
        .expect("spawn crepus native codegen");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let generated = std::fs::read_to_string(out_dir.join("GreetingScreen.swift")).unwrap();
    assert!(generated.contains("public struct GreetingScreen: View"));
    assert!(generated.contains("Text(\"Hello Ada\")"));
    assert!(generated.contains("Button(action: { CrepusActions.perform(\"tap\") })"));
    assert!(generated.contains(".padding(16)"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_codegen_writes_compose_source() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl = tmp.path().join("screen.crepus");
    let out_dir = tmp.path().join("generated");
    std::fs::write(
        &tpl,
        "div flex flex-row gap-2 p-2\n  span text-lg\n    \"Hello {name}\"\n  button @click=tap\n    \"Tap\"",
    )
    .expect("write template");

    let out = crepus()
        .args([
            "native",
            "codegen",
            tpl.to_str().unwrap(),
            "--platform",
            "compose",
            "--out",
            out_dir.to_str().unwrap(),
            "--view-name",
            "GreetingScreen",
            "--var",
            "name=Ada",
        ])
        .output()
        .expect("spawn crepus native codegen");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let generated = std::fs::read_to_string(out_dir.join("GreetingScreen.kt")).unwrap();
    assert!(generated.contains("@Composable"));
    assert!(generated.contains("fun GreetingScreen(modifier: Modifier = Modifier)"));
    assert!(generated.contains("Text(\"Hello Ada\""));
    assert!(generated.contains("Button(onClick = { CrepusActions.perform(\"tap\") })"));
    assert!(generated.contains("modifier.padding(8.dp)"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_codegen_writes_scaffold_compose_package() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl = tmp.path().join("screen.crepus");
    let out_dir = tmp
        .path()
        .join("android/app/src/main/java/dev/crepuscularity/nativeshell/generated");
    std::fs::write(&tpl, "span\n  \"Hello\"").expect("write template");

    let out = crepus()
        .args([
            "native",
            "codegen",
            tpl.to_str().unwrap(),
            "--platform",
            "compose",
            "--out",
            out_dir.to_str().unwrap(),
            "--view-name",
            "GreetingScreen",
        ])
        .output()
        .expect("spawn crepus native codegen");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let generated = std::fs::read_to_string(out_dir.join("GreetingScreen.kt")).unwrap();
    assert!(generated.starts_with("package dev.crepuscularity.nativeshell\n\n"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn mobile_help_lists_core_commands() {
    let out = crepus()
        .args(["mobile", "--help"])
        .output()
        .expect("spawn crepus mobile --help");
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("crepus mobile"));
    assert!(stderr.contains("new <name>"));
    assert!(stderr.contains("dev [--platform"));
    assert!(stderr.contains("doctor [--platform"));
    assert!(stderr.contains("codegen"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn mobile_new_scaffolds_runtime_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = crepus()
        .current_dir(tmp.path())
        .args(["mobile", "new", "phone"])
        .output()
        .expect("spawn crepus mobile new");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let root = tmp.path().join("phone");
    assert!(root.join("crepus.toml").is_file());
    assert!(root.join("views/main.crepus").is_file());
    assert!(root.join("fixture.json").is_file());
    assert!(root.join("ios/project.yml").is_file());
    assert!(root.join("ios/crepus.toml").is_file());
    assert!(root.join("ios/Package.swift").is_file());
    assert!(root.join("ios/App/CrepusMobileApp.swift").is_file());
    assert!(root.join("ios/App/ContentView.swift").is_file());
    assert!(root.join("rust/Cargo.toml").is_file());
    assert!(root.join("rust/src/lib.rs").is_file());

    let rust_actions =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read CrepusRustActions.swift");
    assert!(rust_actions.contains("public final class CrepusStateStore: ObservableObject"));
    assert!(rust_actions.contains("crepusMobileStoreResultJson"));
    assert!(rust_actions.contains("crepusMobileEvalText"));
    assert!(rust_actions.contains("JSONSerialization.data(withJSONObject:"));
    assert!(rust_actions.contains("UIImpactFeedbackGenerator"));
    assert!(rust_actions.contains("UIDevice.current"));
    assert!(rust_actions.contains("Bundle.main.bundleIdentifier"));
    assert!(rust_actions.contains("UserDefaults.standard"));
    assert!(!rust_actions.contains(r#""action":"\(action)""#));
    assert!(!rust_actions.contains("result.contains"));

    let android_actions = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read CrepusRustActions.kt");
    assert!(android_actions.contains("object CrepusStateStore"));
    assert!(android_actions.contains("external fun storeResultJson"));
    assert!(android_actions.contains("external fun evalText"));
    assert!(android_actions.contains("getSharedPreferences(\"crepus_preferences\""));
    assert!(android_actions.contains("Build.MANUFACTURER"));
    assert!(android_actions.contains("packageManager.getPackageInfo"));
    assert!(android_actions.contains("VibrationEffect.createOneShot"));

    let project_yml =
        std::fs::read_to_string(root.join("ios/project.yml")).expect("read project.yml");
    assert!(project_yml
        .contains("$(PROJECT_DIR)/build/rust/aarch64-apple-ios/libcrepus_mobile_actions.a"));
    assert!(project_yml
        .contains("$(PROJECT_DIR)/build/rust/aarch64-apple-ios-sim/libcrepus_mobile_actions.a"));
    assert!(!project_yml
        .contains("$(PROJECT_DIR)/build/rust/$(PLATFORM_NAME)/libcrepus_mobile_actions.a"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn mobile_new_scaffolds_android_runtime_audit_fixes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = crepus()
        .current_dir(tmp.path())
        .args(["mobile", "new", "phone"])
        .output()
        .expect("spawn crepus mobile new");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let root = tmp.path().join("phone");
    let android_actions = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read CrepusRustActions.kt");
    assert!(android_actions.contains("mutableLongStateOf(0L)"));
    assert!(android_actions.contains("CrepusRustActions.storeResultJson(raw)"));
    assert!(android_actions.contains("CrepusRustActions.evalItemsJson"));
    assert!(!android_actions.contains("contains(\"\\\"ok\\\":false\")"));

    let android_generated = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/generated/CrepusGeneratedView.kt"),
    )
    .expect("read generated view");
    assert!(android_generated.contains("object CrepusActions"));
    assert!(android_generated.contains("fun perform(action: String)"));

    let android_gradle = std::fs::read_to_string(root.join("android/app/build.gradle.kts"))
        .expect("read build.gradle.kts");
    assert!(android_gradle.contains("rustJniOutputDir(profile)"));
    assert!(android_gradle.contains("rustJniLibs/$profile"));
    assert!(android_gradle.contains("it.dir(\"arm64-v8a\")"));
    assert!(
        android_gradle.contains("sourceSets[\"debug\"].jniLibs.srcDir(rustJniLibsDir(\"debug\"))")
    );
    assert!(android_gradle
        .contains("sourceSets[\"release\"].jniLibs.srcDir(rustJniLibsDir(\"release\"))"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn mobile_sync_mirrors_view_ir_fixtures() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let new_out = crepus()
        .current_dir(tmp.path())
        .args(["mobile", "new", "phone"])
        .output()
        .expect("spawn crepus mobile new");
    assert!(new_out.status.success());
    let root = tmp.path().join("phone");
    std::fs::write(
        root.join("views/main.crepus"),
        "div\n  span\n    \"Hello {name}\"",
    )
    .expect("write template");

    let out = crepus()
        .current_dir(&root)
        .args(["mobile", "sync", "--var", "name=Ada", "--pretty"])
        .output()
        .expect("spawn crepus mobile sync");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let root_fixture = std::fs::read_to_string(root.join("fixture.json")).unwrap();
    let ios_fixture =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/fixture.json")).unwrap();
    let android_fixture =
        std::fs::read_to_string(root.join("android/app/src/main/assets/fixture.json")).unwrap();
    assert_eq!(root_fixture, ios_fixture);
    assert_eq!(root_fixture, android_fixture);
    assert!(root_fixture.contains("Hello Ada"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn mobile_codegen_writes_swiftui_and_compose_source() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl = tmp.path().join("screen.crepus");
    std::fs::write(&tpl, "div\n  span\n    \"Hello {name}\"").expect("write template");
    let swift_out = tmp.path().join("swift");
    let compose_out = tmp.path().join("compose");

    let swift = crepus()
        .args([
            "mobile",
            "codegen",
            tpl.to_str().unwrap(),
            "--platform",
            "ios",
            "--out",
            swift_out.to_str().unwrap(),
            "--view-name",
            "GreetingScreen",
            "--var",
            "name=Ada",
        ])
        .output()
        .expect("spawn crepus mobile codegen ios");
    assert!(swift.status.success());

    let compose = crepus()
        .args([
            "mobile",
            "codegen",
            tpl.to_str().unwrap(),
            "--platform",
            "android",
            "--out",
            compose_out.to_str().unwrap(),
            "--view-name",
            "GreetingScreen",
            "--var",
            "name=Ada",
        ])
        .output()
        .expect("spawn crepus mobile codegen android");
    assert!(compose.status.success());

    assert!(
        std::fs::read_to_string(swift_out.join("GreetingScreen.swift"))
            .unwrap()
            .contains("Hello Ada")
    );
    assert!(
        std::fs::read_to_string(compose_out.join("GreetingScreen.kt"))
            .unwrap()
            .contains("Hello Ada")
    );
}

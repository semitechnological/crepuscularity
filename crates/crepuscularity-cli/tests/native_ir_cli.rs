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
    assert_eq!(value["version"], 5);
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
    let help = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(help.contains("mobile"));
    assert!(help.contains("new"));
    assert!(help.contains("dev"));
    assert!(help.contains("doctor"));
    assert!(help.contains("codegen"));
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
    assert!(root.join("ios/App/PhoneApp.swift").is_file());
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
    assert!(!rust_actions.contains("UIImpactFeedbackGenerator"));
    assert!(rust_actions.contains("crepusMobileLastResult"));
    assert!(!rust_actions.contains("UIDevice.current"));
    assert!(!rust_actions.contains("Bundle.main.bundleIdentifier"));
    assert!(!rust_actions.contains("UserDefaults.standard"));
    assert!(!rust_actions.contains(r#""action":"\(action)""#));
    assert!(!rust_actions.contains("result.contains"));

    let android_actions = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/phone/CrepusRustActions.kt"),
    )
    .expect("read CrepusRustActions.kt");
    assert!(android_actions.contains("object CrepusStateStore"));
    assert!(android_actions.contains("external fun storeResultJson"));
    assert!(android_actions.contains("external fun evalText"));
    assert!(android_actions.contains("external fun lastResult"));
    assert!(!android_actions.contains("documentPickerValue"));
    assert!(!android_actions.contains("getSharedPreferences(\"crepus_preferences\""));
    assert!(!android_actions.contains("Build.MANUFACTURER"));
    assert!(!android_actions.contains("packageManager.getPackageInfo"));
    assert!(!android_actions.contains("VibrationEffect.createOneShot"));
    let ios_actions =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS actions");
    assert!(!ios_actions.contains("documentPickerValue"));

    let project_yml =
        std::fs::read_to_string(root.join("ios/project.yml")).expect("read project.yml");
    assert!(project_yml.contains("$(PROJECT_DIR)/build/rust/aarch64-apple-ios/libphone_actions.a"));
    assert!(
        project_yml.contains("$(PROJECT_DIR)/build/rust/aarch64-apple-ios-sim/libphone_actions.a")
    );
    assert!(!project_yml.contains("CoreBluetooth"));
    assert!(!project_yml
        .contains("$(PROJECT_DIR)/build/rust/$(PLATFORM_NAME)/libcrepus_mobile_actions.a"));
    let manifest = std::fs::read_to_string(root.join("android/app/src/main/AndroidManifest.xml"))
        .expect("read manifest");
    assert!(!manifest.contains("BLUETOOTH"));
    assert!(!root
        .join("android/app/src/main/java/com/nonpolynomial/btleplug")
        .exists());
    assert!(!root
        .join("android/app/src/main/java/io/github/gedgygedgy")
        .exists());
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn native_add_capability_updates_only_the_scaffold() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("phone");
    assert!(crepus()
        .current_dir(tmp.path())
        .args(["native", "new", "phone"])
        .output()
        .expect("scaffold")
        .status
        .success());
    let default_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read default Android actions");
    let default_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read default iOS actions");
    assert!(!default_android.contains("hapticsValue"));
    assert!(!default_ios.contains("hapticsValue"));
    assert!(!default_android.contains("clipboardValue"));
    assert!(!default_ios.contains("clipboardValue"));
    assert!(!default_android.contains("openUrlValue"));
    assert!(!default_ios.contains("openUrlValue"));
    assert!(!default_android.contains("shareValue"));
    assert!(!default_ios.contains("shareValue"));
    assert!(!default_android.contains("imagePickerValue"));
    assert!(!default_ios.contains("imagePickerValue"));
    assert!(!default_ios.contains("import PhotosUI"));
    assert!(!default_android.contains("documentPickerValue"));
    assert!(!default_ios.contains("documentPickerValue"));
    assert!(!default_android.contains("photoLibraryValue"));
    assert!(!default_ios.contains("photoLibraryValue"));
    assert!(!default_android.contains("cameraValue"));
    assert!(!default_ios.contains("cameraValue"));
    assert!(!default_android.contains("dimensionsValue"));
    assert!(!default_ios.contains("dimensionsValue"));
    assert!(!default_android.contains("dialogValue"));
    assert!(!default_ios.contains("dialogValue"));
    assert!(!default_android.contains("actionSheetValue"));
    assert!(!default_ios.contains("actionSheetValue"));
    assert!(!default_android.contains("appStateValue"));
    assert!(!default_ios.contains("appStateValue"));
    assert!(!default_android.contains("screenOrientationValue"));
    assert!(!default_ios.contains("screenOrientationValue"));
    assert!(!default_android.contains("permissionsValue"));
    assert!(!default_ios.contains("permissionsValue"));
    assert!(!default_android.contains("contactsValue"));
    assert!(!default_ios.contains("contactsValue"));
    assert!(!default_android.contains("inAppBrowserValue"));
    assert!(!default_ios.contains("inAppBrowserValue"));
    assert!(!default_android.contains("systemBarsValue"));
    assert!(!default_ios.contains("systemBarsValue"));
    assert!(crepus()
        .args(["native", "add", "share", "--dir"])
        .arg(&root)
        .output()
        .expect("add share")
        .status
        .success());
    let share_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android share bridge");
    let share_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS share bridge");
    assert!(share_android.contains("Intent.createChooser"));
    assert!(share_ios.contains("UIActivityViewController"));
    assert!(crepus()
        .args(["native", "add", "image-picker", "--dir"])
        .arg(&root)
        .output()
        .expect("add image picker")
        .status
        .success());
    let image_picker_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android image picker bridge");
    let image_picker_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS image picker bridge");
    assert!(image_picker_android.contains("filePicker.launch(arrayOf(\"image/*\", \"video/*\"))"));
    assert!(image_picker_android.contains("\"pick_media\" ->"));
    assert!(image_picker_ios.contains("PHPickerViewController"));
    assert!(image_picker_ios.contains("case \"pick_media\":"));
    assert!(crepus()
        .args(["native", "add", "sensors", "--dir"])
        .arg(&root)
        .output()
        .expect("add sensors")
        .status
        .success());
    let cargo = std::fs::read_to_string(root.join("rust/Cargo.toml")).expect("read Cargo.toml");
    let manifest = std::fs::read_to_string(root.join("android/app/src/main/AndroidManifest.xml"))
        .expect("read manifest");
    assert!(cargo.contains("sensors = []"));
    assert!(manifest.contains("android.hardware.sensor.gyroscope"));
    let android_sensors = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android sensor bridge");
    let ios_sensors =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS sensor bridge");
    assert!(android_sensors.contains("SensorManager.SENSOR_DELAY_GAME"));
    assert!(ios_sensors.contains("CMMotionManager"));
    assert!(ios_sensors.contains("9.80665"));
    assert!(ios_sensors.contains("private static let sensors = SensorBridge()"));
    assert!(crepus()
        .args(["native", "add", "bluetooth", "--dir"])
        .arg(&root)
        .output()
        .expect("add bluetooth")
        .status
        .success());
    let cargo = std::fs::read_to_string(root.join("rust/Cargo.toml")).expect("read Cargo.toml");
    assert!(cargo.contains("bluetooth = []"));
    let bluetooth = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Bluetooth bridge");
    assert!(bluetooth.contains("BluetoothBridge"));
    assert!(bluetooth.contains("bluetooth.device"));
    let ios_bluetooth =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS Bluetooth bridge");
    assert!(ios_bluetooth.contains("CBCentralManager"));
    assert!(ios_bluetooth.contains("bluetooth.device"));
    assert!(ios_bluetooth.contains("CrepusRustActions.successJson"));
    assert!(ios_bluetooth.contains("fileprivate static func successJson"));
    assert!(
        std::fs::read_to_string(root.join("android/app/src/main/AndroidManifest.xml"))
            .expect("read manifest")
            .contains("android.permission.BLUETOOTH_SCAN")
    );
    assert!(crepus()
        .args(["native", "add", "haptics", "--dir"])
        .arg(&root)
        .output()
        .expect("add haptics")
        .status
        .success());
    let haptics_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android haptics bridge");
    let haptics_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS haptics bridge");
    assert!(haptics_android.contains("VibrationEffect.createOneShot"));
    assert!(haptics_ios.contains("UIImpactFeedbackGenerator"));
    assert!(crepus()
        .args(["native", "add", "clipboard", "--dir"])
        .arg(&root)
        .output()
        .expect("add clipboard")
        .status
        .success());
    let clipboard_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android clipboard bridge");
    let clipboard_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS clipboard bridge");
    assert!(clipboard_android.contains("ClipboardManager"));
    assert!(clipboard_ios.contains("UIPasteboard.general"));
    assert!(crepus()
        .args(["native", "add", "linking", "--dir"])
        .arg(&root)
        .output()
        .expect("add browser")
        .status
        .success());
    let browser_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android browser bridge");
    let browser_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS browser bridge");
    assert!(browser_android.contains("Intent.ACTION_VIEW"));
    assert!(browser_ios.contains("UIApplication.shared.open"));
    assert!(crepus()
        .args(["native", "add", "documents", "--dir"])
        .arg(&root)
        .output()
        .expect("add document picker")
        .status
        .success());
    let document_picker_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android document picker bridge");
    let document_picker_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS document picker bridge");
    assert!(document_picker_android.contains("\"documentPicker\" -> documentPickerValue(method)"));
    assert!(document_picker_ios.contains("case \"documentPicker\":"));
    assert!(crepus()
        .args(["native", "add", "photo-library", "--dir"])
        .arg(&root)
        .output()
        .expect("add photo library")
        .status
        .success());
    let photo_library_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android photo library bridge");
    let photo_library_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS photo library bridge");
    assert!(photo_library_android.contains("RequestMultiplePermissions"));
    assert!(photo_library_android.contains("MediaStore.Images.Media.EXTERNAL_CONTENT_URI"));
    assert!(photo_library_ios.contains("PHPhotoLibrary.requestAuthorization"));
    assert!(photo_library_ios.contains("case \"photoLibrary\":"));
    assert!(crepus()
        .args(["native", "add", "camera", "--dir"])
        .arg(&root)
        .output()
        .expect("add camera")
        .status
        .success());
    let camera_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android camera bridge");
    let camera_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS camera bridge");
    assert!(camera_android.contains("TakePicturePreview"));
    assert!(camera_ios.contains("UIImagePickerController"));
    assert!(camera_ios.contains("case \"camera\":"));
    assert!(crepus()
        .args(["native", "add", "dimensions", "--dir"])
        .arg(&root)
        .output()
        .expect("add dimensions")
        .status
        .success());
    let dimensions_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android dimensions bridge");
    let dimensions_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS dimensions bridge");
    assert!(dimensions_android.contains("currentWindowMetrics"));
    assert!(dimensions_ios.contains("UIScreen.main"));
    assert!(crepus()
        .args(["native", "add", "dialog", "--dir"])
        .arg(&root)
        .output()
        .expect("add dialog")
        .status
        .success());
    let dialog_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android dialog bridge");
    let dialog_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS dialog bridge");
    assert!(dialog_android.contains("AlertDialog.Builder"));
    assert!(dialog_ios.contains("UIAlertController"));
    assert!(crepus()
        .args(["native", "add", "action-sheet", "--dir"])
        .arg(&root)
        .output()
        .expect("add action sheet")
        .status
        .success());
    let action_sheet_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android action sheet bridge");
    let action_sheet_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS action sheet bridge");
    assert!(action_sheet_android.contains("setItems(labels)"));
    assert!(action_sheet_ios.contains("preferredStyle: .actionSheet"));
    assert!(crepus()
        .args(["native", "add", "app-state", "--dir"])
        .arg(&root)
        .output()
        .expect("add app state")
        .status
        .success());
    let app_state_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android app state bridge");
    let app_state_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS app state bridge");
    assert!(app_state_android.contains("activity.lifecycle.currentState"));
    assert!(app_state_ios.contains("UIApplication.shared.applicationState"));
    assert!(crepus()
        .args(["native", "add", "app", "--dir"])
        .arg(&root)
        .output()
        .expect("add app")
        .status
        .success());
    let app_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android app bridge");
    let app_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS app bridge");
    assert!(app_android.contains("getPackageInfo"));
    assert!(app_ios.contains("CFBundleShortVersionString"));
    assert!(crepus()
        .args(["native", "add", "screen-orientation", "--dir"])
        .arg(&root)
        .output()
        .expect("add screen orientation")
        .status
        .success());
    let orientation_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android orientation bridge");
    let orientation_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS orientation bridge");
    assert!(orientation_android.contains("ORIENTATION_LANDSCAPE"));
    assert!(orientation_ios.contains("UIScreen.main.bounds"));
    assert!(crepus()
        .args(["native", "add", "accessibility-info", "--dir"])
        .arg(&root)
        .output()
        .expect("add accessibility info")
        .status
        .success());
    let accessibility_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android accessibility bridge");
    let accessibility_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS accessibility bridge");
    assert!(accessibility_android.contains("Settings.Global.ANIMATOR_DURATION_SCALE"));
    assert!(accessibility_ios.contains("UIAccessibility.isReduceMotionEnabled"));
    assert!(accessibility_android.contains("\"accessibilityInfo\", \"screenReader\""));
    assert!(accessibility_ios.contains("case \"accessibilityInfo\", \"screenReader\":"));
    assert!(crepus()
        .args(["native", "add", "device", "--dir"])
        .arg(&root)
        .output()
        .expect("add device")
        .status
        .success());
    let device_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android device bridge");
    let device_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS device bridge");
    assert!(device_android.contains("Build.MANUFACTURER"));
    assert!(device_ios.contains("UIDevice.current"));
    assert!(device_android.contains("\"device\", \"platform\""));
    assert!(device_ios.contains("case \"device\", \"platform\":"));
    assert!(
        browser_android.contains("\"browser\", \"linking\", \"appLauncher\", \"phone\", \"sms\"")
    );
    assert!(
        browser_ios.contains("case \"browser\", \"linking\", \"appLauncher\", \"phone\", \"sms\":")
    );
    assert!(crepus()
        .args(["native", "add", "preferences", "--dir"])
        .arg(&root)
        .output()
        .expect("add preferences")
        .status
        .success());
    let preferences_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android preferences bridge");
    let preferences_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS preferences bridge");
    assert!(preferences_android.contains("getSharedPreferences"));
    assert!(preferences_ios.contains("UserDefaults.standard"));
    assert!(crepus()
        .args(["native", "add", "network", "--dir"])
        .arg(&root)
        .output()
        .expect("add network")
        .status
        .success());
    let network_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android network bridge");
    let network_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS network bridge");
    assert!(network_android.contains("NetworkCapabilities.NET_CAPABILITY_VALIDATED"));
    assert!(network_ios.contains("NWPathMonitor"));
    assert!(crepus()
        .args(["native", "add", "keyboard", "--dir"])
        .arg(&root)
        .output()
        .expect("add keyboard")
        .status
        .success());
    let keyboard_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android keyboard bridge");
    let keyboard_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS keyboard bridge");
    assert!(keyboard_android.contains("hideSoftInputFromWindow"));
    assert!(keyboard_ios.contains("resignFirstResponder"));
    assert!(crepus()
        .args(["native", "add", "settings", "--dir"])
        .arg(&root)
        .output()
        .expect("add settings")
        .status
        .success());
    let settings_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android settings bridge");
    let settings_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS settings bridge");
    assert!(settings_android.contains("ACTION_APPLICATION_DETAILS_SETTINGS"));
    assert!(settings_ios.contains("openSettingsURLString"));
    assert!(crepus()
        .args(["native", "add", "local-notifications", "--dir"])
        .arg(&root)
        .output()
        .expect("add local notifications")
        .status
        .success());
    let notifications_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android notifications bridge");
    let notifications_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS notifications bridge");
    assert!(notifications_android.contains("NotificationChannel"));
    assert!(notifications_ios.contains("UNUserNotificationCenter"));
    assert!(crepus()
        .args(["native", "add", "secure-storage", "--dir"])
        .arg(&root)
        .output()
        .expect("add secure storage")
        .status
        .success());
    let secure_storage_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android secure storage bridge");
    let secure_storage_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS secure storage bridge");
    assert!(secure_storage_android.contains("AndroidKeyStore"));
    assert!(secure_storage_ios.contains("SecItemAdd"));
    assert!(crepus()
        .args(["native", "add", "biometrics", "--dir"])
        .arg(&root)
        .output()
        .expect("add biometrics")
        .status
        .success());
    let biometrics_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android biometrics bridge");
    let biometrics_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS biometrics bridge");
    assert!(biometrics_android.contains("BiometricPrompt"));
    assert!(biometrics_ios.contains("LAContext"));
    assert!(crepus()
        .args(["native", "add", "permissions", "--dir"])
        .arg(&root)
        .output()
        .expect("add permissions")
        .status
        .success());
    let permissions_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android permissions bridge");
    let permissions_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS permissions bridge");
    assert!(permissions_android.contains("android.Manifest.permission.CAMERA"));
    assert!(permissions_ios.contains("AVCaptureDevice.authorizationStatus"));
    assert!(crepus()
        .args(["native", "add", "contacts", "--dir"])
        .arg(&root)
        .output()
        .expect("add contacts")
        .status
        .success());
    let contacts_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android contacts bridge");
    let contacts_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS contacts bridge");
    assert!(contacts_android.contains("ContactsContract.CommonDataKinds.Phone"));
    assert!(contacts_ios.contains("CNContactStore"));
    assert!(crepus()
        .args(["native", "add", "in-app-browser", "--dir"])
        .arg(&root)
        .output()
        .expect("add in-app-browser")
        .status
        .success());
    let in_app_browser_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android in-app-browser bridge");
    let in_app_browser_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS in-app-browser bridge");
    assert!(in_app_browser_android.contains("CustomTabsIntent"));
    assert!(in_app_browser_ios.contains("SFSafariViewController"));
    assert!(crepus()
        .args(["native", "add", "system-bars", "--dir"])
        .arg(&root)
        .output()
        .expect("add system-bars")
        .status
        .success());
    let system_bars_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android system-bars bridge");
    let system_bars_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS system-bars bridge");
    assert!(system_bars_android.contains("statusBarColor"));
    assert!(system_bars_ios.contains("overrideUserInterfaceStyle"));
    assert!(crepus()
        .args(["native", "add", "calendar", "--dir"])
        .arg(&root)
        .output()
        .expect("add calendar")
        .status
        .success());
    let calendar_android = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"),
    )
    .expect("read Android calendar bridge");
    let calendar_ios =
        std::fs::read_to_string(root.join("ios/Sources/NativeShell/CrepusRustActions.swift"))
            .expect("read iOS calendar bridge");
    assert!(calendar_android.contains("CalendarContract.Events.CONTENT_URI"));
    assert!(calendar_ios.contains("EKEventStore"));
    assert!(crepus()
        .args(["native", "add", "filesystem", "--dir"])
        .arg(&root)
        .output()
        .expect("add filesystem")
        .status
        .success());
    let cargo = std::fs::read_to_string(root.join("rust/Cargo.toml")).expect("read Cargo.toml");
    assert!(cargo.contains("haptics = []"));
    assert!(cargo.contains("clipboard = []"));
    assert!(cargo.contains("browser = []"));
    assert!(cargo.contains("share = []"));
    assert!(cargo.contains("image-picker = []"));
    assert!(cargo.contains("documentpicker = []"));
    assert!(cargo.contains("photo-library = []"));
    assert!(cargo.contains("camera = []"));
    assert!(cargo.contains("dimensions = []"));
    assert!(cargo.contains("dialog = []"));
    assert!(cargo.contains("action-sheet = []"));
    assert!(cargo.contains("app-state = []"));
    assert!(cargo.contains("app = []"));
    assert!(cargo.contains("screen-orientation = []"));
    assert!(cargo.contains("accessibility-info = []"));
    assert!(cargo.contains("device = []"));
    assert!(cargo.contains("preferences = []"));
    assert!(cargo.contains("network = []"));
    assert!(cargo.contains("keyboard = []"));
    assert!(cargo.contains("settings = []"));
    assert!(cargo.contains("local-notifications = []"));
    assert!(cargo.contains("secure-storage = []"));
    assert!(cargo.contains("biometrics = []"));
    assert!(cargo.contains("permissions = []"));
    assert!(cargo.contains("calendar = []"));
    assert!(cargo.contains("contacts = []"));
    assert!(haptics_android.contains("\"haptics\", \"vibration\""));
    assert!(haptics_ios.contains("case \"haptics\", \"vibration\":"));
    assert!(cargo.contains("filesystem = []"));
    assert!(cargo.contains("default = [\"filesystem\"]"));
    assert!(
        std::fs::read_to_string(root.join("android/app/src/main/AndroidManifest.xml"))
            .expect("read manifest")
            .contains("android.permission.VIBRATE")
    );
    assert!(
        std::fs::read_to_string(root.join("android/app/src/main/AndroidManifest.xml"))
            .expect("read manifest")
            .contains("android.permission.READ_MEDIA_IMAGES")
    );
    assert!(
        std::fs::read_to_string(root.join("android/app/src/main/AndroidManifest.xml"))
            .expect("read manifest")
            .contains("android.permission.CAMERA")
    );
    assert!(
        std::fs::read_to_string(root.join("android/app/src/main/AndroidManifest.xml"))
            .expect("read manifest")
            .contains("android.permission.READ_CONTACTS")
    );
    assert!(
        std::fs::read_to_string(root.join("android/app/src/main/AndroidManifest.xml"))
            .expect("read manifest")
            .contains("android.permission.READ_CALENDAR")
    );
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
        root.join("android/app/src/main/java/dev/crepuscularity/phone/CrepusRustActions.kt"),
    )
    .expect("read CrepusRustActions.kt");
    assert!(android_actions.contains("mutableLongStateOf(0L)"));
    assert!(android_actions.contains("CrepusRustActions.storeResultJson(raw)"));
    assert!(android_actions.contains("CrepusRustActions.evalItemsJson"));
    assert!(!android_actions.contains("contains(\"\\\"ok\\\":false\")"));

    let android_generated = std::fs::read_to_string(root.join(
        "android/app/src/main/java/dev/crepuscularity/phone/generated/CrepusGeneratedView.kt",
    ))
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
fn native_add_capability_finds_renamed_mobile_action_bridge() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(crepus()
        .current_dir(tmp.path())
        .args(["mobile", "new", "phone"])
        .output()
        .expect("scaffold")
        .status
        .success());
    let root = tmp.path().join("phone");
    assert!(crepus()
        .args(["native", "add", "sensors", "--dir"])
        .arg(&root)
        .output()
        .expect("add sensors")
        .status
        .success());
    let source = std::fs::read_to_string(
        root.join("android/app/src/main/java/dev/crepuscularity/phone/CrepusRustActions.kt"),
    )
    .expect("read renamed Android action bridge");
    assert!(source.contains("SensorBridge"));
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

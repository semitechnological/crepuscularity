use std::fs;
use std::path::Path;

use crepuscularity_native::{generate_native_source, to_json_pretty, NativeCodegenTarget};
use crepuscularity_tauri::TauriProject;

use crate::cli::TauriCommands;

pub fn execute(command: TauriCommands) {
    match command {
        TauriCommands::Audit { dir } => audit(&dir).unwrap_or_else(|e| crate::ui::error(&e)),
        TauriCommands::Convert { dir, out } => {
            convert(&dir, &out).unwrap_or_else(|e| crate::ui::error(&e))
        }
    }
}

fn audit(dir: &Path) -> Result<(), String> {
    let report = TauriProject::open(dir)?.audit();
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
    );
    report.native_ready()
}

fn convert(dir: &Path, out: &Path) -> Result<(), String> {
    let project = TauriProject::open(dir)?;
    project.audit().native_ready()?;
    let metadata = project.metadata();
    let bundle = project.bundle()?;
    let ir = project.native_ir()?;
    let fixture = to_json_pretty(&ir).map_err(|e| e.to_string())?;
    let swift = generate_native_source(&ir, NativeCodegenTarget::SwiftUi, "CrepusGeneratedView");
    let kotlin = format!(
        "package dev.crepuscularity.nativeshell\n\n{}",
        generate_native_source(&ir, NativeCodegenTarget::Compose, "CrepusGeneratedView")
    );
    let staging = out.with_extension(format!("crepus-tmp-{}", std::process::id()));
    if staging.exists() {
        return Err(format!(
            "staging directory '{}' already exists",
            staging.display()
        ));
    }
    let result = (|| {
        crate::native::scaffold_native_app_at(&staging)?;
        apply_metadata(&staging, &metadata)?;
        for (path, source) in &bundle.files {
            let destination = staging.join("views").join(path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(destination, source).map_err(|e| e.to_string())?;
        }
        fs::write(
            staging.join("views/main.crepus"),
            format!("include {}\n", bundle.entry),
        )
        .map_err(|e| e.to_string())?;
        fs::write(staging.join("fixture.json"), &fixture).map_err(|e| e.to_string())?;
        fs::write(
            staging.join("android/app/src/main/assets/fixture.json"),
            &fixture,
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            staging.join("ios/Sources/NativeShell/Generated/CrepusGeneratedView.swift"),
            swift,
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            staging.join(
                "android/app/src/main/java/dev/crepuscularity/nativeshell/generated/CrepusGeneratedView.kt",
            ),
            kotlin,
        )
        .map_err(|e| e.to_string())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    fs::rename(&staging, out).map_err(|e| e.to_string())?;
    println!(
        "converted Tauri {:?} project to {}",
        project.version(),
        out.display()
    );
    Ok(())
}

fn apply_metadata(
    root: &Path,
    metadata: &crepuscularity_tauri::TauriMetadata,
) -> Result<(), String> {
    let product_name = metadata
        .window_title
        .as_deref()
        .or(metadata.product_name.as_deref())
        .unwrap_or("NativeShell");
    replace(
        root.join("android/app/src/main/AndroidManifest.xml"),
        "NativeShell",
        product_name,
    )?;
    if let Some(identifier) = &metadata.identifier {
        replace(
            root.join("android/app/build.gradle.kts"),
            "applicationId = \"dev.crepuscularity.nativeshell\"",
            &format!("applicationId = \"{identifier}\""),
        )?;
        replace(
            root.join("ios/project.yml"),
            "PRODUCT_BUNDLE_IDENTIFIER: dev.crepuscularity.mobile",
            &format!("PRODUCT_BUNDLE_IDENTIFIER: {identifier}"),
        )?;
    }
    if let Some(version) = &metadata.version {
        replace(
            root.join("android/app/build.gradle.kts"),
            "versionName = \"1.0\"",
            &format!("versionName = \"{version}\""),
        )?;
    }
    Ok(())
}

fn replace(path: impl AsRef<Path>, from: &str, to: &str) -> Result<(), String> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    fs::write(path, source.replace(from, to)).map_err(|e| e.to_string())
}

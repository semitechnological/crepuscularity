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

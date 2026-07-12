use std::fs;
use std::path::{Component, Path};

use crepuscularity_native::{generate_native_source, to_json_pretty, NativeCodegenTarget};
use crepuscularity_tauri::TauriProject;

use crate::cli::TauriCommands;

pub fn execute(command: TauriCommands) {
    match command {
        TauriCommands::Convert { dir, out } => {
            convert(&dir, &out).unwrap_or_else(|e| crate::ui::error(&e))
        }
    }
}

fn convert(dir: &Path, out: &Path) -> Result<(), String> {
    let project = TauriProject::open(dir)?;
    let bundle = project.bundle()?;
    let ir = project.native_ir()?;
    crate::native::scaffold_native_app_at(out)?;
    for (path, source) in &bundle.files {
        let relative = Path::new(path);
        if relative
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::RootDir))
        {
            return Err(format!("bundle contains unsafe path {path:?}"));
        }
        let destination = out.join("views").join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(destination, source).map_err(|e| e.to_string())?;
    }
    let entry = bundle
        .files
        .get(&bundle.entry)
        .ok_or_else(|| "bundle entry is missing".to_string())?;
    fs::write(out.join("views/main.crepus"), entry).map_err(|e| e.to_string())?;
    let fixture = to_json_pretty(&ir).map_err(|e| e.to_string())?;
    fs::write(out.join("fixture.json"), &fixture).map_err(|e| e.to_string())?;
    fs::write(
        out.join("android/app/src/main/assets/fixture.json"),
        &fixture,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        out.join("ios/Sources/NativeShell/Generated/CrepusGeneratedView.swift"),
        generate_native_source(&ir, NativeCodegenTarget::SwiftUi, "CrepusGeneratedView"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        out.join(
            "android/app/src/main/java/dev/crepuscularity/nativeshell/generated/CrepusGeneratedView.kt",
        ),
        format!(
            "package dev.crepuscularity.nativeshell\n\n{}",
            generate_native_source(&ir, NativeCodegenTarget::Compose, "CrepusGeneratedView")
        ),
    )
    .map_err(|e| e.to_string())?;
    println!(
        "converted Tauri {:?} project to {}",
        project.version(),
        out.display()
    );
    Ok(())
}

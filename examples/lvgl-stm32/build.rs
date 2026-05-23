use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let manifest = manifest_dir.join("crepus.toml");
    let artifacts =
        crepuscularity::target::build_manifest_file_target(&manifest, Some("stm32-dashboard"))
            .expect("build crepus target");
    let xml = artifacts
        .first()
        .map(|artifact| artifact.contents.as_str())
        .expect("stm32-dashboard artifact");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("out dir"));
    let out_file = out_dir.join("stm32_dashboard.xml");
    std::fs::write(&out_file, xml).expect("write LVGL XML");

    println!("cargo:rerun-if-changed={}", manifest.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("ui.crepus").display()
    );
}

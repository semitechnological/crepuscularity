fn main() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("crepus.toml");
    let artifacts =
        crepuscularity::target::build_manifest_file_target(&manifest, Some("dashboard"))
            .expect("example target should render to LVGL XML");
    print!("{}", artifacts[0].contents);
}

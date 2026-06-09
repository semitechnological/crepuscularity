use std::process::Command;

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

#[test]
#[cfg_attr(
    windows,
    ignore = "default desktop crepus.exe does not spawn reliably on Windows CI"
)]
fn embedded_snapshot_writes_ppm_header() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl = tmp.path().join("screen.crepus");
    let out = tmp.path().join("screen.ppm");
    std::fs::write(
        &tpl,
        "motion w-full h-full bg-zinc-950\n  div text-white\n    \"Hi\"",
    )
    .expect("write template");

    let status = crepus()
        .args([
            "embedded",
            "snapshot",
            tpl.to_str().unwrap(),
            "--width",
            "64",
            "--height",
            "48",
            "--out",
            out.to_str().unwrap(),
        ])
        .status()
        .expect("spawn crepus embedded snapshot");
    assert!(status.success(), "embedded snapshot failed");

    let bytes = std::fs::read(&out).expect("read ppm");
    let header_len = "P6\n64 48\n255\n".len();
    assert!(bytes.len() > header_len);
    let header = std::str::from_utf8(&bytes[..header_len]).expect("ppm ascii header");
    assert_eq!(header, "P6\n64 48\n255\n");
}

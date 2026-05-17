use std::process::Command;

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

#[test]
#[cfg_attr(
    windows,
    ignore = "crepus embedded subcommand does not run reliably on Windows CI (see embedded_cli.rs)"
)]
fn embedded_check_valid_template() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let tpl = tmp.path().join("ok.crepus");
    std::fs::write(&tpl, "motion w-full h-full\n  \"ok\"").expect("write");

    let status = crepus()
        .args(["embedded", "check", tpl.to_str().unwrap()])
        .status()
        .expect("spawn");
    assert!(status.success());
}

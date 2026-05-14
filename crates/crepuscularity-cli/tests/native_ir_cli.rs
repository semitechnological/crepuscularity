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

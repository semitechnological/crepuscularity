//! Integration tests for `crepus web`.

use std::process::Command;

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

#[test]
fn web_new_then_build_emits_html() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let status = crepus()
        .current_dir(tmp.path())
        .args(["web", "new", "Acme Labs"])
        .status()
        .expect("spawn crepus web new");
    assert!(status.success());

    let site_dir = tmp.path().join("acme-labs");
    assert!(site_dir.join("site.json").is_file());
    let site_json = std::fs::read_to_string(site_dir.join("site.json")).expect("read site.json");
    assert!(site_json.contains("Acme Labs"));

    let out = site_dir.join("dist").join("index.html");
    let status = crepus()
        .current_dir(tmp.path())
        .args([
            "web",
            "build",
            "--site",
            "acme-labs",
            "-o",
            out.to_str().unwrap(),
        ])
        .status()
        .expect("spawn crepus web build");
    assert!(status.success());

    let html = std::fs::read_to_string(&out).expect("read html");
    assert!(html.contains("<html"));
    assert!(html.contains("Acme Labs"));
}

#[test]
fn web_build_from_stdin_dash() {
    // Minimal SiteBuilder JSON (matches `starter_site_payload` shape) without a dev-dependency.
    // Use `r##"..."##` so hex colors like `"#fafafa"` do not terminate the raw string.
    let json = r##"{
  "businessName": "Stdin Co",
  "domain": null,
  "seo": {
    "title": "Stdin Co",
    "description": "test",
    "ogImage": null
  },
  "theme": {
    "accent": "#3b82f6",
    "accentSoft": "#60a5fa",
    "surface": "#09090b",
    "text": "#fafafa",
    "muted": "#a1a1aa",
    "border": "#27272a"
  },
  "elements": [
    {
      "type": "hero",
      "id": "hero-1",
      "props": {
        "eyebrow": "Welcome",
        "headline": "Meet Stdin Co",
        "subheadline": "Ship a polished landing page from structured data.",
        "primary": { "label": "Get started", "href": "#contact", "external": null },
        "secondary": null,
        "media": null
      }
    }
  ],
  "analytics": null,
  "turnstile": null
}"##
    .to_string();

    let out = tempfile::tempdir().expect("tempdir");
    let html_path = out.path().join("p.html");

    let mut child = crepus()
        .current_dir(out.path())
        .args(["web", "build", "-", "-o", html_path.to_str().unwrap()])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();
    let status = child.wait().expect("wait");
    assert!(status.success());
    let html = std::fs::read_to_string(&html_path).expect("html");
    assert!(html.contains("Stdin Co"));
}

#[test]
fn web_new_fails_when_dir_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let _ = std::fs::create_dir_all(tmp.path().join("dup-site"));
    let status = crepus()
        .current_dir(tmp.path())
        .args(["web", "new", "dup-site"])
        .status()
        .expect("spawn");
    assert!(!status.success());
}

#[test]
fn root_help_lists_web_commands() {
    let out = crepus().args(["--help"]).output().expect("help");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(
        text.contains("web new") && text.contains("web build"),
        "usage should list web commands:\n{text}"
    );
}

#[test]
fn web_build_missing_json_file_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ghost = tmp.path().join("nope.json");
    let status = crepus()
        .current_dir(tmp.path())
        .args(["web", "build", "--json"])
        .arg(&ghost)
        .status()
        .expect("spawn");
    assert!(!status.success());
}

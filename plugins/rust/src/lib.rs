use std::process::{Command, Stdio};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ViewIr {
    pub version: u32,
    pub root: Vec<ViewNode>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ViewNode {
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "stack")]
    Stack {
        axis: String,
        children: Vec<ViewNode>,
    },
    #[serde(rename = "scroll")]
    Scroll {
        axis: String,
        children: Vec<ViewNode>,
    },
    #[serde(rename = "button")]
    Button { label: String },
    #[serde(other)]
    Other,
}

pub fn render_ir(path: &str) -> Result<ViewIr, Box<dyn std::error::Error>> {
    let bin = std::env::var("CREPUS_BIN").unwrap_or_else(|_| "crepus".to_string());
    let out = Command::new(bin)
        .args(["native", "ir", path])
        .stdout(Stdio::piped())
        .output()?;
    if !out.status.success() {
        return Err(format!(
            "crepus native ir failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}

pub fn render_html(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(render_ir(path)?.root.iter().map(render_node).collect())
}

fn render_node(node: &ViewNode) -> String {
    match node {
        ViewNode::Text { content } => escape_html(content),
        ViewNode::Stack { axis, children } | ViewNode::Scroll { axis, children } => {
            format!(
                "<div data-crepus-kind=\"stack\" data-axis=\"{}\">{}</div>",
                escape_html(axis),
                children.iter().map(render_node).collect::<String>()
            )
        }
        ViewNode::Button { label } => format!("<button>{}</button>", escape_html(label)),
        ViewNode::Other => String::new(),
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_html() {
        let html = crate::render_html("../fixtures/hello.crepus").expect("render html");
        assert!(html.contains("data-crepus-kind=\"stack\""));
    }
}

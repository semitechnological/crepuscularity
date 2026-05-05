use ratatui::{backend::TestBackend, Terminal};
use std::fs;
use std::path::PathBuf;

use crate::{render_component, render_template, TemplateContext, TemplateValue};

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Render a template into a TestBackend of the given dimensions and return the
/// full buffer as a Vec of row strings (one per terminal row).
fn render(width: u16, height: u16, template: &str, ctx: &TemplateContext) -> Vec<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            let area = frame.area();
            render_template(template, ctx, frame, area).expect("render_template returned an error");
        })
        .unwrap();

    let buf = terminal.backend().buffer();
    let w = buf.area.width as usize;
    let h = buf.area.height as usize;
    (0..h)
        .map(|y| {
            buf.content[y * w..(y + 1) * w]
                .iter()
                .map(|c| c.symbol())
                .collect::<String>()
        })
        .collect()
}

/// Collect all rows into one string (newline-separated) for easy `contains` checks.
fn all_text(rows: &[String]) -> String {
    rows.join("\n")
}

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

// ─── Basic rendering ──────────────────────────────────────────────────────────

#[test]
fn plain_text_renders() {
    let mut ctx = TemplateContext::new();
    ctx.set("name", "terminal");
    let rows = render(
        40,
        5,
        "div w-full h-full flex-col\n  div\n    \"Hello {name}\"",
        &ctx,
    );
    assert!(
        all_text(&rows).contains("Hello terminal"),
        "expected 'Hello terminal' in output:\n{}",
        all_text(&rows)
    );
}

#[test]
fn raw_expression_renders() {
    let mut ctx = TemplateContext::new();
    ctx.set("score", 42i64);
    let rows = render(30, 3, "div\n  {score}", &ctx);
    assert!(
        all_text(&rows).contains("42"),
        "expected '42' in:\n{}",
        all_text(&rows)
    );
}

// ─── JSX / React-style syntax ─────────────────────────────────────────────────
//
// The parser auto-detects JSX when the first content token starts with `<`.
// All the same layout and styling classes work — just angle-bracket style.

#[test]
fn jsx_syntax_renders() {
    let mut ctx = TemplateContext::new();
    ctx.set("user", "ada");
    // Same template as indentation syntax but written in JSX/HTML style.
    let tpl = r#"<div class="w-full h-full flex-col">
  <div class="h-[1]">Hello {user}</div>
  <div class="flex-1">Content area</div>
</div>"#;
    let rows = render(40, 5, tpl, &ctx);
    assert!(
        all_text(&rows).contains("Hello ada"),
        "JSX: expected 'Hello ada' in:\n{}",
        all_text(&rows)
    );
    assert!(
        all_text(&rows).contains("Content area"),
        "JSX: expected 'Content area' in:\n{}",
        all_text(&rows)
    );
}

#[test]
fn jsx_and_indent_are_equivalent() {
    let ctx = TemplateContext::new();

    let indent = "div flex-col\n  div\n    \"Row A\"\n  div\n    \"Row B\"";
    let jsx = "<div class=\"flex-col\"><div>Row A</div><div>Row B</div></div>";

    let indent_rows = render(30, 5, indent, &ctx);
    let jsx_rows = render(30, 5, jsx, &ctx);

    // Both should contain the same text regardless of syntax.
    assert!(
        all_text(&indent_rows).contains("Row A"),
        "indent: {}",
        all_text(&indent_rows)
    );
    assert!(
        all_text(&jsx_rows).contains("Row A"),
        "jsx: {}",
        all_text(&jsx_rows)
    );
    assert!(
        all_text(&indent_rows).contains("Row B"),
        "indent: {}",
        all_text(&indent_rows)
    );
    assert!(
        all_text(&jsx_rows).contains("Row B"),
        "jsx: {}",
        all_text(&jsx_rows)
    );
}

#[test]
fn jsx_class_name_alias() {
    // React uses `className` instead of `class`; the parser accepts both.
    let ctx = TemplateContext::new();
    let tpl = r#"<div className="flex-col"><div>className works</div></div>"#;
    let rows = render(30, 3, tpl, &ctx);
    assert!(
        all_text(&rows).contains("className works"),
        "{}",
        all_text(&rows)
    );
}

// ─── Layout ───────────────────────────────────────────────────────────────────

#[test]
fn flex_col_stacks_vertically() {
    // Each child gets its own row.
    let ctx = TemplateContext::new();
    let tpl = "div flex-col\n  div h-[1]\n    \"Top\"\n  div h-[1]\n    \"Bottom\"";
    let rows = render(20, 4, tpl, &ctx);
    // "Top" should appear before "Bottom" in the row list.
    let top_row = rows.iter().position(|r| r.contains("Top")).unwrap();
    let bot_row = rows.iter().position(|r| r.contains("Bottom")).unwrap();
    assert!(
        top_row < bot_row,
        "flex-col: Top should be above Bottom. rows:\n{}",
        all_text(&rows)
    );
}

#[test]
fn flex_row_stacks_horizontally() {
    let ctx = TemplateContext::new();
    let tpl = "div flex-row\n  div w-[8]\n    \"Left\"\n  div flex-1\n    \"Right\"";
    let rows = render(30, 3, tpl, &ctx);
    // "Left" and "Right" should appear on the same row.
    let row_with_left = rows.iter().find(|r| r.contains("Left")).unwrap();
    assert!(
        row_with_left.contains("Right"),
        "flex-row: Left and Right should be on the same row. rows:\n{}",
        all_text(&rows)
    );
}

#[test]
fn fixed_height_constraint() {
    let ctx = TemplateContext::new();
    // Header takes exactly 2 rows, body gets the rest.
    let tpl = "div w-full h-full flex-col\n  div h-[2]\n    \"Header\"\n  div flex-1\n    \"Body\"";
    let rows = render(20, 6, tpl, &ctx);
    assert!(all_text(&rows).contains("Header"), "{}", all_text(&rows));
    assert!(all_text(&rows).contains("Body"), "{}", all_text(&rows));
}

#[test]
fn full_screen_tui_layout() {
    // Classic TUI: header / sidebar + main / status-bar.
    let ctx = TemplateContext::new();
    let tpl = r#"div w-full h-full flex-col
  div h-[1]
    "Header"
  div flex-1 flex-row
    div w-[10]
      "Sidebar"
    div flex-1
      "Main"
  div h-[1]
    "Status""#;
    let rows = render(40, 8, tpl, &ctx);
    assert!(all_text(&rows).contains("Header"), "{}", all_text(&rows));
    assert!(all_text(&rows).contains("Sidebar"), "{}", all_text(&rows));
    assert!(all_text(&rows).contains("Main"), "{}", all_text(&rows));
    assert!(all_text(&rows).contains("Status"), "{}", all_text(&rows));
}

// ─── Control flow ─────────────────────────────────────────────────────────────

#[test]
fn if_true_branch_shown() {
    let mut ctx = TemplateContext::new();
    ctx.set("ok", true);
    let tpl = "div\n if {ok}\n  \"Yes\"\n else\n  \"No\"";
    let rows = render(20, 3, tpl, &ctx);
    assert!(all_text(&rows).contains("Yes"), "{}", all_text(&rows));
    assert!(!all_text(&rows).contains("No"), "{}", all_text(&rows));
}

#[test]
fn if_false_branch_shown() {
    let mut ctx = TemplateContext::new();
    ctx.set("ok", false);
    let tpl = "div\n if {ok}\n  \"Yes\"\n else\n  \"No\"";
    let rows = render(20, 3, tpl, &ctx);
    assert!(!all_text(&rows).contains("Yes"), "{}", all_text(&rows));
    assert!(all_text(&rows).contains("No"), "{}", all_text(&rows));
}

#[test]
fn if_no_else_empty_on_false() {
    let mut ctx = TemplateContext::new();
    ctx.set("flag", false);
    let tpl = "div\n if {flag}\n  \"Visible\"";
    let rows = render(20, 3, tpl, &ctx);
    assert!(!all_text(&rows).contains("Visible"), "{}", all_text(&rows));
}

#[test]
fn for_loop_renders_items() {
    let mut ctx = TemplateContext::new();
    let items: Vec<TemplateContext> = ["Alpha", "Beta", "Gamma"]
        .iter()
        .map(|s| {
            let mut c = TemplateContext::new();
            c.set("value", *s);
            c
        })
        .collect();
    ctx.set("items", TemplateValue::List(items));

    let tpl = "div flex-col\n for item in {items}\n  div\n    \"{item}\"";
    let rows = render(20, 6, tpl, &ctx);
    assert!(all_text(&rows).contains("Alpha"), "{}", all_text(&rows));
    assert!(all_text(&rows).contains("Beta"), "{}", all_text(&rows));
    assert!(all_text(&rows).contains("Gamma"), "{}", all_text(&rows));
}

#[test]
fn match_correct_arm() {
    let mut ctx = TemplateContext::new();
    ctx.set("status", "active");
    // Arms are at the SAME indent as `match`, bodies one level deeper.
    let tpl = "div\n match {status}\n \"active\" =>\n  \"Online\"\n _ =>\n  \"Offline\"";
    let rows = render(20, 3, tpl, &ctx);
    assert!(all_text(&rows).contains("Online"), "{}", all_text(&rows));
    assert!(!all_text(&rows).contains("Offline"), "{}", all_text(&rows));
}

#[test]
fn match_wildcard_arm() {
    let mut ctx = TemplateContext::new();
    ctx.set("status", "unknown");
    let tpl = "div\n match {status}\n \"active\" =>\n  \"Online\"\n _ =>\n  \"Offline\"";
    let rows = render(20, 3, tpl, &ctx);
    assert!(all_text(&rows).contains("Offline"), "{}", all_text(&rows));
}

// ─── Let declarations ─────────────────────────────────────────────────────────

#[test]
fn let_decl_sets_variable() {
    let ctx = TemplateContext::new();
    let tpl = "div\n $: let total = {10 + 5}\n div\n  \"{total}\"";
    let rows = render(20, 3, tpl, &ctx);
    assert!(all_text(&rows).contains("15"), "{}", all_text(&rows));
}

// ─── Styling (smoke tests — verify no panics) ─────────────────────────────────

#[test]
fn colour_classes_dont_panic() {
    let ctx = TemplateContext::new();
    let tpl = "div bg-zinc-950 text-white flex-col\n  div text-green-400\n    \"Green\"\n  div text-red-500\n    \"Red\"\n  div bg-[#1e1e2e] text-[#cdd6f4]\n    \"Catppuccin\"";
    let rows = render(30, 5, tpl, &ctx);
    assert!(all_text(&rows).contains("Green"), "{}", all_text(&rows));
    assert!(
        all_text(&rows).contains("Catppuccin"),
        "{}",
        all_text(&rows)
    );
}

#[test]
fn border_classes_dont_panic() {
    let ctx = TemplateContext::new();
    // 6 rows: 1 top border + 1 top padding + 1 content + 1 bottom padding + 1 bottom border = 5
    let tpl = "div border rounded p-1\n  \"Boxed\"";
    let rows = render(20, 6, tpl, &ctx);
    assert!(all_text(&rows).contains("Boxed"), "{}", all_text(&rows));
}

#[test]
fn text_modifiers_dont_panic() {
    let ctx = TemplateContext::new();
    let tpl = "div font-bold italic underline\n  \"Styled\"";
    let rows = render(20, 3, tpl, &ctx);
    assert!(all_text(&rows).contains("Styled"), "{}", all_text(&rows));
}

#[test]
fn gap_spacing_dont_panic() {
    let ctx = TemplateContext::new();
    let tpl = "div flex-col gap-1\n  div h-[1]\n    \"A\"\n  div h-[1]\n    \"B\"";
    let rows = render(20, 5, tpl, &ctx);
    assert!(all_text(&rows).contains("A"), "{}", all_text(&rows));
    assert!(all_text(&rows).contains("B"), "{}", all_text(&rows));
}

// ─── Multi-component files ────────────────────────────────────────────────────

#[test]
fn multi_component_render() {
    let ctx = TemplateContext::new();
    let content = "--- Card\ndiv border rounded p-1\n  slot\n    \"fallback\"";
    let rows = {
        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_component(content, "Card", &ctx, frame, area).unwrap();
            })
            .unwrap();
        let buf = terminal.backend().buffer();
        let w = buf.area.width as usize;
        let h = buf.area.height as usize;
        (0..h)
            .map(|y| {
                buf.content[y * w..(y + 1) * w]
                    .iter()
                    .map(|c| c.symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
    };
    assert!(all_text(&rows).contains("fallback"), "{}", all_text(&rows));
}

#[test]
fn include_uses_isolated_context() {
    let mut ctx = TemplateContext::new();
    ctx.base_dir = Some(PathBuf::from("/virtual"));
    ctx.set("secret", "leak");
    ctx.virtual_files
        .insert("child.crepus".into(), "div\n  \"{secret}\"".into());

    let tpl = "div\n include child.crepus";
    let rows = render(30, 4, tpl, &ctx);
    assert!(!all_text(&rows).contains("leak"), "{}", all_text(&rows));
}

#[test]
fn include_reads_virtual_file_by_suffix_match() {
    let mut ctx = TemplateContext::new();
    ctx.base_dir = Some(PathBuf::from("/virtual"));
    ctx.virtual_files
        .insert("child.crepus".into(), "div\n  \"From virtual\"".into());

    let tpl = "div\n include child.crepus";
    let rows = render(30, 4, tpl, &ctx);
    assert!(
        all_text(&rows).contains("From virtual"),
        "{}",
        all_text(&rows)
    );
}

#[test]
fn include_rejects_parent_dir_escape() {
    let mut ctx = TemplateContext::new();
    ctx.base_dir = Some(PathBuf::from("/virtual"));
    ctx.virtual_files
        .insert("../secret.crepus".into(), "div\n  \"secret\"".into());

    let tpl = "div\n include ../secret.crepus";
    let rows = render(40, 4, tpl, &ctx);
    let text = all_text(&rows);
    assert!(text.contains("include path outside base dir"), "{text}");
    assert!(!text.contains("secret"), "{text}");
}

#[test]
fn include_rejects_absolute_path() {
    let mut ctx = TemplateContext::new();
    ctx.base_dir = Some(PathBuf::from("/virtual"));
    ctx.virtual_files
        .insert("/tmp/secret.crepus".into(), "div\n  \"secret\"".into());

    let tpl = "div\n include /tmp/secret.crepus";
    let rows = render(40, 4, tpl, &ctx);
    let text = all_text(&rows);
    assert!(text.contains("include path outside base dir"), "{text}");
    assert!(!text.contains("secret"), "{text}");
}

#[test]
fn demo_example_renders_without_error() {
    let template = fs::read_to_string(examples_dir().join("demo.crepus")).unwrap();

    let mut ctx = TemplateContext::new();
    ctx.set("title", "Demo");
    ctx.set("show_badge", true);
    ctx.set("score", 91i64);
    ctx.set("status", "active");
    let items: Vec<TemplateContext> = ["One", "Two"]
        .iter()
        .map(|s| {
            let mut c = TemplateContext::new();
            c.set("value", *s);
            c
        })
        .collect();
    ctx.set("items", TemplateValue::List(items));

    let rows = render(120, 40, &template, &ctx);
    let text = all_text(&rows);
    assert!(text.contains("Demo"), "{}", text);
    assert!(!text.contains("⚠"), "{}", text);
}

#[test]
fn jsx_demo_example_renders_include() {
    let template = fs::read_to_string(examples_dir().join("jsx-demo.crepus")).unwrap();

    let mut ctx = TemplateContext::new();
    ctx.base_dir = Some(examples_dir());
    ctx.set("show_header", true);
    ctx.set("name", "Ada");
    ctx.set("score", 88i64);
    ctx.set("status", "active");
    let items: Vec<TemplateContext> = ["Alpha", "Beta"]
        .iter()
        .map(|s| {
            let mut c = TemplateContext::new();
            c.set("value", *s);
            c
        })
        .collect();
    ctx.set("items", TemplateValue::List(items));

    let rows = render(120, 40, &template, &ctx);
    let text = all_text(&rows);
    assert!(!text.contains("⚠"), "{}", text);
}

#[test]
fn components_demo_renders_slot_content() {
    let template = fs::read_to_string(examples_dir().join("components-demo.crepus")).unwrap();

    let mut ctx = TemplateContext::new();
    ctx.base_dir = Some(examples_dir());
    let rows = render(100, 25, &template, &ctx);
    let text = all_text(&rows);
    assert!(!text.contains("⚠"), "{}", text);
}

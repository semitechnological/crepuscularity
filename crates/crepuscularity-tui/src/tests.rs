use ratatui::{backend::TestBackend, style::Color, Terminal};
use std::fs;
use std::path::PathBuf;

use crate::{
    draw as draw_template, render_component, render_template, template, HotTemplate, ReloadOutcome,
    TemplateContext, TemplateValue,
};

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

fn buffer_rows(terminal: &Terminal<TestBackend>) -> Vec<String> {
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

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn temp_case(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "crepuscularity-tui-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::create_dir_all(&dir);
    dir
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
fn file_template_builder_renders() {
    let dir = temp_case("builder");
    let path = dir.join("ui.crepus");
    fs::write(
        &path,
        "div w-full h-full flex-col\n  div h-[1]\n    \"{title}\"\n  div h-[1]\n    \"{input}\"",
    )
    .unwrap();

    let mut ui = template(&path).unwrap();
    ui.set("title", "My App");
    ui.set("input", "input contents");

    let backend = TestBackend::new(40, 4);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| ui.draw_full(frame).expect("template draws"))
        .unwrap();

    let text = all_text(&buffer_rows(&terminal));
    assert!(text.contains("My App"), "{text}");
    assert!(text.contains("input contents"), "{text}");
    assert_eq!(ui.context().base_dir.as_deref(), Some(dir.as_path()));
}

#[test]
fn draw_helper_owns_terminal_draw_pass() {
    let dir = temp_case("draw-helper");
    let path = dir.join("ui.crepus");
    fs::write(&path, "div\n  \"{title}\"").unwrap();

    let backend = TestBackend::new(40, 4);
    let mut terminal = Terminal::new(backend).unwrap();
    draw_template(&mut terminal, &path, |ui| {
        ui.set("title", "Draw Helper");
    })
    .unwrap();

    let text = all_text(&buffer_rows(&terminal));
    assert!(text.contains("Draw Helper"), "{text}");
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

#[test]
fn renderer_target_defaults_are_available_to_templates() {
    let ctx = TemplateContext::new();
    let rows = render(
        40,
        5,
        "div\n  if {is_tui && crepus_target == \"tui\"}\n    div tui-panel\n      \"TUI target\"",
        &ctx,
    );
    assert!(
        all_text(&rows).contains("TUI target"),
        "{}",
        all_text(&rows)
    );
}

#[test]
fn renderer_target_defaults_do_not_override_context() {
    let mut ctx = TemplateContext::new();
    ctx.set("crepus_target", "gui");
    ctx.set("is_tui", false);
    ctx.set("is_gui", true);
    let rows = render(
        40,
        5,
        "div\n  if {is_gui && crepus_target == \"gui\"}\n    div\n      \"GUI target\"",
        &ctx,
    );
    assert!(
        all_text(&rows).contains("GUI target"),
        "{}",
        all_text(&rows)
    );
}

#[test]
fn target_prefixed_classes_apply_only_for_matching_renderer() {
    let ctx = TemplateContext::new();
    let rows = render(
        40,
        5,
        "div tui:background-black gpui:background-white web:background-red macos:background-grey\n  \"target styles\"",
        &ctx,
    );
    assert!(
        all_text(&rows).contains("target styles"),
        "{}",
        all_text(&rows)
    );
}

#[test]
fn button_tag_gets_terminal_button_chrome() {
    let ctx = TemplateContext::new();
    let rows = render(24, 3, "button w-[12] h-[3]\n  \"Run\"", &ctx);
    let text = all_text(&rows);
    assert!(text.contains("Run"), "{text}");
    assert!(text.contains("╭") || text.contains("┌"), "{text}");
}

#[test]
fn overflow_y_scroll_uses_scroll_offset() {
    let mut ctx = TemplateContext::new();
    ctx.set("offset", 2i64);
    let rows = render(
        20,
        3,
        "div h-[3] overflow-y-scroll scroll-offset={offset}\n  div h-[1]\n    \"zero\"\n  div h-[1]\n    \"one\"\n  div h-[1]\n    \"two\"\n  div h-[1]\n    \"three\"",
        &ctx,
    );
    let text = all_text(&rows);
    assert!(!text.contains("zero"), "{text}");
    assert!(!text.contains("one"), "{text}");
    assert!(text.contains("two"), "{text}");
    assert!(text.contains("three"), "{text}");
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
fn fixed_height_with_one_sided_border_keeps_content_visible() {
    let ctx = TemplateContext::new();
    let tpl = "div w-full h-full flex-col\n  div h-[1] border-b\n    \"Header\"\n  div flex-1\n    \"Body\"";
    let rows = render(30, 5, tpl, &ctx);
    assert!(all_text(&rows).contains("Header"), "{}", all_text(&rows));
    assert!(all_text(&rows).contains("Body"), "{}", all_text(&rows));
}

#[test]
fn child_text_inherits_parent_style() {
    let backend = TestBackend::new(30, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let ctx = TemplateContext::new();
    terminal
        .draw(|frame| {
            let area = frame.area();
            render_template(
                "div text-[#cdd6f4]\n  div\n    \"Inherited\"",
                &ctx,
                frame,
                area,
            )
            .expect("render_template returned an error");
        })
        .unwrap();

    let buf = terminal.backend().buffer();
    let cell = buf
        .content
        .iter()
        .find(|c| c.symbol() == "I")
        .expect("expected inherited text to render");
    assert_eq!(cell.fg, Color::Rgb(205, 214, 244));
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

// ─── Hot reload ───────────────────────────────────────────────────────────────

fn render_hot(width: u16, height: u16, hot: &mut HotTemplate) -> Vec<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            let _ = hot.poll_and_draw_full(frame);
        })
        .unwrap();
    buffer_rows(&terminal)
}

#[test]
fn template_reload_picks_up_new_source() {
    let dir = temp_case("reload");
    let path = dir.join("ui.crepus");
    fs::write(&path, "div\n  \"first version\"").unwrap();

    let mut tpl = template(&path).unwrap();
    let backend = TestBackend::new(40, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| tpl.draw_full(frame).unwrap())
        .unwrap();
    assert!(
        all_text(&buffer_rows(&terminal)).contains("first version"),
        "{}",
        all_text(&buffer_rows(&terminal))
    );

    fs::write(&path, "div\n  \"second version\"").unwrap();
    tpl.reload().expect("reload succeeds after rewrite");

    terminal
        .draw(|frame| tpl.draw_full(frame).unwrap())
        .unwrap();
    let text = all_text(&buffer_rows(&terminal));
    assert!(text.contains("second version"), "{text}");
    assert!(!text.contains("first version"), "{text}");
}

#[test]
fn template_reload_without_path_fails() {
    let mut tpl = crate::Template::from_source("div\n  \"x\"");
    let err = tpl.reload().expect_err("from_source has no path");
    assert!(err.contains("no path"), "{err}");
}

#[test]
fn hot_template_initial_render_uses_disk_source() {
    let dir = temp_case("hot-initial");
    let path = dir.join("ui.crepus");
    fs::write(&path, "div\n  \"original\"").unwrap();

    let mut hot = HotTemplate::watch(&path).unwrap();
    let rows = render_hot(40, 3, &mut hot);
    assert!(all_text(&rows).contains("original"), "{}", all_text(&rows));
    assert_eq!(hot.template().context().base_dir.as_deref(), Some(&*dir));
}

#[test]
fn hot_template_poll_unchanged_when_flag_clear() {
    let dir = temp_case("hot-noop");
    let path = dir.join("ui.crepus");
    fs::write(&path, "div\n  \"x\"").unwrap();

    let mut hot = HotTemplate::watch(&path).unwrap();
    assert_eq!(hot.poll_reload(), ReloadOutcome::Unchanged);
}

#[test]
fn hot_template_poll_reloads_when_flag_set() {
    let dir = temp_case("hot-flag-set");
    let path = dir.join("ui.crepus");
    fs::write(&path, "div\n  \"before\"").unwrap();

    let mut hot = HotTemplate::watch(&path).unwrap();
    let rows = render_hot(40, 3, &mut hot);
    assert!(all_text(&rows).contains("before"), "{}", all_text(&rows));

    fs::write(&path, "div\n  \"after\"").unwrap();
    *hot.changed_handle().lock().unwrap() = true;

    assert_eq!(hot.poll_reload(), ReloadOutcome::Reloaded);

    let rows = render_hot(40, 3, &mut hot);
    let text = all_text(&rows);
    assert!(text.contains("after"), "{text}");
    assert!(!text.contains("before"), "{text}");
}

#[test]
fn hot_template_preserves_context_across_reload() {
    let dir = temp_case("hot-ctx");
    let path = dir.join("ui.crepus");
    fs::write(&path, "div\n  \"v1: {label}\"").unwrap();

    let mut hot = HotTemplate::watch(&path).unwrap();
    hot.template_mut().set("label", "kept");

    let rows = render_hot(40, 3, &mut hot);
    assert!(all_text(&rows).contains("v1: kept"), "{}", all_text(&rows));

    fs::write(&path, "div\n  \"v2: {label}\"").unwrap();
    *hot.changed_handle().lock().unwrap() = true;
    assert_eq!(hot.poll_reload(), ReloadOutcome::Reloaded);

    let rows = render_hot(40, 3, &mut hot);
    let text = all_text(&rows);
    assert!(text.contains("v2: kept"), "{text}");
}

#[test]
fn hot_template_poll_returns_error_for_missing_file() {
    let dir = temp_case("hot-err");
    let path = dir.join("ui.crepus");
    fs::write(&path, "div\n  \"alive\"").unwrap();

    let mut hot = HotTemplate::watch(&path).unwrap();
    let _ = render_hot(40, 3, &mut hot);

    fs::remove_file(&path).unwrap();
    *hot.changed_handle().lock().unwrap() = true;

    match hot.poll_reload() {
        ReloadOutcome::Error(_) => {}
        other => panic!("expected Error after deleting file, got {other:?}"),
    }

    let rows = render_hot(40, 3, &mut hot);
    assert!(all_text(&rows).contains("alive"), "{}", all_text(&rows));
}

#[test]
fn hot_template_consumes_change_flag() {
    let dir = temp_case("hot-flag-consumed");
    let path = dir.join("ui.crepus");
    fs::write(&path, "div\n  \"x\"").unwrap();

    let mut hot = HotTemplate::watch(&path).unwrap();
    *hot.changed_handle().lock().unwrap() = true;
    assert_eq!(hot.poll_reload(), ReloadOutcome::Reloaded);
    assert_eq!(hot.poll_reload(), ReloadOutcome::Unchanged);
    assert!(!hot.has_pending_change());
}

#[test]
fn hot_template_drop_releases_watcher() {
    // Smoke test: instantiate and drop many `HotTemplate`s in a row. Before
    // moving the watcher's ownership into the struct, every `watch()` call
    // permanently leaked a parked OS thread.
    let dir = temp_case("hot-drop");
    let path = dir.join("ui.crepus");
    fs::write(&path, "div\n  \"x\"").unwrap();

    for _ in 0..16 {
        let hot = HotTemplate::watch(&path).expect("watch should succeed");
        drop(hot);
    }
}

// ─── Watcher event filter (pure) ──────────────────────────────────────────────

mod watcher_filter {
    use crate::hot_reload::event_touches_relevant_path;
    use notify::event::{ModifyKind, RemoveKind};
    use notify::{Event, EventKind};
    use std::fs;
    use std::path::PathBuf;

    fn ev(kind: EventKind, paths: Vec<PathBuf>) -> Event {
        Event {
            kind,
            paths,
            attrs: Default::default(),
        }
    }

    #[test]
    fn matches_target_template() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        fs::write(&target, "div").unwrap();
        let target = target.canonicalize().unwrap();
        let e = ev(EventKind::Modify(ModifyKind::Any), vec![target.clone()]);
        assert!(event_touches_relevant_path(&e, &target, dir.path()));
    }

    #[test]
    fn matches_sibling_include() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        let include = dir.path().join("card.crepus");
        fs::write(&target, "div").unwrap();
        fs::write(&include, "div").unwrap();
        let target = target.canonicalize().unwrap();
        let include = include.canonicalize().unwrap();
        let e = ev(EventKind::Modify(ModifyKind::Any), vec![include]);
        assert!(event_touches_relevant_path(&e, &target, dir.path()));
    }

    #[test]
    fn matches_target_after_atomic_save_remove() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        fs::write(&target, "div").unwrap();
        let target = target.canonicalize().unwrap();
        fs::remove_file(&target).unwrap();
        let e = ev(EventKind::Remove(RemoveKind::File), vec![target.clone()]);
        assert!(event_touches_relevant_path(&e, &target, dir.path()));
    }

    #[test]
    fn ignores_unrelated_extension() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        let other = dir.path().join("README.md");
        fs::write(&target, "div").unwrap();
        fs::write(&other, "x").unwrap();
        let target = target.canonicalize().unwrap();
        let other = other.canonicalize().unwrap();
        let e = ev(EventKind::Modify(ModifyKind::Any), vec![other]);
        assert!(!event_touches_relevant_path(&e, &target, dir.path()));
    }
}

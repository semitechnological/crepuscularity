//! AST + context → ratatui `Frame` rendering.
//!
//! The renderer works in two phases:
//!
//! 1. **Build** — walks the `.crepus` AST (evaluating control-flow and expressions)
//!    and produces a [`WidgetNode`] tree that mirrors the DOM hierarchy.
//!
//! 2. **Paint** — recursively splits the terminal `Rect` using ratatui `Layout` and
//!    renders each [`WidgetNode`] into the resulting chunks.
//!
//! The split mirrors CSS flexbox: `flex-col` (default) stacks children vertically;
//! `flex-row` stacks them horizontally.  Size classes (`w-[20]`, `h-[3]`, `flex-1`,
//! `w-1/3`, …) become ratatui `Constraint`s so the layout is fully driven by the
//! template, not imperative Rust code.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{block::Padding, Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crepuscularity_core::ast::*;
use crepuscularity_core::context::{value_to_str, TemplateContext, TemplateValue};
use crepuscularity_core::eval::eval_expr;
use crepuscularity_core::parser::{parse_component_file, parse_template};
use crepuscularity_core::preprocess::slot_rotate_child_phrases;

use crate::style::{parse_classes, StyleHints};

fn read_file(ctx: &TemplateContext, path: &std::path::Path) -> Result<String, String> {
    let key = path.to_string_lossy();
    if let Some(content) = ctx.virtual_files.get(key.as_ref()) {
        return Ok(content.clone());
    }
    for (vkey, content) in &ctx.virtual_files {
        if vkey.ends_with(key.as_ref()) || key.ends_with(vkey.as_str()) {
            return Ok(content.clone());
        }
    }
    if cfg!(not(target_arch = "wasm32")) {
        std::fs::read_to_string(path).map_err(|e| format!("include error: {:?}: {}", path, e))
    } else {
        Err(format!(
            "include error: file not in virtual bundle: {}",
            key
        ))
    }
}

fn resolve_include_path(
    base_dir: Option<&std::path::Path>,
    path: &str,
) -> Result<std::path::PathBuf, String> {
    let requested = std::path::Path::new(path);
    if requested.is_absolute()
        || requested
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!("include path outside base dir: {path}"));
    }

    let candidate = if let Some(base) = base_dir {
        base.join(requested)
    } else {
        requested.to_path_buf()
    };
    if cfg!(not(target_arch = "wasm32")) {
        let resolved = std::fs::canonicalize(&candidate).unwrap_or(candidate);
        if let Some(base) = base_dir {
            if let Ok(base) = std::fs::canonicalize(base) {
                if !resolved.starts_with(&base) {
                    return Err(format!("include path outside base dir: {path}"));
                }
            }
        }
        Ok(resolved)
    } else {
        Ok(candidate)
    }
}

// ─── Intermediate widget tree ─────────────────────────────────────────────────

/// An intermediate representation of the layout, independent of screen size.
/// Built from the AST, then painted into a ratatui `Frame`.
pub enum WidgetNode {
    /// A flex container that splits its area into sub-areas for its children.
    Container {
        direction: Direction,
        gap: u16,
        children: Vec<WidgetChild>,
        style: Style,
        block: Option<BlockSpec>,
        scroll_offset: usize,
    },
    /// A leaf node rendered as a ratatui `Paragraph`.
    Content {
        lines: Vec<Line<'static>>,
        style: Style,
        block: Option<BlockSpec>,
        alignment: Alignment,
    },
    /// Occupies no space (collapsed if-branch, empty for-loop, etc.).
    Empty,
}

pub struct WidgetChild {
    pub node: WidgetNode,
    /// How much space this child requests from its parent in the layout axis.
    pub constraint: Constraint,
}

/// Parameters needed to build a `Block` widget (borders + optional title + padding).
#[derive(Clone)]
pub struct BlockSpec {
    borders: Borders,
    border_type: BorderType,
    border_style: Style,
    title: Option<String>,
    title_alignment: Alignment,
    padding: Padding,
}

impl BlockSpec {
    fn to_block(&self) -> Block<'static> {
        let mut b = Block::default()
            .borders(self.borders)
            .border_type(self.border_type)
            .border_style(self.border_style)
            .padding(self.padding);
        if let Some(ref t) = self.title {
            b = b.title(t.clone()).title_alignment(self.title_alignment);
        }
        b
    }
}

// ─── Public render entry points ───────────────────────────────────────────────

/// Parse `template` and paint it into `frame` within `area`.
///
/// `area` is typically `frame.area()` for a full-screen root element, or a
/// sub-`Rect` when embedding a crepus view inside a larger ratatui application.
pub fn render_template(
    template: &str,
    ctx: &TemplateContext,
    frame: &mut Frame,
    area: Rect,
) -> Result<(), String> {
    let nodes = parse_template(template)?;
    render_nodes(&nodes, ctx, frame, area)
}

/// Paint pre-parsed `nodes` into `frame` within `area`.
pub fn render_nodes(
    nodes: &[Node],
    ctx: &TemplateContext,
    frame: &mut Frame,
    area: Rect,
) -> Result<(), String> {
    // The root is implicitly a vertical flex container filling the whole area.
    let render_ctx = with_tui_target(ctx);
    let children = build_children(nodes, &render_ctx, Direction::Vertical, Style::default());
    paint_children(&children, frame, area, Direction::Vertical, 0, 0);
    Ok(())
}

/// Parse a named component from a multi-component `.crepus` file and paint it.
pub fn render_component(
    content: &str,
    component_name: &str,
    ctx: &TemplateContext,
    frame: &mut Frame,
    area: Rect,
) -> Result<(), String> {
    let file = parse_component_file(content)?;
    let component = file
        .components
        .get(component_name)
        .ok_or_else(|| format!("component not found: {component_name}"))?;

    let mut child_ctx = with_tui_target(ctx);
    for (key, expr) in &component.meta.defaults {
        child_ctx
            .vars
            .entry(key.clone())
            .or_insert_with(|| eval_expr(expr, &TemplateContext::default()));
    }

    render_nodes(&component.nodes, &child_ctx, frame, area)
}

fn with_tui_target(ctx: &TemplateContext) -> TemplateContext {
    let mut ctx = ctx.clone();
    ctx.vars
        .entry("crepus_target".to_string())
        .or_insert_with(|| TemplateValue::Str("tui".to_string()));
    ctx.vars
        .entry("is_tui".to_string())
        .or_insert(TemplateValue::Bool(true));
    ctx.vars
        .entry("is_gui".to_string())
        .or_insert(TemplateValue::Bool(false));
    ctx.vars
        .entry("is_web".to_string())
        .or_insert(TemplateValue::Bool(false));
    ctx.vars
        .entry("crepus_platform".to_string())
        .or_insert_with(|| TemplateValue::Str(std::env::consts::OS.to_string()));
    ctx.vars
        .entry("is_macos".to_string())
        .or_insert(TemplateValue::Bool(cfg!(target_os = "macos")));
    ctx.vars
        .entry("is_windows".to_string())
        .or_insert(TemplateValue::Bool(cfg!(target_os = "windows")));
    ctx.vars
        .entry("is_linux".to_string())
        .or_insert(TemplateValue::Bool(cfg!(target_os = "linux")));
    ctx
}

// ─── Phase 1: Build ───────────────────────────────────────────────────────────

/// Convert a slice of nodes into a list of `WidgetChild`s.
///
/// `parent_dir` controls which `SizeHint` dimension (width vs. height) is used
/// to derive each child's `Constraint`.
fn build_children(
    nodes: &[Node],
    ctx: &TemplateContext,
    parent_dir: Direction,
    inherited: Style,
) -> Vec<WidgetChild> {
    let mut ctx = ctx.clone();
    let mut out: Vec<WidgetChild> = Vec::new();
    // Consecutive text/expression nodes are buffered and emitted as one Content.
    let mut text_buf: Vec<Line<'static>> = Vec::new();

    for node in nodes {
        match node {
            // Variable declarations mutate context for subsequent siblings.
            Node::LetDecl(decl) => {
                let val = eval_expr(&decl.expr, &ctx);
                if decl.is_default {
                    ctx.vars.entry(decl.name.clone()).or_insert(val);
                } else {
                    ctx.vars.insert(decl.name.clone(), val);
                }
            }
            Node::Text(parts) => {
                text_buf.push(build_line(parts, &ctx));
            }
            Node::RawText(expr) => {
                text_buf.push(Line::raw(value_to_str(&eval_expr(expr, &ctx))));
            }
            // Structural nodes: flush buffered text first, then emit.
            Node::Element(el) => {
                flush_text(&mut text_buf, &mut out, parent_dir, inherited);
                out.push(build_element(el, &ctx, parent_dir, inherited));
            }
            Node::If(block) => {
                flush_text(&mut text_buf, &mut out, parent_dir, inherited);
                out.extend(build_if(block, &ctx, parent_dir, inherited));
            }
            Node::For(block) => {
                flush_text(&mut text_buf, &mut out, parent_dir, inherited);
                out.extend(build_for(block, &ctx, parent_dir, inherited));
            }
            Node::Match(block) => {
                flush_text(&mut text_buf, &mut out, parent_dir, inherited);
                out.extend(build_match(block, &ctx, parent_dir, inherited));
            }
            Node::Include(inc) => {
                flush_text(&mut text_buf, &mut out, parent_dir, inherited);
                out.extend(build_include(inc, &ctx, parent_dir, inherited));
            }
        }
    }

    flush_text(&mut text_buf, &mut out, parent_dir, inherited);
    out
}

/// Emit accumulated text lines as a single `Content` child and clear the buffer.
fn flush_text(
    buf: &mut Vec<Line<'static>>,
    out: &mut Vec<WidgetChild>,
    parent_dir: Direction,
    inherited: Style,
) {
    if buf.is_empty() {
        return;
    }
    let lines = std::mem::take(buf);
    let height = lines.len() as u16;
    let constraint = match parent_dir {
        // In a vertical stack, text takes exactly as many rows as it has lines.
        Direction::Vertical => Constraint::Length(height),
        // In a horizontal stack, text shares space equally with siblings.
        Direction::Horizontal => Constraint::Fill(1),
    };
    out.push(WidgetChild {
        node: WidgetNode::Content {
            lines,
            style: inherited,
            block: None,
            alignment: Alignment::Left,
        },
        constraint,
    });
}

fn build_element(
    el: &Element,
    ctx: &TemplateContext,
    parent_dir: Direction,
    inherited: Style,
) -> WidgetChild {
    // ── Special pseudo-elements ───────────────────────────────────────────────
    match el.tag.as_str() {
        "slot" => return build_slot(el, ctx, parent_dir, inherited),
        "slot-rotate" => return build_slot_rotate(el, ctx, parent_dir, inherited),
        _ => {}
    }

    // ── Evaluate conditional classes ──────────────────────────────────────────
    let mut classes = active_classes(el, ctx);
    if el.tag == "button"
        && !classes.iter().any(|class| {
            matches!(
                class.as_str(),
                "tui-button" | "button" | "tui-button-active" | "button-active"
            )
        })
    {
        classes.push("tui-button".to_string());
    }
    let mut hints = parse_classes(&classes);

    // Allow a `title={expr}` binding to set the block title.
    if let Some(title_val) = el
        .bindings
        .iter()
        .find(|b| b.prop == "title")
        .map(|b| value_to_str(&eval_expr(&b.value, ctx)))
    {
        hints.title = Some(title_val);
    }

    let constraint = adjusted_constraint(&hints, parent_dir);
    let style = inherited.patch(hints.to_style());

    // ── Detect pure-text vs. mixed/structural children ────────────────────────
    let is_pure_text = !el.children.is_empty()
        && el
            .children
            .iter()
            .all(|n| matches!(n, Node::Text(_) | Node::RawText(_) | Node::LetDecl(_)));

    if is_pure_text {
        build_content_element(el, &hints, ctx, constraint, style)
    } else if el.children.is_empty() {
        build_empty_box(&hints, constraint, style)
    } else {
        build_container_element(el, &hints, ctx, parent_dir, constraint, style, &classes)
    }
}

fn adjusted_constraint(hints: &StyleHints, parent_dir: Direction) -> Constraint {
    match hints.constraint_for(parent_dir) {
        Constraint::Length(n) => Constraint::Length(match parent_dir {
            Direction::Horizontal => n.max(min_width(hints)),
            Direction::Vertical => n.max(min_height(hints)),
        }),
        constraint => constraint,
    }
}

fn min_width(hints: &StyleHints) -> u16 {
    let borders = u16::from(hints.borders.intersects(Borders::LEFT))
        + u16::from(hints.borders.intersects(Borders::RIGHT));
    borders + hints.padding.left + hints.padding.right + 1
}

fn min_height(hints: &StyleHints) -> u16 {
    let borders = u16::from(hints.borders.intersects(Borders::TOP))
        + u16::from(hints.borders.intersects(Borders::BOTTOM));
    borders + hints.padding.top + hints.padding.bottom + 1
}

/// Element whose children are all text → render as a `Paragraph`.
fn build_content_element(
    el: &Element,
    hints: &StyleHints,
    ctx: &TemplateContext,
    constraint: Constraint,
    style: Style,
) -> WidgetChild {
    let mut child_ctx = ctx.clone();
    let mut lines: Vec<Line<'static>> = Vec::new();

    for child in &el.children {
        match child {
            Node::Text(parts) => lines.push(build_line(parts, &child_ctx)),
            Node::RawText(expr) => {
                lines.push(Line::raw(value_to_str(&eval_expr(expr, &child_ctx))))
            }
            Node::LetDecl(decl) => {
                let val = eval_expr(&decl.expr, &child_ctx);
                if decl.is_default {
                    child_ctx.vars.entry(decl.name.clone()).or_insert(val);
                } else {
                    child_ctx.vars.insert(decl.name.clone(), val);
                }
            }
            _ => {}
        }
    }

    WidgetChild {
        node: WidgetNode::Content {
            lines,
            style,
            block: hints_to_block(hints),
            alignment: hints.alignment,
        },
        constraint,
    }
}

/// Element with no children (spacer, background panel, etc.).
fn build_empty_box(hints: &StyleHints, constraint: Constraint, style: Style) -> WidgetChild {
    WidgetChild {
        node: WidgetNode::Container {
            direction: hints.direction,
            gap: hints.gap,
            children: vec![],
            style,
            block: hints_to_block(hints),
            scroll_offset: 0,
        },
        constraint,
    }
}

/// Element with structural children → split its area for each child.
fn build_container_element(
    el: &Element,
    hints: &StyleHints,
    ctx: &TemplateContext,
    _parent_dir: Direction,
    constraint: Constraint,
    style: Style,
    classes: &[String],
) -> WidgetChild {
    let child_dir = hints.direction;
    let children = build_children(&el.children, ctx, child_dir, style);
    let scroll_offset = scroll_offset(el, ctx, classes);

    WidgetChild {
        node: WidgetNode::Container {
            direction: child_dir,
            gap: hints.gap,
            children,
            style,
            block: hints_to_block(hints),
            scroll_offset,
        },
        constraint,
    }
}

/// `<slot>` — renders the caller's slot children (or the element's own children
/// as fallback when no slot was passed).
fn build_slot(
    el: &Element,
    ctx: &TemplateContext,
    _parent_dir: Direction,
    inherited: Style,
) -> WidgetChild {
    let (nodes, slot_ctx) = if let Some((slot_nodes, slot_ctx_box)) = &ctx.slot {
        (slot_nodes.as_slice(), slot_ctx_box.as_ref())
    } else {
        (el.children.as_slice(), ctx)
    };

    let children = build_children(nodes, slot_ctx, Direction::Vertical, inherited);
    WidgetChild {
        node: WidgetNode::Container {
            direction: Direction::Vertical,
            gap: 0,
            children,
            style: inherited,
            block: None,
            scroll_offset: 0,
        },
        constraint: Constraint::Fill(1),
    }
}

/// `<slot-rotate>` — TUI does not have CSS animations, but the renderer is
/// invoked on every poll-and-draw cycle (typically every ~100 ms via
/// [`crate::HotTemplate::poll_and_draw_full`]), so we cycle the displayed
/// phrase from wall-clock time. The visible phrase index is
/// `floor(now_ms / interval_ms) % phrases.len()`, so all `slot-rotate`
/// elements with the same interval stay in phase across re-renders and
/// across hot-reloads.
fn build_slot_rotate(
    el: &Element,
    ctx: &TemplateContext,
    parent_dir: Direction,
    inherited: Style,
) -> WidgetChild {
    let phrases = slot_rotate_child_phrases(&el.children).unwrap_or_default();
    let interval_ms = slot_rotate_interval_ms(el, ctx);
    let line = current_slot_rotate_line(&phrases, interval_ms, current_unix_ms());

    let classes = active_classes(el, ctx);
    let hints = parse_classes(&classes);
    let constraint = hints.constraint_for(parent_dir);
    let style = inherited.patch(hints.to_style());

    WidgetChild {
        node: WidgetNode::Content {
            lines: vec![line],
            style,
            block: hints_to_block(&hints),
            alignment: hints.alignment,
        },
        constraint,
    }
}

/// Pull the `interval={ms}` binding off `el`, evaluate it against `ctx`, and
/// fall back to 3200 ms (the same default used by the web/SSR backends) on
/// any failure to match the cross-target visual behaviour of `slot-rotate`.
fn slot_rotate_interval_ms(el: &Element, ctx: &TemplateContext) -> u64 {
    const DEFAULT_INTERVAL_MS: u64 = 3200;
    for b in &el.bindings {
        if b.prop == "interval" {
            let v = value_to_str(&eval_expr(&b.value, ctx));
            let trimmed = v.trim_matches('"').trim();
            return trimmed.parse().unwrap_or(DEFAULT_INTERVAL_MS);
        }
    }
    DEFAULT_INTERVAL_MS
}

/// Compute the `Line` for the current phrase given the rotation parameters.
///
/// Pulled out so the time-based selection is unit-testable (the production
/// caller passes `current_unix_ms()`; tests pass deterministic values).
fn current_slot_rotate_line(phrases: &[String], interval_ms: u64, now_ms: u64) -> Line<'static> {
    if phrases.is_empty() {
        return Line::raw("");
    }
    let interval = interval_ms.max(1);
    let idx = ((now_ms / interval) as usize) % phrases.len();
    Line::raw(phrases[idx].clone())
}

fn current_unix_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn build_if(
    block: &IfBlock,
    ctx: &TemplateContext,
    parent_dir: Direction,
    inherited: Style,
) -> Vec<WidgetChild> {
    let body = if ctx.eval_condition(&block.condition) {
        &block.then_children
    } else if let Some(ref else_children) = block.else_children {
        else_children
    } else {
        return vec![];
    };
    build_children(body, ctx, parent_dir, inherited)
}

fn build_for(
    block: &ForBlock,
    ctx: &TemplateContext,
    parent_dir: Direction,
    inherited: Style,
) -> Vec<WidgetChild> {
    let items = ctx.get_list(&block.iterator);
    let mut out = Vec::new();
    for item_ctx in items {
        let mut child_ctx = ctx.clone();
        // Merge all fields from the item context into the child context.
        for (k, v) in &item_ctx.vars {
            child_ctx.vars.insert(k.clone(), v.clone());
        }
        // Bind the loop pattern variable to the item's "value" field if present.
        let pattern = block.pattern.trim();
        if !pattern.is_empty() {
            let item_str = item_ctx.get_str("value");
            if !item_str.is_empty() {
                child_ctx
                    .vars
                    .insert(pattern.to_string(), TemplateValue::Str(item_str));
            }
        }
        out.extend(build_children(
            &block.body,
            &child_ctx,
            parent_dir,
            inherited,
        ));
    }
    out
}

fn build_match(
    block: &MatchBlock,
    ctx: &TemplateContext,
    parent_dir: Direction,
    inherited: Style,
) -> Vec<WidgetChild> {
    let val = value_to_str(&eval_expr(&block.expr, ctx));
    for arm in &block.arms {
        let pattern = arm.pattern.trim();
        let matched = if pattern == "_" {
            true
        } else if pattern.starts_with('"') && pattern.ends_with('"') {
            pattern[1..pattern.len() - 1] == *val
        } else {
            pattern == val
        };
        if matched {
            return build_children(&arm.body, ctx, parent_dir, inherited);
        }
    }
    vec![]
}

fn build_include(
    inc: &IncludeNode,
    ctx: &TemplateContext,
    parent_dir: Direction,
    inherited: Style,
) -> Vec<WidgetChild> {
    // Split `path#ComponentName` if present.
    let (file_path, component_name) = if let Some((f, c)) = inc.path.split_once('#') {
        (f, Some(c))
    } else {
        (inc.path.as_str(), None)
    };

    let full_path = match resolve_include_path(ctx.base_dir.as_deref(), file_path) {
        Ok(path) => path,
        Err(e) => return error_children(&e),
    };

    // Load file content: virtual files take priority, then filesystem.
    let content = match read_file(ctx, &full_path) {
        Ok(c) => c,
        Err(e) => return error_children(&e),
    };

    let mut child_ctx = TemplateContext::new();
    child_ctx.base_dir = full_path.parent().map(|p| p.to_path_buf());
    child_ctx.virtual_files = ctx.virtual_files.clone();
    for (key, expr) in &inc.props {
        let val = eval_expr(expr, ctx);
        child_ctx.vars.insert(key.clone(), val);
    }
    if !inc.slot.is_empty() {
        child_ctx.slot = Some((inc.slot.clone(), Box::new(ctx.clone())));
    }

    // Parse and recurse.
    let nodes = match component_name {
        Some(name) => match parse_component_file(&content) {
            Ok(file) => match file.components.get(name) {
                Some(comp) => {
                    for (k, v) in &comp.meta.defaults {
                        child_ctx
                            .vars
                            .entry(k.clone())
                            .or_insert_with(|| eval_expr(v, &TemplateContext::default()));
                    }
                    comp.nodes.clone()
                }
                None => return error_children(&format!("component not found: {name}")),
            },
            Err(e) => return error_children(&format!("parse error in '{file_path}': {e}")),
        },
        None => match parse_template(&content) {
            Ok(n) => n,
            Err(e) => return error_children(&format!("parse error in '{file_path}': {e}")),
        },
    };

    build_children(&nodes, &child_ctx, parent_dir, inherited)
}

// ─── Phase 2: Paint ───────────────────────────────────────────────────────────

/// Recursively render a `WidgetNode` tree into `frame` at `area`.
pub fn paint_node(node: &WidgetNode, frame: &mut Frame, area: Rect) {
    match node {
        WidgetNode::Container {
            direction,
            gap,
            children,
            style,
            block,
            scroll_offset,
        } => {
            // Fill background / draw border, then recurse into the inner area.
            let inner = if let Some(spec) = block {
                let b = spec.to_block().style(*style);
                let inner = b.inner(area);
                frame.render_widget(b, area);
                inner
            } else if *style != Style::default() {
                // Background colour only — no border.
                frame.render_widget(Block::default().style(*style), area);
                area
            } else {
                area
            };

            paint_children(children, frame, inner, *direction, *gap, *scroll_offset);
        }

        WidgetNode::Content {
            lines,
            style,
            block,
            alignment,
        } => {
            let text = Text::from(lines.clone());
            let para = Paragraph::new(text)
                .style(*style)
                .alignment(*alignment)
                .wrap(Wrap { trim: false });
            if let Some(spec) = block {
                frame.render_widget(para.block(spec.to_block()), area);
            } else {
                frame.render_widget(para, area);
            }
        }

        WidgetNode::Empty => {}
    }
}

fn paint_children(
    children: &[WidgetChild],
    frame: &mut Frame,
    area: Rect,
    direction: Direction,
    gap: u16,
    scroll_offset: usize,
) {
    if children.is_empty() {
        return;
    }
    let visible_children = if direction == Direction::Vertical && scroll_offset > 0 {
        &children[scroll_offset.min(children.len())..]
    } else {
        children
    };
    if visible_children.is_empty() {
        return;
    }
    let constraints: Vec<Constraint> = visible_children.iter().map(|c| c.constraint).collect();
    let chunks = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .spacing(gap)
        .split(area);

    for (i, child) in visible_children.iter().enumerate() {
        if let Some(&chunk) = chunks.get(i) {
            paint_node(&child.node, frame, chunk);
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Resolve active classes for an element (static + conditional).
fn active_classes(el: &Element, ctx: &TemplateContext) -> Vec<String> {
    let mut classes: Vec<String> = el
        .classes
        .iter()
        .filter_map(|c| active_class(ctx, &ctx.interpolate(c)))
        .collect();
    for cc in &el.conditional_classes {
        if ctx.eval_condition(&cc.condition) {
            if let Some(class) = active_class(ctx, &ctx.interpolate(&cc.class)) {
                classes.push(class);
            }
        }
    }
    classes
}

fn active_class(ctx: &TemplateContext, class: &str) -> Option<String> {
    if let Some((prefix, rest)) = class.split_once(':') {
        if matches_target_prefix(ctx, prefix) {
            return Some(rest.to_string());
        }
        if known_target_prefix(prefix) {
            return None;
        }
    }
    Some(class.to_string())
}

fn scroll_offset(el: &Element, ctx: &TemplateContext, classes: &[String]) -> usize {
    let scrollable = classes.iter().any(|class| {
        matches!(
            class.as_str(),
            "overflow-y-scroll" | "overflow-scroll" | "virtual-scroll"
        )
    });
    if !scrollable {
        return 0;
    }
    el.bindings
        .iter()
        .find(|binding| {
            matches!(
                binding.prop.as_str(),
                "scroll-offset" | "scroll-y" | "scroll"
            )
        })
        .map(|binding| value_to_str(&eval_expr(&binding.value, ctx)))
        .or_else(|| {
            let value = ctx.get_str("scroll_offset");
            (!value.is_empty()).then_some(value)
        })
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

fn matches_target_prefix(ctx: &TemplateContext, prefix: &str) -> bool {
    match prefix {
        "tui" => ctx.get_bool("is_tui") || ctx.get_str("crepus_target") == "tui",
        "gpui" | "gui" => ctx.get_bool("is_gui") || ctx.get_str("crepus_target") == "gpui",
        "web" => ctx.get_bool("is_web") || ctx.get_str("crepus_target") == "web",
        "macos" | "darwin" => ctx.get_bool("is_macos") || ctx.get_str("crepus_platform") == "macos",
        "windows" | "win32" => {
            ctx.get_bool("is_windows") || ctx.get_str("crepus_platform") == "windows"
        }
        "linux" => ctx.get_bool("is_linux") || ctx.get_str("crepus_platform") == "linux",
        _ => false,
    }
}

fn known_target_prefix(prefix: &str) -> bool {
    matches!(
        prefix,
        "tui" | "gpui" | "gui" | "web" | "macos" | "darwin" | "windows" | "win32" | "linux"
    )
}

/// Build a ratatui `Line` from a `TextPart` slice.
fn build_line(parts: &[TextPart], ctx: &TemplateContext) -> Line<'static> {
    let spans: Vec<Span<'static>> = parts
        .iter()
        .map(|part| match part {
            TextPart::Literal(s) => Span::raw(s.clone()),
            TextPart::Expr(expr) => Span::raw(value_to_str(&eval_expr(expr, ctx))),
        })
        .collect();
    Line::from(spans)
}

/// Convert `StyleHints` into a `BlockSpec` if borders / title / padding are needed.
fn hints_to_block(hints: &StyleHints) -> Option<BlockSpec> {
    if !hints.needs_block() {
        return None;
    }
    let mut border_style = Style::default();
    if let Some(fg) = hints.border_fg {
        border_style = border_style.fg(fg);
    }
    Some(BlockSpec {
        borders: hints.borders,
        border_type: hints.border_type,
        border_style,
        title: hints.title.clone(),
        title_alignment: hints.title_alignment,
        padding: hints.padding,
    })
}

/// Return a single-line error `WidgetChild` in red (mirrors the runtime convention).
fn error_children(msg: &str) -> Vec<WidgetChild> {
    vec![WidgetChild {
        node: WidgetNode::Content {
            lines: vec![Line::from(vec![Span::styled(
                format!(" ⚠ {msg} "),
                Style::default().fg(Color::Red),
            )])],
            style: Style::default(),
            block: None,
            alignment: Alignment::Left,
        },
        constraint: Constraint::Length(1),
    }]
}

#[cfg(test)]
mod slot_rotate_tests {
    use super::*;

    fn lines(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| (*s).to_string()).collect()
    }

    fn rendered(line: Line<'static>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
    }

    #[test]
    fn current_slot_rotate_line_cycles_with_time() {
        let phrases = lines(&["one", "two", "three"]);
        let interval = 1000u64;

        // t=0   → idx 0
        // t=999 → idx 0 (still in the first window)
        // t=1000→ idx 1
        // t=2999→ idx 2
        // t=3000→ wraps back to idx 0
        assert_eq!(
            rendered(current_slot_rotate_line(&phrases, interval, 0)),
            "one"
        );
        assert_eq!(
            rendered(current_slot_rotate_line(&phrases, interval, 999)),
            "one"
        );
        assert_eq!(
            rendered(current_slot_rotate_line(&phrases, interval, 1000)),
            "two"
        );
        assert_eq!(
            rendered(current_slot_rotate_line(&phrases, interval, 2999)),
            "three"
        );
        assert_eq!(
            rendered(current_slot_rotate_line(&phrases, interval, 3000)),
            "one"
        );
    }

    #[test]
    fn current_slot_rotate_line_handles_empty() {
        let line = current_slot_rotate_line(&[], 1000, 12345);
        assert_eq!(rendered(line), "");
    }

    #[test]
    fn current_slot_rotate_line_handles_zero_interval() {
        // Guard against the obvious division-by-zero footgun.
        let phrases = lines(&["a", "b"]);
        let line = current_slot_rotate_line(&phrases, 0, 4242);
        // Falls back to a 1ms interval, so idx = 4242 % 2 = 0 → "a".
        assert_eq!(rendered(line), "a");
    }

    #[test]
    fn current_slot_rotate_line_handles_single_phrase() {
        let phrases = lines(&["solo"]);
        // Any time should still produce "solo" — modulo 1 is 0.
        for t in [0u64, 1, 1000, 1_000_000_000] {
            assert_eq!(
                rendered(current_slot_rotate_line(&phrases, 1000, t)),
                "solo"
            );
        }
    }
}

use crate::ir::{StackAxis, ViewIr, ViewNode, ViewStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCodegenTarget {
    SwiftUi,
    Compose,
}

pub fn generate_native_source(ir: &ViewIr, target: NativeCodegenTarget, view_name: &str) -> String {
    match target {
        NativeCodegenTarget::SwiftUi => generate_swiftui(ir, view_name),
        NativeCodegenTarget::Compose => generate_compose(ir, view_name),
    }
}

fn generate_swiftui(ir: &ViewIr, view_name: &str) -> String {
    let body = swiftui_nodes(&ir.root, 2, None, None);
    format!(
        "import Foundation\nimport SwiftUI\n\n@MainActor\npublic enum CrepusActions {{\n    public static let model = CrepusStateStore.shared\n    public static var dispatch: (String) -> String = {{ _ in \"{{}}\" }}\n    public static var resultSink: (String) -> Void = {{ _ in }}\n\n    public static func applyResult(_ json: String) {{\n        model.applyResult(json)\n    }}\n\n    public static func perform(_ action: String) {{\n        resultSink(dispatch(action))\n    }}\n\n    public static func performChange(_ action: String?, bind: String, value: Any) {{\n        resultSink(CrepusRustActions.dispatchChangeStored(action ?? \"\", bind: bind, value: value))\n    }}\n}}\n\n@MainActor\npublic struct {view_name}: View {{\n    @ObservedObject private var model = CrepusActions.model\n\n    public init() {{}}\n\n    public var body: some View {{\n{body}\n    }}\n}}\n"
    )
}

fn swiftui_nodes(
    nodes: &[ViewNode],
    indent: usize,
    scope_name: Option<&str>,
    scope_var: Option<&str>,
) -> String {
    if nodes.len() == 1 {
        swiftui_node(&nodes[0], indent, scope_name, scope_var)
    } else {
        let pad = indent_str(indent);
        let inner = nodes
            .iter()
            .map(|node| swiftui_node(node, indent + 1, scope_name, scope_var))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{pad}Group {{\n{inner}\n{pad}}}")
    }
}

fn swiftui_node(
    node: &ViewNode,
    indent: usize,
    scope_name: Option<&str>,
    scope_var: Option<&str>,
) -> String {
    let pad = indent_str(indent);
    match node {
        ViewNode::Text {
            content,
            bind,
            style,
            ..
        } => {
            let text = bind
                .as_deref()
                .map(|bind| swift_model_text(bind, scope_name, scope_var))
                .unwrap_or_else(|| format!("\"{}\"", swift_escape(content)));
            let mut out = format!("{pad}Text({text})");
            swiftui_style(&mut out, style.as_ref(), true, indent);
            out
        }
        ViewNode::If {
            condition,
            then_children,
            else_children,
            ..
        } => {
            let then_inner = swiftui_children(then_children, indent + 1, scope_name, scope_var);
            let condition = swift_model_bool(condition, scope_name, scope_var);
            if let Some(else_children) = else_children {
                let else_inner = swiftui_children(else_children, indent + 1, scope_name, scope_var);
                format!(
                    "{pad}if {condition} {{\n{then_inner}\n{pad}}} else {{\n{else_inner}\n{pad}}}"
                )
            } else {
                format!("{pad}if {condition} {{\n{then_inner}\n{pad}}}")
            }
        }
        ViewNode::ForEach {
            bind,
            item_name,
            item_body,
            ..
        } => {
            let item_var = swift_identifier(item_name);
            let items = swift_model_items(bind, scope_name, scope_var);
            let inner = swiftui_children(item_body, indent + 1, Some(item_name), Some(&item_var));
            format!(
                "{pad}ForEach(Array({items}.enumerated()), id: \\.offset) {{ _, {item_var} in\n{inner}\n{pad}}}"
            )
        }
        ViewNode::Stack {
            axis,
            spacing,
            align_items,
            style,
            children,
            ..
        } => {
            let align = swiftui_stack_alignment(*axis, align_items.as_deref());
            let gap = spacing.unwrap_or(8.0);
            let inner = swiftui_children(children, indent + 1, scope_name, scope_var);
            let mut out = if *axis == StackAxis::Row
                && style.as_ref().and_then(|style| style.flex_wrap) == Some(true)
            {
                format!(
                    "{pad}LazyVGrid(columns: [GridItem(.adaptive(minimum: 72), spacing: {gap:.1})], alignment: {align}, spacing: {gap:.1}) {{\n{inner}\n{pad}}}"
                )
            } else {
                let view = match axis {
                    StackAxis::Row => "HStack",
                    StackAxis::Column => "VStack",
                };
                format!("{pad}{view}(alignment: {align}, spacing: {gap:.1}) {{\n{inner}\n{pad}}}")
            };
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Button {
            label,
            on_click,
            style,
        } => {
            let action = swiftui_action(on_click.as_deref());
            let mut out = format!(
                "{pad}Button(action: {{ {action} }}) {{\n{}Text(\"{}\")\n{pad}}}",
                indent_str(indent + 1),
                swift_escape(label)
            );
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Toggle {
            label,
            bind,
            checked,
            on_change,
            style,
            ..
        } => {
            let binding = bind
                .as_deref()
                .map(|bind| {
                    format!(
                        "Binding(get: {{ {} }}, set: {{ newValue in {} }})",
                        swift_model_bool(bind, scope_name, scope_var),
                        swiftui_change(on_change.as_deref(), bind, "newValue")
                    )
                })
                .unwrap_or_else(|| format!(".constant({})", swift_bool(*checked)));
            let mut out = format!("{pad}Toggle(\"{}\", isOn: {binding})", swift_escape(label));
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Checkbox {
            label,
            bind,
            checked,
            on_change,
            style,
            ..
        } => {
            let binding = bind
                .as_deref()
                .map(|bind| {
                    format!(
                        "Binding(get: {{ {} }}, set: {{ newValue in {} }})",
                        swift_model_bool(bind, scope_name, scope_var),
                        swiftui_change(on_change.as_deref(), bind, "newValue")
                    )
                })
                .unwrap_or_else(|| format!(".constant({})", swift_bool(*checked)));
            let mut out = format!("{pad}Toggle(\"{}\", isOn: {binding})", swift_escape(label));
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Slider {
            label,
            bind,
            value,
            min,
            max,
            step,
            on_change,
            style,
            ..
        } => {
            let step_arg = step.map(|v| format!(", step: {v:.1}")).unwrap_or_default();
            let slider_value = bind
                .as_deref()
                .map(|bind| {
                    format!(
                        "Binding(get: {{ {} }}, set: {{ newValue in {} }})",
                        swift_model_number(bind, scope_name, scope_var),
                        swiftui_change(on_change.as_deref(), bind, "newValue")
                    )
                })
                .unwrap_or_else(|| format!(".constant({value:.3})"));
            let control =
                format!("Slider(value: {slider_value}, in: {min:.3}...{max:.3}{step_arg})");
            let mut out = if let Some(label) = label {
                format!(
                    "{pad}VStack(alignment: .leading, spacing: 8.0) {{\n{}Text(\"{}\")\n{}{control}\n{pad}}}",
                    indent_str(indent + 1),
                    swift_escape(label),
                    indent_str(indent + 1)
                )
            } else {
                format!("{pad}{control}")
            };
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Progress {
            label,
            value,
            max,
            style,
        } => {
            let mut out = if let Some(label) = label {
                format!(
                    "{pad}ProgressView(\"{}\", value: {value:.3}, total: {max:.3})",
                    swift_escape(label)
                )
            } else {
                format!("{pad}ProgressView(value: {value:.3}, total: {max:.3})")
            };
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Meter {
            label,
            value,
            max,
            style,
            ..
        } => {
            let text = label
                .as_deref()
                .map(|label| format!("{label}: {value:.1}/{max:.1}"))
                .unwrap_or_else(|| format!("{value:.1}/{max:.1}"));
            let mut out = format!("{pad}Text(\"{}\")", swift_escape(&text));
            swiftui_style(&mut out, style.as_ref(), true, indent);
            out
        }
        ViewNode::Badge { label, style, .. } => {
            let mut out = format!("{pad}Text(\"{}\")", swift_escape(label));
            swiftui_style(&mut out, style.as_ref(), true, indent);
            out
        }
        ViewNode::Divider { style, .. } => {
            let mut out = format!("{pad}Divider()");
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Spacer { size, style } => {
            let mut out = size
                .map(|v| format!("{pad}Spacer(minLength: {v:.1})"))
                .unwrap_or_else(|| format!("{pad}Spacer()"));
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Dropzone {
            label,
            style,
            children,
            ..
        } => {
            let inner = if children.is_empty() {
                format!(
                    "{}Text(\"{}\")",
                    indent_str(indent + 1),
                    swift_escape(label)
                )
            } else {
                swiftui_children(children, indent + 1, scope_name, scope_var)
            };
            let mut out =
                format!("{pad}VStack(alignment: .leading, spacing: 8.0) {{\n{inner}\n{pad}}}");
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::FilePicker {
            label,
            on_pick,
            style,
            ..
        } => {
            let action = swiftui_action(on_pick.as_deref());
            let mut out = format!(
                "{pad}Button(action: {{ {action} }}) {{\n{}Text(\"{}\")\n{pad}}}",
                indent_str(indent + 1),
                swift_escape(label)
            );
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Image {
            src, alt, style, ..
        } => {
            let label = alt.as_deref().unwrap_or(src);
            let mut out = format!("{pad}Text(\"{}\")", swift_escape(label));
            swiftui_style(&mut out, style.as_ref(), true, indent);
            out
        }
        ViewNode::Scroll {
            axis,
            style,
            children,
        } => {
            let scroll_axis = match axis {
                StackAxis::Row => ".horizontal",
                StackAxis::Column => ".vertical",
            };
            let inner = swiftui_node(
                &ViewNode::Stack {
                    axis: *axis,
                    spacing: Some(8.0),
                    align_items: None,
                    justify_content: None,
                    style: None,
                    children: children.clone(),
                },
                indent + 1,
                scope_name,
                scope_var,
            );
            let mut out = format!("{pad}ScrollView({scroll_axis}) {{\n{inner}\n{pad}}}");
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::List {
            ordered,
            style,
            children,
        } => {
            let rows = children
                .iter()
                .enumerate()
                .map(|(idx, child)| {
                    let prefix = if *ordered {
                        format!("{}. ", idx + 1)
                    } else {
                        String::new()
                    };
                    format!(
                        "{}HStack {{\n{}Text(\"{}\")\n{}\n{}}}",
                        indent_str(indent + 1),
                        indent_str(indent + 2),
                        swift_escape(&prefix),
                        swiftui_node(child, indent + 2, scope_name, scope_var),
                        indent_str(indent + 1)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let mut out =
                format!("{pad}VStack(alignment: .leading, spacing: 8.0) {{\n{rows}\n{pad}}}");
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::ListItem { style, children } => {
            let inner = swiftui_children(children, indent + 1, scope_name, scope_var);
            let mut out =
                format!("{pad}VStack(alignment: .leading, spacing: 4.0) {{\n{inner}\n{pad}}}");
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::SlotRotate { phrases, style, .. } => {
            let mut out = format!(
                "{pad}Text(\"{}\")",
                swift_escape(phrases.first().map(String::as_str).unwrap_or(""))
            );
            swiftui_style(&mut out, style.as_ref(), true, indent);
            out
        }
        ViewNode::Input {
            placeholder,
            bind,
            multiline,
            secure,
            on_change,
            style,
            ..
        } => {
            let mut out = if *multiline {
                format!(
                    "{pad}TextEditor(text: Binding(get: {{ {} }}, set: {{ newValue in {} }}))",
                    swift_model_text(bind, scope_name, scope_var),
                    swiftui_change(on_change.as_deref(), bind, "newValue")
                )
            } else if *secure {
                format!(
                    "{pad}SecureField(\"{}\", text: Binding(get: {{ {} }}, set: {{ newValue in {} }}))",
                    swift_escape(placeholder),
                    swift_model_text(bind, scope_name, scope_var),
                    swiftui_change(on_change.as_deref(), bind, "newValue")
                )
            } else {
                format!(
                    "{pad}TextField(\"{}\", text: Binding(get: {{ {} }}, set: {{ newValue in {} }}))",
                    swift_escape(placeholder)
                    ,
                    swift_model_text(bind, scope_name, scope_var),
                    swiftui_change(on_change.as_deref(), bind, "newValue")
                )
            };
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Picker {
            bind,
            options,
            on_change,
            style,
            ..
        } => {
            let first = options.first().map(|o| o.value.as_str()).unwrap_or("");
            let rows = options
                .iter()
                .map(|option| {
                    format!(
                        "{}Text(\"{}\").tag(\"{}\")",
                        indent_str(indent + 1),
                        swift_escape(&option.label),
                        swift_escape(&option.value)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let mut out = format!(
                "{pad}Picker(\"\", selection: Binding(get: {{ {} }}, set: {{ newValue in {} }})) {{\n{rows}\n{pad}}}",
                swift_model_text(bind, scope_name, scope_var),
                swiftui_change(on_change.as_deref(), bind, "newValue")
            );
            swiftui_style(&mut out, style.as_ref(), false, indent);
            if first.is_empty() {
                out = out.replace("Binding(get: {  }, set: { _ in })", ".constant(\"\")");
            }
            out
        }
    }
}

fn swiftui_action(on_click: Option<&str>) -> String {
    on_click
        .map(|action| format!("CrepusActions.perform(\"{}\")", swift_escape(action)))
        .unwrap_or_default()
}

fn swiftui_change(on_change: Option<&str>, bind: &str, value: &str) -> String {
    let action = on_change
        .map(|action| format!("\"{}\"", swift_escape(action)))
        .unwrap_or_else(|| "nil".to_string());
    format!(
        "CrepusActions.performChange({}, bind: \"{}\", value: {})",
        action,
        swift_escape(bind),
        value
    )
}

fn swiftui_children(
    children: &[ViewNode],
    indent: usize,
    scope_name: Option<&str>,
    scope_var: Option<&str>,
) -> String {
    children
        .iter()
        .map(|child| swiftui_node(child, indent, scope_name, scope_var))
        .collect::<Vec<_>>()
        .join("\n")
}

fn swift_identifier(name: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in name.chars().enumerate() {
        if (idx == 0 && (ch.is_ascii_alphabetic() || ch == '_'))
            || (idx > 0 && (ch.is_ascii_alphanumeric() || ch == '_'))
        {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "item".to_string()
    } else {
        out
    }
}

fn swift_scope_args(scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    match (scope_name, scope_var) {
        (Some(scope_name), Some(scope_var)) => format!(
            ", scopeName: \"{}\", scope: {}",
            swift_escape(scope_name),
            scope_var
        ),
        _ => String::new(),
    }
}

fn swift_model_text(expr: &str, scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    format!(
        "CrepusActions.model.text(\"{}\"{})",
        swift_escape(expr),
        swift_scope_args(scope_name, scope_var)
    )
}

fn swift_model_bool(expr: &str, scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    format!(
        "CrepusActions.model.bool(\"{}\"{})",
        swift_escape(expr),
        swift_scope_args(scope_name, scope_var)
    )
}

fn swift_model_number(expr: &str, scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    format!(
        "CrepusActions.model.number(\"{}\"{})",
        swift_escape(expr),
        swift_scope_args(scope_name, scope_var)
    )
}

fn swift_model_items(expr: &str, scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    format!(
        "CrepusActions.model.items(\"{}\"{})",
        swift_escape(expr),
        swift_scope_args(scope_name, scope_var)
    )
}

fn swiftui_stack_alignment(axis: StackAxis, align_items: Option<&str>) -> &'static str {
    match axis {
        StackAxis::Column => match align_items {
            Some("center") => ".center",
            Some("end") => ".trailing",
            _ => ".leading",
        },
        StackAxis::Row => match align_items {
            Some("center") => ".center",
            Some("end") => ".bottom",
            _ => ".top",
        },
    }
}

fn swiftui_style(out: &mut String, style: Option<&ViewStyle>, is_text: bool, indent: usize) {
    let Some(style) = style else {
        return;
    };
    let pad = indent_str(indent + 1);
    if let Some(size) = style.font_size {
        out.push_str(&format!("\n{pad}.font(.system(size: {size:.1}))"));
    }
    if let Some(weight) = style.font_weight {
        out.push_str(&format!(
            "\n{pad}.fontWeight({})",
            swiftui_font_weight(weight)
        ));
    }
    if let Some(color) = &style.foreground_color {
        out.push_str(&format!("\n{pad}.foregroundStyle({})", swift_color(color)));
    }
    if is_text {
        if let Some(align) = swiftui_text_align(style.text_align.as_deref()) {
            out.push_str(&format!("\n{pad}.multilineTextAlignment({align})"));
        }
        if style.italic == Some(true) {
            out.push_str(&format!("\n{pad}.italic()"));
        }
        if style.underline == Some(true) {
            out.push_str(&format!("\n{pad}.underline()"));
        }
        if style.strikethrough == Some(true) {
            out.push_str(&format!("\n{pad}.strikethrough()"));
        }
        if let Some(lines) = style.line_clamp {
            out.push_str(&format!("\n{pad}.lineLimit({lines})"));
        }
    }
    swiftui_frame(out, style, &pad);
    swiftui_spacing(out, style, &pad, "padding");
    swiftui_spacing(out, style, &pad, "margin");
    if let Some(opacity) = style.opacity {
        out.push_str(&format!("\n{pad}.opacity({opacity:.3})"));
    }
    if style.hidden == Some(true) {
        out.push_str(&format!("\n{pad}.opacity(0)"));
    }
    if let Some(background) = &style.background_color {
        out.push_str(&format!("\n{pad}.background({})", swift_color(background)));
    }
    if let Some(radius) = style.corner_radius {
        out.push_str(&format!(
            "\n{pad}.clipShape(RoundedRectangle(cornerRadius: {radius:.1}))"
        ));
    }
    if let Some(width) = style.border_width {
        let color = style
            .border_color
            .as_deref()
            .map(swift_color)
            .unwrap_or_else(|| "Color.gray".to_string());
        out.push_str(&format!("\n{pad}.border({color}, width: {width:.1})"));
    }
    if style.overflow_hidden == Some(true) {
        out.push_str(&format!("\n{pad}.clipped()"));
    }
    if let Some(radius) = style.shadow_radius {
        let color = style
            .shadow_color
            .as_deref()
            .map(swift_color)
            .unwrap_or_else(|| "Color.black.opacity(0.25)".to_string());
        let x = style.shadow_offset_x.unwrap_or(0.0);
        let y = style.shadow_offset_y.unwrap_or(0.0);
        out.push_str(&format!(
            "\n{pad}.shadow(color: {color}, radius: {radius:.1}, x: {x:.1}, y: {y:.1})"
        ));
    }
    if style.translate_x.is_some() || style.translate_y.is_some() {
        out.push_str(&format!(
            "\n{pad}.offset(x: {:.1}, y: {:.1})",
            style.translate_x.unwrap_or(0.0),
            style.translate_y.unwrap_or(0.0)
        ));
    }
    if let Some(rotate) = style.rotate {
        out.push_str(&format!("\n{pad}.rotationEffect(.degrees({rotate:.1}))"));
    }
    if style.scale_x.is_some() || style.scale_y.is_some() {
        out.push_str(&format!(
            "\n{pad}.scaleEffect(x: {:.3}, y: {:.3})",
            style.scale_x.unwrap_or(1.0),
            style.scale_y.unwrap_or(1.0)
        ));
    }
}

fn swiftui_spacing(out: &mut String, style: &ViewStyle, pad: &str, kind: &str) {
    let values = if kind == "padding" {
        [
            (style.padding, ""),
            (style.padding_horizontal, ".horizontal"),
            (style.padding_vertical, ".vertical"),
            (style.padding_top, ".top"),
            (style.padding_bottom, ".bottom"),
            (style.padding_left, ".leading"),
            (style.padding_right, ".trailing"),
        ]
    } else {
        [
            (style.margin, ""),
            (style.margin_horizontal, ".horizontal"),
            (style.margin_vertical, ".vertical"),
            (style.margin_top, ".top"),
            (style.margin_bottom, ".bottom"),
            (style.margin_left, ".leading"),
            (style.margin_right, ".trailing"),
        ]
    };
    for (value, edge) in values {
        if let Some(value) = value {
            if edge.is_empty() {
                out.push_str(&format!("\n{pad}.padding({value:.0})"));
            } else {
                out.push_str(&format!("\n{pad}.padding({edge}, {value:.0})"));
            }
        }
    }
}

fn swiftui_frame(out: &mut String, style: &ViewStyle, pad: &str) {
    if style.width == Some(-1.0) || style.height == Some(-1.0) {
        let max_width = if style.width == Some(-1.0) {
            ".infinity".to_string()
        } else {
            "nil".to_string()
        };
        let max_height = if style.height == Some(-1.0) {
            ".infinity".to_string()
        } else {
            "nil".to_string()
        };
        out.push_str(&format!(
            "\n{pad}.frame(maxWidth: {max_width}, maxHeight: {max_height}, alignment: .topLeading)"
        ));
        return;
    }
    if style.width.is_some() || style.height.is_some() {
        let width = style
            .width
            .filter(|v| *v > 0.0)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "nil".to_string());
        let height = style
            .height
            .filter(|v| *v > 0.0)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "nil".to_string());
        out.push_str(&format!(
            "\n{pad}.frame(width: {width}, height: {height}, alignment: .topLeading)"
        ));
    }
}

fn swiftui_text_align(value: Option<&str>) -> Option<&'static str> {
    match value {
        Some("center") => Some(".center"),
        Some("right") | Some("end") | Some("trailing") => Some(".trailing"),
        Some("justify") => Some(".leading"),
        Some("left") | Some("start") | Some("leading") => Some(".leading"),
        _ => None,
    }
}

fn swiftui_font_weight(weight: u16) -> &'static str {
    match weight {
        0..=299 => ".thin",
        300..=399 => ".light",
        400..=499 => ".regular",
        500..=599 => ".medium",
        600..=699 => ".semibold",
        700..=799 => ".bold",
        _ => ".heavy",
    }
}

fn generate_compose(ir: &ViewIr, view_name: &str) -> String {
    let body = compose_nodes(&ir.root, 1, None, None);
    format!(
        "import androidx.compose.foundation.background\nimport androidx.compose.foundation.border\nimport androidx.compose.foundation.clickable\nimport androidx.compose.foundation.horizontalScroll\nimport androidx.compose.foundation.layout.Arrangement\nimport androidx.compose.foundation.layout.Column\nimport androidx.compose.foundation.layout.FlowRow\nimport androidx.compose.foundation.layout.PaddingValues\nimport androidx.compose.foundation.layout.Row\nimport androidx.compose.foundation.layout.Spacer\nimport androidx.compose.foundation.layout.fillMaxHeight\nimport androidx.compose.foundation.layout.fillMaxSize\nimport androidx.compose.foundation.layout.fillMaxWidth\nimport androidx.compose.foundation.layout.height\nimport androidx.compose.foundation.layout.offset\nimport androidx.compose.foundation.layout.padding\nimport androidx.compose.foundation.layout.width\nimport androidx.compose.foundation.rememberScrollState\nimport androidx.compose.foundation.shape.RoundedCornerShape\nimport androidx.compose.foundation.verticalScroll\nimport androidx.compose.material3.Button\nimport androidx.compose.material3.ButtonDefaults\nimport androidx.compose.material3.Divider\nimport androidx.compose.material3.LinearProgressIndicator\nimport androidx.compose.material3.LocalContentColor\nimport androidx.compose.material3.Slider\nimport androidx.compose.material3.Switch\nimport androidx.compose.material3.Text\nimport androidx.compose.material3.TextField\nimport androidx.compose.runtime.Composable\nimport androidx.compose.runtime.CompositionLocalProvider\nimport androidx.compose.ui.Modifier\nimport androidx.compose.ui.draw.alpha\nimport androidx.compose.ui.draw.clip\nimport androidx.compose.ui.draw.rotate\nimport androidx.compose.ui.draw.scale\nimport androidx.compose.ui.graphics.Color\nimport androidx.compose.ui.text.font.FontStyle\nimport androidx.compose.ui.text.font.FontWeight\nimport androidx.compose.ui.text.input.PasswordVisualTransformation\nimport androidx.compose.ui.text.style.TextAlign\nimport androidx.compose.ui.text.style.TextDecoration\nimport androidx.compose.ui.unit.dp\nimport androidx.compose.ui.unit.sp\nimport kotlinx.serialization.json.JsonElement\nimport kotlinx.serialization.json.JsonPrimitive\n\nobject CrepusActions {{\n    var dispatch: (String) -> String = {{ \"{{}}\" }}\n    var resultSink: (String) -> Unit = {{}}\n\n    fun applyResult(raw: String) {{\n        CrepusStateStore.applyResult(raw)\n    }}\n\n    fun perform(action: String) {{\n        resultSink(dispatch(action))\n    }}\n\n    fun performChange(action: String?, bind: String, value: JsonElement) {{\n        resultSink(CrepusRustActions.dispatchChangeJson(action ?: \"\", bind, value.toString()))\n    }}\n}}\n\n@Composable\nfun {view_name}(modifier: Modifier = Modifier) {{\n{body}\n}}\n"
    )
}

fn compose_nodes(
    nodes: &[ViewNode],
    indent: usize,
    scope_name: Option<&str>,
    scope_var: Option<&str>,
) -> String {
    if nodes.len() == 1 {
        compose_node_with_base(
            &nodes[0],
            indent,
            Some("modifier".to_string()),
            scope_name,
            scope_var,
        )
    } else {
        let pad = indent_str(indent);
        let inner = compose_children(nodes, indent + 1, scope_name, scope_var);
        format!("{pad}Column {{\n{inner}\n{pad}}}")
    }
}

fn compose_node(
    node: &ViewNode,
    indent: usize,
    scope_name: Option<&str>,
    scope_var: Option<&str>,
) -> String {
    compose_node_with_base(node, indent, None, scope_name, scope_var)
}

fn compose_node_with_base(
    node: &ViewNode,
    indent: usize,
    base_modifier: Option<String>,
    scope_name: Option<&str>,
    scope_var: Option<&str>,
) -> String {
    let pad = indent_str(indent);
    match node {
        ViewNode::Text {
            content,
            bind,
            style,
            ..
        } => {
            let args = compose_text_args(style.as_ref());
            let text = bind
                .as_deref()
                .map(|bind| compose_model_text(bind, scope_name, scope_var))
                .unwrap_or_else(|| format!("\"{}\"", kotlin_escape(content)));
            format!("{pad}Text({text}{args})")
        }
        ViewNode::If {
            condition,
            then_children,
            else_children,
            ..
        } => {
            let condition = compose_model_bool(condition, scope_name, scope_var);
            let then_inner = compose_children(then_children, indent + 1, scope_name, scope_var);
            if let Some(else_children) = else_children {
                let else_inner = compose_children(else_children, indent + 1, scope_name, scope_var);
                format!("{pad}if ({condition}) {{\n{then_inner}\n{pad}}} else {{\n{else_inner}\n{pad}}}")
            } else {
                format!("{pad}if ({condition}) {{\n{then_inner}\n{pad}}}")
            }
        }
        ViewNode::ForEach {
            bind,
            item_name,
            item_body,
            ..
        } => {
            let item_var = kotlin_identifier(item_name);
            let items = compose_model_items(bind, scope_name, scope_var);
            let inner = compose_children(item_body, indent + 1, Some(item_name), Some(&item_var));
            format!("{pad}{items}.forEachIndexed {{ _, {item_var} ->\n{inner}\n{pad}}}")
        }
        ViewNode::Stack {
            axis,
            spacing,
            style,
            children,
            ..
        } => {
            let wrap = *axis == StackAxis::Row
                && style.as_ref().and_then(|style| style.flex_wrap) == Some(true);
            let view = match (axis, wrap) {
                (StackAxis::Row, true) => "FlowRow",
                (StackAxis::Row, false) => "Row",
                (StackAxis::Column, _) => "Column",
            };
            let mut args = Vec::new();
            if let Some(modifier) = compose_modifier_chain(base_modifier, style.as_ref()) {
                args.push(format!("modifier = {modifier}"));
            }
            let arrangement = match axis {
                StackAxis::Row => "horizontalArrangement",
                StackAxis::Column => "verticalArrangement",
            };
            args.push(format!(
                "{arrangement} = Arrangement.spacedBy({:.0}.dp)",
                spacing.unwrap_or(8.0)
            ));
            if wrap {
                args.push(format!(
                    "verticalArrangement = Arrangement.spacedBy({:.0}.dp)",
                    spacing.unwrap_or(8.0)
                ));
            }
            let args = args.join(", ");
            let inner = compose_children(children, indent + 1, scope_name, scope_var);
            let out = format!("{pad}{view}({args}) {{\n{inner}\n{pad}}}");
            compose_content_color_wrapper(out, style.as_ref(), indent)
        }
        ViewNode::Button {
            label,
            on_click,
            style,
        } => {
            let modifier = compose_button_args(style.as_ref());
            let action = compose_action(on_click.as_deref());
            format!(
                "{pad}Button(onClick = {{ {action} }}{modifier}) {{\n{}Text(\"{}\")\n{pad}}}",
                indent_str(indent + 1),
                kotlin_escape(label)
            )
        }
        ViewNode::Toggle {
            label,
            bind,
            checked,
            on_change,
            style,
            ..
        }
        | ViewNode::Checkbox {
            label,
            bind,
            checked,
            on_change,
            style,
            ..
        } => {
            let modifier = compose_modifier_call_args(style.as_ref());
            let checked_value = bind
                .as_deref()
                .map(|bind| compose_model_bool(bind, scope_name, scope_var))
                .unwrap_or_else(|| kotlin_bool(*checked).to_string());
            let on_change = bind
                .as_deref()
                .map(|bind| compose_change(on_change.as_deref(), bind, "JsonPrimitive(it)"))
                .unwrap_or_else(|| compose_action(on_change.as_deref()));
            format!(
                "{pad}Row{modifier} {{\n{}Text(\"{}\")\n{}Switch(checked = {checked_value}, onCheckedChange = {{ {on_change} }})\n{pad}}}",
                indent_str(indent + 1),
                kotlin_escape(label),
                indent_str(indent + 1)
            )
        }
        ViewNode::Slider {
            label,
            bind,
            value,
            min,
            max,
            on_change,
            style,
            ..
        } => {
            let modifier = compose_modifier_call_args(style.as_ref());
            let label = label
                .as_deref()
                .map(|label| {
                    format!(
                        "{}Text(\"{}\")\n",
                        indent_str(indent + 1),
                        kotlin_escape(label)
                    )
                })
                .unwrap_or_default();
            let slider_value = bind
                .as_deref()
                .map(|bind| compose_model_number(bind, scope_name, scope_var))
                .unwrap_or_else(|| format!("{value:.3}f"));
            let on_value_change = bind
                .as_deref()
                .map(|bind| {
                    compose_change(on_change.as_deref(), bind, "JsonPrimitive(it.toDouble())")
                })
                .unwrap_or_default();
            format!(
                "{pad}Column{modifier} {{\n{label}{}Slider(value = {slider_value}, onValueChange = {{ {on_value_change} }}, valueRange = {min:.3}f..{max:.3}f)\n{pad}}}",
                indent_str(indent + 1)
            )
        }
        ViewNode::Progress {
            label,
            value,
            max,
            style,
        } => {
            let modifier = compose_modifier_call_args(style.as_ref());
            let label = label
                .as_deref()
                .map(|label| {
                    format!(
                        "{}Text(\"{}\")\n",
                        indent_str(indent + 1),
                        kotlin_escape(label)
                    )
                })
                .unwrap_or_default();
            format!(
                "{pad}Column{modifier} {{\n{label}{}LinearProgressIndicator(progress = {value:.3}f / {max:.3}f)\n{pad}}}",
                indent_str(indent + 1)
            )
        }
        ViewNode::Meter {
            label, value, max, ..
        } => {
            let text = label
                .as_deref()
                .map(|label| format!("{label}: {value:.1}/{max:.1}"))
                .unwrap_or_else(|| format!("{value:.1}/{max:.1}"));
            format!("{pad}Text(\"{}\")", kotlin_escape(&text))
        }
        ViewNode::Badge { label, style, .. } => {
            let args = compose_text_args(style.as_ref());
            format!("{pad}Text(\"{}\"{args})", kotlin_escape(label))
        }
        ViewNode::Divider { .. } => format!("{pad}Divider()"),
        ViewNode::Spacer { size, .. } => {
            format!(
                "{pad}Spacer(modifier = Modifier.height({:.0}.dp))",
                size.unwrap_or(8.0)
            )
        }
        ViewNode::Dropzone {
            label,
            style,
            children,
            ..
        } => {
            let modifier = compose_modifier_call_args(style.as_ref());
            let inner = if children.is_empty() {
                format!(
                    "{}Text(\"{}\")",
                    indent_str(indent + 1),
                    kotlin_escape(label)
                )
            } else {
                compose_children(children, indent + 1, scope_name, scope_var)
            };
            format!("{pad}Column{modifier} {{\n{inner}\n{pad}}}")
        }
        ViewNode::FilePicker {
            label,
            on_pick,
            style,
            ..
        } => {
            let modifier = compose_button_args(style.as_ref());
            let action = compose_action(on_pick.as_deref());
            format!(
                "{pad}Button(onClick = {{ {action} }}{modifier}) {{\n{}Text(\"{}\")\n{pad}}}",
                indent_str(indent + 1),
                kotlin_escape(label)
            )
        }
        ViewNode::Image {
            src, alt, style, ..
        } => {
            let args = compose_text_args(style.as_ref());
            format!(
                "{pad}Text(\"{}\"{args})",
                kotlin_escape(alt.as_deref().unwrap_or(src))
            )
        }
        ViewNode::Scroll {
            axis,
            style,
            children,
        } => {
            let base = match axis {
                StackAxis::Row => "Modifier.horizontalScroll(rememberScrollState())",
                StackAxis::Column => "Modifier.verticalScroll(rememberScrollState())",
            };
            let modifier = compose_modifier_chain(Some(base.to_string()), style.as_ref());
            let view = match axis {
                StackAxis::Row => "Row",
                StackAxis::Column => "Column",
            };
            let inner = compose_children(children, indent + 1, scope_name, scope_var);
            format!(
                "{pad}{view}(modifier = {}) {{\n{inner}\n{pad}}}",
                modifier.unwrap_or_else(|| "Modifier".to_string())
            )
        }
        ViewNode::List { children, .. } | ViewNode::ListItem { children, .. } => {
            let inner = compose_children(children, indent + 1, scope_name, scope_var);
            format!("{pad}Column {{\n{inner}\n{pad}}}")
        }
        ViewNode::SlotRotate { phrases, style, .. } => {
            let args = compose_text_args(style.as_ref());
            format!(
                "{pad}Text(\"{}\"{args})",
                kotlin_escape(phrases.first().map(String::as_str).unwrap_or(""))
            )
        }
        ViewNode::Input {
            placeholder,
            bind,
            secure,
            on_change,
            style,
            ..
        } => {
            let on_value_change = compose_change(on_change.as_deref(), bind, "JsonPrimitive(it)");
            let modifier = compose_modifier_param(style.as_ref());
            let secure_arg = if *secure {
                ", visualTransformation = PasswordVisualTransformation()"
            } else {
                ""
            };
            format!(
                "{pad}TextField(value = {}, onValueChange = {{ {on_value_change} }}, placeholder = {{ Text(\"{}\") }}{secure_arg}{modifier})",
                compose_model_text(bind, scope_name, scope_var),
                kotlin_escape(placeholder),
            )
        }
        ViewNode::Picker {
            bind,
            options,
            on_change,
            style,
            ..
        } => {
            let modifier = compose_modifier_call_args(style.as_ref());
            let current = compose_model_text(bind, scope_name, scope_var);
            let inner = options
                .iter()
                .map(|option| {
                    let click = compose_change(
                        on_change.as_deref(),
                        bind,
                        &format!("JsonPrimitive(\"{}\")", kotlin_escape(&option.value)),
                    );
                    format!(
                        "{}Text(text = if ({current} == \"{}\") \"{}\" else \"{}\", modifier = Modifier.clickable {{ {click} }})",
                        indent_str(indent + 1),
                        kotlin_escape(&option.value),
                        kotlin_escape(&option.label),
                        kotlin_escape(&option.label)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("{pad}Column{modifier} {{\n{inner}\n{pad}}}")
        }
    }
}

fn compose_action(on_click: Option<&str>) -> String {
    on_click
        .map(|action| format!("CrepusActions.perform(\"{}\")", kotlin_escape(action)))
        .unwrap_or_default()
}

fn compose_change(on_change: Option<&str>, bind: &str, value: &str) -> String {
    let action = on_change
        .map(|action| format!("\"{}\"", kotlin_escape(action)))
        .unwrap_or_else(|| "null".to_string());
    format!(
        "CrepusActions.performChange({}, \"{}\", {})",
        action,
        kotlin_escape(bind),
        value
    )
}

fn compose_content_color_wrapper(out: String, style: Option<&ViewStyle>, indent: usize) -> String {
    let Some(color) = style
        .and_then(|style| style.foreground_color.as_deref())
        .map(compose_hex_argb)
    else {
        return out;
    };
    let pad = indent_str(indent);
    let inner = out
        .lines()
        .map(|line| format!("{}{}", indent_str(1), line))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{pad}CompositionLocalProvider(LocalContentColor provides Color(0x{color})) {{\n{inner}\n{pad}}}"
    )
}

fn compose_children(
    children: &[ViewNode],
    indent: usize,
    scope_name: Option<&str>,
    scope_var: Option<&str>,
) -> String {
    children
        .iter()
        .map(|child| compose_node(child, indent, scope_name, scope_var))
        .collect::<Vec<_>>()
        .join("\n")
}

fn kotlin_identifier(name: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in name.chars().enumerate() {
        if (idx == 0 && (ch.is_ascii_alphabetic() || ch == '_'))
            || (idx > 0 && (ch.is_ascii_alphanumeric() || ch == '_'))
        {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "item".to_string()
    } else {
        out
    }
}

fn kotlin_scope_args(scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    match (scope_name, scope_var) {
        (Some(scope_name), Some(scope_var)) => format!(
            ", scopeName = \"{}\", scope = {}",
            kotlin_escape(scope_name),
            scope_var
        ),
        _ => String::new(),
    }
}

fn compose_model_text(expr: &str, scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    format!(
        "CrepusStateStore.text(\"{}\"{})",
        kotlin_escape(expr),
        kotlin_scope_args(scope_name, scope_var)
    )
}

fn compose_model_bool(expr: &str, scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    format!(
        "CrepusStateStore.bool(\"{}\"{})",
        kotlin_escape(expr),
        kotlin_scope_args(scope_name, scope_var)
    )
}

fn compose_model_number(expr: &str, scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    format!(
        "CrepusStateStore.number(\"{}\"{})",
        kotlin_escape(expr),
        kotlin_scope_args(scope_name, scope_var)
    )
}

fn compose_model_items(expr: &str, scope_name: Option<&str>, scope_var: Option<&str>) -> String {
    format!(
        "CrepusStateStore.items(\"{}\"{})",
        kotlin_escape(expr),
        kotlin_scope_args(scope_name, scope_var)
    )
}

fn compose_text_args(style: Option<&ViewStyle>) -> String {
    let mut args = Vec::new();
    if let Some(modifier) = compose_modifier(style) {
        args.push(format!("modifier = {modifier}"));
    }
    if let Some(style) = style {
        if let Some(size) = style.font_size {
            args.push(format!("fontSize = {size:.1}.sp"));
        }
        if let Some(weight) = style.font_weight {
            args.push(format!("fontWeight = {}", compose_font_weight(weight)));
        }
        if let Some(color) = &style.foreground_color {
            args.push(format!("color = Color(0x{})", compose_hex_argb(color)));
        }
        if let Some(align) = compose_text_align(style.text_align.as_deref()) {
            args.push(format!("textAlign = {align}"));
        }
        if style.italic == Some(true) {
            args.push("fontStyle = FontStyle.Italic".to_string());
        }
        if style.underline == Some(true) && style.strikethrough == Some(true) {
            args.push(
                "textDecoration = TextDecoration.combine(listOf(TextDecoration.Underline, TextDecoration.LineThrough))"
                    .to_string(),
            );
        } else if style.underline == Some(true) {
            args.push("textDecoration = TextDecoration.Underline".to_string());
        } else if style.strikethrough == Some(true) {
            args.push("textDecoration = TextDecoration.LineThrough".to_string());
        }
        if let Some(line_height) = style.line_height {
            args.push(format!("lineHeight = {:.1}.sp", line_height * 16.0));
        }
        if let Some(lines) = style.line_clamp {
            args.push(format!("maxLines = {lines}"));
        }
    }
    if args.is_empty() {
        String::new()
    } else {
        format!(", {}", args.join(", "))
    }
}

fn compose_modifier_call_args(style: Option<&ViewStyle>) -> String {
    compose_modifier(style)
        .map(|modifier| format!("(modifier = {modifier})"))
        .unwrap_or_else(|| "()".to_string())
}

fn compose_modifier_param(style: Option<&ViewStyle>) -> String {
    compose_modifier(style)
        .map(|modifier| format!(", modifier = {modifier}"))
        .unwrap_or_default()
}

fn compose_button_args(style: Option<&ViewStyle>) -> String {
    let mut args = Vec::new();
    if let Some(style) = style {
        let mut modifier_style = style.clone();
        modifier_style.background_color = None;
        modifier_style.foreground_color = None;
        modifier_style.padding = None;
        modifier_style.padding_horizontal = None;
        modifier_style.padding_vertical = None;
        modifier_style.padding_top = None;
        modifier_style.padding_bottom = None;
        modifier_style.padding_left = None;
        modifier_style.padding_right = None;
        if let Some(modifier) = compose_modifier(Some(&modifier_style)) {
            args.push(format!("modifier = {modifier}"));
        }
        if style.background_color.is_some() || style.foreground_color.is_some() {
            let container = style
                .background_color
                .as_deref()
                .map(compose_hex_argb)
                .unwrap_or_else(|| "FF6750A4".to_string());
            let content = style
                .foreground_color
                .as_deref()
                .map(compose_hex_argb)
                .unwrap_or_else(|| "FFFFFFFF".to_string());
            args.push(format!(
                "colors = ButtonDefaults.buttonColors(containerColor = Color(0x{container}), contentColor = Color(0x{content}))"
            ));
            args.push(
                "elevation = ButtonDefaults.buttonElevation(defaultElevation = 0.dp, pressedElevation = 0.dp, focusedElevation = 0.dp, hoveredElevation = 0.dp, disabledElevation = 0.dp)"
                    .to_string(),
            );
        }
        if let Some(padding) = compose_button_content_padding(style) {
            args.push(format!("contentPadding = {padding}"));
        }
    }
    if args.is_empty() {
        String::new()
    } else {
        format!(", {}", args.join(", "))
    }
}

fn compose_button_content_padding(style: &ViewStyle) -> Option<String> {
    if let Some(value) = style.padding {
        return Some(format!("PaddingValues({value:.0}.dp)"));
    }
    if style.padding_horizontal.is_some() || style.padding_vertical.is_some() {
        return Some(format!(
            "PaddingValues(horizontal = {:.0}.dp, vertical = {:.0}.dp)",
            style.padding_horizontal.unwrap_or(0.0),
            style.padding_vertical.unwrap_or(0.0)
        ));
    }
    if style.padding_top.is_some()
        || style.padding_bottom.is_some()
        || style.padding_left.is_some()
        || style.padding_right.is_some()
    {
        return Some(format!(
            "PaddingValues(start = {:.0}.dp, top = {:.0}.dp, end = {:.0}.dp, bottom = {:.0}.dp)",
            style.padding_left.unwrap_or(0.0),
            style.padding_top.unwrap_or(0.0),
            style.padding_right.unwrap_or(0.0),
            style.padding_bottom.unwrap_or(0.0)
        ));
    }
    None
}

fn compose_modifier(style: Option<&ViewStyle>) -> Option<String> {
    compose_modifier_chain(None, style)
}

fn compose_modifier_chain(base: Option<String>, style: Option<&ViewStyle>) -> Option<String> {
    let mut modifier = base.unwrap_or_else(|| "Modifier".to_string());
    let mut used = modifier != "Modifier";
    if let Some(style) = style {
        for value in compose_spacing_values(style, "margin") {
            modifier.push_str(&format!(".padding({value})"));
            used = true;
        }
        if style.width == Some(-1.0) && style.height == Some(-1.0) {
            modifier.push_str(".fillMaxSize()");
            used = true;
        } else {
            if style.width == Some(-1.0) || style.max_width == Some(-1.0) {
                modifier.push_str(".fillMaxWidth()");
                used = true;
            } else if let Some(width) = style.width.filter(|v| *v > 0.0) {
                modifier.push_str(&format!(".width({width:.0}.dp)"));
                used = true;
            }
            if style.height == Some(-1.0) || style.max_height == Some(-1.0) {
                modifier.push_str(".fillMaxHeight()");
                used = true;
            } else if let Some(height) = style.height.filter(|v| *v > 0.0) {
                modifier.push_str(&format!(".height({height:.0}.dp)"));
                used = true;
            }
        }
        if let Some(color) = &style.background_color {
            if let Some(radius) = style.corner_radius {
                modifier.push_str(&format!(".clip(RoundedCornerShape({radius:.0}.dp))"));
            }
            modifier.push_str(&format!(
                ".background(Color(0x{}))",
                compose_hex_argb(color)
            ));
            used = true;
        }
        if let Some(width) = style.border_width {
            let color = style
                .border_color
                .as_deref()
                .map(compose_hex_argb)
                .unwrap_or_else(|| "FF888888".to_string());
            let radius = style.corner_radius.unwrap_or(0.0);
            modifier.push_str(&format!(
                ".border({width:.0}.dp, Color(0x{color}), RoundedCornerShape({radius:.0}.dp))"
            ));
            used = true;
        }
        for value in compose_spacing_values(style, "padding") {
            modifier.push_str(&format!(".padding({value})"));
            used = true;
        }
        if let Some(opacity) = style.opacity {
            modifier.push_str(&format!(".alpha({opacity:.3}f)"));
            used = true;
        }
        if style.hidden == Some(true) {
            modifier.push_str(".alpha(0f)");
            used = true;
        }
        if style.translate_x.is_some() || style.translate_y.is_some() {
            modifier.push_str(&format!(
                ".offset(x = {:.0}.dp, y = {:.0}.dp)",
                style.translate_x.unwrap_or(0.0),
                style.translate_y.unwrap_or(0.0)
            ));
            used = true;
        }
        if let Some(rotate) = style.rotate {
            modifier.push_str(&format!(".rotate({rotate:.1}f)"));
            used = true;
        }
        if style.scale_x.is_some() || style.scale_y.is_some() {
            modifier.push_str(&format!(
                ".scale(scaleX = {:.3}f, scaleY = {:.3}f)",
                style.scale_x.unwrap_or(1.0),
                style.scale_y.unwrap_or(1.0)
            ));
            used = true;
        }
    }
    used.then_some(modifier)
}

fn compose_spacing_values(style: &ViewStyle, kind: &str) -> Vec<String> {
    let (all, horizontal, vertical, top, bottom, left, right) = if kind == "padding" {
        (
            style.padding,
            style.padding_horizontal,
            style.padding_vertical,
            style.padding_top,
            style.padding_bottom,
            style.padding_left,
            style.padding_right,
        )
    } else {
        (
            style.margin,
            style.margin_horizontal,
            style.margin_vertical,
            style.margin_top,
            style.margin_bottom,
            style.margin_left,
            style.margin_right,
        )
    };
    let mut out = Vec::new();
    if let Some(value) = all {
        out.push(format!("{value:.0}.dp"));
    }
    if horizontal.is_some() || vertical.is_some() {
        out.push(format!(
            "horizontal = {:.0}.dp, vertical = {:.0}.dp",
            horizontal.unwrap_or(0.0),
            vertical.unwrap_or(0.0)
        ));
    }
    if top.is_some() || bottom.is_some() || left.is_some() || right.is_some() {
        out.push(format!(
            "start = {:.0}.dp, top = {:.0}.dp, end = {:.0}.dp, bottom = {:.0}.dp",
            left.unwrap_or(0.0),
            top.unwrap_or(0.0),
            right.unwrap_or(0.0),
            bottom.unwrap_or(0.0)
        ));
    }
    out
}

fn compose_text_align(value: Option<&str>) -> Option<&'static str> {
    match value {
        Some("center") => Some("TextAlign.Center"),
        Some("right") | Some("end") | Some("trailing") => Some("TextAlign.End"),
        Some("justify") => Some("TextAlign.Justify"),
        Some("left") | Some("start") | Some("leading") => Some("TextAlign.Start"),
        _ => None,
    }
}

fn compose_font_weight(weight: u16) -> &'static str {
    match weight {
        0..=299 => "FontWeight.Thin",
        300..=399 => "FontWeight.Light",
        400..=499 => "FontWeight.Normal",
        500..=599 => "FontWeight.Medium",
        600..=699 => "FontWeight.SemiBold",
        700..=799 => "FontWeight.Bold",
        _ => "FontWeight.ExtraBold",
    }
}

fn compose_hex_argb(color: &str) -> String {
    let trimmed = color.trim_start_matches('#');
    match trimmed.len() {
        6 => format!("FF{}", trimmed.to_ascii_uppercase()),
        8 => trimmed.to_ascii_uppercase(),
        _ => "FF888888".to_string(),
    }
}

fn swift_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn swift_color(color: &str) -> String {
    let trimmed = color.trim_start_matches('#');
    let Some((r, g, b, a)) = parse_hex_rgba(trimmed) else {
        return "Color.gray".to_string();
    };
    format!(
        "Color(red: {:.3}, green: {:.3}, blue: {:.3}, opacity: {:.3})",
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0
    )
}

fn parse_hex_rgba(hex: &str) -> Option<(u8, u8, u8, u8)> {
    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    if hex.len() == 8 {
        Some((
            ((value & 0x00ff0000) >> 16) as u8,
            ((value & 0x0000ff00) >> 8) as u8,
            (value & 0x000000ff) as u8,
            ((value & 0xff000000) >> 24) as u8,
        ))
    } else {
        Some((
            ((value & 0xff0000) >> 16) as u8,
            ((value & 0x00ff00) >> 8) as u8,
            (value & 0x0000ff) as u8,
            255,
        ))
    }
}

fn kotlin_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn swift_bool(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn kotlin_bool(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn indent_str(level: usize) -> String {
    "    ".repeat(level)
}

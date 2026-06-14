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
    let body = swiftui_nodes(&ir.root, 2);
    format!(
        "import SwiftUI\n\npublic enum CrepusActions {{\n    public static var dispatch: (String) -> Void = {{ _ in }}\n}}\n\npublic struct {view_name}: View {{\n    public init() {{}}\n\n    public var body: some View {{\n{body}\n    }}\n}}\n"
    )
}

fn swiftui_nodes(nodes: &[ViewNode], indent: usize) -> String {
    if nodes.len() == 1 {
        swiftui_node(&nodes[0], indent)
    } else {
        let pad = indent_str(indent);
        let inner = nodes
            .iter()
            .map(|node| swiftui_node(node, indent + 1))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{pad}Group {{\n{inner}\n{pad}}}")
    }
}

fn swiftui_node(node: &ViewNode, indent: usize) -> String {
    let pad = indent_str(indent);
    match node {
        ViewNode::Text { content, style } => {
            let mut out = format!("{pad}Text(\"{}\")", swift_escape(content));
            swiftui_style(&mut out, style.as_ref(), true, indent);
            out
        }
        ViewNode::Stack {
            axis,
            spacing,
            align_items,
            style,
            children,
            ..
        } => {
            let view = match axis {
                StackAxis::Row => "HStack",
                StackAxis::Column => "VStack",
            };
            let align = swiftui_stack_alignment(*axis, align_items.as_deref());
            let gap = spacing.unwrap_or(8.0);
            let inner = swiftui_children(children, indent + 1);
            let mut out =
                format!("{pad}{view}(alignment: {align}, spacing: {gap:.1}) {{\n{inner}\n{pad}}}");
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
            checked,
            style,
            ..
        } => {
            let mut out = format!(
                "{pad}Toggle(\"{}\", isOn: .constant({}))",
                swift_escape(label),
                swift_bool(*checked)
            );
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Checkbox {
            label,
            checked,
            style,
            ..
        } => {
            let mut out = format!(
                "{pad}Toggle(\"{}\", isOn: .constant({}))",
                swift_escape(label),
                swift_bool(*checked)
            );
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Slider {
            label,
            value,
            min,
            max,
            step,
            style,
            ..
        } => {
            let step_arg = step.map(|v| format!(", step: {v:.1}")).unwrap_or_default();
            let control =
                format!("Slider(value: .constant({value:.3}), in: {min:.3}...{max:.3}{step_arg})");
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
                swiftui_children(children, indent + 1)
            };
            let mut out =
                format!("{pad}VStack(alignment: .leading, spacing: 8.0) {{\n{inner}\n{pad}}}");
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
                        swiftui_node(child, indent + 2),
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
            let inner = swiftui_children(children, indent + 1);
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
            multiline,
            style,
            ..
        } => {
            let mut out = if *multiline {
                format!(
                    "{pad}TextEditor(text: .constant(\"{}\"))",
                    swift_escape(placeholder)
                )
            } else {
                format!(
                    "{pad}TextField(\"{}\", text: .constant(\"\"))",
                    swift_escape(placeholder)
                )
            };
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
        ViewNode::Picker { options, style, .. } => {
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
                "{pad}Picker(\"\", selection: .constant(\"{}\")) {{\n{rows}\n{pad}}}",
                swift_escape(first)
            );
            swiftui_style(&mut out, style.as_ref(), false, indent);
            out
        }
    }
}

fn swiftui_action(on_click: Option<&str>) -> String {
    on_click
        .map(|action| format!("CrepusActions.dispatch(\"{}\")", swift_escape(action)))
        .unwrap_or_default()
}

fn swiftui_children(children: &[ViewNode], indent: usize) -> String {
    children
        .iter()
        .map(|child| swiftui_node(child, indent))
        .collect::<Vec<_>>()
        .join("\n")
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
    if is_text {
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
    let body = compose_nodes(&ir.root, 1);
    format!(
        "import androidx.compose.foundation.background\nimport androidx.compose.foundation.border\nimport androidx.compose.foundation.horizontalScroll\nimport androidx.compose.foundation.layout.Arrangement\nimport androidx.compose.foundation.layout.Column\nimport androidx.compose.foundation.layout.Row\nimport androidx.compose.foundation.layout.Spacer\nimport androidx.compose.foundation.layout.fillMaxHeight\nimport androidx.compose.foundation.layout.fillMaxSize\nimport androidx.compose.foundation.layout.fillMaxWidth\nimport androidx.compose.foundation.layout.height\nimport androidx.compose.foundation.layout.offset\nimport androidx.compose.foundation.layout.padding\nimport androidx.compose.foundation.layout.width\nimport androidx.compose.foundation.rememberScrollState\nimport androidx.compose.foundation.shape.RoundedCornerShape\nimport androidx.compose.foundation.verticalScroll\nimport androidx.compose.material3.Button\nimport androidx.compose.material3.Divider\nimport androidx.compose.material3.LinearProgressIndicator\nimport androidx.compose.material3.Slider\nimport androidx.compose.material3.Switch\nimport androidx.compose.material3.Text\nimport androidx.compose.material3.TextField\nimport androidx.compose.runtime.Composable\nimport androidx.compose.ui.Modifier\nimport androidx.compose.ui.draw.alpha\nimport androidx.compose.ui.draw.clip\nimport androidx.compose.ui.draw.rotate\nimport androidx.compose.ui.draw.scale\nimport androidx.compose.ui.graphics.Color\nimport androidx.compose.ui.text.font.FontStyle\nimport androidx.compose.ui.text.font.FontWeight\nimport androidx.compose.ui.text.style.TextAlign\nimport androidx.compose.ui.text.style.TextDecoration\nimport androidx.compose.ui.unit.dp\nimport androidx.compose.ui.unit.sp\n\nobject CrepusActions {{\n    var dispatch: (String) -> Unit = {{}}\n}}\n\n@Composable\nfun {view_name}(modifier: Modifier = Modifier) {{\n{body}\n}}\n"
    )
}

fn compose_nodes(nodes: &[ViewNode], indent: usize) -> String {
    if nodes.len() == 1 {
        compose_node_with_base(&nodes[0], indent, Some("modifier".to_string()))
    } else {
        let pad = indent_str(indent);
        let inner = compose_children(nodes, indent + 1);
        format!("{pad}Column {{\n{inner}\n{pad}}}")
    }
}

fn compose_node(node: &ViewNode, indent: usize) -> String {
    compose_node_with_base(node, indent, None)
}

fn compose_node_with_base(node: &ViewNode, indent: usize, base_modifier: Option<String>) -> String {
    let pad = indent_str(indent);
    match node {
        ViewNode::Text { content, style } => {
            let args = compose_text_args(style.as_ref());
            format!("{pad}Text(\"{}\"{args})", kotlin_escape(content))
        }
        ViewNode::Stack {
            axis,
            spacing,
            style,
            children,
            ..
        } => {
            let view = match axis {
                StackAxis::Row => "Row",
                StackAxis::Column => "Column",
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
            let args = args.join(", ");
            let inner = compose_children(children, indent + 1);
            format!("{pad}{view}({args}) {{\n{inner}\n{pad}}}")
        }
        ViewNode::Button {
            label,
            on_click,
            style,
        } => {
            let modifier = compose_modifier_param(style.as_ref());
            let action = compose_action(on_click.as_deref());
            format!(
                "{pad}Button(onClick = {{ {action} }}{modifier}) {{\n{}Text(\"{}\")\n{pad}}}",
                indent_str(indent + 1),
                kotlin_escape(label)
            )
        }
        ViewNode::Toggle {
            label,
            checked,
            style,
            ..
        }
        | ViewNode::Checkbox {
            label,
            checked,
            style,
            ..
        } => {
            let modifier = compose_modifier_call_args(style.as_ref());
            format!(
                "{pad}Row{modifier} {{\n{}Text(\"{}\")\n{}Switch(checked = {}, onCheckedChange = {{}})\n{pad}}}",
                indent_str(indent + 1),
                kotlin_escape(label),
                indent_str(indent + 1),
                kotlin_bool(*checked)
            )
        }
        ViewNode::Slider {
            label,
            value,
            min,
            max,
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
            format!(
                "{pad}Column{modifier} {{\n{label}{}Slider(value = {value:.3}f, onValueChange = {{}}, valueRange = {min:.3}f..{max:.3}f)\n{pad}}}",
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
                compose_children(children, indent + 1)
            };
            format!("{pad}Column{modifier} {{\n{inner}\n{pad}}}")
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
            let inner = compose_children(children, indent + 1);
            format!(
                "{pad}{view}(modifier = {}) {{\n{inner}\n{pad}}}",
                modifier.unwrap_or_else(|| "Modifier".to_string())
            )
        }
        ViewNode::List { children, .. } | ViewNode::ListItem { children, .. } => {
            let inner = compose_children(children, indent + 1);
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
            placeholder, style, ..
        } => {
            let modifier = compose_modifier_param(style.as_ref());
            format!(
                "{pad}TextField(value = \"\", onValueChange = {{}}, placeholder = {{ Text(\"{}\") }}{modifier})",
                kotlin_escape(placeholder)
            )
        }
        ViewNode::Picker { options, style, .. } => {
            let modifier = compose_modifier_call_args(style.as_ref());
            let inner = options
                .iter()
                .map(|option| {
                    format!(
                        "{}Text(\"{}\")",
                        indent_str(indent + 1),
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
        .map(|action| format!("CrepusActions.dispatch(\"{}\")", kotlin_escape(action)))
        .unwrap_or_default()
}

fn compose_children(children: &[ViewNode], indent: usize) -> String {
    children
        .iter()
        .map(|child| compose_node(child, indent))
        .collect::<Vec<_>>()
        .join("\n")
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

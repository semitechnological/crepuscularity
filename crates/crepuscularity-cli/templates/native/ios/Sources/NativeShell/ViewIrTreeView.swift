import SwiftUI

public struct ViewIrRootView: View {
    private let ir: ViewIr

    public init(ir: ViewIr) {
        self.ir = ir
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            ForEach(Array(ir.root.enumerated()), id: \.offset) { _, node in
                ViewNodeView(node: node)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

public struct ViewNodeView: View {
    let node: ViewNode
    @ObservedObject private var actionStore = CrepusActionStore.shared

    public init(node: ViewNode) {
        self.node = node
    }

    public var body: some View {
        switch node {
        case .text(let content, let style):
            Text(content)
                .applyViewStyle(style, isText: true)

        case .stack(let axis, let spacing, let alignItems, let justifyContent, let style, let children):
            let gap = CGFloat(spacing ?? 8)
            let hAlign = horizontalAlignment(from: alignItems)
            Group {
                switch axis {
                case .column:
                    VStack(alignment: hAlign, spacing: gap) {
                        ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                            ViewNodeView(node: child)
                        }
                    }
                case .row:
                    HStack(alignment: rowAlignment(from: alignItems), spacing: gap) {
                        ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                            ViewNodeView(node: child)
                        }
                    }
                }
            }
            .applyViewStyle(style, isText: false)
            .modifier(JustifyOverlay(justify: justifyContent))

        case .button(let label, let onClick, let style):
            Button(action: {
                if let onClick { actionStore.dispatch(onClick) }
            }) {
                Text(label)
                    .modifier(ConditionalForeground(color: style?.foregroundColor.flatMap { Color(hex: $0) }))
            }
            .applyViewStyle(style, isText: false)

        case .toggle(let label, _, let checked, let onChange, let style):
            ToggleNodeView(label: label, checked: checked, onChange: onChange, style: style)

        case .checkbox(let label, _, let checked, let onChange, let style):
            CheckboxNodeView(label: label, checked: checked, onChange: onChange, style: style)

        case .slider(let label, _, let value, let min, let max, let step, let style):
            SliderNodeView(label: label, value: value, min: min, max: max, step: step, style: style)

        case .progress(let label, let value, let max, let style):
            ProgressNodeView(label: label, value: value, min: 0, max: max, style: style)

        case .meter(let label, let value, let min, let max, let style):
            ProgressNodeView(label: label, value: value, min: min, max: max, style: style)

        case .badge(let label, let tone, let style):
            Text(label)
                .font(.caption.weight(.semibold))
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(badgeColor(tone))
                .foregroundStyle(.white)
                .clipShape(Capsule())
                .applyViewStyle(style, isText: false)

        case .divider(let axis, let style):
            DividerNodeView(axis: axis)
                .applyViewStyle(style, isText: false)

        case .spacer(let size, let style):
            Color.clear
                .frame(height: CGFloat(size ?? 8))
                .applyViewStyle(style, isText: false)

        case .dropzone(let label, _, let onDrop, let style, let children):
            Button(action: {
                if let onDrop { actionStore.dispatch(onDrop) }
            }) {
                VStack(alignment: .leading, spacing: 8) {
                    if children.isEmpty {
                        Text(label)
                    } else {
                        ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                            ViewNodeView(node: child)
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .buttonStyle(.plain)
            .applyViewStyle(style, isText: false)

        case .filePicker(let label, _, _, let onPick, let style):
            Button(action: {
                if let onPick { actionStore.dispatch(onPick) }
            }) {
                Label(label, systemImage: "paperclip")
            }
            .applyViewStyle(style, isText: false)

        case .image(let src, let alt, let style):
            Group {
                if let url = URL(string: src) {
                    AsyncImage(url: url) { phase in
                        switch phase {
                        case .success(let image):
                            image.resizable().scaledToFit()
                        default:
                            VStack(alignment: .leading) {
                                Text(alt ?? src)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                Text("Loading image")
                                    .font(.caption2)
                            }
                        }
                    }
                } else {
                    Text(alt ?? src)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .applyViewStyle(style, isText: false)

        case .scroll(let axis, let style, let children):
            let gap: CGFloat = 8
            ScrollView {
                switch axis {
                case .column:
                    VStack(alignment: .leading, spacing: gap) {
                        ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                            ViewNodeView(node: child)
                        }
                    }
                case .row:
                    HStack(alignment: .top, spacing: gap) {
                        ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                            ViewNodeView(node: child)
                        }
                    }
                }
            }
            .applyViewStyle(style, isText: false)

        case .list(let ordered, let style, let children):
            VStack(alignment: .leading, spacing: 8) {
                ForEach(Array(children.enumerated()), id: \.offset) { index, child in
                    HStack(alignment: .top, spacing: 8) {
                        Text(ordered ? "\(index + 1)." : "•")
                        ViewNodeView(node: child)
                    }
                }
            }
            .applyViewStyle(style, isText: false)

        case .listItem(let style, let children):
            VStack(alignment: .leading, spacing: 8) {
                ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                    ViewNodeView(node: child)
                }
            }
            .applyViewStyle(style, isText: false)

        case .slotRotate(let phrases, _, let style):
            // Full rotation needs a timer; show first phrase in the shell demo.
            Text(phrases.first ?? "")
                .applyViewStyle(style, isText: true)

        case .input(let placeholder, let bind, let multiline, let style):
            InputNodeView(
                placeholder: placeholder,
                initialValue: "",
                multiline: multiline,
                style: style
            )

        case .picker(let bind, let options, let style):
            PickerNodeView(bind: bind, options: options, style: style)
        }
    }

    private func horizontalAlignment(from alignItems: String?) -> HorizontalAlignment {
        switch alignItems {
        case "end": return .trailing
        case "center": return .center
        default: return .leading
        }
    }

    private func rowAlignment(from alignItems: String?) -> VerticalAlignment {
        switch alignItems {
        case "end": return .bottom
        case "center": return .center
        case "stretch": return .center
        default: return .top
        }
    }
}

private struct ToggleNodeView: View {
    let label: String
    let onChange: String?
    let style: ViewStyle?
    @ObservedObject private var actionStore = CrepusActionStore.shared
    @State private var value: Bool

    init(label: String, checked: Bool, onChange: String?, style: ViewStyle?) {
        self.label = label
        self.onChange = onChange
        self.style = style
        _value = State(initialValue: checked)
    }

    var body: some View {
        Toggle(label, isOn: $value)
            .onChange(of: value) { _, _ in
                if let onChange { actionStore.dispatch(onChange) }
            }
            .applyViewStyle(style, isText: false)
    }
}

private struct CheckboxNodeView: View {
    let label: String
    let onChange: String?
    let style: ViewStyle?
    @ObservedObject private var actionStore = CrepusActionStore.shared
    @State private var value: Bool

    init(label: String, checked: Bool, onChange: String?, style: ViewStyle?) {
        self.label = label
        self.onChange = onChange
        self.style = style
        _value = State(initialValue: checked)
    }

    var body: some View {
        Button(action: {
            value.toggle()
            if let onChange { actionStore.dispatch(onChange) }
        }) {
            HStack(spacing: 8) {
                Image(systemName: value ? "checkmark.square.fill" : "square")
                Text(label)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .buttonStyle(.plain)
        .applyViewStyle(style, isText: false)
    }
}

private struct SliderNodeView: View {
    let label: String?
    let min: Float
    let max: Float
    let step: Float?
    let style: ViewStyle?
    @State private var value: Double

    init(label: String?, value: Float, min: Float, max: Float, step: Float?, style: ViewStyle?) {
        self.label = label
        self.min = min
        self.max = max
        self.step = step
        self.style = style
        _value = State(initialValue: Double(value))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if let label {
                Text("\(label) \(Int(value.rounded()))")
            }
            Slider(
                value: $value,
                in: Double(min)...Double(max),
                step: Double(step ?? 1)
            )
        }
        .applyViewStyle(style, isText: false)
    }
}

private struct ProgressNodeView: View {
    let label: String?
    let value: Float
    let min: Float
    let max: Float
    let style: ViewStyle?

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if let label {
                Text(label)
            }
            ProgressView(value: progressValue)
        }
        .applyViewStyle(style, isText: false)
    }

    private var progressValue: Double {
        let span = max - min
        guard span > 0 else { return 0 }
        return Double((value - min) / span)
    }
}

private struct DividerNodeView: View {
    let axis: StackAxis

    var body: some View {
        switch axis {
        case .column:
            Divider()
        case .row:
            Rectangle()
                .fill(Color.secondary.opacity(0.2))
                .frame(width: 1, height: 24)
        }
    }
}

private struct InputNodeView: View {
    let placeholder: String
    let multiline: Bool
    let style: ViewStyle?
    @State private var value: String

    init(placeholder: String, initialValue: String, multiline: Bool, style: ViewStyle?) {
        self.placeholder = placeholder
        self.multiline = multiline
        self.style = style
        _value = State(initialValue: initialValue)
    }

    var body: some View {
        Group {
            if multiline {
                TextEditor(text: $value)
                    .frame(minHeight: 96)
            } else {
                TextField(placeholder, text: $value)
                    .textFieldStyle(.roundedBorder)
            }
        }
        .applyViewStyle(style, isText: false)
    }
}

private struct PickerNodeView: View {
    let options: [PickerOption]
    let style: ViewStyle?
    @State private var selection: String

    init(bind: String, options: [PickerOption], style: ViewStyle?) {
        self.options = options
        self.style = style
        let fallback = options.first?.value ?? normalizedBindingValue(bind)
        _selection = State(initialValue: fallback)
    }

    var body: some View {
        Picker("", selection: $selection) {
            ForEach(options, id: \.value) { option in
                Text(option.label).tag(option.value)
            }
        }
        .pickerStyle(.segmented)
        .applyViewStyle(style, isText: false)
    }
}

/// Best-effort `justify-content` for simple stacks (shell demo).
private struct JustifyOverlay: ViewModifier {
    let justify: String?

    func body(content: Content) -> some View {
        switch justify {
        case "center":
            content.frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
        case "end":
            content.frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .trailing)
        default:
            content
        }
    }
}

extension View {
    @ViewBuilder
    fileprivate func applyViewStyle(_ style: ViewStyle?, isText: Bool) -> some View {
        if let s = style {
            self.modifier(ViewStyleModifier(style: s, isText: isText))
        } else {
            self
        }
    }
}

private struct ViewStyleModifier: ViewModifier {
    let style: ViewStyle
    let isText: Bool

    func body(content: Content) -> some View {
        let pad = EdgeInsets(
            top: CGFloat(style.paddingTop ?? style.paddingVertical ?? style.padding ?? 0),
            leading: CGFloat(style.paddingLeft ?? style.paddingHorizontal ?? style.padding ?? 0),
            bottom: CGFloat(style.paddingBottom ?? style.paddingVertical ?? style.padding ?? 0),
            trailing: CGFloat(style.paddingRight ?? style.paddingHorizontal ?? style.padding ?? 0)
        )
        let hasPadding = style.padding != nil || style.paddingHorizontal != nil
            || style.paddingVertical != nil || style.paddingTop != nil || style.paddingBottom != nil
            || style.paddingLeft != nil || style.paddingRight != nil

        let frameWidth = frameAxis(width: style.width, min: style.minWidth, max: style.maxWidth)
        let frameHeight = frameAxis(width: style.height, min: style.minHeight, max: style.maxHeight)

        let base = content
            .frame(
                minWidth: frameWidth.min,
                idealWidth: frameWidth.ideal,
                maxWidth: frameWidth.max,
                minHeight: frameHeight.min,
                idealHeight: frameHeight.ideal,
                maxHeight: frameHeight.max,
                alignment: .topLeading
            )
            .modifier(ConditionalPadding(pad: pad, enabled: hasPadding))
            .background((style.backgroundColor.flatMap { Color(hex: $0) }) ?? Color.clear)
            .modifier(ConditionalForeground(color: style.foregroundColor.flatMap { Color(hex: $0) }))
            .overlay(
                RoundedRectangle(cornerRadius: CGFloat(style.cornerRadius ?? 0))
                    .stroke(
                        style.borderColor.flatMap { Color(hex: $0) } ?? Color.secondary.opacity(style.borderWidth == nil ? 0 : 0.3),
                        lineWidth: CGFloat(style.borderWidth ?? 0)
                    )
            )
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(style.cornerRadius ?? 0)))

        Group {
            if isText {
                textDecorations(base)
            } else {
                base
            }
        }
    }

    @ViewBuilder
    private func textDecorations(_ v: some View) -> some View {
        let w = style.fontWeight.map(fontWeightSwiftUI)
        let withFont = v
            .font(style.fontSize.map { .system(size: CGFloat($0)) } ?? .body)
            .fontWeight(w ?? .regular)
            .modifier(ConditionalForeground(color: style.foregroundColor.flatMap { Color(hex: $0) }))
            .italic(style.italic == true)
            .underline(style.underline == true)
            .strikethrough(style.strikethrough == true)
        switch style.textAlign {
        case "center":
            withFont.frame(maxWidth: .infinity, alignment: .center)
        case "trailing":
            withFont.frame(maxWidth: .infinity, alignment: .trailing)
        default:
            withFont
        }
    }
}

private func badgeColor(_ tone: String?) -> Color {
    switch tone {
    case "success": return .green
    case "warning": return .orange
    case "danger": return .red
    default: return Color.black.opacity(0.75)
    }
}

private func normalizedBindingValue(_ bind: String) -> String {
    let trimmed = bind.trimmingCharacters(in: .whitespacesAndNewlines)
    if trimmed.count >= 2, trimmed.hasPrefix("\""), trimmed.hasSuffix("\"") {
        return String(trimmed.dropFirst().dropLast())
    }
    return trimmed
}

private func frameAxis(width: Float?, min: Float?, max: Float?) -> (min: CGFloat?, ideal: CGFloat?, max: CGFloat?) {
    let minValue = min.map { CGFloat($0) }
    let idealValue = width.flatMap { $0 > 0 ? CGFloat($0) : nil }
    let maxValue: CGFloat?
    if width == -1 || max == -1 {
        maxValue = .infinity
    } else {
        maxValue = max.flatMap { $0 > 0 ? CGFloat($0) : nil }
    }
    return (minValue, idealValue, maxValue)
}

private struct ConditionalForeground: ViewModifier {
    let color: Color?

    func body(content: Content) -> some View {
        if let color {
            content.foregroundStyle(color)
        } else {
            content
        }
    }
}

private struct ConditionalPadding: ViewModifier {
    let pad: EdgeInsets
    let enabled: Bool

    func body(content: Content) -> some View {
        if enabled {
            content.padding(pad)
        } else {
            content
        }
    }
}

private func fontWeightSwiftUI(_ w: UInt16) -> Font.Weight {
    switch w {
    case ..<300: return .thin
    case ..<400: return .light
    case ..<500: return .regular
    case ..<600: return .medium
    case ..<700: return .semibold
    case ..<800: return .bold
    default: return .heavy
    }
}

extension Color {
    init?(hex: String) {
        var t = hex.trimmingCharacters(in: .whitespaces)
        if t.hasPrefix("#") { t.removeFirst() }
        guard t.count == 6 || t.count == 8 else { return nil }
        var n: UInt64 = 0
        guard Scanner(string: t).scanHexInt64(&n) else { return nil }
        let a: Double
        let r: Double
        let g: Double
        let b: Double
        if t.count == 8 {
            a = Double((n & 0xFF00_0000) >> 24) / 255
            r = Double((n & 0x00FF_0000) >> 16) / 255
            g = Double((n & 0x0000_FF00) >> 8) / 255
            b = Double(n & 0x0000_00FF) / 255
        } else {
            a = 1
            r = Double((n & 0xFF0000) >> 16) / 255
            g = Double((n & 0x00FF00) >> 8) / 255
            b = Double(n & 0x0000FF) / 255
        }
        self = Color(.sRGB, red: r, green: g, blue: b, opacity: a)
    }
}

/// Loads bundled fixture and shows the tree (for Xcode Previews or embedding in an app).
public struct FixtureRootView: View {
    @State private var ir: ViewIr?
    @State private var errorText: String?

    private let bundle: Bundle

    public init(bundle: Bundle = NativeShellResources.bundle) {
        self.bundle = bundle
    }

    public var body: some View {
        Group {
            if let ir {
                ViewIrRootView(ir: ir)
            } else if let errorText {
                Text(errorText).foregroundStyle(.red)
            } else {
                ProgressView()
            }
        }
        .task {
            do {
                ir = try ViewIr.loadFixture(bundle: bundle)
            } catch {
                errorText = String(describing: error)
            }
        }
    }
}

#if DEBUG
#Preview("Fixture") {
    FixtureRootView()
}
#endif

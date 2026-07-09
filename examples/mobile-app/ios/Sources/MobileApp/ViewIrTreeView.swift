import SwiftUI

/// Root view that loads `fixture.json` from the SwiftPM bundle and renders the View IR tree.
public struct FixtureRootView: View {
    @State private var ir: ViewIr?

    public init() {}

    public var body: some View {
        Group {
            if let ir {
                ViewIrRootView(ir: ir)
            } else {
                ProgressView()
                    .task { loadFixture() }
            }
        }
    }

    private func loadFixture() {
        guard let url = MobileAppResources.bundle.url(forResource: "fixture", withExtension: "json"),
              let data = try? Data(contentsOf: url),
              let decoded = try? JSONDecoder().decode(ViewIr.self, from: data)
        else { return }
        ir = decoded
    }
}

/// Renders the top-level `ViewIr` by composing all root nodes.
public struct ViewIrRootView: View {
    public let ir: ViewIr

    public init(ir: ViewIr) {
        self.ir = ir
    }

    public var body: some View {
        VStack(spacing: 0) {
            ForEach(Array(ir.root.enumerated()), id: \.offset) { _, node in
                ViewNodeView(node: node)
            }
        }
    }
}

/// Recursively renders a single `ViewNode` into SwiftUI.
public struct ViewNodeView: View {
    public let node: ViewNode

    public init(node: ViewNode) {
        self.node = node
    }

    public var body: some View {
        switch node {
        case .text(let t):
            textView(t)
        case .button(let b):
            buttonView(b)
        case .toggle(let t):
            toggleView(t)
        case .slider(let s):
            sliderView(s)
        case .input(let i):
            inputView(i)
        case .stack(let s):
            stackView(s)
        case .scroll(let s):
            scrollView(s)
        case .ifNode(let i):
            ifView(i)
        case .forEach(let f):
            forEachView(f)
        case .tabs(let t):
            tabsView(t)
        case .tab(let t):
            tabView(t)
        case .unknown:
            EmptyView()
        }
    }

    @ViewBuilder
    private func textView(_ node: TextNode) -> some View {
        let text = node.content ?? node.bind ?? ""
        styledText(Text(text), style: node.style)
    }

    @ViewBuilder
    private func buttonView(_ node: ButtonNode) -> some View {
        let s = node.style
        Button(action: { performAction(node.onClick) }) {
            styledText(Text(node.label), style: s)
                .padding(.horizontal, cg(s?.paddingHorizontal ?? s?.padding))
                .padding(.vertical, cg(s?.paddingVertical ?? s?.padding))
                .background(colorFrom(s?.backgroundColor))
                .foregroundColor(colorFrom(s?.foregroundColor))
                .cornerRadius(cg(s?.cornerRadius))
        }
    }

    @ViewBuilder
    private func toggleView(_ node: ToggleNode) -> some View {
        let label = node.label ?? ""
        Toggle(label, isOn: .constant(node.checked ?? false))
            .onChange(of: node.checked ?? false) { _ in performAction(node.onChange) }
    }

    @ViewBuilder
    private func sliderView(_ node: SliderNode) -> some View {
        Slider(value: .constant(Double(node.value ?? 0)),
               in: Double(node.min ?? 0)...Double(node.max ?? 100),
               step: Double(node.step ?? 1))
    }

    @ViewBuilder
    private func inputView(_ node: InputNode) -> some View {
        if node.multiline == true {
            TextEditor(text: .constant(""))
                .frame(minHeight: 80)
        } else {
            TextField(node.placeholder ?? "", text: .constant(""))
        }
    }

    @ViewBuilder
    private func stackView(_ node: StackNode) -> some View {
        let axis = node.axis ?? .column
        let spacing = cg(node.spacing)
        let children = node.children ?? []
        if axis == .row {
            HStack(spacing: spacing) {
                ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                    ViewNodeView(node: child)
                }
            }
            .applyStyle(node.style)
        } else {
            VStack(spacing: spacing) {
                ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                    ViewNodeView(node: child)
                }
            }
            .applyStyle(node.style)
        }
    }

    @ViewBuilder
    private func scrollView(_ node: ScrollNode) -> some View {
        let axis = node.axis ?? .column
        let children = node.children ?? []
        if axis == .row {
            ScrollView(.horizontal) {
                HStack(spacing: 0) {
                    ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                        ViewNodeView(node: child)
                    }
                }
            }
            .applyStyle(node.style)
        } else {
            ScrollView(.vertical) {
                VStack(spacing: 0) {
                    ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                        ViewNodeView(node: child)
                    }
                }
            }
            .applyStyle(node.style)
        }
    }

    @ViewBuilder
    private func ifView(_ node: IfNode) -> some View {
        let children = node.thenChildren ?? []
        ForEach(Array(children.enumerated()), id: \.offset) { _, child in
            ViewNodeView(node: child)
        }
    }

    @ViewBuilder
    private func forEachView(_ node: ForEachNode) -> some View {
        let body = node.itemBody ?? []
        ForEach(Array(body.enumerated()), id: \.offset) { _, child in
            ViewNodeView(node: child)
        }
    }

    @ViewBuilder
    private func tabsView(_ node: TabsNode) -> some View {
        let children = node.children ?? []
        TabView {
            ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                ViewNodeView(node: child)
            }
        }
    }

    @ViewBuilder
    private func tabView(_ node: TabNode) -> some View {
        let children = node.children ?? []
        VStack(spacing: 0) {
            ForEach(Array(children.enumerated()), id: \.offset) { _, child in
                ViewNodeView(node: child)
            }
        }
        .tabItem {
            Label(node.label, systemImage: node.icon ?? "circle")
        }
    }

    // MARK: - Helpers

    private func performAction(_ action: String?) {
        guard let action else { return }
        // ponytail: action dispatch placeholder; integrate with Rust bridge in real apps
        print("action: \(action)")
    }

    private func styledText(_ text: Text, style: ViewStyle?) -> some View {
        var result = text
        if let size = style?.fontSize {
            result = result.font(.system(size: CGFloat(size)))
        }
        if let weight = style?.fontWeight {
            result = result.fontWeight(.init(rawValue: weight))
        }
        if let color = style?.foregroundColor {
            result = result.foregroundColor(colorFrom(color))
        }
        return result
    }

    private func colorFrom(_ hex: String?) -> Color {
        guard let hex, !hex.isEmpty else { return .primary }
        let cleaned = hex.hasPrefix("#") ? String(hex.dropFirst()) : hex
        guard let value = UInt64(cleaned, radix: 16) else { return .primary }
        let r = Double((value >> 16) & 0xFF) / 255
        let g = Double((value >> 8) & 0xFF) / 255
        let b = Double(value & 0xFF) / 255
        return Color(red: r, green: g, blue: b)
    }

    private func cg(_ value: Float?) -> CGFloat? {
        value.map { CGFloat($0) }
    }

    private func cg(_ value: Float?) -> CGFloat {
        value.map { CGFloat($0) } ?? 0
    }
}

// MARK: - Style Modifier

private extension View {
    @ViewBuilder
    func applyStyle(_ style: ViewStyle?) -> some View {
        if let style {
            self
                .padding(.horizontal, CGFloat(style.paddingHorizontal ?? style.padding ?? 0))
                .padding(.vertical, CGFloat(style.paddingVertical ?? style.padding ?? 0))
                .padding(.top, CGFloat(style.paddingTop ?? 0))
                .padding(.bottom, CGFloat(style.paddingBottom ?? 0))
                .background(colorFrom(style.backgroundColor))
                .foregroundColor(colorFrom(style.foregroundColor))
                .cornerRadius(CGFloat(style.cornerRadius ?? 0))
        } else {
            self
        }
    }

    private func colorFrom(_ hex: String?) -> Color {
        guard let hex, !hex.isEmpty else { return .primary }
        let cleaned = hex.hasPrefix("#") ? String(hex.dropFirst()) : hex
        guard let value = UInt64(cleaned, radix: 16) else { return .primary }
        let r = Double((value >> 16) & 0xFF) / 255
        let g = Double((value >> 8) & 0xFF) / 255
        let b = Double(value & 0xFF) / 255
        return Color(red: r, green: g, blue: b)
    }
}

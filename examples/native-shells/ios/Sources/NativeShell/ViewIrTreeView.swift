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
        .padding()
    }
}

public struct ViewNodeView: View {
    let node: ViewNode

    public init(node: ViewNode) {
        self.node = node
    }

    public var body: some View {
        switch node {
        case .text(let content):
            Text(content)
        case .stack(let axis, let spacing, let children):
            let gap = CGFloat(spacing ?? 8)
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

import Foundation
import SwiftUI

/// Bundle that contains `fixture.json` for the NativeShell Swift target.
public enum NativeShellResources {
    public static var bundle: Bundle {
        let main = Bundle(for: _NativeShellResources.self)
        if let url = main.url(forResource: "NativeShell_NativeShell", withExtension: "bundle"),
           let resourceBundle = Bundle(url: url)
        {
            return resourceBundle
        }
        return main
    }
}

private final class _NativeShellResources {}

public struct ViewIr: Decodable, Sendable {
    public var version: Int
    public var root: [ViewNode]
}

public enum StackAxis: String, Decodable, Sendable {
    case row
    case column
}

public struct ViewStyle: Decodable, Sendable, Hashable {
    public let padding: Float?
    public let paddingHorizontal: Float?
    public let paddingVertical: Float?
    public let paddingTop: Float?
    public let paddingBottom: Float?
    public let paddingLeft: Float?
    public let paddingRight: Float?
    public let margin: Float?
    public let marginHorizontal: Float?
    public let marginVertical: Float?
    public let marginTop: Float?
    public let marginBottom: Float?
    public let marginLeft: Float?
    public let marginRight: Float?
    public let width: Float?
    public let height: Float?
    public let minWidth: Float?
    public let minHeight: Float?
    public let maxWidth: Float?
    public let maxHeight: Float?
    public let fontSize: Float?
    public let fontWeight: UInt16?
    public let textAlign: String?
    public let foregroundColor: String?
    public let backgroundColor: String?
    public let cornerRadius: Float?
    public let italic: Bool?
    public let underline: Bool?
    public let strikethrough: Bool?

    enum CodingKeys: String, CodingKey {
        case padding
        case paddingHorizontal
        case paddingVertical
        case paddingTop
        case paddingBottom
        case paddingLeft
        case paddingRight
        case margin
        case marginHorizontal
        case marginVertical
        case marginTop
        case marginBottom
        case marginLeft
        case marginRight
        case width
        case height
        case minWidth
        case minHeight
        case maxWidth
        case maxHeight
        case fontSize
        case fontWeight
        case textAlign
        case foregroundColor
        case backgroundColor
        case cornerRadius
        case italic
        case underline
        case strikethrough
    }
}

public enum ViewNode: Decodable, Sendable {
    case text(content: String, style: ViewStyle?)
    case stack(
        axis: StackAxis,
        spacing: Float?,
        alignItems: String?,
        justifyContent: String?,
        style: ViewStyle?,
        children: [ViewNode]
    )
    case button(label: String, onClick: String?, style: ViewStyle?)
    case image(src: String, alt: String?, style: ViewStyle?)
    case scroll(axis: StackAxis, style: ViewStyle?, children: [ViewNode])
    case slotRotate(phrases: [String], intervalMs: UInt64, style: ViewStyle?)

    enum CodingKeys: String, CodingKey {
        case kind
        case content
        case axis
        case spacing
        case alignItems
        case justifyContent
        case style
        case children
        case label
        case onClick
        case src
        case alt
        case phrases
        case intervalMs
    }

    enum Kind: String, Decodable {
        case text
        case stack
        case button
        case image
        case scroll
        case slotRotate
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try c.decode(Kind.self, forKey: .kind)
        switch kind {
        case .text:
            let content = try c.decode(String.self, forKey: .content)
            let style = try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            self = .text(content: content, style: style)
        case .stack:
            let axis = try c.decode(StackAxis.self, forKey: .axis)
            let spacing = try c.decodeIfPresent(Float.self, forKey: .spacing)
            let alignItems = try c.decodeIfPresent(String.self, forKey: .alignItems)
            let justifyContent = try c.decodeIfPresent(String.self, forKey: .justifyContent)
            let style = try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            let children = try c.decode([ViewNode].self, forKey: .children)
            self = .stack(
                axis: axis,
                spacing: spacing,
                alignItems: alignItems,
                justifyContent: justifyContent,
                style: style,
                children: children
            )
        case .button:
            let label = try c.decode(String.self, forKey: .label)
            let onClick = try c.decodeIfPresent(String.self, forKey: .onClick)
            let style = try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            self = .button(label: label, onClick: onClick, style: style)
        case .image:
            let src = try c.decode(String.self, forKey: .src)
            let alt = try c.decodeIfPresent(String.self, forKey: .alt)
            let style = try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            self = .image(src: src, alt: alt, style: style)
        case .scroll:
            let axis = try c.decode(StackAxis.self, forKey: .axis)
            let style = try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            let children = try c.decode([ViewNode].self, forKey: .children)
            self = .scroll(axis: axis, style: style, children: children)
        case .slotRotate:
            let phrases = try c.decode([String].self, forKey: .phrases)
            let intervalMs = try c.decode(UInt64.self, forKey: .intervalMs)
            let style = try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            self = .slotRotate(phrases: phrases, intervalMs: intervalMs, style: style)
        }
    }
}

public struct HotReloadEnvelope: Decodable, Sendable {
    public let sequence: UInt64
    public let message: HotReloadMessage
}

public enum HotReloadMessage: Decodable, Sendable {
    case noop
    case patch(mutations: [IrMutation])
    case fullReload(ir: ViewIr, reason: String)
    case error(message: String)
    case unsupported

    enum CodingKeys: String, CodingKey {
        case kind
        case mutations
        case ir
        case reason
        case message
    }

    enum Kind: String, Decodable {
        case noop
        case patch
        case fullReload
        case error
        case devHello
        case swiftgenStatus
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        switch try container.decode(Kind.self, forKey: .kind) {
        case .noop:
            self = .noop
        case .patch:
            self = .patch(mutations: try container.decode([IrMutation].self, forKey: .mutations))
        case .fullReload:
            self = .fullReload(
                ir: try container.decode(ViewIr.self, forKey: .ir),
                reason: try container.decode(String.self, forKey: .reason)
            )
        case .error:
            self = .error(message: try container.decode(String.self, forKey: .message))
        case .devHello, .swiftgenStatus:
            self = .unsupported
        }
    }
}

public enum IrMutation: Decodable, Sendable {
    case replaceRoot(root: [ViewNode])
    case replaceNode(path: [Int], node: ViewNode)
    case insertNode(parentPath: [Int], index: Int, node: ViewNode)
    case removeNode(path: [Int])
    case updateText(path: [Int], content: String)
    case updateStyle(path: [Int], style: ViewStyle?)

    enum CodingKeys: String, CodingKey {
        case op
        case root
        case path
        case node
        case parentPath
        case index
        case content
        case style
    }

    enum Operation: String, Decodable {
        case replaceRoot
        case replaceNode
        case insertNode
        case removeNode
        case updateText
        case updateStyle
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        switch try container.decode(Operation.self, forKey: .op) {
        case .replaceRoot:
            self = .replaceRoot(root: try container.decode([ViewNode].self, forKey: .root))
        case .replaceNode:
            self = .replaceNode(
                path: try container.decode([Int].self, forKey: .path),
                node: try container.decode(ViewNode.self, forKey: .node)
            )
        case .insertNode:
            self = .insertNode(
                parentPath: try container.decode([Int].self, forKey: .parentPath),
                index: try container.decode(Int.self, forKey: .index),
                node: try container.decode(ViewNode.self, forKey: .node)
            )
        case .removeNode:
            self = .removeNode(path: try container.decode([Int].self, forKey: .path))
        case .updateText:
            self = .updateText(
                path: try container.decode([Int].self, forKey: .path),
                content: try container.decode(String.self, forKey: .content)
            )
        case .updateStyle:
            self = .updateStyle(
                path: try container.decode([Int].self, forKey: .path),
                style: try container.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        }
    }
}

public enum ViewIrLoadError: Error {
    case missingResource
    case decode(Error)
}

extension ViewIr {
    public static func loadFixture(bundle: Bundle = NativeShellResources.bundle) throws -> ViewIr {
        guard let url = bundle.url(forResource: "fixture", withExtension: "json") else {
            throw ViewIrLoadError.missingResource
        }
        do {
            let data = try Data(contentsOf: url)
            return try JSONDecoder().decode(ViewIr.self, from: data)
        } catch {
            throw ViewIrLoadError.decode(error)
        }
    }

    public mutating func apply(_ envelope: HotReloadEnvelope) throws {
        switch envelope.message {
        case .noop, .unsupported:
            return
        case .error(let message):
            throw ViewIrMutationError.invalidMutation(message)
        case .fullReload(let ir, _):
            self = ir
        case .patch(let mutations):
            try apply(mutations)
        }
    }

    public mutating func apply(_ mutations: [IrMutation]) throws {
        for mutation in mutations {
            switch mutation {
            case .replaceRoot(let root):
                self.root = root
            case .replaceNode(let path, let node):
                try mutateNode(path: path) { target in
                    target = node
                }
            case .insertNode(let parentPath, let index, let node):
                try mutateChildren(path: parentPath) { children in
                    guard index <= children.count else {
                        throw ViewIrMutationError.invalidPath(parentPath)
                    }
                    children.insert(node, at: index)
                }
            case .removeNode(let path):
                guard !path.isEmpty else {
                    throw ViewIrMutationError.invalidMutation("cannot remove root list directly")
                }
                try mutateChildren(path: Array(path.dropLast())) { children in
                    let index = path[path.count - 1]
                    guard children.indices.contains(index) else {
                        throw ViewIrMutationError.invalidPath(path)
                    }
                    children.remove(at: index)
                }
            case .updateText(let path, let content):
                try mutateNode(path: path) { target in
                    switch target {
                    case .text(_, let style):
                        target = .text(content: content, style: style)
                    default:
                        throw ViewIrMutationError.invalidPath(path)
                    }
                }
            case .updateStyle(let path, let style):
                try mutateNode(path: path) { target in
                    target = target.withStyle(style)
                }
            }
        }
    }

    private mutating func mutateChildren(
        path: [Int],
        body: (inout [ViewNode]) throws -> Void
    ) throws {
        if path.isEmpty {
            try body(&root)
            return
        }
        try mutateNode(path: path) { target in
            switch target {
            case .stack(let axis, let spacing, let alignItems, let justifyContent, let style, var children):
                try body(&children)
                target = .stack(
                    axis: axis,
                    spacing: spacing,
                    alignItems: alignItems,
                    justifyContent: justifyContent,
                    style: style,
                    children: children
                )
            case .scroll(let axis, let style, var children):
                try body(&children)
                target = .scroll(axis: axis, style: style, children: children)
            default:
                throw ViewIrMutationError.invalidPath(path)
            }
        }
    }

    private mutating func mutateNode(
        path: [Int],
        body: (inout ViewNode) throws -> Void
    ) throws {
        guard let index = path.first, root.indices.contains(index) else {
            throw ViewIrMutationError.invalidPath(path)
        }
        root[index] = try root[index].mutated(path: Array(path.dropFirst()), body: body)
    }
}

public enum ViewIrMutationError: Error {
    case invalidPath([Int])
    case invalidMutation(String)
}

private extension ViewNode {
    func mutated(
        path: [Int],
        body: (inout ViewNode) throws -> Void
    ) throws -> ViewNode {
        var copy = self
        if path.isEmpty {
            try body(&copy)
            return copy
        }
        let index = path[0]
        switch copy {
        case .stack(let axis, let spacing, let alignItems, let justifyContent, let style, var children):
            guard children.indices.contains(index) else {
                throw ViewIrMutationError.invalidPath(path)
            }
            children[index] = try children[index].mutated(path: Array(path.dropFirst()), body: body)
            return .stack(
                axis: axis,
                spacing: spacing,
                alignItems: alignItems,
                justifyContent: justifyContent,
                style: style,
                children: children
            )
        case .scroll(let axis, let style, var children):
            guard children.indices.contains(index) else {
                throw ViewIrMutationError.invalidPath(path)
            }
            children[index] = try children[index].mutated(path: Array(path.dropFirst()), body: body)
            return .scroll(axis: axis, style: style, children: children)
        default:
            throw ViewIrMutationError.invalidPath(path)
        }
    }

    func withStyle(_ style: ViewStyle?) -> ViewNode {
        switch self {
        case .text(let content, _):
            return .text(content: content, style: style)
        case .stack(let axis, let spacing, let alignItems, let justifyContent, _, let children):
            return .stack(
                axis: axis,
                spacing: spacing,
                alignItems: alignItems,
                justifyContent: justifyContent,
                style: style,
                children: children
            )
        case .button(let label, let onClick, _):
            return .button(label: label, onClick: onClick, style: style)
        case .image(let src, let alt, _):
            return .image(src: src, alt: alt, style: style)
        case .scroll(let axis, _, let children):
            return .scroll(axis: axis, style: style, children: children)
        case .slotRotate(let phrases, let intervalMs, _):
            return .slotRotate(phrases: phrases, intervalMs: intervalMs, style: style)
        }
    }
}

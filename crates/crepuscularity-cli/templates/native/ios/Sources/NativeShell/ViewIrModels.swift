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
    public let version: Int
    public let root: [ViewNode]
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
}

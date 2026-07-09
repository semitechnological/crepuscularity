import Foundation
import SwiftUI

/// Bundle that contains `fixture.json` for the MobileApp Swift target.
public enum MobileAppResources {
    public static var bundle: Bundle {
        let main = Bundle(for: _MobileAppResources.self)
        if let url = main.url(forResource: "MobileApp_MobileApp", withExtension: "bundle"),
           let resourceBundle = Bundle(url: url)
        {
            return resourceBundle
        }
        return main
    }
}

private final class _MobileAppResources {}

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
    public let flexGrow: Float?
    public let width: Float?
    public let height: Float?

    enum CodingKeys: String, CodingKey {
        case padding, paddingHorizontal, paddingVertical
        case paddingTop, paddingBottom, paddingLeft, paddingRight
        case margin, marginHorizontal, marginVertical
        case marginTop, marginBottom, marginLeft, marginRight
        case fontSize, fontWeight, textAlign
        case foregroundColor, backgroundColor, cornerRadius
        case italic, underline, strikethrough
        case flexGrow, width, height
    }
}

public enum ViewNode: Decodable, Sendable {
    case text(TextNode)
    case button(ButtonNode)
    case toggle(ToggleNode)
    case slider(SliderNode)
    case input(InputNode)
    case stack(StackNode)
    case scroll(ScrollNode)
    case ifNode(IfNode)
    case forEach(ForEachNode)
    case tabs(TabsNode)
    case tab(TabNode)
    case unknown

    enum CodingKeys: String, CodingKey {
        case kind
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try container.decode(String.self, forKey: .kind)
        switch kind {
        case "text":
            self = .text(try TextNode(from: decoder))
        case "button":
            self = .button(try ButtonNode(from: decoder))
        case "toggle":
            self = .toggle(try ToggleNode(from: decoder))
        case "slider":
            self = .slider(try SliderNode(from: decoder))
        case "input":
            self = .input(try InputNode(from: decoder))
        case "stack":
            self = .stack(try StackNode(from: decoder))
        case "scroll":
            self = .scroll(try ScrollNode(from: decoder))
        case "if":
            self = .ifNode(try IfNode(from: decoder))
        case "forEach":
            self = .forEach(try ForEachNode(from: decoder))
        case "tabs":
            self = .tabs(try TabsNode(from: decoder))
        case "tab":
            self = .tab(try TabNode(from: decoder))
        default:
            self = .unknown
        }
    }
}

public struct TextNode: Decodable, Sendable {
    public let content: String?
    public let bind: String?
    public let style: ViewStyle?
}

public struct ButtonNode: Decodable, Sendable {
    public let label: String
    public let onClick: String?
    public let style: ViewStyle?
}

public struct ToggleNode: Decodable, Sendable {
    public let label: String?
    public let bind: String?
    public let checked: Bool?
    public let onChange: String?
}

public struct SliderNode: Decodable, Sendable {
    public let bind: String?
    public let value: Float?
    public let min: Float?
    public let max: Float?
    public let step: Float?
    public let style: ViewStyle?
}

public struct InputNode: Decodable, Sendable {
    public let placeholder: String?
    public let bind: String?
    public let multiline: Bool?
}

public struct StackNode: Decodable, Sendable {
    public let axis: StackAxis?
    public let spacing: Float?
    public let alignItems: String?
    public let justifyContent: String?
    public let children: [ViewNode]?
    public let style: ViewStyle?
}

public struct ScrollNode: Decodable, Sendable {
    public let axis: StackAxis?
    public let children: [ViewNode]?
    public let style: ViewStyle?
}

public struct IfNode: Decodable, Sendable {
    public let condition: String
    public let thenChildren: [ViewNode]?
    public let elseChildren: [ViewNode]?

    enum CodingKeys: String, CodingKey {
        case condition, thenChildren, elseChildren
    }
}

public struct ForEachNode: Decodable, Sendable {
    public let bind: String
    public let itemName: String?
    public let itemBody: [ViewNode]?
}

public struct TabsNode: Decodable, Sendable {
    public let bind: String?
    public let children: [ViewNode]?
    public let style: ViewStyle?
}

public struct TabNode: Decodable, Sendable {
    public let value: String
    public let label: String
    public let icon: String?
    public let children: [ViewNode]?
}

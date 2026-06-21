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
    public let borderWidth: Float?
    public let borderColor: String?
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
        case borderWidth
        case borderColor
        case italic
        case underline
        case strikethrough
    }
}

public struct PickerOption: Decodable, Sendable, Hashable {
    public let value: String
    public let label: String
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
    case toggle(label: String, bind: String?, checked: Bool, onChange: String?, style: ViewStyle?)
    case checkbox(label: String, bind: String?, checked: Bool, onChange: String?, style: ViewStyle?)
    case slider(label: String?, bind: String?, value: Float, min: Float, max: Float, step: Float?, style: ViewStyle?)
    case progress(label: String?, value: Float, max: Float, style: ViewStyle?)
    case meter(label: String?, value: Float, min: Float, max: Float, style: ViewStyle?)
    case badge(label: String, tone: String?, style: ViewStyle?)
    case divider(axis: StackAxis, style: ViewStyle?)
    case spacer(size: Float?, style: ViewStyle?)
    case dropzone(label: String, accept: String?, onDrop: String?, style: ViewStyle?, children: [ViewNode])
    case filePicker(label: String, accept: [String], multiple: Bool, onPick: String?, style: ViewStyle?)
    case image(src: String, alt: String?, style: ViewStyle?)
    case scroll(axis: StackAxis, style: ViewStyle?, children: [ViewNode])
    case list(ordered: Bool, style: ViewStyle?, children: [ViewNode])
    case listItem(style: ViewStyle?, children: [ViewNode])
    case slotRotate(phrases: [String], intervalMs: UInt64, style: ViewStyle?)
    case input(placeholder: String, bind: String, multiline: Bool, style: ViewStyle?)
    case picker(bind: String, options: [PickerOption], style: ViewStyle?)

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
        case bind
        case checked
        case onClick
        case onChange
        case value
        case min
        case max
        case step
        case tone
        case size
        case accept
        case onDrop
        case multiple
        case onPick
        case src
        case alt
        case ordered
        case phrases
        case intervalMs
        case placeholder
        case multiline
        case options
    }

    enum Kind: String, Decodable {
        case text
        case stack
        case button
        case toggle
        case checkbox
        case slider
        case progress
        case meter
        case badge
        case divider
        case spacer
        case dropzone
        case filePicker
        case image
        case scroll
        case list
        case listItem
        case slotRotate
        case input
        case picker
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
        case .toggle:
            self = .toggle(
                label: try c.decode(String.self, forKey: .label),
                bind: try c.decodeIfPresent(String.self, forKey: .bind),
                checked: try c.decode(Bool.self, forKey: .checked),
                onChange: try c.decodeIfPresent(String.self, forKey: .onChange),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        case .checkbox:
            self = .checkbox(
                label: try c.decode(String.self, forKey: .label),
                bind: try c.decodeIfPresent(String.self, forKey: .bind),
                checked: try c.decode(Bool.self, forKey: .checked),
                onChange: try c.decodeIfPresent(String.self, forKey: .onChange),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        case .slider:
            self = .slider(
                label: try c.decodeIfPresent(String.self, forKey: .label),
                bind: try c.decodeIfPresent(String.self, forKey: .bind),
                value: try c.decode(Float.self, forKey: .value),
                min: try c.decode(Float.self, forKey: .min),
                max: try c.decode(Float.self, forKey: .max),
                step: try c.decodeIfPresent(Float.self, forKey: .step),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        case .progress:
            self = .progress(
                label: try c.decodeIfPresent(String.self, forKey: .label),
                value: try c.decode(Float.self, forKey: .value),
                max: try c.decode(Float.self, forKey: .max),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        case .meter:
            self = .meter(
                label: try c.decodeIfPresent(String.self, forKey: .label),
                value: try c.decode(Float.self, forKey: .value),
                min: try c.decode(Float.self, forKey: .min),
                max: try c.decode(Float.self, forKey: .max),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        case .badge:
            self = .badge(
                label: try c.decode(String.self, forKey: .label),
                tone: try c.decodeIfPresent(String.self, forKey: .tone),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        case .divider:
            self = .divider(
                axis: try c.decode(StackAxis.self, forKey: .axis),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        case .spacer:
            self = .spacer(
                size: try c.decodeIfPresent(Float.self, forKey: .size),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        case .dropzone:
            self = .dropzone(
                label: try c.decode(String.self, forKey: .label),
                accept: try c.decodeIfPresent(String.self, forKey: .accept),
                onDrop: try c.decodeIfPresent(String.self, forKey: .onDrop),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style),
                children: try c.decode([ViewNode].self, forKey: .children)
            )
        case .filePicker:
            self = .filePicker(
                label: try c.decode(String.self, forKey: .label),
                accept: try c.decodeIfPresent([String].self, forKey: .accept) ?? [],
                multiple: try c.decodeIfPresent(Bool.self, forKey: .multiple) ?? false,
                onPick: try c.decodeIfPresent(String.self, forKey: .onPick),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
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
        case .list:
            self = .list(
                ordered: try c.decode(Bool.self, forKey: .ordered),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style),
                children: try c.decode([ViewNode].self, forKey: .children)
            )
        case .listItem:
            self = .listItem(
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style),
                children: try c.decode([ViewNode].self, forKey: .children)
            )
        case .slotRotate:
            let phrases = try c.decode([String].self, forKey: .phrases)
            let intervalMs = try c.decode(UInt64.self, forKey: .intervalMs)
            let style = try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            self = .slotRotate(phrases: phrases, intervalMs: intervalMs, style: style)
        case .input:
            self = .input(
                placeholder: try c.decode(String.self, forKey: .placeholder),
                bind: try c.decode(String.self, forKey: .bind),
                multiline: try c.decodeIfPresent(Bool.self, forKey: .multiline) ?? false,
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
            )
        case .picker:
            self = .picker(
                bind: try c.decode(String.self, forKey: .bind),
                options: try c.decode([PickerOption].self, forKey: .options),
                style: try c.decodeIfPresent(ViewStyle.self, forKey: .style)
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
}

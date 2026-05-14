import Foundation

public struct ViewIr: Decodable {
    public let version: Int
    public let root: [Node]

    public typealias Node = [String: JSONValue]
}

public enum JSONValue: Decodable {
    case string(String)
    case number(Double)
    case bool(Bool)
    case object([String: JSONValue])
    case array([JSONValue])
    case null

    public init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if c.decodeNil() {
            self = .null
        } else if let v = try? c.decode(Bool.self) {
            self = .bool(v)
        } else if let v = try? c.decode(Double.self) {
            self = .number(v)
        } else if let v = try? c.decode(String.self) {
            self = .string(v)
        } else if let v = try? c.decode([JSONValue].self) {
            self = .array(v)
        } else {
            self = .object(try c.decode([String: JSONValue].self))
        }
    }
}

public func renderIr(path: String) throws -> ViewIr {
    let bin = ProcessInfo.processInfo.environment["CREPUS_BIN"] ?? "crepus"
    let process = Process()
    let output = Pipe()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    process.arguments = [bin, "native", "ir", path]
    process.standardOutput = output
    try process.run()
    process.waitUntilExit()
    let data = output.fileHandleForReading.readDataToEndOfFile()
    if process.terminationStatus != 0 {
        throw NSError(domain: "Crepuscularity", code: Int(process.terminationStatus))
    }
    return try JSONDecoder().decode(ViewIr.self, from: data)
}

public func renderHtml(path: String) throws -> String {
    try renderIr(path: path).root.map(renderNode).joined()
}

private func renderNode(_ node: ViewIr.Node) -> String {
    guard case let .string(kind)? = node["kind"] else {
        return ""
    }
    switch kind {
    case "text":
        if case let .string(content)? = node["content"] {
            return escapeHtml(content)
        }
        return ""
    case "stack", "scroll":
        let axis: String
        if case let .string(value)? = node["axis"] {
            axis = value
        } else {
            axis = "column"
        }
        let children: String
        if case let .array(values)? = node["children"] {
            children = values.compactMap { value in
                if case let .object(child) = value {
                    return renderNode(child)
                }
                return nil
            }.joined()
        } else {
            children = ""
        }
        return "<div data-crepus-kind=\"\(escapeHtml(kind))\" data-axis=\"\(escapeHtml(axis))\">\(children)</div>"
    case "button":
        if case let .string(label)? = node["label"] {
            return "<button>\(escapeHtml(label))</button>"
        }
        return "<button></button>"
    default:
        return ""
    }
}

private func escapeHtml(_ value: String) -> String {
    value.replacingOccurrences(of: "&", with: "&amp;")
        .replacingOccurrences(of: "<", with: "&lt;")
        .replacingOccurrences(of: ">", with: "&gt;")
        .replacingOccurrences(of: "\"", with: "&quot;")
}

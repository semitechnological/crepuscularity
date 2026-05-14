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

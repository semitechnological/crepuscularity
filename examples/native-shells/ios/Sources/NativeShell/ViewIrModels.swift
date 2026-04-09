import Foundation

/// Bundle that contains `fixture.json` for the NativeShell Swift target.
public enum NativeShellResources {
    public static var bundle: Bundle {
        let main = Bundle(for: _NativeShellResources.self)
        // SPM places resources in NativeShell_NativeShell.bundle next to the built module.
        if let url = main.url(forResource: "NativeShell_NativeShell", withExtension: "bundle"),
           let resourceBundle = Bundle(url: url)
        {
            return resourceBundle
        }
        return main
    }
}

private final class _NativeShellResources {}

/// Root document: mirrors Rust `crepuscularity_native::ViewIr` JSON (camelCase, `kind` tag).
public struct ViewIr: Decodable, Sendable {
    public let version: Int
    public let root: [ViewNode]

    enum CodingKeys: String, CodingKey {
        case version
        case root
    }

    public init(version: Int, root: [ViewNode]) {
        self.version = version
        self.root = root
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        self.version = try c.decode(Int.self, forKey: .version)
        self.root = try c.decode([ViewNode].self, forKey: .root)
    }
}

public enum StackAxis: String, Decodable, Sendable {
    case row
    case column
}

public enum ViewNode: Decodable, Sendable {
    case text(content: String)
    case stack(axis: StackAxis, spacing: Float?, children: [ViewNode])

    enum CodingKeys: String, CodingKey {
        case kind
        case content
        case axis
        case spacing
        case children
    }

    enum Kind: String, Decodable {
        case text
        case stack
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try c.decode(Kind.self, forKey: .kind)
        switch kind {
        case .text:
            let content = try c.decode(String.self, forKey: .content)
            self = .text(content: content)
        case .stack:
            let axis = try c.decode(StackAxis.self, forKey: .axis)
            let spacing = try c.decodeIfPresent(Float.self, forKey: .spacing)
            let children = try c.decode([ViewNode].self, forKey: .children)
            self = .stack(axis: axis, spacing: spacing, children: children)
        }
    }
}

public enum ViewIrLoadError: Error {
    case missingResource
    case decode(Error)
}

extension ViewIr {
    /// Load bundled `fixture.json` from a bundle (use ``NativeShellResources/bundle`` for this package).
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

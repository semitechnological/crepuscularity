import Darwin
import Foundation
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

@_silgen_name("crepus_mobile_dispatch")
private func crepusMobileDispatch(_ action: UnsafePointer<CChar>, _ length: UInt) -> Bool

@_silgen_name("crepus_mobile_dispatch_json")
private func crepusMobileDispatchJson(_ action: UnsafePointer<CChar>, _ length: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt

public enum CrepusRustActions {
    public static func install() {
        CrepusActions.dispatch = { action in
            if let host = dispatchHostAction(action) {
                return host
            }
            return action.withCString { pointer in
                let capacity = 4096
                let output = UnsafeMutablePointer<CChar>.allocate(capacity: capacity)
                defer { output.deallocate() }
                let written = crepusMobileDispatchJson(pointer, UInt(strlen(pointer)), output, UInt(capacity))
                if written >= capacity {
                    return oversizedResultJson(action: action)
                }
                return String(cString: output)
            }
        }
        CrepusActions.resultSink = { result in
            Task { @MainActor in
                CrepusActionStore.shared.record(result)
            }
        }
    }

    private static func oversizedResultJson(action: String) -> String {
        let payload: [String: Any] = [
            "ok": false,
            "action": action,
            "error": "action result too large",
        ]
        if let data = try? JSONSerialization.data(withJSONObject: payload),
           let json = String(data: data, encoding: .utf8) {
            return json
        }
        return "{\"ok\":false,\"error\":\"action result too large\"}"
    }

    private static func dispatchHostAction(_ action: String) -> String? {
        guard let data = action.data(using: .utf8),
              let root = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              (root["kind"] as? String) == "plugin",
              (root["capability"] as? String) == "clipboard",
              let method = root["method"] as? String
        else {
            return nil
        }
        let payload = root["payload"] as? [String: Any]
        let actionName = "clipboard.\(method)"
        do {
            let value = try clipboardValue(method: method, payload: payload)
            return successJson(action: actionName, capability: "clipboard", method: method, value: value)
        } catch {
            return errorJson(action: actionName, error: error.localizedDescription)
        }
    }

    private static func clipboardValue(method: String, payload: [String: Any]?) throws -> Any {
        switch method {
        case "get":
            return ["text": currentClipboardText() as Any]
        case "set":
            guard let text = payload?["text"] as? String else {
                throw HostActionError("clipboard.set requires payload.text")
            }
            setClipboardText(text)
            return ["text": text]
        case "clear":
            clearClipboard()
            return ["cleared": true]
        default:
            throw HostActionError("unsupported clipboard method: \(method)")
        }
    }

    private static func successJson(action: String, capability: String, method: String, value: Any) -> String {
        let payload: [String: Any] = [
            "ok": true,
            "action": action,
            "value": [
                "capability": capability,
                "method": method,
                "value": value,
            ],
        ]
        return stringify(payload)
    }

    private static func errorJson(action: String, error: String) -> String {
        stringify([
            "ok": false,
            "action": action,
            "error": error,
        ])
    }

    private static func stringify(_ payload: [String: Any]) -> String {
        if let data = try? JSONSerialization.data(withJSONObject: payload),
           let json = String(data: data, encoding: .utf8) {
            return json
        }
        return "{\"ok\":false,\"error\":\"json encode failure\"}"
    }
}

private struct HostActionError: LocalizedError {
    let message: String

    init(_ message: String) {
        self.message = message
    }

    var errorDescription: String? { message }
}

#if canImport(UIKit)
private func currentClipboardText() -> String? {
    UIPasteboard.general.string
}

private func setClipboardText(_ text: String) {
    UIPasteboard.general.string = text
}

private func clearClipboard() {
    UIPasteboard.general.items = []
}
#elseif canImport(AppKit)
private func currentClipboardText() -> String? {
    NSPasteboard.general.string(forType: .string)
}

private func setClipboardText(_ text: String) {
    let pasteboard = NSPasteboard.general
    pasteboard.clearContents()
    pasteboard.setString(text, forType: .string)
}

private func clearClipboard() {
    NSPasteboard.general.clearContents()
}
#else
private func currentClipboardText() -> String? { nil }

private func setClipboardText(_ text: String) {}

private func clearClipboard() {}
#endif

private struct CrepusActionResult: Decodable {
    let ok: Bool?
    let error: String?
}

@MainActor
public final class CrepusActionStore: ObservableObject {
    public static let shared = CrepusActionStore()

    @Published public private(set) var lastResult: String = "{}"
    @Published public private(set) var lastError: String?

    public func dispatch(_ action: String) {
        record(CrepusActions.dispatch(action))
    }

    public func record(_ result: String) {
        lastResult = result
        let data = Data(result.utf8)
        if let payload = try? JSONDecoder().decode(CrepusActionResult.self, from: data),
           payload.ok == false {
            lastError = payload.error ?? result
        } else {
            lastError = nil
        }
    }
}

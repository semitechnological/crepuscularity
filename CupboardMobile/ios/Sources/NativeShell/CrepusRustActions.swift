import Darwin
import Foundation
import SwiftUI

@_silgen_name("crepus_mobile_dispatch")
private func crepusMobileDispatch(_ action: UnsafePointer<CChar>, _ length: UInt) -> Bool

@_silgen_name("crepus_mobile_dispatch_json")
private func crepusMobileDispatchJson(_ action: UnsafePointer<CChar>, _ length: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt
@_silgen_name("crepus_mobile_store_result_json")
private func crepusMobileStoreResultJson(_ json: UnsafePointer<CChar>, _ length: UInt) -> Bool
@_silgen_name("crepus_mobile_eval_text")
private func crepusMobileEvalText(_ expr: UnsafePointer<CChar>, _ exprLength: UInt, _ scopeName: UnsafePointer<CChar>?, _ scopeNameLength: UInt, _ scope: UnsafePointer<CChar>?, _ scopeLength: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt
@_silgen_name("crepus_mobile_eval_bool")
private func crepusMobileEvalBool(_ expr: UnsafePointer<CChar>, _ exprLength: UInt, _ scopeName: UnsafePointer<CChar>?, _ scopeNameLength: UInt, _ scope: UnsafePointer<CChar>?, _ scopeLength: UInt) -> Bool
@_silgen_name("crepus_mobile_eval_number")
private func crepusMobileEvalNumber(_ expr: UnsafePointer<CChar>, _ exprLength: UInt, _ scopeName: UnsafePointer<CChar>?, _ scopeNameLength: UInt, _ scope: UnsafePointer<CChar>?, _ scopeLength: UInt) -> Double
@_silgen_name("crepus_mobile_eval_items_json")
private func crepusMobileEvalItemsJson(_ expr: UnsafePointer<CChar>, _ exprLength: UInt, _ scopeName: UnsafePointer<CChar>?, _ scopeNameLength: UInt, _ scope: UnsafePointer<CChar>?, _ scopeLength: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt

public enum CrepusRustActions {
    public static func install() {
        CrepusActions.dispatch = { action in
            action.withCString { pointer in
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
}

@MainActor
public final class CrepusStateStore: ObservableObject {
    public static let shared = CrepusStateStore()

    @Published public private(set) var revision: UInt64 = 0

    public func applyResult(_ json: String) {
        let stored = json.withCString { pointer in
            crepusMobileStoreResultJson(pointer, UInt(strlen(pointer)))
        }
        if stored {
            revision &+= 1
        }
    }

    public func text(_ expr: String, scopeName: String? = nil, scope: Any? = nil) -> String {
        _ = revision
        return readString(expr: expr, scopeName: scopeName, scopeJson: scopeJson(scope), reader: crepusMobileEvalText)
    }

    public func bool(_ expr: String, scopeName: String? = nil, scope: Any? = nil) -> Bool {
        _ = revision
        return expr.withCString { exprPointer in
            callOptionalArgs(scopeName, scopeJson(scope)) { scopeNamePointer, scopeNameLength, scopePointer, scopeLength in
                crepusMobileEvalBool(exprPointer, UInt(strlen(exprPointer)), scopeNamePointer, scopeNameLength, scopePointer, scopeLength)
            }
        }
    }

    public func number(_ expr: String, scopeName: String? = nil, scope: Any? = nil) -> Double {
        _ = revision
        return expr.withCString { exprPointer in
            callOptionalArgs(scopeName, scopeJson(scope)) { scopeNamePointer, scopeNameLength, scopePointer, scopeLength in
                crepusMobileEvalNumber(exprPointer, UInt(strlen(exprPointer)), scopeNamePointer, scopeNameLength, scopePointer, scopeLength)
            }
        }
    }

    public func items(_ expr: String, scopeName: String? = nil, scope: Any? = nil) -> [Any] {
        _ = revision
        let json = readString(expr: expr, scopeName: scopeName, scopeJson: scopeJson(scope), reader: crepusMobileEvalItemsJson)
        guard let data = json.data(using: .utf8),
              let array = try? JSONSerialization.jsonObject(with: data) as? [Any]
        else {
            return []
        }
        return array
    }

    private func scopeJson(_ scope: Any?) -> String? {
        guard let scope else {
            return nil
        }
        switch scope {
        case let text as String:
            return "\"\(text.replacingOccurrences(of: "\"", with: "\\\""))\""
        case let number as NSNumber:
            return number.stringValue
        case is NSNull:
            return "null"
        default:
            guard JSONSerialization.isValidJSONObject(scope),
                  let data = try? JSONSerialization.data(withJSONObject: scope),
                  let json = String(data: data, encoding: .utf8)
            else {
                return nil
            }
            return json
        }
    }

    private func readString(
        expr: String,
        scopeName: String?,
        scopeJson: String?,
        reader: (UnsafePointer<CChar>, UInt, UnsafePointer<CChar>?, UInt, UnsafePointer<CChar>?, UInt, UnsafeMutablePointer<CChar>, UInt) -> UInt
    ) -> String {
        expr.withCString { exprPointer in
            let capacity = 4096
            let output = UnsafeMutablePointer<CChar>.allocate(capacity: capacity)
            defer { output.deallocate() }
            let written = callOptionalArgs(scopeName, scopeJson) { scopeNamePointer, scopeNameLength, scopePointer, scopeLength in
                reader(exprPointer, UInt(strlen(exprPointer)), scopeNamePointer, scopeNameLength, scopePointer, scopeLength, output, UInt(capacity))
            }
            if written >= capacity {
                return ""
            }
            return String(cString: output)
        }
    }

    private func callOptionalArgs<T>(
        _ scopeName: String?,
        _ scopeJson: String?,
        body: (UnsafePointer<CChar>?, UInt, UnsafePointer<CChar>?, UInt) -> T
    ) -> T {
        if let scopeName {
            return scopeName.withCString { scopeNamePointer in
                if let scopeJson {
                    return scopeJson.withCString { scopePointer in
                        body(scopeNamePointer, UInt(strlen(scopeNamePointer)), scopePointer, UInt(strlen(scopePointer)))
                    }
                }
                return body(scopeNamePointer, UInt(strlen(scopeNamePointer)), nil, 0)
            }
        }
        if let scopeJson {
            return scopeJson.withCString { scopePointer in
                body(nil, 0, scopePointer, UInt(strlen(scopePointer)))
            }
        }
        return body(nil, 0, nil, 0)
    }
}

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

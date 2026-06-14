import Darwin
import Foundation

@_silgen_name("crepus_mobile_dispatch")
private func crepusMobileDispatch(_ action: UnsafePointer<CChar>, _ length: UInt) -> Bool

@_silgen_name("crepus_mobile_dispatch_json")
private func crepusMobileDispatchJson(_ action: UnsafePointer<CChar>, _ length: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt

public enum CrepusRustActions {
    public static func install() {
        CrepusActions.dispatch = { action in
            action.withCString { pointer in
                let capacity = 4096
                let output = UnsafeMutablePointer<CChar>.allocate(capacity: capacity)
                defer { output.deallocate() }
                let written = crepusMobileDispatchJson(pointer, UInt(strlen(pointer)), output, UInt(capacity))
                if written >= capacity {
                    return "{\"ok\":false,\"action\":\"\(action)\",\"error\":\"action result too large\"}"
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
        lastError = result.contains("\"ok\":false") ? result : nil
    }
}

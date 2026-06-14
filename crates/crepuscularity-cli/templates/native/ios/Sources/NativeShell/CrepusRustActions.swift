import Darwin
import Foundation

@_silgen_name("crepus_mobile_dispatch")
private func crepusMobileDispatch(_ action: UnsafePointer<CChar>, _ length: UInt) -> Bool

public enum CrepusRustActions {
    public static func install() {
        CrepusActions.dispatch = { action in
            action.withCString { pointer in
                _ = crepusMobileDispatch(pointer, UInt(strlen(pointer)))
            }
        }
    }
}

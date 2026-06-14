import NativeShell
import SwiftUI

@main
struct CrepusMobileApp: App {
    init() {
        CrepusRustActions.install()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

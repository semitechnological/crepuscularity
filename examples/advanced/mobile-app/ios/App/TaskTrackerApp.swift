import MobileApp
import SwiftUI

@main
struct TaskTrackerApp: App {
    init() {
        CrepusRustActions.install()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    var body: some View {
        TaskTrackerView()
    }
}

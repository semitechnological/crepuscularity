import NativeShell
import SwiftUI

struct ContentView: View {
    var body: some View {
        ZStack(alignment: .topLeading) {
            Color(red: 16 / 255, green: 22 / 255, blue: 36 / 255).ignoresSafeArea()
            CrepusMobileRuntimeRootView()
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .ignoresSafeArea()
        }
    }
}

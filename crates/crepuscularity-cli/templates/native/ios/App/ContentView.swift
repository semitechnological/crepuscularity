import NativeShell
import SwiftUI

struct ContentView: View {
    var body: some View {
        ZStack(alignment: .topLeading) {
            Color(red: 0.063, green: 0.086, blue: 0.141)
            #if DEBUG
            CrepusMobileRuntimeRootView()
                .padding(.top, 64)
            #else
            CrepusGeneratedView()
                .padding(.top, 64)
            #endif
        }
        .ignoresSafeArea()
    }
}

import Foundation
import SwiftUI

@MainActor
public final class CrepusMobileRuntime: ObservableObject {
    @Published public private(set) var ir: ViewIr?
    @Published public private(set) var errorText: String?

    private let baseURL: URL
    private let bundle: Bundle

    public init(
        baseURL: URL = URL(string: "http://127.0.0.1:4001")!,
        bundle: Bundle = NativeShellResources.bundle
    ) {
        self.baseURL = baseURL
        self.bundle = bundle
    }

    public func start() async {
        do {
            ir = try ViewIr.loadFixture(bundle: bundle)
        } catch {
            errorText = String(describing: error)
        }

        #if DEBUG
        guard ProcessInfo.processInfo.environment["CREPUS_DEV_SERVER"] == "1" else {
            return
        }

        while !Task.isCancelled {
            await refreshFromDevServer()
            try? await Task.sleep(nanoseconds: 1_000_000_000)
        }
        #endif
    }

    private func refreshFromDevServer() async {
        do {
            let (data, _) = try await URLSession.shared.data(from: baseURL.appendingPathComponent("ir"))
            ir = try JSONDecoder().decode(ViewIr.self, from: data)
            errorText = nil
            _ = try? await URLSession.shared.data(from: baseURL.appendingPathComponent("events"))
        } catch {
            if ir == nil {
                errorText = String(describing: error)
            }
        }
    }
}

public struct CrepusMobileRuntimeRootView: View {
    @StateObject private var runtime: CrepusMobileRuntime

    public init() {
        _runtime = StateObject(wrappedValue: CrepusMobileRuntime())
    }

    public init(runtime: CrepusMobileRuntime) {
        _runtime = StateObject(wrappedValue: runtime)
    }

    public var body: some View {
        Group {
            if let ir = runtime.ir {
                ViewIrRootView(ir: ir)
            } else if let errorText = runtime.errorText {
                Text(errorText).foregroundStyle(.red)
            } else {
                ProgressView()
            }
        }
        .task {
            await runtime.start()
        }
    }
}

// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "NativeShell",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "NativeShell", targets: ["NativeShell"]),
    ],
    targets: [
        .target(
            name: "NativeShell",
            path: "Sources/NativeShell",
            resources: [.copy("fixture.json")]
        ),
    ]
)

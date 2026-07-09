// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "MobileApp",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "MobileApp", targets: ["MobileApp"]),
    ],
    targets: [
        .target(
            name: "MobileApp",
            path: "Sources/MobileApp",
            resources: [.copy("fixture.json")]
        ),
    ]
)

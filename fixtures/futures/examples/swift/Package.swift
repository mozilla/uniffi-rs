// swift-tools-version: 5.6
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "demo",
    platforms: [.macOS(.v12)],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "demo",
            dependencies: [],
            swiftSettings: [
                .unsafeFlags(["-I", "../../../../target/release"])
            ],
            linkerSettings: [
                .linkedLibrary("uniffi_futures"),
                .unsafeFlags(["-L../../../../target/release/"])
            ]),
    ]
)

// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Piercast",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "Piercast", targets: ["Piercast"])
    ],
    targets: [
        .executableTarget(
            name: "Piercast",
            path: "Sources/Piercast",
            resources: [.process("../Resources")]
        )
    ]
)

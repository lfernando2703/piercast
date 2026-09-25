import SwiftUI

@main
struct PiercastApp: App {
    @StateObject private var daemon = DaemonClient.shared

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(daemon)
                .task { await daemon.ensureRunning() }
        }
        .commands {
            CommandGroup(replacing: .newItem) {}
        }

        MenuBarExtra("Piercast", systemImage: "antenna.radiowaves.left.and.right") {
            MenuBarView()
                .environmentObject(daemon)
        }
    }
}

import SwiftUI

@main
struct PiercastiOSApp: App {
    @StateObject private var session = PairingSession.shared

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(session)
        }
    }
}

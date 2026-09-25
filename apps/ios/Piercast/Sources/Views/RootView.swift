import UIKit
import SwiftUI

struct RootView: View {
    @EnvironmentObject private var session: PairingSession

    var body: some View {
        TabView {
            NavigationStack {
                HomeView()
            }
            .tabItem { Label("Apps", systemImage: "square.grid.2x2") }

            NavigationStack {
                SettingsView()
            }
            .tabItem { Label("Settings", systemImage: "gear") }
        }
        .tint(Color(red: 0.48, green: 0.42, blue: 1.0))
    }
}

struct HomeView: View {
    @EnvironmentObject private var session: PairingSession

    var body: some View {
        List(session.apps) { app in
            NavigationLink(value: app.id) {
                HStack {
                    VStack(alignment: .leading) {
                        Text(app.name).font(.headline)
                        Text(app.status).font(.caption).foregroundStyle(.secondary)
                    }
                    Spacer()
                }
                .accessibilityLabel("\(app.name), \(app.status)")
            }
        }
        .navigationTitle("Piercast")
        .navigationDestination(for: String.self) { id in
            if let app = session.apps.first(where: { $0.id == id }) {
                AppDetailView(app: app)
            }
        }
        .refreshable { await session.refreshApps() }
        .task { await session.refreshApps() }
        .overlay {
            if session.active == nil {
                ContentUnavailableView("Pair a desktop", systemImage: "qrcode.viewfinder", description: Text("Scan the QR from Piercast desktop Settings → Mobile Access."))
            }
        }
    }
}

struct AppDetailView: View {
    @EnvironmentObject private var session: PairingSession
    let app: MobileAppInfo

    var body: some View {
        List {
            Section("Status") {
                Text(app.status)
            }
            Section("Actions") {
                Button("Open") { Task { await open() } }
                Button("Start") { Task { await session.action(appId: app.id, path: "start") } }
                Button("Stop") { Task { await session.action(appId: app.id, path: "stop") } }
                Button("Restart") { Task { await session.action(appId: app.id, path: "restart") } }
                Button("Kill", role: .destructive) { Task { await session.action(appId: app.id, path: "kill") } }
            }
        }
        .navigationTitle(app.name)
    }

    private func open() async {
        await session.action(appId: app.id, path: "open")
        let urlString = app.expose_url ?? app.open_url
        if let urlString, let url = URL(string: urlString) {
            await UIApplication.shared.open(url)
        }
    }
}

struct SettingsView: View {
    @EnvironmentObject private var session: PairingSession
    @State private var showScanner = false

    var body: some View {
        List {
            Section("Paired hosts") {
                ForEach(session.hosts) { host in
                    Button {
                        session.active = host
                        Task { await session.refreshApps() }
                    } label: {
                        HStack {
                            Text(host.hostName)
                            Spacer()
                            if session.active == host {
                                Image(systemName: "checkmark.circle.fill")
                            }
                        }
                    }
                }
            }
            Section {
                Button("Scan pairing QR") { showScanner = true }
            }
        }
        .navigationTitle("Settings")
        .sheet(isPresented: $showScanner) {
            Text("QR scanner hooks to AVFoundation Camera; paste bootstrap JSON in debug builds.")
                .padding()
        }
    }
}

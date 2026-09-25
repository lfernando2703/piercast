import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var daemon: DaemonClient
    @State private var filter: LibraryFilter = .all
    @State private var query = ""
    @State private var showPalette = false

    var body: some View {
        NavigationSplitView {
            List(selection: $filter) {
                Section("Library") {
                    ForEach(LibraryFilter.allCases) { f in
                        Label(f.title, systemImage: f.symbol).tag(f)
                    }
                }
            }
            .navigationTitle("Piercast")
            .accessibilityLabel("Library filters")
        } detail: {
            AppGridView(apps: filteredApps)
                .navigationTitle(filter.title)
                .searchable(text: $query)
                .toolbar {
                    ToolbarItem(placement: .primaryAction) {
                        Button {
                            showPalette = true
                        } label: {
                            Label("Command Palette", systemImage: "magnifyingglass")
                        }
                        .keyboardShortcut("k", modifiers: .command)
                    }
                }
        }
        .sheet(isPresented: $showPalette) {
            CommandPaletteView()
                .environmentObject(daemon)
        }
    }

    private var filteredApps: [PiercastAppInfo] {
        daemon.apps.filter { app in
            switch filter {
            case .all: return true
            case .running: return app.status.lowercased() == "running"
            case .unhealthy: return app.status.lowercased() == "unhealthy"
            case .favorites: return false // wired when favorites API lands
            }
        }
        .filter { query.isEmpty || $0.name.localizedCaseInsensitiveContains(query) || $0.id.localizedCaseInsensitiveContains(query) }
    }
}

enum LibraryFilter: String, CaseIterable, Identifiable {
    case all, running, unhealthy, favorites
    var id: String { rawValue }
    var title: String {
        switch self {
        case .all: return "All"
        case .running: return "Running"
        case .unhealthy: return "Unhealthy"
        case .favorites: return "Favorites"
        }
    }
    var symbol: String {
        switch self {
        case .all: return "square.grid.2x2"
        case .running: return "play.circle"
        case .unhealthy: return "exclamationmark.triangle"
        case .favorites: return "star"
        }
    }
}

struct AppGridView: View {
    @EnvironmentObject private var daemon: DaemonClient
    let apps: [PiercastAppInfo]

    private let columns = [GridItem(.adaptive(minimum: 180), spacing: 16)]

    var body: some View {
        if apps.isEmpty {
            ContentUnavailableView("No apps yet", systemImage: "antenna.radiowaves.left.and.right", description: Text("Register a piercast.yml or ask your agent to upsert."))
        } else {
            ScrollView {
                LazyVGrid(columns: columns, spacing: 16) {
                    ForEach(apps) { app in
                        AppTile(app: app)
                    }
                }
                .padding(16)
            }
        }
    }
}

struct AppTile: View {
    @EnvironmentObject private var daemon: DaemonClient
    let app: PiercastAppInfo

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(app.name).font(.headline)
                Spacer()
                StatusPill(status: app.status)
            }
            Text(app.id).font(.caption).foregroundStyle(.secondary)
            HStack {
                Button("Open") { Task { await daemon.open(id: app.id) } }
                    .keyboardShortcut(.defaultAction)
                Button("Stop") { Task { await daemon.stop(id: app.id) } }
            }
        }
        .padding(12)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 12))
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(app.name), \(app.status)")
    }
}

struct StatusPill: View {
    let status: String
    var body: some View {
        Text(status.capitalized)
            .font(.caption2.weight(.semibold))
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(color.opacity(0.2), in: Capsule())
            .foregroundStyle(color)
            .accessibilityLabel("Status \(status)")
    }
    private var color: Color {
        switch status.lowercased() {
        case "running": return Color(red: 0.19, green: 0.82, blue: 0.35)
        case "unhealthy": return Color(red: 1.0, green: 0.27, blue: 0.23)
        case "starting": return Color(red: 1.0, green: 0.62, blue: 0.04)
        default: return .secondary
        }
    }
}

struct MenuBarView: View {
    @EnvironmentObject private var daemon: DaemonClient
    var body: some View {
        Text("Running: \(daemon.runningCount)")
        Divider()
        Button("Open Piercast") {
            NSApp.activate(ignoringOtherApps: true)
        }
        Button("Quit") { NSApp.terminate(nil) }
    }
}

struct CommandPaletteView: View {
    @EnvironmentObject private var daemon: DaemonClient
    @Environment(\.dismiss) private var dismiss
    @State private var q = ""

    var body: some View {
        VStack(spacing: 0) {
            TextField("Search apps and actions", text: $q)
                .textFieldStyle(.plain)
                .padding()
            List(filtered, id: \.id) { app in
                HStack {
                    Text(app.name)
                    Spacer()
                    Button("Open") { Task { await daemon.open(id: app.id); dismiss() } }
                    Button("Kill") { Task { await daemon.kill(id: app.id); dismiss() } }
                }
            }
        }
        .frame(width: 480, height: 360)
    }

    private var filtered: [PiercastAppInfo] {
        daemon.apps.filter { q.isEmpty || $0.name.localizedCaseInsensitiveContains(q) }
    }
}

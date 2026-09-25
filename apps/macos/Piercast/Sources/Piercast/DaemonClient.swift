import Foundation
import Combine

@MainActor
final class DaemonClient: ObservableObject {
    static let shared = DaemonClient()

    @Published var apps: [PiercastAppInfo] = []
    @Published var runningCount: Int = 0
    @Published var connected: Bool = false

    private let baseURL: URL
    private var token: String = ""

    init(baseURL: URL = URL(string: ProcessInfo.processInfo.environment["PIERCAST_URL"] ?? "http://127.0.0.1:47923")!) {
        self.baseURL = baseURL
    }

    func ensureRunning() async {
        if await healthOK() {
            connected = true
            await refreshApps()
            return
        }
        launchEmbeddedDaemon()
        for _ in 0..<40 {
            try? await Task.sleep(nanoseconds: 250_000_000)
            if await healthOK() {
                connected = true
                await refreshApps()
                return
            }
        }
        connected = false
    }

    private func healthOK() async -> Bool {
        var req = URLRequest(url: baseURL.appendingPathComponent("v1/health"))
        req.timeoutInterval = 1.5
        do {
            let (_, resp) = try await URLSession.shared.data(for: req)
            return (resp as? HTTPURLResponse)?.statusCode == 200
        } catch {
            return false
        }
    }

    private func launchEmbeddedDaemon() {
        let bundled = Bundle.main.url(forResource: "piercastd", withExtension: nil)
        let path = bundled?.path ?? "piercastd"
        let proc = Process()
        proc.executableURL = URL(fileURLWithPath: path)
        proc.arguments = []
        try? proc.run()
    }

    func refreshApps() async {
        guard let req = authedRequest(path: "v1/apps") else { return }
        do {
            let (data, _) = try await URLSession.shared.data(for: req)
            let decoded = try JSONDecoder().decode([PiercastAppInfo].self, from: data)
            apps = decoded
            runningCount = decoded.filter { $0.status == "running" || $0.status == "Running" }.count
        } catch {
            // keep prior list
        }
    }

    func open(id: String) async {
        _ = try? await post(path: "v1/apps/\(id)/open")
        await refreshApps()
    }

    func stop(id: String) async {
        _ = try? await post(path: "v1/apps/\(id)/stop")
        await refreshApps()
    }

    func kill(id: String) async {
        _ = try? await post(path: "v1/apps/\(id)/kill")
        await refreshApps()
    }

    private func post(path: String) async throws -> Data {
        var req = authedRequest(path: path) ?? URLRequest(url: baseURL)
        req.httpMethod = "POST"
        let (data, _) = try await URLSession.shared.data(for: req)
        return data
    }

    private func authedRequest(path: String) -> URLRequest? {
        var req = URLRequest(url: baseURL.appendingPathComponent(path))
        if token.isEmpty {
            token = loadPairingToken() ?? ""
        }
        if !token.isEmpty {
            req.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        return req
    }

    private func loadPairingToken() -> String? {
        let home = FileManager.default.homeDirectoryForCurrentUser
        let url = home.appendingPathComponent("Library/Application Support/Piercast/pairing.json")
        guard let data = try? Data(contentsOf: url),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let t = json["token"] as? String else { return nil }
        return t
    }
}

struct PiercastAppInfo: Codable, Identifiable {
    var id: String
    var name: String
    var status: String
    var tags: [String]?
}

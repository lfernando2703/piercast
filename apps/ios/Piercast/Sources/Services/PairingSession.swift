import Foundation
import Combine
import UIKit

struct PairedHost: Codable, Identifiable, Equatable {
    var id: String { hostName + "|" + baseURL }
    var baseURL: String
    var token: String
    var hostName: String
}

struct BootstrapPayload: Codable {
    var base_url: String
    var token: String
    var host_name: String
    var expires_at: String
}

@MainActor
final class PairingSession: ObservableObject {
    static let shared = PairingSession()

    @Published var hosts: [PairedHost] = []
    @Published var active: PairedHost?
    @Published var apps: [MobileAppInfo] = []

    private let defaultsKey = "piercast.paired.hosts"

    init() {
        load()
        active = hosts.first
    }

    func completePairing(payload: BootstrapPayload) async throws {
        // Exchange short-lived bootstrap token for long-lived device token
        guard let url = URL(string: payload.base_url + "/v1/pair/complete") else {
            throw URLError(.badURL)
        }
        var req = URLRequest(url: url)
        req.httpMethod = "POST"
        req.setValue("Bearer \(payload.token)", forHTTPHeaderField: "Authorization")
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try JSONSerialization.data(withJSONObject: [
            "device_name": UIDevice.current.name,
            "platform": "ios"
        ])
        let (data, resp) = try await URLSession.shared.data(for: req)
        guard let http = resp as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
            throw URLError(.userAuthenticationRequired)
        }
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        let deviceToken = (json?["token"] as? String) ?? payload.token
        let host = PairedHost(baseURL: payload.base_url, token: deviceToken, hostName: payload.host_name)
        hosts.removeAll { $0.hostName == host.hostName }
        hosts.append(host)
        active = host
        save()
        await refreshApps()
    }

    func refreshApps() async {
        guard let host = active else { apps = []; return }
        guard let url = URL(string: host.baseURL + "/v1/apps") else { return }
        var req = URLRequest(url: url)
        req.setValue("Bearer \(host.token)", forHTTPHeaderField: "Authorization")
        do {
            let (data, _) = try await URLSession.shared.data(for: req)
            apps = try JSONDecoder().decode([MobileAppInfo].self, from: data)
        } catch {
            apps = []
        }
    }

    func action(appId: String, path: String) async {
        guard let host = active,
              let url = URL(string: "\(host.baseURL)/v1/apps/\(appId)/\(path)") else { return }
        var req = URLRequest(url: url)
        req.httpMethod = "POST"
        req.setValue("Bearer \(host.token)", forHTTPHeaderField: "Authorization")
        _ = try? await URLSession.shared.data(for: req)
        await refreshApps()
    }

    private func save() {
        if let data = try? JSONEncoder().encode(hosts) {
            UserDefaults.standard.set(data, forKey: defaultsKey)
        }
    }

    private func load() {
        guard let data = UserDefaults.standard.data(forKey: defaultsKey),
              let decoded = try? JSONDecoder().decode([PairedHost].self, from: data) else { return }
        hosts = decoded
    }
}

struct MobileAppInfo: Codable, Identifiable {
    var id: String
    var name: String
    var status: String
    var open_url: String?
    var expose_url: String?
}

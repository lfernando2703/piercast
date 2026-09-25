using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Text.Json.Serialization;

namespace Piercast.Services;

public sealed class DaemonClient
{
    public static DaemonClient Shared { get; } = new();

    private readonly HttpClient _http = new() { Timeout = TimeSpan.FromSeconds(5) };
    private readonly string _baseUrl;
    private string _token = string.Empty;

    public event Action? Changed;
    public List<PiercastAppInfo> Apps { get; private set; } = [];
    public int RunningCount => Apps.Count(a => a.Status.Equals("running", StringComparison.OrdinalIgnoreCase));
    public bool Connected { get; private set; }

    private DaemonClient()
    {
        _baseUrl = Environment.GetEnvironmentVariable("PIERCAST_URL")?.TrimEnd('/')
                   ?? "http://127.0.0.1:47923";
    }

    public async Task EnsureRunningAsync()
    {
        if (await HealthOkAsync())
        {
            Connected = true;
            await RefreshAppsAsync();
            return;
        }

        TryLaunchDaemon();
        for (var i = 0; i < 40; i++)
        {
            await Task.Delay(250);
            if (await HealthOkAsync())
            {
                Connected = true;
                await RefreshAppsAsync();
                return;
            }
        }

        Connected = false;
        Changed?.Invoke();
    }

    public async Task RefreshAppsAsync()
    {
        EnsureAuth();
        try
        {
            var list = await _http.GetFromJsonAsync<List<PiercastAppInfo>>($"{_baseUrl}/v1/apps");
            Apps = list ?? [];
            Changed?.Invoke();
        }
        catch
        {
        }
    }

    public Task OpenAsync(string id) => PostAsync($"v1/apps/{id}/open");
    public Task StopAsync(string id) => PostAsync($"v1/apps/{id}/stop");
    public Task KillAsync(string id) => PostAsync($"v1/apps/{id}/kill");
    public Task RestartAsync(string id) => PostAsync($"v1/apps/{id}/restart");
    public Task StartAsync(string id, string mode = "development") =>
        PostAsync($"v1/apps/{id}/start", new { mode });

    private async Task PostAsync(string path, object? body = null)
    {
        EnsureAuth();
        using var resp = body is null
            ? await _http.PostAsync($"{_baseUrl}/{path}", null)
            : await _http.PostAsJsonAsync($"{_baseUrl}/{path}", body);
        await RefreshAppsAsync();
    }

    private async Task<bool> HealthOkAsync()
    {
        try
        {
            using var resp = await _http.GetAsync($"{_baseUrl}/v1/health");
            return resp.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }

    private void EnsureAuth()
    {
        if (!string.IsNullOrEmpty(_token)) return;
        _token = LoadPairingToken() ?? string.Empty;
        if (!string.IsNullOrEmpty(_token))
        {
            _http.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", _token);
        }
    }

    private static string? LoadPairingToken()
    {
        var appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        var path = Path.Combine(appData, "Piercast", "pairing.json");
        if (!File.Exists(path)) return null;
        try
        {
            var json = System.Text.Json.JsonDocument.Parse(File.ReadAllText(path));
            return json.RootElement.TryGetProperty("token", out var t) ? t.GetString() : null;
        }
        catch
        {
            return null;
        }
    }

    private static void TryLaunchDaemon()
    {
        try
        {
            System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
            {
                FileName = "piercastd",
                UseShellExecute = true,
                CreateNoWindow = true
            });
        }
        catch
        {
        }
    }
}

public sealed class PiercastAppInfo
{
    [JsonPropertyName("id")] public string Id { get; set; } = "";
    [JsonPropertyName("name")] public string Name { get; set; } = "";
    [JsonPropertyName("status")] public string Status { get; set; } = "stopped";
    [JsonPropertyName("tags")] public List<string>? Tags { get; set; }
}

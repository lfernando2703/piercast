namespace Piercast.Services;

/// <summary>
/// System tray via WinUI + notify icon placeholder.
/// Full CommunityToolkit tray hooks wire in packaging step.
/// </summary>
public static class TrayService
{
    public static void Initialize(Action openMain)
    {
        _ = openMain;
        _ = DaemonClient.Shared;
    }

    public static void SetAutostart(bool enabled)
    {
        const string runKey = @"Software\Microsoft\Windows\CurrentVersion\Run";
        using var key = Microsoft.Win32.Registry.CurrentUser.OpenSubKey(runKey, writable: true)
                        ?? Microsoft.Win32.Registry.CurrentUser.CreateSubKey(runKey);
        if (enabled)
        {
            var exe = Environment.ProcessPath ?? "Piercast.exe";
            key.SetValue("Piercast", $"\"{exe}\"");
        }
        else
        {
            key.DeleteValue("Piercast", throwOnMissingValue: false);
        }
    }
}

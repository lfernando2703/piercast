using Microsoft.UI.Xaml;
using Piercast.Services;

namespace Piercast;

public partial class App : Application
{
    private Window? _window;

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        _window = new MainWindow();
        _window.Activate();
        _ = DaemonClient.Shared.EnsureRunningAsync();
        TrayService.Initialize(() =>
        {
            _window.Activate();
        });
    }
}

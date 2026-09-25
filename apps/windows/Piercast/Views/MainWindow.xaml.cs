using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Piercast.Services;

namespace Piercast;

public sealed partial class MainWindow : Window
{
    private string _filter = "all";

    public MainWindow()
    {
        InitializeComponent();
        DaemonClient.Shared.Changed += Refresh;
        Refresh();
    }

    private void Refresh()
    {
        var apps = DaemonClient.Shared.Apps.AsEnumerable();
        apps = _filter switch
        {
            "running" => apps.Where(a => a.Status.Equals("running", StringComparison.OrdinalIgnoreCase)),
            "unhealthy" => apps.Where(a => a.Status.Equals("unhealthy", StringComparison.OrdinalIgnoreCase)),
            _ => apps
        };
        AppList.ItemsSource = apps.ToList();
    }

    private void Nav_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
    {
        if (args.SelectedItem is NavigationViewItem item && item.Tag is string tag)
        {
            _filter = tag;
            Refresh();
        }
    }

    private async void Open_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: string id })
            await DaemonClient.Shared.OpenAsync(id);
    }

    private async void Stop_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: string id })
            await DaemonClient.Shared.StopAsync(id);
    }
}

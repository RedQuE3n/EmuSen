using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Avalonia.Platform;
using EmuSen.Mistress.Views;

namespace EmuSen.Mistress
{
    public partial class App : Application
    {
        internal static readonly Uri IconUri = new("avares://EmuSen.Mistress/Assets/Icon/png/emusen-256.png");

        private static WindowIcon? _icon;
        private static bool _iconOnEveryWindow;

        // The crescent-and-E icon every Mistress window shows in its frame and the taskbar - see EmuSen_Settings_Reference.md §4.55.
        internal static WindowIcon Icon => _icon ??= new WindowIcon(AssetLoader.Open(IconUri));

        internal static void ShowIconOnEveryWindow()
        {
            if (_iconOnEveryWindow) return;
            _iconOnEveryWindow = true;
            Window.WindowOpenedEvent.AddClassHandler<Window>((window, _) => window.Icon ??= Icon);
        }

        public override void Initialize()
        {
            AvaloniaXamlLoader.Load(this);
        }

        public override void OnFrameworkInitializationCompleted()
        {
            ShowIconOnEveryWindow();
            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                desktop.MainWindow = new MainWindow();
            }

            base.OnFrameworkInitializationCompleted();
        }
    }
}

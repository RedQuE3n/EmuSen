using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;

namespace EmuSen.Shell
{
    public partial class App : Application
    {
        // A factory, not a window: Avalonia has to be running before any Window is constructed - see EmuSen_Frontend_Driver.md §1.
        private readonly Func<Window> _mainWindow;

        public App(Func<Window> mainWindow)
        {
            _mainWindow = mainWindow;
        }

        public override void Initialize()
        {
            AvaloniaXamlLoader.Load(this);
        }

        public override void OnFrameworkInitializationCompleted()
        {
            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                // The shell window outlives the game window it opens, so the last one out ends the process.
                desktop.ShutdownMode = ShutdownMode.OnLastWindowClose;
                desktop.MainWindow = _mainWindow();
            }

            base.OnFrameworkInitializationCompleted();
        }
    }
}

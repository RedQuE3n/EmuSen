using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using EmuSen.Cores;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Hotaru.Views;

namespace EmuSen.Hotaru
{
    public partial class App : Application
    {
        // A factory, not a window: Avalonia has to be running before any Window is constructed.
        private readonly Func<Window> _mainWindow;

        // Takes every already-constructed piece of emulator/debug state
        // Program.cs's Main built before Avalonia ever starts (see that
        // file's own comment on why - RunStandaloneShell's plain-console
        // shell has to fully resolve a ROM, build the core BEFORE any
        // Avalonia type is touched at all) rather than reconstructing any
        // of it here - Main hands it to
        // AppBuilder.Configure<App>(() => new App(...)) instead of the
        // usual parameterless Configure<App>(). Takes extraCommands, not
        // a pre-built DianaOSInterpreter, since GameWindow itself now
        // owns building (and REbuilding, after a `core <name> <path>`
        // swap) the actual interpreter - see that class's own
        // RebuildDebugTargetAndCommands().
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

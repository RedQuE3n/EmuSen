using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.Hotaru.Views;

namespace EmuSen.Hotaru
{
    public partial class App : Application
    {
        private readonly VenusCore _core;
        private readonly SnesDebugTarget _debugTarget;
        private readonly DianaOSInterpreter _debugCmd;
        private readonly FrameRecorder _frameRecorder;
        private readonly string _statePath;

        // Takes every already-constructed piece of emulator/debug state
        // Program.cs's Main built before Avalonia ever starts (see that
        // file's own comment on why - RunStandaloneShell's plain-console
        // shell has to fully resolve a ROM, build the core, and wire up
        // the debug toolchain BEFORE any Avalonia type is touched at all)
        // rather than reconstructing any of it here - Main hands it to
        // AppBuilder.Configure<App>(() => new App(...)) instead of the
        // usual parameterless Configure<App>().
        public App(VenusCore core, SnesDebugTarget debugTarget, DianaOSInterpreter debugCmd, FrameRecorder frameRecorder, string statePath)
        {
            _core = core;
            _debugTarget = debugTarget;
            _debugCmd = debugCmd;
            _frameRecorder = frameRecorder;
            _statePath = statePath;
        }

        public override void Initialize()
        {
            AvaloniaXamlLoader.Load(this);
        }

        public override void OnFrameworkInitializationCompleted()
        {
            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                desktop.MainWindow = new GameWindow(_core, _debugTarget, _debugCmd, _frameRecorder, _statePath);
            }

            base.OnFrameworkInitializationCompleted();
        }
    }
}

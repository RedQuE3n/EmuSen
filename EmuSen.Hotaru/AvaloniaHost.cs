using System;
using System.Threading;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using Avalonia.Themes.Fluent;
using EmuSen.DianaOS;
using EmuSen.Hotaru.Views;

namespace EmuSen.Hotaru
{
    // Boots a minimal Avalonia UI stack on its OWN dedicated background
    // thread, entirely separate from Program.cs's own main thread (which
    // Raylib's window/game loop owns) - lets `coretop -w` open a real,
    // independent GUI window without Raylib and Avalonia fighting over
    // "the" main thread the way two GUI toolkits normally would. Started
    // lazily, exactly once, the first time `-w` is ever used in a given
    // process - most sessions never touch this at all, so there's no
    // cost paid for carrying the Avalonia dependency unless it's
    // actually exercised.
    //
    // A known, accepted risk worth being explicit about: the
    // IDebugTarget methods CoretopWindow's own refresh timer calls
    // (GetCpuRegisters, GetHardwareLoad, ...) run on THIS dedicated
    // Avalonia thread while Program.cs's main loop concurrently calls
    // core.RunFrame() on its own thread, with no synchronization between
    // the two - the same class of narrow data race EmuSen.Mistress9's
    // own console window has (see EmulationControlCommands.cs's
    // pause/resume commands, built specifically to close that gap
    // there). Nothing here pauses the console build's emulation loop the
    // way Mistress9's `pause` command can, since this frontend never had
    // a pause mechanism to begin with - accepted for now as a read-
    // mostly debug view, not something feeding back into gameplay logic.
    internal static class AvaloniaHost
    {
        private static readonly object _startLock = new();
        private static Thread? _uiThread;
        private static readonly ManualResetEventSlim _ready = new(false);
        private static CoretopWindow? _coretopWindow;
        private static FeedWindow? _feedWindow;

        // Called from CoretopCommand's own `-w` handling (via a plain
        // Action<IDebugTarget> delegate - see that class's own comment on
        // why it takes one instead of referencing this Avalonia-specific
        // type directly). Safe to call repeatedly: the first call starts
        // the Avalonia thread; every call (first or not) posts onto it to
        // create-or-reuse-and-update the one CoretopWindow instance, the
        // same at-most-one/reuse pattern EmuSen.Mistress9's own
        // OpenCoretopWindow uses.
        public static void ShowCoretopWindow(IDebugTarget target)
        {
            EnsureStarted();
            Dispatcher.UIThread.Post(() =>
            {
                if (_coretopWindow is null)
                {
                    _coretopWindow = new CoretopWindow(target);
                    _coretopWindow.Closed += (_, _) => _coretopWindow = null;
                    _coretopWindow.Show();
                }
                else
                {
                    _coretopWindow.UpdateTarget(target);
                    _coretopWindow.Activate();
                }
            });
        }

        // Called from the `feed -w` handling in Program.cs's own
        // RunDebugPrompt - a live mirror of the actual game picture
        // (ICore.GetFrameBufferRgba()), not hardware/debug data the way
        // ShowCoretopWindow's target is. Takes a plain frame-provider
        // delegate instead of an ICore/VenusCore reference so this class
        // (and FeedWindow itself) stay just as core-agnostic as
        // ShowCoretopWindow already is via IDebugTarget - Program.cs
        // supplies `() => (core.GetFrameBufferRgba(), core.ScreenWidth,
        // core.ScreenHeight)` as that provider. Same at-most-one/reuse
        // pattern as ShowCoretopWindow: the provider closure is only
        // ever bound once per process (Hotaru has no ROM-hot-swap - a
        // new ROM means a new process), so reusing an existing window
        // just needs to bring it forward, never rebind it to a new
        // target the way coretop's UpdateTarget does.
        public static void ShowFeedWindow(Func<(byte[] Rgba, int Width, int Height)> frameProvider)
        {
            EnsureStarted();
            Dispatcher.UIThread.Post(() =>
            {
                if (_feedWindow is null)
                {
                    _feedWindow = new FeedWindow(frameProvider);
                    _feedWindow.Closed += (_, _) => _feedWindow = null;
                    _feedWindow.Show();
                }
                else
                {
                    _feedWindow.Activate();
                }
            });
        }

        private static void EnsureStarted()
        {
            lock (_startLock)
            {
                if (_uiThread is null)
                {
                    _uiThread = new Thread(RunAvaloniaDispatcherLoop)
                    {
                        IsBackground = true,
                        Name = "Avalonia-Coretop",
                    };
                    // STA only matters (and is only supported at all) on
                    // Windows - some Avalonia backends/interop paths
                    // there assume it. SetApartmentState throws
                    // PlatformNotSupportedException on every other OS.
                    if (OperatingSystem.IsWindows()) _uiThread.SetApartmentState(ApartmentState.STA);
                    _uiThread.Start();
                }
            }

            _ready.Wait();
        }

        private static void RunAvaloniaDispatcherLoop()
        {
            AppBuilder.Configure<Application>()
                .UsePlatformDetect()
                .WithInterFont()
                .SetupWithoutStarting();

            // Not using the normal App.axaml-driven startup (there's no
            // XAML application resource file in this project at all - a
            // console/Raylib project has never needed one before this),
            // so the Fluent theme has to be attached by hand here instead
            // of via <Application.Styles> - without it, controls like
            // ProgressBar have no visual template at all.
            if (Application.Current is not null)
            {
                Application.Current.Styles.Add(new FluentTheme());
            }

            _ready.Set();

            // Blocks THIS thread forever, pumping Avalonia's own
            // dispatcher queue (window creation, timer ticks, redraws,
            // input) - the equivalent of a normal Avalonia app's
            // Application.Run, just not tied to this process's main
            // thread the way that call normally would be.
            Dispatcher.UIThread.MainLoop(CancellationToken.None);
        }
    }
}

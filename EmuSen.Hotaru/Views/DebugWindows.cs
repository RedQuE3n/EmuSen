using System;
using Avalonia.Threading;
using EmuSen.DianaOS;

namespace EmuSen.Hotaru.Views
{
    // coretop -w / feed -w window management - replaces AvaloniaHost.cs's
    // static state (deleted - see git history) now that Avalonia is
    // Hotaru's primary application (see Program.cs/App.axaml.cs) instead
    // of a second dispatcher thread bootstrapped alongside a separate
    // Raylib main loop. Dispatcher.UIThread is already pumping by the
    // time either method here can ever be called - both are only
    // reachable via commands typed at the F4 prompt or 'feed'/'feed -w'
    // (see GameWindow.axaml.cs's RunDebugPrompt), which only run after
    // GameWindow's constructor - and therefore
    // App.OnFrameworkInitializationCompleted - has already started the
    // Avalonia dispatcher, so there's no "start a UI thread first" step
    // left to do at all, just the same at-most-one/reuse-or-create window
    // pattern AvaloniaHost already had.
    internal static class DebugWindows
    {
        private static CoretopWindow? _coretopWindow;
        private static FeedWindow? _feedWindow;

        public static void ShowCoretopWindow(IDebugTarget target)
        {
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

        // Keeps an already-open coretop window from silently going stale
        // after a `core <name> <path>` swap (GameWindow.SwapCore) rebuilds
        // _debugTarget - unlike ShowCoretopWindow above, this never
        // CREATES the window (a swap shouldn't pop one up for a user who
        // never asked for one) and never Activate()s it (stealing window
        // focus on every swap, mid-gameplay, would be its own new
        // annoyance) - it only refreshes whatever's already open, so it's
        // safe to call unconditionally on every swap regardless of
        // whether coretop is even in use this session.
        public static void UpdateCoretopWindowTargetIfOpen(IDebugTarget target)
        {
            if (_coretopWindow is null) return;
            Dispatcher.UIThread.Post(() => _coretopWindow?.UpdateTarget(target));
        }

        public static void ShowFeedWindow(Func<(byte[] Rgba, int Width, int Height)> frameProvider)
        {
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
    }
}

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

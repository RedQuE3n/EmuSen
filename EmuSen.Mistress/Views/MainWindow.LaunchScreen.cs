using System;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;

namespace EmuSen.Mistress.Views
{
    // ES-DE's launch screen between the resume question and a big-picture game's first frame - see EmuSen_Settings_Reference.md §4.71.
    public partial class MainWindow
    {
        private LaunchScreen? _launchScreen;
        private TaskCompletionSource<bool>? _launchWait;
        private DispatcherTimer? _launchTimer;
        private bool _launchFrameAsked;

        internal LaunchScreen? LaunchScreenShown => _launchScreen is { IsOpen: true } screen ? screen : null;

        internal bool LaunchScreenOpen => _launchScreen?.IsOpen == true;

        // The game's name, its system and its marquee, else its cover, each by the library's own order of sources (§4.60).
        internal (string Name, string System, string? Art) LaunchContent(SceneGame game)
        {
            var shelf = EmuSen.Cores.CoreCatalog.ShelfByName(EmuSen.Cores.CoreCatalog.ShelfFor(game.File));
            string system = shelf?.EsdeFullName is { Length: > 0 } full ? full : shelf?.Name ?? "";
            string? art = null;
            if (shelf?.EsdeSystem is { Length: > 0 } esde)
            {
                var sources = MediaSourcesNow();
                art = sources.Locate(esde, game.File, "marquee").Path ?? sources.Cover(esde, game.File);
            }
            return (game.Name, system, art);
        }

        // True when the game should start: after the screen's time, at once when it is Disabled; false when the window closed first.
        private async Task<bool> ShowLaunchScreenAsync(SceneGame game)
        {
            string setting = _appSettings.BigPictureInterface.LaunchScreenDuration;
            TimeSpan duration = LaunchScreen.DurationOf(setting);
            if (duration <= TimeSpan.Zero) return true;

            if (_launchScreen is null)
            {
                _launchScreen = new LaunchScreen { Name = "LaunchScreen" };
                if (Sheets.Parent is Panel root) root.Children.Insert(root.Children.IndexOf(Sheets), _launchScreen);
            }
            (string name, string system, string? art) = LaunchContent(game);
            _launchScreen.Backdrop = ThemedLibraryHost;
            LetGoOfTheThemedDirection(UiClock());
            _launchScreen.Open(name, system, art, LaunchScreen.IsPopup(setting), UiClock(), duration);
            _launchWait = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
            ScheduleLaunchScreen();
            return await _launchWait.Task;
        }

        // Steps the screen to the interface clock: its entrance, then the start at its end.
        internal void AdvanceLaunchScreen()
        {
            if (_launchScreen is not { IsOpen: true } screen) return;
            TimeSpan now = UiClock();
            screen.Advance(now);
            if (now >= screen.Due)
            {
                _launchTimer?.Stop();
                _launchWait?.TrySetResult(true);
                return;
            }
            ScheduleLaunchScreen();
        }

        private void ScheduleLaunchScreen()
        {
            if (_launchScreen is not { IsOpen: true } screen) return;
            TimeSpan now = UiClock();
            if (screen.Moving(now))
            {
                if (_launchFrameAsked) return;
                _launchFrameAsked = true;
                RequestAnimationFrame(_ => { _launchFrameAsked = false; AdvanceLaunchScreen(); });
                return;
            }
            _launchTimer ??= new DispatcherTimer();
            _launchTimer.Stop();
            _launchTimer.Interval = screen.Due > now ? screen.Due - now : TimeSpan.FromMilliseconds(1);
            _launchTimer.Tick -= OnLaunchTimer;
            _launchTimer.Tick += OnLaunchTimer;
            _launchTimer.Start();
        }

        private void OnLaunchTimer(object? sender, EventArgs e)
        {
            _launchTimer?.Stop();
            AdvanceLaunchScreen();
        }

        // Taken down when the game's first frame is due, or when the load failed and the library is back.
        private void CloseLaunchScreen()
        {
            _launchTimer?.Stop();
            _launchScreen?.Close();
            _launchWait?.TrySetResult(false);
            _launchWait = null;
        }
    }
}

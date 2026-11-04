using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;

namespace EmuSen.Mistress.Views
{
    // The launch screen between the resume question and a game's first frame, under every look - see EmuSen_Interface.md §18.3.
    public partial class MainWindow
    {
        private LaunchScreen? _launchScreen;
        private TaskCompletionSource<bool>? _launchWait;
        private DispatcherTimer? _launchTimer;
        private bool _launchFrameAsked;

        internal LaunchScreen? LaunchScreenShown => _launchScreen is { IsOpen: true } screen ? screen : null;

        internal bool LaunchScreenOpen => _launchScreen?.IsOpen == true;

        // This window's switch, from LaunchScreen.ShownByDefault; a test about the screen sets it.
        internal bool LaunchScreens { get; set; } = LaunchScreen.ShownByDefault;

        private bool LaunchScreenOn => LaunchScreens && LaunchScreen.Shows(_appSettings.BigPictureInterface.LaunchScreenDuration);

        public const string LoadingLine = "Loading the game", ResumingLine = "Loading where you left off", NoDiscLine = "Starting with no disc in it";

        // The line for a state chosen on the game's page: its slot by name, or where the player left off.
        private string StateLineFor(string path, string state) =>
            ((IGamePageHost)this).States(path).FirstOrDefault(s => s.Path == state)?.Slot is { } slot && slot != GamePage.AutoSlot ? $"Loading the state in {slot}" : ResumingLine;

        // A game's card: its name, console and engine, and its marquee else its cover, each through the library's own media lookup (§4.60).
        internal LaunchCard LaunchCardFor(string path, string? name, string line)
        {
            GameFacts facts = FactsShown(path);
            GameArt art = ((IGamePageHost)this).Art(path);
            string console = RuntimeCatalog.ConsoleForRom(path) ?? "";
            string? engine = GameMetadata.EngineFor(path, ((IGamePageHost)this).Engine(path)) ?? RuntimeCatalog.SystemOf(console)?.Engines.FirstOrDefault();
            string system = engine is { Length: > 0 } ? $"{facts.Console} · {engine}" : facts.Console;
            return new LaunchCard(name ?? facts.Title, system, art.Marquee ?? art.Cover, art.Marquee is not null, line);
        }

        // A console started empty: its name and engine, no game's name and no picture.
        public static LaunchCard LaunchCardForConsole(string console)
        {
            var system = RuntimeCatalog.SystemOf(console);
            string name = system?.DisplayName is { Length: > 0 } shown ? shown : system?.Name ?? console;
            string? engine = system?.Engines.FirstOrDefault();
            return new LaunchCard(null, engine is { Length: > 0 } ? $"{name} · {engine}" : name, null, false, NoDiscLine);
        }

        // True when the start should go on: after the hold, at once when the screen is off; false when the window closed first.
        private async Task<bool> ShowLaunchScreenAsync(LaunchCard card)
        {
            if (!LaunchScreenOn) return true;
            if (_launchScreen is null)
            {
                _launchScreen = new LaunchScreen();
                if (Sheets.Parent is Panel root) root.Children.Insert(root.Children.IndexOf(Sheets), _launchScreen);
            }
            if (ThemedLibraryShown) LetGoOfTheThemedDirection(UiClock());
            _launchScreen.Open(card, UiClock(), LaunchScreen.HoldFor(_appSettings.BigPictureInterface.LaunchScreenDuration));
            _launchWait = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
            ScheduleLaunchScreen();
            return await _launchWait.Task;
        }

        // A start that is not a game file's: the screen, then the start, then the screen down once it is on.
        private async Task UnderLaunchScreenAsync(LaunchCard card, Action start)
        {
            if (!await ShowLaunchScreenAsync(card)) return;
            if (_recordsClosed) return;
            start();
            CloseLaunchScreen();
        }

        // Steps the screen to the interface clock: its bar, then the start at its end.
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

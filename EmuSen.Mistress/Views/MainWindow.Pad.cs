using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Threading;
using EmuSen.Mistress.Input;
using SDL3;

namespace EmuSen.Mistress.Views
{
    // The interface steered from a pad: the library, a menu over either screen, and any other window through its keys - see EmuSen_Settings_Reference.md §4.29.
    public partial class MainWindow
    {
        private readonly PadNavigator _padNavigator = new();
        private readonly Stopwatch _padClock = Stopwatch.StartNew();
        private readonly List<PadMenuEntry> _padMenuEntries = new();
        private DispatcherTimer? _padTimer;
        private bool _padMenuOpen;
        private bool _padMenuPaused;

        // Set when a menu closes, until every button is let go, so the press that closed it is not the game's.
        private bool _padQuiet;

        private const double PadStickThreshold = 0.55;

        private bool GameOnScreen => GameFrame.IsVisible && _session is { IsRomLoaded: true };

        // Asked for by the setting, by --bigscreen, or by a Deck's Game Mode, which SteamDeck=1 alone does not tell from Desktop Mode - see EmuSen_Settings_Reference.md §4.43.
        private bool WantsBigScreen() => WantsBigScreen(_appSettings.BigScreen, Environment.GetCommandLineArgs(), Environment.GetEnvironmentVariable);

        internal static bool WantsBigScreen(bool setting, string[] args, Func<string, string?> environment) =>
            setting || args.Contains("--bigscreen") || InGameModeSession(environment);

        internal static bool InGameModeSession(Func<string, string?> environment)
        {
            string? desktop = environment("XDG_CURRENT_DESKTOP");
            if (string.IsNullOrEmpty(desktop)) return environment("SteamDeck") == "1";
            return desktop.Split(':').Any(d => d.Equals("gamescope", StringComparison.OrdinalIgnoreCase));
        }

        private void StartPadNavigation()
        {
            // Wired once; what big screen changes is ApplyBigScreen's, which a switch runs again - see EmuSen_Settings_Reference.md §4.54.
            ApplyPadHints();
            Sheets.PresentedChanged += OnSheetsChanged;
            SizeChanged += (_, e) => Sheets.Scale = Math.Clamp(e.NewSize.Height / 720.0, 1.0, 2.0);
            SizeChanged += (_, e) => PadNotice.FontSize = _bigScreen ? 18 * Math.Clamp(e.NewSize.Height / 720.0, 1.0, 2.0) : 15;
            SetUpBigPictureSwitch();
            SetUpBigMenus();
            ApplyBigScreen(WantsBigScreen());

            PadMenuList.Label = entry => entry.Submenu is null ? entry.Text() : entry.Text() + "  ›";
            SetUpPadMenuPointer();
            _padTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(16) };
            _padTimer.Tick += (_, _) => PadTick();
            _padTimer.Start();
        }

        private void PadTick()
        {
            // The launch screen takes no input, as ES-DE's does not (§4.71).
            if (LaunchScreenOpen)
            {
                _gamepad.Poll();
                AdvanceLaunchScreen();
                if (LaunchScreenOpen) _padNavigator.Forget(PadHeld);
                return;
            }
            _gamepad.Poll();
            _menuOpening.Advance(UiClock());
            // Before anything else reads the pad: the press that wakes the screensaver is its own (§4.75).
            if (StepScreensaver()) return;
            if (!KeyboardSteers() && !_gamepad.IsConnected) return;
            TrackControllers();

            // A game on screen owns the pad; only the chord reaches the interface - see §4.29.
            if (GameOnScreen && !_padMenuOpen && OtherWindow() is null)
            {
                bool chord = _padNavigator.MenuChord(PadHeld);
                _padNavigator.Forget(PadHeld);
                if (chord) OpenPadMenu();
                return;
            }

            _padNavigator.MenuChord(PadHeld);
            TimeSpan now = UiClock();

            // The themed view takes the directions as held and let go, for its own repeats - see EmuSen_Settings_Reference.md §4.52.
            bool themed = ThemedTakesThePad;
            if (ThemedLibraryShown) _themed!.SetPadLayout(HelpFamily, _appSettings.SwapPadButtons);
            if (themed) ThemedDirections(now);
            else LetGoOfTheThemedDirection(now);

            foreach (UiButton press in _padNavigator.Feed(PadHeld, now).ToArray())
            {
                if (themed && Directions.Contains(press)) continue;
                OnPadCommand(press);
            }

            if (ThemedLibraryShown) ScheduleThemedFrame();
        }

        // True while the game must not hear the pad: a menu is over it, or the button that closed one is still down.
        private bool PadBelongsToTheInterface()
        {
            if (_padMenuOpen) return true;
            if (OtherWindow() is not null) return _padQuiet = true;
            if (!_padQuiet) return false;
            if (Enum.GetValues<UiButton>().Any(PadHeld)) return true;
            _padQuiet = false;
            return false;
        }

        private bool _sheetPaused;

        // A sheet over a game pauses it, and its last one resumes only what it paused - the pad menu's rule (§4.29).
        private void OnSheetsChanged()
        {
            UpdateMenuBackdrop();
            if (Sheets.IsPresenting)
            {
                if (_sheetPaused || !GameOnScreen || IsPaused) return;
                PauseEmulation();
                _sheetPaused = true;
                return;
            }

            if (_sheetPaused && GameOnScreen && IsPaused && !_padMenuOpen) ResumeEmulation();
            _sheetPaused = false;
        }

        // A sheet first; else another window, which has the pad only while it is the active one, so a debug window left open beside a game takes nothing.
        private Window? OtherWindow() =>
            Sheets.Current
            ?? OwnedWindows.LastOrDefault(w => w.IsVisible && w.IsActive)
            ?? (Application.Current?.ApplicationLifetime as IClassicDesktopStyleApplicationLifetime)?.Windows
                .LastOrDefault(w => !ReferenceEquals(w, this) && w.IsVisible && w.IsActive);

        internal void OnPadCommand(UiButton button)
        {
            if (button == UiButton.Guide) button = UiButton.Menu;
            if (ScreensaverFromButton(button)) return;

            if (OtherWindow() is { } other) PadWindowRouter.Send(other, button);
            else if (EmuSen.LunaP.Controls.OnScreenKeyboard.OpenOver(this) is { } keyboard) PadKeyboard.Send(keyboard, button);
            else if (EmuSen.LunaP.Controls.MenuTextPopup.OpenOver(this) is { } popup) PadKeyboard.Send(popup, button);
            else if (_padMenuOpen) PadMenuCommand(button);
            else if (LibraryView.IsVisible && ThemedLibraryShown) ThemedPadCommand(button);
            else if (LibraryView.IsVisible) LibraryPadCommand(button);
        }

        private void LibraryPadCommand(UiButton button)
        {
            switch (button)
            {
                // Covers move in two dimensions, so the shoulders take the console instead - see EmuSen_Settings_Reference.md §4.33.
                case UiButton.Up: MoveLibrarySelection(GridShown ? -LibraryGrid.Columns : -1); break;
                case UiButton.Down: MoveLibrarySelection(GridShown ? LibraryGrid.Columns : 1); break;
                case UiButton.PageUp: if (GridShown) StepLibraryConsole(-1); else MoveLibrarySelection(-10); break;
                case UiButton.PageDown: if (GridShown) StepLibraryConsole(1); else MoveLibrarySelection(10); break;
                case UiButton.First: MoveLibrarySelection(int.MinValue / 2); break;
                case UiButton.Last: MoveLibrarySelection(int.MaxValue / 2); break;
                case UiButton.Left: if (GridShown) MoveLibrarySelection(-1); else StepLibraryConsole(-1); break;
                case UiButton.Right: if (GridShown) MoveLibrarySelection(1); else StepLibraryConsole(1); break;
                case UiButton.Options: if (LibraryList.Selected is EmuSen.Mistress.Library.RomEntry chosen) ToggleFavourite(chosen); break;
                case UiButton.Accept: LaunchSelectedLibraryEntry(); break;
                case UiButton.Search: SearchFromThePad(); break;
                case UiButton.Back: if (!LeaveTheSearchBox() && _session is { IsRomLoaded: true }) ToggleLibrary(); break;
                case UiButton.Menu: OpenPadMenu(); break;
            }
        }

        // The search box takes the focus and the on-screen keyboard opens on it; once it is put away the list still moves under the box - see §4.45.6.
        private void SearchFromThePad()
        {
            LibraryFilter.FocusSearch();
            if (FocusManager?.GetFocusedElement() is TextBox box) PadKeyboard.Open(box);
        }

        // Back out of the search box to the list, keeping what was typed.
        private bool LeaveTheSearchBox()
        {
            if (FocusManager?.GetFocusedElement() is not TextBox) return false;
            // The rows take the focus, not the list; with no row the window takes it.
            if (LibraryList.ContainerFromIndex(Math.Max(LibraryList.SelectedIndex, 0)) is { } row) row.Focus();
            else Focus();
            return true;
        }

        private void MoveLibrarySelection(int by)
        {
            int count = LibraryList.ItemCount;
            if (count == 0) return;

            int index = Math.Clamp(Math.Max(LibraryList.SelectedIndex, 0) + by, 0, count - 1);
            LibraryList.SelectedIndex = index;
            LibraryList.ScrollIntoView(index);
        }

        // The console filter stepped round, as choosing it from the bar would do.
        private void StepLibraryConsole(int by)
        {
            IReadOnlyList<string> choices = EmuSen.Cores.CoreCatalog.FilterChoices;
            int at = Math.Max(0, choices.ToList().IndexOf(LibraryFilter.Facet as string ?? SelectedConsole));
            string next = choices[(at + by + choices.Count) % choices.Count];

            LibraryFilter.SetFacets(choices, next);
            OnLibraryFilterChanged();
        }

        private string PadLibraryHint()
        {
            string back = _session is { IsRomLoaded: true } ? $"B  Back to {_currentDisplayName}      " : "";
            string moves = GridShown ? "L1 R1  Console      L2 R2  Top, End" : "L1 R1  Page      L2 R2  Top, End      Left Right  Console";
            return PadHints.Face($"A  Play      {back}Y  Search      Select  Favourite      {moves}      Start  Menu");
        }
    }
}

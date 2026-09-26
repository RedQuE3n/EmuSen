using System;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // Big picture entered and left while the window runs; on the desktop it is the window's full screen, however that was reached - see EmuSen_Settings_Reference.md §4.54.
    public partial class MainWindow
    {
        private bool _bigScreen;

        // A Game Mode session is full screen and big picture for its whole life, so nothing offers to leave it.
        private bool _bigScreenForced;

        private LunaAction? _bigPictureAction;

        // Where the desktop library stood when big picture was entered, given back when it is left.
        private DesktopPlace? _desktopPlace;

        private sealed record DesktopPlace(string Console, string Search, string? Game);

        internal bool BigPictureOn => _bigScreen;

        internal bool BigPictureForced => _bigScreenForced;

        // Whether the interface's sound stream is held; a test reads it.
        internal bool UiSoundsHeld => _uiSounds is not null;

        // A platform option read once, so only a session that is big-screen for its whole life embeds every popup; a switch embeds the window's own (§4.54).
        public static bool EmbedsPopupsAtStart(string[] args, Func<string, string?> environment) =>
            args.Contains("--bigscreen") || InGameModeSession(environment);

        private void SetUpBigPictureSwitch()
        {
            _bigScreenForced = InGameModeSession(Environment.GetEnvironmentVariable);
            _bigPictureAction = new LunaAction("Fullscreen", () => SetBigPicture(!_bigScreen)) { Shortcut = _fullscreen.Shortcut };
            BigPictureButtons.ItemsSource = new Control[] { new ActionButton(_bigPictureAction) { Name = "BigPictureButton" } };
            FullScreenChanged += on => { if (!_bigScreenForced) SetBigPicture(on); };
            Opened += (_, _) => { if (_bigScreen) IsFullScreen = true; };
        }

        // Everything big screen changes in the window, set for either answer, so a switch is this run again.
        private void ApplyBigScreen(bool on)
        {
            _bigScreen = on;
            MenuStrip.IsVisible = !on;
            // A desktop sidebar costs a handheld its width, so the console choice goes back in the filter bar - see EmuSen_Settings_Reference.md §4.33.
            LibrarySidebarPane.IsVisible = !on;
            LibraryFilter.ShowFacet = on;
            Enlarge(LibraryList, TemplatedControl.FontSizeProperty, on, 24);
            Enlarge(LibraryHeaderText, TextBlock.FontSizeProperty, on, 17);
            Enlarge(LibraryHintText, TextBlock.FontSizeProperty, on, 17);

            // One window on screen in a big-screen session, so the others are drawn inside this one - see EmuSen_Settings_Reference.md §4.45.2.
            Sheets.PresentsWindows = on;
            EmbeddedPopups.SetIsEnabled(this, on);

            BigPictureButtons.IsVisible = !_bigScreenForced;
            _bigPictureAction!.Text = on ? "Exit Big Picture" : "Fullscreen";
            _bigPictureAction.HelpText = on
                ? "Back to the desktop library, as the window was"
                : "Opens big picture mode: the library full screen, for a pad or a television. Esc or the same key comes back.";

            if (on && _themed is null) SetUpThemedLibrary();
        }

        private static void Enlarge(AvaloniaObject control, StyledProperty<double> size, bool on, double points)
        {
            if (on) control.SetValue(size, points);
            else control.ClearValue(size);
        }

        // Enters or leaves big picture: the layout, the library, the popups and sheets, the window's full screen, and the setting the next start reads.
        internal void SetBigPicture(bool on)
        {
            if (on == _bigScreen || (!on && _bigScreenForced)) return;

            if (on) _desktopPlace = DesktopPlaceNow();
            ApplyBigScreen(on);

            // Leaving stops the themed view: ReturnTo's showing hides its host, which stops the wake, and the sound stream goes here.
            if (on) ShowLibraryEntries();
            else
            {
                ReleaseUiSounds();
                ReturnTo(_desktopPlace ?? DesktopPlaceNow());
                _desktopPlace = null;
            }

            _appSettings.BigScreen = on;
            _appSettings.Save();
            IsFullScreen = on;
        }

        // Esc leaves only where it has nothing else to do: the library showing, no game behind it, nothing over it.
        private bool EscapeLeavesBigPicture =>
            _bigScreen && !_bigScreenForced && LibraryView.IsVisible && _session is not { IsRomLoaded: true } && !_padMenuOpen && !Sheets.IsPresenting
            && OnScreenKeyboard.OpenOver(this) is null;

        private DesktopPlace DesktopPlaceNow() =>
            new(SelectedConsole, LibraryFilter.SearchText, LibraryList.Selected?.FullPath);

        private void ReturnTo(DesktopPlace place)
        {
            LibraryFilter.SearchText = place.Search;
            if (place.Console != SelectedConsole)
            {
                LibraryFilter.SetFacets(EmuSen.Cores.CoreCatalog.FilterChoices, place.Console);
                OnLibraryFilterChanged();
            }
            ShowLibraryEntries();

            if (_shownEntries.FirstOrDefault(e => e.FullPath == place.Game) is not { } game) return;
            LibraryList.Select(game);
            LibraryGrid.Select(game);
        }

        // The interface's second stream let go, so the desktop holds none on the device; the next showing preloads and the next sound opens it again.
        private void ReleaseUiSounds()
        {
            _uiSounds?.Dispose();
            _uiSounds = null;
        }
    }
}

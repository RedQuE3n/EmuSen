using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Threading;
using EmuSen.Endymion;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // A big-screen session's library drawn from an ES-DE theme, steered by the pad, and the render loop that draws it only while it changes - see EmuSen_Settings_Reference.md §4.52.
    public partial class MainWindow
    {
        private ThemedLibrary? _themed;
        private TextBox? _themedSearch;
        private Border? _themedSearchBar;
        private UiSoundPlayer? _uiSounds;
        private DispatcherTimer? _themedWake;
        private bool _themedFrameAsked;
        private UiButton? _themedHeld;

        private static readonly UiButton[] Directions = [UiButton.Up, UiButton.Down, UiButton.Left, UiButton.Right];

        private Func<TimeSpan>? _uiClock;

        // The interface's clock: the pad's repeats and the themed view's motion read it, and a test replaces it.
        internal Func<TimeSpan> UiClock
        {
            get => _uiClock ??= () => _padClock.Elapsed;
            set => _uiClock = value;
        }

        // Where a navigation sound goes; a test records them instead of opening the audio device.
        internal Action<string>? UiSoundSink { get; set; }

        // When the themed view next wants a frame: null for never, the present for the next vsync, else a later moment.
        internal TimeSpan? ThemedWakeAt { get; private set; }

        internal int ThemedFramesDrawn { get; private set; }

        internal ThemedLibrary? Themed => _themed;

        internal bool ThemedLibraryShown => ThemedLibraryHost.IsVisible;

        private bool ThemedStyleWanted =>
            _bigScreen && _appSettings.LibraryStyle == AppSettings.LibraryStyleTheme && !string.IsNullOrWhiteSpace(_appSettings.BigPictureTheme);

        private void SetUpThemedLibrary()
        {
            _themed = new ThemedLibrary(() => UiClock());
            _themedSearch = new TextBox { Name = "ThemedSearchBox", Watermark = "Search titles", FontSize = 22, MinWidth = 420 };
            _themedSearch.TextChanged += (_, _) => { _themed.SetFilter(_themedSearch.Text ?? "", UiClock()); ShowThemedSearchBar(); ScheduleThemedFrame(); };
            _themedSearchBar = new Border
            {
                Child = _themedSearch, IsVisible = false, HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Top,
                Margin = new Thickness(0, 24, 0, 0), Padding = new Thickness(10), CornerRadius = new CornerRadius(10),
                Background = new Avalonia.Media.SolidColorBrush(Avalonia.Media.Color.FromArgb(0xE0, 0x10, 0x10, 0x10)),
            };
            ThemedLibraryHost.Children.Add(_themed.Root);
            ThemedLibraryHost.Children.Add(_themedSearchBar);
            LibraryView.SizeChanged += (_, _) => { if (ThemedStyleWanted && LibraryView.IsVisible) ShowThemedLibrary(); };
            LibraryView.PropertyChanged += (_, e) =>
            {
                if (e.Property != IsVisibleProperty) return;
                LetGoOfTheThemedDirection(UiClock());
                ApplyStatusBar();
                ScheduleThemedFrame();
            };
        }

        // Called on every showing of the library: the theme's view in its place when the style asks for it and it loads, else Mistress's own.
        private void ShowThemedLibrary()
        {
            bool shown = false;
            if (_themed is not null && ThemedStyleWanted)
            {
                _themed.Status = DeviceStatusReader.Read();
                _themed.PlaySound = _appSettings.NavigationSounds ? PlayUiSound : null;
                _themed.SetFamily(PadFamilies.Of(_gamepad.ControllerType, _gamepad.ControllerName));
                string mediaKey = $"{_appSettings.EsdeMediaDirectory}|{System.Runtime.CompilerServices.RuntimeHelpers.GetHashCode(_artwork)}";
                shown = _themed.Show(_appSettings.BigPictureTheme!, LibraryView.Bounds.Size, ThemedShelves(), ThemedMedia(), mediaKey);
                if (!shown && _themed.Error is { } why && LibraryView.Bounds.Width > 0) StatusText.Text = why;
                if (shown && _themed.PlaySound is not null && UiSoundSink is null) (_uiSounds ??= new UiSoundPlayer()).Preload(_themed.SoundFiles);
            }

            LibraryContent.IsVisible = !shown;
            ThemedLibraryHost.IsVisible = shown;
            ApplyStatusBar();
            ScheduleThemedFrame();
        }

        private void PlayUiSound(string path)
        {
            if (UiSoundSink is { } sink) sink(path);
            else (_uiSounds ??= new UiSoundPlayer()).Play(path);
        }

        // Each shelf under ES-DE's name for it, favourites first and then by title, as ES-DE lists a gamelist by default (§13.8).
        private IReadOnlyList<ThemedShelf> ThemedShelves() => EmuSen.Cores.CoreCatalog.ShelvesInReleaseOrder
            .Select(s => new ThemedShelf(new ThemeSystem(s.EsdeSystem, s.EsdeFullName, s.EsdeSystem),
                _allScan.Entries.Where(e => e.Shelf == s.Name).Select(ThemedGame)
                    .OrderByDescending(g => g.Favorite).ThenBy(g => g.Name, StringComparer.OrdinalIgnoreCase).ToList()))
            .ToList();

        private SceneGame ThemedGame(RomEntry entry)
        {
            GameRecord? r = _recordSnapshot.GetValueOrDefault(entry.FullPath);
            return new SceneGame(entry.Title, entry.FullPath)
            {
                Favorite = r?.Favourite == true, LastPlayed = r?.LastPlayed, PlayCount = r?.PlayCount ?? 0,
                PlayTime = r is { PlaySeconds: > 0 } ? TimeSpan.FromSeconds(r.PlaySeconds) : null,
            };
        }

        // An ES-DE downloaded_media folder when one is set, and the library's own covers for the cover type.
        private ISceneMedia ThemedMedia()
        {
            EsdeMediaFolder? esde = string.IsNullOrWhiteSpace(_appSettings.EsdeMediaDirectory) ? null : new EsdeMediaFolder(_appSettings.EsdeMediaDirectory);
            var entries = _allScan.Entries.ToDictionary(e => e.FullPath, StringComparer.Ordinal);
            return new LibraryMedia(esde, file => entries.TryGetValue(file, out RomEntry? e) ? CoverPathFor(e) : null);
        }

        private sealed class LibraryMedia(EsdeMediaFolder? esde, Func<string, string?> cover) : ISceneMedia
        {
            public string? Find(ThemeSystem system, SceneGame game, string mediaType) =>
                esde?.Find(system, game, mediaType) ?? (mediaType == "cover" ? cover(game.File) : null);
        }

        // Whether the themed view is what the pad is steering now: nothing over it, no game on screen.
        private bool ThemedTakesThePad =>
            ThemedLibraryShown && LibraryView.IsVisible && !_padMenuOpen && OtherWindow() is null && OnScreenKeyboard.OpenOver(this) is null;

        // Directions go to the view as held and let go, so its own repeats (§14.7) run rather than the navigator's.
        private void ThemedDirections(TimeSpan now)
        {
            UiButton? held = _themedHeld is { } h && PadHeld(h) ? h : Directions.Where(PadHeld).Select(d => (UiButton?)d).FirstOrDefault();
            if (held == _themedHeld)
            {
                if (held is not null) _themed!.Advance(now);
                return;
            }

            if (_themedHeld is not null) _themed!.ReleaseDirection(now);
            _themedHeld = held;
            if (held is { } press) _themed!.PressDirection(press, now);
        }

        private void LetGoOfTheThemedDirection(TimeSpan now)
        {
            if (_themedHeld is null) return;
            _themed?.ReleaseDirection(now);
            _themedHeld = null;
        }

        private async void ThemedPadCommand(UiButton button)
        {
            if (_themed is null) return;
            ThemedCommand command = _themed.Command(button, UiClock());
            ScheduleThemedFrame();
            switch (command.Action)
            {
                case ThemedAction.Launch when command.Game is { } game:
                    await StartGameAsync(game.File, Path.GetFileName(game.File));
                    break;
                case ThemedAction.Favourite when command.Game is { } game:
                    _records.ToggleFavourite(game.File);
                    IdentifyLater(game.File);
                    StatusText.Text = _records.IsFavourite(game.File) ? $"Added {game.Name} to Favourites" : $"Removed {game.Name} from Favourites";
                    ShowLibraryEntries();
                    break;
                case ThemedAction.Search when _themedSearch is not null:
                    ShowThemedSearchBar(open: true);
                    PadKeyboard.Open(_themedSearch);
                    break;
                case ThemedAction.ClearSearch when _themedSearch is not null:
                    _themedSearch.Text = "";
                    break;
                case ThemedAction.Menu:
                    OpenPadMenu();
                    break;
                case ThemedAction.Leave when _session is { IsRomLoaded: true }:
                    ToggleLibrary();
                    break;
            }
        }

        private void ShowThemedSearchBar(bool open = false)
        {
            if (_themedSearchBar is null) return;
            _themedSearchBar.IsVisible = open || (_themed?.Filter.Length ?? 0) > 0 || OnScreenKeyboard.OpenOver(this) is not null;
        }

        // Asks for the next frame only while something moves, sleeps until a still text's pause ends, and draws nothing when all is still (§15, P39).
        private void ScheduleThemedFrame()
        {
            _themedWake?.Stop();
            if (_themed?.Stage is null || !ThemedLibraryShown || !LibraryView.IsVisible)
            {
                ThemedWakeAt = null;
                return;
            }

            TimeSpan now = UiClock();
            TimeSpan? next = _themed.NextChange(now);
            ThemedWakeAt = next;
            if (next is not { } due) return;
            if (due <= now)
            {
                if (_themedFrameAsked) return;
                _themedFrameAsked = true;
                RequestAnimationFrame(_ => { _themedFrameAsked = false; ThemedFrame(); });
                return;
            }

            _themedWake ??= new DispatcherTimer();
            _themedWake.Interval = due - now;
            _themedWake.Tick -= OnThemedWake;
            _themedWake.Tick += OnThemedWake;
            _themedWake.Start();
        }

        private void OnThemedWake(object? sender, EventArgs e)
        {
            _themedWake?.Stop();
            ThemedFrame();
        }

        // One frame of the render loop: the view's clock stepped to now, then the next frame asked for or not.
        internal void ThemedFrame()
        {
            if (_themed?.Stage is null || !ThemedLibraryShown) return;
            ThemedFramesDrawn++;
            _themed.Advance(UiClock());
            ShowThemedSearchBar();
            ScheduleThemedFrame();
        }
    }
}

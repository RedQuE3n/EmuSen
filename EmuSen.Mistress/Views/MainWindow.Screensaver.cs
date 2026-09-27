using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Threading;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // ES-DE's screensaver over the big-screen library: the idle clock, the input that wakes it, its controls, and frames only while it changes - see EmuSen_Settings_Reference.md §4.75.
    public partial class MainWindow
    {
        private Screensaver? _saver;
        private DispatcherTimer? _saverTimer;
        private bool _saverFrameAsked;
        private TimeSpan _idleFrom;
        private TimeSpan? _lastSaverFrame;
        private HashSet<UiButton> _saverHeld = new();
        private bool _saverQuiet;
        private TimeSpan _nextSwap;
        private Slide? _slide;
        private List<RomEntry>? _slideGames;
        private List<string>? _slideFiles;
        private readonly Dictionary<string, string?> _slidePictures = new(StringComparer.Ordinal);

        // A picture the slideshow shows: a game's, with the system it is under, or a file of the custom folder.
        private sealed record Slide(string Picture, SceneGame? Game, string? System, string? Esde);

        internal Random ScreensaverRandom { get; set; } = new();

        // Where Game Mode is read from; a test gives its own.
        internal Func<string, string?> ScreensaverEnvironment { get; set; } = Environment.GetEnvironmentVariable;

        internal Screensaver? ScreensaverShown => _saver is { IsOpen: true } s ? s : null;

        internal bool ScreensaverOpen => _saver?.IsOpen == true;

        internal TimeSpan? ScreensaverWakeAt { get; private set; }

        internal int ScreensaverFramesDrawn { get; private set; }

        internal TimeSpan ScreensaverIdleFrom => _idleFrom;

        private BigPictureInterface SaverSettings => _appSettings.BigPictureInterface;

        // Nothing over the library and no game on screen; ES-DE starts none while a menu is open either.
        private bool ScreensaverMayStart =>
            _bigScreen && !_themedClosed && LibraryView.IsVisible && !GameOnScreen && !_padMenuOpen && !Sheets.IsPresenting && !LaunchScreenOpen
            && OtherWindow() is null && OnScreenKeyboard.OpenOver(this) is null
            && (SaverSettings.ScreensaverInGameMode || !InGameModeSession(ScreensaverEnvironment));

        // Each poll: any button held keeps the idle clock at now or wakes the screensaver; true when the poll is the screensaver's and goes no further.
        private bool StepScreensaver()
        {
            TimeSpan now = UiClock();
            var held = Enum.GetValues<UiButton>().Where(PadHeld).ToHashSet();
            HashSet<UiButton> before = _saverHeld;
            _saverHeld = held;
            if (_saver is { IsOpen: true } saver)
            {
                UiButton[] pressed = held.Where(b => !before.Contains(b)).Order().ToArray();
                if (pressed.Length > 0) ScreensaverButton(pressed[0], now);
                else if (ScreensaverWakeAt is { } due && due <= now) ScreensaverFrame();
                _padNavigator.Forget(PadHeld);
                return true;
            }

            // The button that woke it does nothing more until it is let go.
            if (_saverQuiet)
            {
                if (held.Count > 0)
                {
                    _padNavigator.Forget(PadHeld);
                    _idleFrom = now;
                    return true;
                }
                _saverQuiet = false;
            }

            if (held.Count > 0 || !ScreensaverMayStart)
            {
                _idleFrom = now;
                return false;
            }

            int timer = SaverSettings.ScreensaverTimer;
            if (timer <= 0 || now - _idleFrom < TimeSpan.FromMilliseconds(timer)) return false;
            StartScreensaver(now);
            return true;
        }

        // A key: activity that keeps it away, or the press that wakes it or steers it.
        private bool ScreensaverKey(Key key, bool pressed)
        {
            TimeSpan now = UiClock();
            if (!pressed) return false;
            _idleFrom = now;
            if (_saver is not { IsOpen: true }) return false;
            ScreensaverButton(ThemedKeyButton(key) ?? UiButton.Guide, now);
            return true;
        }

        private void ScreensaverPointer(bool pressed)
        {
            _idleFrom = UiClock();
            if (pressed && _saver is { IsOpen: true }) StopScreensaver(UiClock());
        }

        // X in the system view starts it at once, as ES-DE's screensaver controls allow; true when it did.
        private bool ScreensaverFromButton(UiButton button)
        {
            if (button != UiButton.Screensaver || !SaverSettings.ScreensaverControls || !ThemedLibraryShown || _themed?.ViewName != "system" || !ScreensaverMayStart) return false;
            StartScreensaver(UiClock());
            _saverHeld = Enum.GetValues<UiButton>().Where(PadHeld).ToHashSet();
            return true;
        }

        // ES-DE's controls: left and right another game, A that game started, Y its gamelist; any other button, or any with the controls off, only wakes the view.
        private void ScreensaverButton(UiButton button, TimeSpan now)
        {
            bool controls = SaverSettings.ScreensaverControls && _saver?.Kind == BigPictureInterface.SaverSlideshow && _slide is not null;
            switch (button)
            {
                case UiButton.Left or UiButton.Right when controls:
                    NextSlide(now);
                    _nextSwap = now + SwapInterval;
                    ScheduleScreensaver();
                    return;
                case UiButton.Accept when controls && _slide is { Game: { } game, Esde: { } esde }:
                    StopScreensaver(now);
                    ShowSlideGame(esde, game, now);
                    _themed?.Sound("launch");
                    _ = StartGameAsync(game.File, Path.GetFileName(game.File), ThemedLibraryShown ? game : null);
                    return;
                case UiButton.Search when controls && _slide is { Game: { } game, Esde: { } esde }:
                    StopScreensaver(now);
                    ShowSlideGame(esde, game, now);
                    return;
                default:
                    StopScreensaver(now);
                    return;
            }
        }

        // The game the slideshow showed, selected in its gamelist, or in the sidebar library's list.
        private void ShowSlideGame(string esde, SceneGame game, TimeSpan now)
        {
            if (ThemedLibraryShown && _themed is not null)
            {
                _themed.ShowGame(esde, game.File, now);
                ScheduleThemedFrame();
                return;
            }
            for (int i = 0; i < LibraryList.ItemCount; i++)
            {
                if (LibraryList.Items[i] is not RomEntry e || e.FullPath != game.File) continue;
                LibraryList.SelectedIndex = i;
                LibraryList.ScrollIntoView(i);
                return;
            }
        }

        private TimeSpan SwapInterval => TimeSpan.FromMilliseconds(Math.Clamp(SaverSettings.ScreensaverSwapImageTimeout, 2000, 120000));

        // Video falls back to Dim until videos exist, and a slideshow with nothing to show does too, as ES-DE's do.
        internal void StartScreensaver(TimeSpan now)
        {
            if (_saver is null)
            {
                _saver = new Screensaver { Name = "Screensaver" };
                if (Sheets.Parent is Panel root) root.Children.Insert(root.Children.IndexOf(Sheets), _saver);
            }
            LetGoOfTheThemedDirection(now);
            _padNavigator.Forget(PadHeld);
            _slide = null;
            _slideGames = null;
            _slideFiles = null;
            _slidePictures.Clear();
            string type = SaverSettings.ScreensaverType;
            if (type == BigPictureInterface.SaverSlideshow)
            {
                _saver.OpenSlideshow(now);
                if (NextSlide(now))
                {
                    _nextSwap = now + SwapInterval;
                    ScheduleThemedFrame();
                    ScheduleScreensaver();
                    return;
                }
            }
            _saver.OpenDim(type == BigPictureInterface.SaverBlack, now);
            ScheduleThemedFrame();
            ScheduleScreensaver();
        }

        internal void StopScreensaver(TimeSpan now)
        {
            if (_saver is not { IsOpen: true } saver) return;
            _saverTimer?.Stop();
            saver.Close();
            ScreensaverWakeAt = null;
            _slide = null;
            _idleFrom = now;
            _saverQuiet = true;
            ScheduleThemedFrame();
        }

        // Another picture, never the one just shown while there is another; false when there is none at all.
        private bool NextSlide(TimeSpan now)
        {
            Slide? next = SaverSettings.ScreensaverSlideshowCustomImages ? NextCustomSlide() : NextGameSlide();
            if (next is null) return false;
            _slide = next;
            bool star = next.Game is { Favorite: true } && !SaverSettings.ScreensaverSlideshowOnlyFavorites;
            bool overlay = next.Game is not null && SaverSettings.ScreensaverSlideshowGameInfo;
            _saver!.ShowPicture(next.Picture, overlay ? next.Game!.Name : null, overlay ? next.System : null, star, SaverSettings.ScreensaverStretchImages, now);
            return true;
        }

        private Slide? NextGameSlide()
        {
            _slideGames ??= _allScan.Entries.Where(e => Listed(e) && (!SaverSettings.ScreensaverSlideshowOnlyFavorites || _recordSnapshot.GetValueOrDefault(e.FullPath)?.Favourite == true)).ToList();
            EmuSen.Mistress.Scraping.MediaSources sources = MediaSourcesNow();
            while (_slideGames.Count > 0)
            {
                List<RomEntry> pool = _slideGames.Count > 1 && _slide?.Game is { } last ? _slideGames.Where(e => e.FullPath != last.File).ToList() : _slideGames;
                RomEntry pick = pool[ScreensaverRandom.Next(pool.Count)];
                string? esde = EmuSen.Cores.CoreCatalog.ShelfByName(pick.Shelf)?.EsdeSystem is { Length: > 0 } s ? s : null;
                if (!_slidePictures.TryGetValue(pick.FullPath, out string? picture))
                    _slidePictures[pick.FullPath] = picture = esde is null ? null : Screensaver.SlideKinds.Select(k => sources.Locate(esde, pick.FullPath, k).Path).FirstOrDefault(p => p is not null);
                if (picture is null)
                {
                    _slideGames.Remove(pick);
                    continue;
                }
                SceneGame game = ThemedGame(pick);
                return new Slide(picture, game, LaunchContent(game).System, esde);
            }
            return null;
        }

        private Slide? NextCustomSlide()
        {
            _slideFiles ??= Screensaver.CustomImages(SaverSettings.ScreensaverSlideshowCustomDir, SaverSettings.ScreensaverSlideshowRecurse, _appSettings.RomDirectory);
            if (_slideFiles.Count == 0) return null;
            List<string> pool = _slideFiles.Count > 1 && _slide is { } last ? _slideFiles.Where(f => f != last.Picture).ToList() : _slideFiles;
            return new Slide(pool[ScreensaverRandom.Next(pool.Count)], null, null, null);
        }

        // Asks for a frame only while it changes and sleeps until the next swap otherwise (P114, §15's rule).
        private void ScheduleScreensaver()
        {
            _saverTimer?.Stop();
            if (_saver is not { IsOpen: true } saver || _themedClosed)
            {
                ScreensaverWakeAt = null;
                return;
            }

            TimeSpan now = UiClock();
            TimeSpan? next = saver.NextChange(now);
            if (saver.Kind == BigPictureInterface.SaverSlideshow && _slide is not null && (next is null || _nextSwap < next)) next = _nextSwap;
            ScreensaverWakeAt = next;
            if (next is not { } due) return;
            if (due <= now)
            {
                if (_saverFrameAsked) return;
                _saverFrameAsked = true;
                RequestAnimationFrame(_ => { _saverFrameAsked = false; ScreensaverFrame(); });
                return;
            }

            _saverTimer ??= new DispatcherTimer();
            _saverTimer.Interval = due - now;
            _saverTimer.Tick -= OnSaverTimer;
            _saverTimer.Tick += OnSaverTimer;
            _saverTimer.Start();
        }

        private void OnSaverTimer(object? sender, EventArgs e)
        {
            _saverTimer?.Stop();
            ScreensaverFrame();
        }

        // One frame: the swap when it is due, the fades stepped to now, and the next frame asked for or not.
        internal void ScreensaverFrame()
        {
            if (_saver is not { IsOpen: true } saver || _themedClosed) return;
            TimeSpan now = UiClock();
            if (_lastSaverFrame == now) return;
            _lastSaverFrame = now;
            ScreensaverFramesDrawn++;
            if (saver.Kind == BigPictureInterface.SaverSlideshow && _slide is not null && now >= _nextSwap)
            {
                NextSlide(now);
                _nextSwap = now - _nextSwap >= SwapInterval ? now + SwapInterval : _nextSwap + SwapInterval;
            }
            saver.Advance(now);
            ScheduleScreensaver();
        }

        // A closed window's timer ends with it (§15.14's lesson).
        private void CloseScreensaver()
        {
            _saverTimer?.Stop();
            _saver?.Close();
            ScreensaverWakeAt = null;
        }
    }
}

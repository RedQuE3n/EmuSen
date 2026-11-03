using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using SDL3;
using Xunit;

namespace EmuSen.WiseMan.Mistress
{
    // The Pause Card's own behaviour: the rewind pane, X for a screenshot, the volume row in each session, the swap in its help bar and the Guide chords - see EmuSen_BigPicture.md §48.
    [Collection(TestCollections.ProcessGlobals)]
    public class PauseCardTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PauseCardTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPauseCardTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PauseCardTests()
        {
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private (MainWindow Window, PadDriver Pad) Playing(Action<AppSettings>? more = null, bool gameMode = false)
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Cobalt Harbor (Synthetic).sfc"), PadRewindReelTests.ChangingBackdrop());
            var app = new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = true, StateDirectory = Path.Combine(_root, "States") };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            more?.Invoke(app);
            app.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            if (gameMode) typeof(MainWindow).GetField("_bigScreenForced", Hidden)!.SetValue(window, true);
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            OwnLook.Play(window, pad, swapped: app.SwapPadButtons);
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            return (window, pad);
        }

        private static object? Field(MainWindow w, string name) => typeof(MainWindow).GetField(name, Hidden)!.GetValue(w);
        private static AppSettings Settings(MainWindow w) => (AppSettings)Field(w, "_appSettings")!;
        private static string State(MainWindow w, string root, int slot) => SaveLibrary.StatePathFor((string)Field(w, "_currentRomPath")!, slot, Path.Combine(root, "States"));

        private static void Done(MainWindow window)
        {
            typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(window, null);
            window.Close();
        }

        private static void WaitFor(Func<bool> condition, string what)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out waiting for " + what);
                Thread.Sleep(5);
            }
        }

        [Fact]
        public void Guide_alone_is_the_menu_as_it_is_let_go_and_with_up_down_or_R1_a_chord_instead()
        {
            var r = new GuideChordReader();
            Assert.Equal(GuideChord.None, r.Feed(true, false, false, false));
            Assert.Equal(GuideChord.Menu, r.Feed(false, false, false, false));
            Assert.Equal(GuideChord.None, r.Feed(false, false, false, false));

            Assert.Equal(GuideChord.None, r.Feed(true, false, false, false));
            Assert.Equal(GuideChord.Save, r.Feed(true, true, false, false));
            Assert.Equal(GuideChord.None, r.Feed(true, true, false, false));
            Assert.Equal(GuideChord.Load, r.Feed(true, false, true, false));
            Assert.Equal(GuideChord.Screenshot, r.Feed(true, false, false, true));
            Assert.Equal(GuideChord.None, r.Feed(false, false, false, false));

            // A direction already held when Guide goes down is not a chord, and Guide let go then opens the menu.
            Assert.Equal(GuideChord.None, r.Feed(true, true, false, false));
            Assert.Equal(GuideChord.None, r.Feed(true, true, false, false));
            Assert.Equal(GuideChord.Menu, r.Feed(false, true, false, false));

            // A Guide held across a menu opens nothing when it is let go.
            r.Reset(guideHeld: true);
            Assert.Equal(GuideChord.None, r.Feed(false, false, false, false));
        }

        [Fact]
        public Task With_the_chords_on_guide_and_up_saves_without_the_menu_and_guide_alone_opens_it() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(a => a.GuideChords = AppSettings.GuideChordsOn);
            pad.Pad.Press(SDL.GamepadButton.Guide);
            pad.Tick();
            Assert.False(PadMenu.IsOpen(window));
            pad.Tap(SDL.GamepadButton.DPadUp);
            pad.Pad.Release(SDL.GamepadButton.Guide);
            pad.Tick();
            Assert.False(PadMenu.IsOpen(window));
            WaitFor(() => File.Exists(State(window, _root, 1)), "the chord's save");

            pad.Guide();
            Assert.True(PadMenu.IsOpen(window));
            Assert.True(window.IsPaused);
            Done(window);
        }, default);

        [Fact]
        public Task In_game_mode_auto_leaves_the_chords_off_so_guide_opens_the_menu_at_once_and_up_is_the_menu_s() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(gameMode: true);
            Assert.Equal(AppSettings.GuideChordsAuto, Settings(window).GuideChords);
            pad.Pad.Press(SDL.GamepadButton.Guide);
            pad.Tick();
            Assert.True(PadMenu.IsOpen(window));
            pad.Pad.Release(SDL.GamepadButton.Guide);
            pad.Tick();
            Assert.False(File.Exists(State(window, _root, 1)));
            Done(window);
        }, default);

        [Fact]
        public Task X_takes_a_screenshot_from_any_page_and_the_menu_stays() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing();
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            PadMenu.Choose(window, pad, MainWindow.LeaveMenu, exact: true);
            pad.X();
            Assert.True(PadMenu.IsOpen(window));
            Assert.Equal(2, PadMenu.Depth(window));
            WaitFor(() => Directory.Exists(DataStore.Screenshots) && Directory.EnumerateFiles(DataStore.Screenshots, "*.png").Any(), "the screenshot");
            Done(window);
        }, default);

        [Fact]
        public Task On_the_desktop_session_the_volume_row_steps_the_volume_and_in_game_mode_it_defers_to_the_quick_access_menu() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(a => a.Volume = 0.5);
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            PadMenu.Choose(window, pad, MainWindow.GameOptionsMenu, exact: true);
            int at = Array.FindIndex(PadMenu.Lines(window), l => l.StartsWith(MainWindow.VolumeRow, StringComparison.Ordinal));
            pad.Down(at);
            Assert.Equal("50%", PadMenu.Entries(window)[at].Value!());
            pad.Right();
            Assert.Equal(0.6, Settings(window).Volume, 3);
            Assert.Equal("60%", PadMenu.Entries(window)[at].Value!());
            Done(window);

            (window, pad) = Playing(gameMode: true);
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            PadMenu.Choose(window, pad, MainWindow.GameOptionsMenu, exact: true);
            PadMenuEntry volume = PadMenu.Entries(window).Single(e => e.Label?.Invoke() == MainWindow.VolumeRow);
            Assert.Null(volume.Adjust);
            Assert.Equal("Quick Access Menu", volume.Value!());
            Done(window);
        }, default);

        [Fact]
        public Task The_rewind_row_s_pane_shows_the_reel_or_why_it_cannot_open() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing();
            for (int i = 0; i < 60; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(10); }
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            pad.Down();
            WindowFitAuditTests.Settle(window);
            PauseCardPane pane = PadMenu.Pane(window);
            Assert.Equal("reel", pane.Showing);
            Assert.Contains(PadMenuTests.PaneWords(pane), t => t.Contains(" available. A opens the reel", StringComparison.Ordinal) || t.StartsWith("Rewind: ", StringComparison.Ordinal));
            Done(window);
        }, default);

        // The oldest and newest moments the reel holds, by frame.
        private static (long Oldest, long Newest, int Count) Reel(MainWindow w)
        {
            var session = (EmuSen.Mistress.GameSession)Field(w, "_session")!;
            using var reel = System.Text.Json.JsonDocument.Parse(session.Crystal.RewindReel());
            long[] frames = reel.RootElement.GetProperty("moments").EnumerateArray().Select(m => m.GetProperty("frame").GetInt64()).ToArray();
            return frames.Length == 0 ? (-1, -1, 0) : (frames[0], frames[^1], frames.Length);
        }

        // Load from the card leaves the moments before it in the reel, so Rewind undoes a wrong load; the Rust runtime keeps them, the C# half (untouched until B10) still clears them (EmuSen_BigPicture.md §48.8).
        [Fact]
        public Task Load_keeps_the_reel_from_before_it_so_rewind_undoes_it() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing();
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            pad.Down(2);
            pad.A();
            WaitFor(() => File.Exists(State(window, _root, 1)), "the save");
            WaitFor(() => Reel(window).Count >= 3, "moments in the reel");
            (long oldest, long newest, _) = Reel(window);
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            pad.Down(3);
            pad.A();
            WaitFor(() => window.GetControl<TextBlock>("StatusText").Text?.StartsWith("State loaded", StringComparison.Ordinal) == true, "the load");
            Assert.False(PadMenu.IsOpen(window));
            (long after, long latest, int count) = Reel(window);
            if (EmuSen.WiseMan.Crystal.DivergenceRowTests.KeepsRewindAcrossALoad)
            {
                Assert.Equal(oldest, after);
                Assert.True(latest >= newest && count >= 3, $"the reel kept up to {newest}: {after}..{latest}, {count} moments");
            }
            Done(window);
        }, default);

        [Fact]
        public Task With_the_swap_the_card_s_help_bar_names_east_as_select_and_south_as_back() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(a => a.SwapPadButtons = true);
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            HintEntry[] hints = PadMenu.Big(window).Hints!.ToArray();
            Assert.Equal(PadGlyphButton.East, hints[0].Button);
            Assert.Equal(PadGlyphButton.South, hints[1].Button);
            Assert.Equal(PadGlyphButton.North, hints.Single(h => h.Label == "Screenshot").Button);
            Done(window);
        }, default);
    }
}

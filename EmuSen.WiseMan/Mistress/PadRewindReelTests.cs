using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // The rewind reel from the pad and the pointer: it opens paused, steps, strides, rewinds to exactly the moment, and cancels to exactly where it was - see EmuSen_Settings_Reference.md §4.49.
    [Collection(TestCollections.ProcessGlobals)]
    public class PadRewindReelTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PadRewindReelTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPadRewindReelTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PadRewindReelTests(ITestOutputHelper output)
        {
            _out = output;
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

        // After the boot stub: the screen on, then each vertical blank the backdrop colour stepped, so every moment has its own picture.
        public static byte[] ChangingBackdrop() => SyntheticRom.Build((5, new byte[]
        {
            0xA9, 0x0F, 0x8D, 0x00, 0x21,       // LDA #$0F; STA $2100
            0xAD, 0x12, 0x42, 0x10, 0xFB,       // loop: LDA $4212; BPL loop
            0x9C, 0x21, 0x21,                   // STZ $2121
            0xE6, 0x10, 0xA5, 0x10,             // INC $10; LDA $10
            0x8D, 0x22, 0x21,                   // STA $2122
            0xA5, 0x10, 0x4A, 0x4A,             // LDA $10; LSR; LSR
            0x8D, 0x22, 0x21,                   // STA $2122
            0xAD, 0x12, 0x42, 0x30, 0xFB,       // wait: LDA $4212; BMI wait
            0x80, 0xE3,                         // BRA loop
        }));

        private (MainWindow Window, PadDriver Pad) WithAGame(bool bigScreen)
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), ChangingBackdrop());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen }.Save();

            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            return (window, pad);
        }

        // Unthrottled with every frame drawn, so seconds of history take a fraction of one, then back to normal speed.
        private static void Play(MainWindow window, int frames)
        {
            var speed = (SpeedController)Field(window, "_speed");
            speed.MaxFrameSkip = 0;
            SetField(window, "_baseSpeedPercent", SpeedController.UnthrottledPercent);
            long until = Game(window).TotalFrames + frames;
            WaitFor(() => Game(window).TotalFrames >= until);
            SetField(window, "_baseSpeedPercent", SpeedController.NormalPercent);
        }

        private static void Choose(MainWindow window, PadDriver pad, string entry)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            ChooseInOpenMenu(window, pad, entry);
        }

        private static void ChooseInOpenMenu(MainWindow window, PadDriver pad, string entry)
        {
            string[] lines = MenuLines(window);
            int at = Array.FindIndex(lines, l => l.StartsWith(entry, StringComparison.Ordinal));
            Assert.True(at >= 0, $"No '{entry}' in the pad menu: {string.Join(", ", lines)}");
            pad.Down(at);
            pad.A();
        }

        private static string[] MenuLines(MainWindow window) =>
            window.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();

        private static RewindReelWindow OpenReel(MainWindow window, PadDriver pad)
        {
            Choose(window, pad, "Rewind");
            WaitFor(() => Sheets(window).Current is RewindReelWindow);
            Settle(window);
            return (RewindReelWindow)Sheets(window).Current!;
        }

        private static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");
        private static Control Sheet(MainWindow w) => Sheets(w).SheetOf(Sheets(w).Current!)!;
        private static RewindBuffer Rewind(MainWindow w) => (RewindBuffer)Field(w, "_rewind");
        private static EmulatorSession Game(MainWindow w) => (EmulatorSession)Field(w, "_session");
        private static TextBlock Status(MainWindow w) => w.GetControl<TextBlock>("StatusText");
        private static object Field(MainWindow w, string name) => typeof(MainWindow).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(w)!;
        private static void SetField(MainWindow w, string name, object value) => typeof(MainWindow).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(w, value);
        private static void Stop(MainWindow w) => typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(w, null);

        private static byte[] Save(ICore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        private static void Settle(Window window)
        {
            Dispatcher.UIThread.RunJobs();
            UiTest.Capture(window);
            Dispatcher.UIThread.RunJobs();
        }

        private static void WaitFor(Func<bool> condition)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out waiting for the emulation thread");
                Thread.Sleep(2);
            }
        }

        // Until the frame count holds still, so the emulation thread is parked and the test may read the buffer.
        private static long WaitUntilParked(MainWindow window)
        {
            long frames;
            do
            {
                frames = Game(window).TotalFrames;
                Thread.Sleep(100);
                Dispatcher.UIThread.RunJobs();
            }
            while (Game(window).TotalFrames != frames);
            return frames;
        }

        private static string Shown(RewindReelWindow reel) => reel.Strip.Selected!.Label;

        [Fact]
        public Task A_rewinds_to_the_moment_the_reel_shows_byte_for_byte_and_leaves_a_paused_game_paused() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 240);
            window.PauseEmulation();
            WaitUntilParked(window);
            ICore core = Game(window).Core!;
            long frameBefore = Rewind(window).Frame;

            RewindReelWindow reel = OpenReel(window, pad);
            Assert.True(window.IsPaused);
            List<ReelMoment> moments = Moments(reel);
            // Four snapshots a second of the console's own frames - see EmuSen_Settings_Reference.md §4.50.
            Assert.Equal((int)Math.Round(Game(window).FrameRateHz / 4), Rewind(window).IntervalFrames);
            Assert.True(moments.Count >= 240 / Rewind(window).IntervalFrames, $"{moments.Count} tiles at one every {Rewind(window).IntervalFrames} frames");
            Assert.True(moments[^1].IsNow);
            Assert.Same(moments[^1], reel.Strip.Selected);
            Assert.Equal(frameBefore, moments[^1].Frame);
            Assert.All(moments.SkipLast(1), m => Assert.False(m.IsNow));
            Assert.Equal(moments.Select(m => m.Frame).OrderBy(f => f), moments.Select(m => m.Frame));
            UiTest.Dump("rewind-reel-sheet-now", UiTest.Capture(window));

            pad.Left(3);
            Settle(window);
            ReelMoment target = moments[^4];
            Assert.Same(target, reel.Strip.Selected);
            Assert.Equal(target.Label, window.GetVisualDescendants().OfType<TextBlock>().First(t => t.Name == "ReelCaption").Text);
            Assert.Equal(ReelMoment.Ago((frameBefore - target.Frame) / Game(window).FrameRateHz), target.Label);
            byte[] expected = Rewind(window).StateAt(target.Frame)!;
            UiTest.Dump("rewind-reel-sheet-three-back", UiTest.Capture(window));

            pad.A();
            WaitFor(() => Status(window).Text?.StartsWith("Rewound", StringComparison.Ordinal) == true);
            Assert.False(Sheets(window).IsPresenting);
            Assert.True(window.IsPaused);
            WaitUntilParked(window);

            Assert.Equal(expected, Save(core));
            Assert.Equal(target.Frame, Rewind(window).Frame);
            Assert.Equal(target.Frame, Rewind(window).Moments()[^1].Frame);
            _out.WriteLine($"{moments.Count} tiles; rewound from frame {frameBefore} to {target.Frame} ({target.Label}), {expected.Length}-byte state identical");

            Stop(window);
            window.Close();
        }, default);

        private static List<ReelMoment> Moments(RewindReelWindow reel)
        {
            var seen = new List<ReelMoment>();
            ReelMoment? keep = reel.Strip.Selected;
            reel.Strip.Move(int.MinValue / 2);
            while (true)
            {
                seen.Add(reel.Strip.Selected!);
                if (!reel.Strip.Move(1)) break;
            }
            reel.Strip.Select(keep);
            return seen;
        }

        [Fact]
        public Task B_cancels_leaving_the_machine_and_the_history_exactly_as_they_were() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 240);

            // Paused by the player, so it stays paused after B and the state after can be compared; measured before the reel reads anything.
            window.PauseEmulation();
            WaitUntilParked(window);
            ICore core = Game(window).Core!;
            byte[] before = Save(core);
            long[] history = Rewind(window).Moments().Select(m => m.Frame).ToArray();
            int depth = Rewind(window).Depth;

            RewindReelWindow reel = OpenReel(window, pad);
            pad.Left(5);
            pad.L1();
            Settle(window);
            Assert.False(reel.Strip.Selected!.IsNow);

            pad.B();
            WaitFor(() => !Sheets(window).IsPresenting);
            Settle(window);
            WaitUntilParked(window);
            Assert.True(window.IsPaused);
            Assert.Equal(before, Save(core));
            Assert.Equal(history, Rewind(window).Moments().Select(m => m.Frame).ToArray());
            Assert.Equal(depth, Rewind(window).Depth);

            Stop(window);
            window.Close();
        }, default);

        // The present is a tile, and choosing it is no rewind: the machine and the history are untouched.
        [Fact]
        public Task A_on_now_changes_nothing() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 120);
            window.PauseEmulation();
            WaitUntilParked(window);
            ICore core = Game(window).Core!;
            byte[] before = Save(core);
            long[] history = Rewind(window).Moments().Select(m => m.Frame).ToArray();

            RewindReelWindow reel = OpenReel(window, pad);
            pad.Left(2);
            pad.Right(2);
            Assert.True(reel.Strip.Selected!.IsNow);
            pad.A();
            WaitFor(() => !Sheets(window).IsPresenting);
            Settle(window);
            WaitUntilParked(window);

            Assert.Equal(before, Save(core));
            Assert.Equal(history, Rewind(window).Moments().Select(m => m.Frame).ToArray());
            Assert.True(window.IsPaused);
            Assert.False(Status(window).Text?.StartsWith("Rewind", StringComparison.Ordinal), $"choosing Now said '{Status(window).Text}'");

            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task A_from_the_menu_rewinds_and_resumes_the_game_the_menu_paused() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 240);

            RewindReelWindow reel = OpenReel(window, pad);
            pad.Left(10);
            Settle(window);
            ReelMoment target = reel.Strip.Selected!;
            pad.A();
            WaitFor(() => Status(window).Text?.StartsWith("Rewound", StringComparison.Ordinal) == true);
            Assert.False(window.IsPaused);
            Assert.Contains(target.Frame, Rewind(window).Moments().Select(m => m.Frame));

            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task The_shoulders_stride_five_seconds_and_the_triggers_go_to_either_end() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 900);

            RewindReelWindow reel = OpenReel(window, pad);
            List<ReelMoment> moments = Moments(reel);
            double reach = moments[0].SecondsBack;
            Assert.True(reach > 12, $"{reach:F1} seconds held");

            // Every label is its distance at the console's own rate, which is not 60 - the SNES's is 60.0988.
            long now = moments[^1].Frame;
            double hz = Game(window).FrameRateHz;
            Assert.All(moments, m => Assert.Equal(m.IsNow ? "Now" : ReelMoment.Ago((now - m.Frame) / hz), m.Label));

            pad.L1();
            Settle(window);
            ReelMoment once = reel.Strip.Selected!;
            Assert.InRange(once.SecondsBack, RewindReelWindow.StrideSeconds, RewindReelWindow.StrideSeconds + 0.5);
            ReelMoment newer = moments[moments.IndexOf(once) + 1];
            Assert.True(newer.SecondsBack < RewindReelWindow.StrideSeconds);

            pad.L1();
            Settle(window);
            ReelMoment twice = reel.Strip.Selected!;
            Assert.InRange(twice.SecondsBack - once.SecondsBack, RewindReelWindow.StrideSeconds, RewindReelWindow.StrideSeconds + 0.5);

            pad.R1();
            Settle(window);
            Assert.InRange(twice.SecondsBack - reel.Strip.Selected!.SecondsBack, RewindReelWindow.StrideSeconds, RewindReelWindow.StrideSeconds + 0.5);

            pad.Press(UiButton.First);
            Assert.Same(moments[0], reel.Strip.Selected);
            pad.Press(UiButton.Last);
            Assert.Same(moments[^1], reel.Strip.Selected);
            pad.Press(UiButton.First);
            pad.Left();
            Assert.Same(moments[0], reel.Strip.Selected);

            // B resumes the game the menu paused.
            pad.B();
            WaitFor(() => !Sheets(window).IsPresenting);
            Assert.False(window.IsPaused);
            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task Every_control_on_the_reel_is_reached_by_the_pad() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 120);
            RewindReelWindow reel = OpenReel(window, pad);

            Control sheet = Sheet(window);
            Assert.Same(reel.Strip, window.FocusManager!.GetFocusedElement());
            List<InputElement> operable = PadAudit.Operable(sheet);
            operable.Add(reel.Strip);
            HashSet<InputElement> reached = PadAudit.Reachable(sheet, pad);
            _out.WriteLine($"operable: {string.Join(", ", operable.Select(PadAudit.Describe))}; reached: {string.Join(", ", reached.Select(PadAudit.Describe))}");
            Assert.Empty(operable.Where(e => !reached.Contains(e)).Select(PadAudit.Describe));
            Assert.Contains(operable, e => e is Button { Name: "RewindHereButton" });
            Assert.Contains(operable, e => e is Button { Name: "CancelRewindButton" });

            // A on the Cancel button is the button's, not the strip's.
            var cancel = (Button)PadAudit.Reach(sheet, pad, e => e is Button { Name: "CancelRewindButton" });
            pad.A();
            WaitFor(() => !Sheets(window).IsPresenting);
            Assert.False(window.IsPaused);

            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task With_no_history_the_entry_says_so_and_opens_nothing() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            WaitFor(() => Rewind(window).SnapshotBytes > 0);
            window.PauseEmulation();
            WaitUntilParked(window);
            Rewind(window).Clear();

            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            string[] lines = MenuLines(window);
            int at = Array.FindIndex(lines, l => l.StartsWith("Rewind", StringComparison.Ordinal));
            Assert.Equal("Rewind  (nothing to go back to yet)", lines[at]);
            pad.Down(at);
            pad.A();
            Settle(window);
            Assert.False(Sheets(window).IsPresenting);
            Assert.True(window.GetControl<Control>("PadMenuPanel").IsVisible);
            pad.B();

            Stop(window);
            window.Close();
        }, default);

        // EMUSEN_REWIND_REEL_GAME="<rom>|<state>" renders the reel over a real game, for a person to look at; unset, it does nothing.
        [Fact]
        public Task A_real_game_on_the_reel_is_rendered_for_a_look() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_REWIND_REEL_GAME")?.Split('|') is not [string rom, string state])
            {
                _out.WriteLine("EMUSEN_REWIND_REEL_GAME unset, not run");
                return;
            }

            foreach (bool bigScreen in new[] { true, false })
            {
                foreach (string old in Directory.GetFiles(_romDir)) File.Delete(old);
                File.Copy(rom, Path.Combine(_romDir, Path.GetFileName(rom)));
                new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen }.Save();
                var window = new MainWindow { Width = 1280, Height = 800 };
                window.Show();
                var pad = new PadDriver(window);
                window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
                pad.A();
                window.PauseEmulation();
                WaitUntilParked(window);
                typeof(MainWindow).GetMethod("LoadStateFromConsole", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { state });
                window.ResumeEmulation();
                Play(window, 720);
                string name = Path.GetFileNameWithoutExtension(rom) + (bigScreen ? "-sheet" : "-desktop");

                if (bigScreen)
                {
                    RewindReelWindow reel = OpenReel(window, pad);
                    pad.Left(6);
                    Settle(window);
                    UiTest.Dump("rewind-reel-" + name, UiTest.Capture(window));
                    pad.L1();
                    Settle(window);
                    UiTest.Dump("rewind-reel-" + name + "-stride", UiTest.Capture(window));
                    pad.B();
                }
                else
                {
                    typeof(MainWindow).GetMethod("OpenRewindReel", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { false });
                    WaitFor(() => window.OwnedWindows.OfType<RewindReelWindow>().Any());
                    RewindReelWindow reel = window.OwnedWindows.OfType<RewindReelWindow>().Single();
                    reel.Strip.Move(-6);
                    Settle(reel);
                    UiTest.Dump("rewind-reel-" + name, UiTest.Capture(reel));
                    reel.Close();
                }

                Stop(window);
                window.Close();
            }
        }, default);

        [Fact]
        public Task On_the_desktop_a_click_chooses_and_a_double_click_rewinds() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver _) = WithAGame(bigScreen: false);
            Play(window, 240);
            window.PauseEmulation();
            WaitUntilParked(window);
            ICore core = Game(window).Core!;

            typeof(MainWindow).GetMethod("OpenRewindReel", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { false });
            WaitFor(() => window.OwnedWindows.OfType<RewindReelWindow>().Any());
            RewindReelWindow reel = window.OwnedWindows.OfType<RewindReelWindow>().Single();
            Settle(reel);
            UiTest.Dump("rewind-reel-desktop-now", UiTest.Capture(reel));

            TileGridItem tile = reel.Strip.GetVisualDescendants().OfType<TileGridItem>().Where(t => t.IsVisible && !t.IsSelected).OrderBy(t => t.Bounds.X).Last();
            Point at = tile.TranslatePoint(new Point(tile.Bounds.Width / 2, tile.Bounds.Height / 2), reel)!.Value;
            reel.MouseDown(at, MouseButton.Left);
            reel.MouseUp(at, MouseButton.Left);
            Settle(reel);
            ReelMoment target = reel.Strip.Selected!;
            Assert.False(target.IsNow);
            byte[] expected = Rewind(window).StateAt(target.Frame)!;
            UiTest.Dump("rewind-reel-desktop-chosen", UiTest.Capture(reel));

            // The second press is the double-tap, and the reel is gone before its release.
            reel.MouseDown(at, MouseButton.Left);
            WaitFor(() => Status(window).Text?.StartsWith("Rewound", StringComparison.Ordinal) == true);
            Assert.Same(target, reel.DialogResult);
            WaitUntilParked(window);
            Assert.Equal(expected, Save(core));
            Assert.True(window.IsPaused);

            Stop(window);
            window.Close();
        }, default);
    }
}

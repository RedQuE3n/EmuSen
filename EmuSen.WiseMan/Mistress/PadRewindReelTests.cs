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
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // The rewind reel as a page of the Pause Card: it opens on the card, steps, strides, goes back to exactly the moment, and B leaves everything as it was - see EmuSen_Interface.md §18.5.
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

        // After the boot stub: the screen on, then each vertical blank the backdrop colour stepped and WRAM $10 counted, so every moment has its own picture and its own count.
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
            var app = new AppSettings { BigPictureInterface = { LaunchScreenDuration = EmuSen.Galaxia.Models.BigPictureInterface.LaunchDisabled }, RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            app.Save();

            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            OwnLook.Play(window, pad);
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            return (window, pad);
        }

        // Unthrottled with every frame drawn, so seconds of history take a fraction of one, then back to normal speed.
        private static void Play(MainWindow window, int frames) => MistressGame.Play(window, frames, (until, _) => WaitFor(until));

        // The card opened and A on its Rewind row: the reel's page, on the newest held moment.
        private static void OpenReel(MainWindow window, PadDriver pad)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            PadMenu.Choose(window, pad, "Rewind");
            Settle(window);
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal(MainWindow.MomentRow, PadMenu.Entries(window)[1].Label!());
            Assert.Equal("moment", PadMenu.Pane(window).Showing);
        }

        private static string Status(MainWindow w) => w.GetControl<TextBlock>("StatusText").Text ?? "";
        private static void Stop(MainWindow w) => typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(w, null);
        private static byte Count(MainWindow w) => MistressGame.Peek(w, "WRAM", 0x10);
        private static string Reel(MainWindow w) => MistressGame.Of(w).Crystal.RewindReel();

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

        // Until the frame count holds still, so the emulation thread is parked.
        private static long WaitUntilParked(MainWindow window)
        {
            long frames;
            do
            {
                frames = MistressGame.TotalFrames(window);
                Thread.Sleep(100);
                Dispatcher.UIThread.RunJobs();
            }
            while (MistressGame.TotalFrames(window) != frames);
            return frames;
        }

        [Fact]
        public Task A_goes_back_to_exactly_the_moment_the_page_shows_and_leaves_a_paused_game_paused() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 240);
            window.PauseEmulation();
            long frameBefore = WaitUntilParked(window);
            byte countBefore = Count(window);

            OpenReel(window, pad);
            Assert.True(window.IsPaused);
            IReadOnlyList<MainWindow.ReelMark> moments = window.ReelShown;
            double hz = MistressGame.Of(window).FrameRateHz;
            Assert.True(moments[0].IsNow);
            Assert.Equal(frameBefore, moments[0].Frame);
            Assert.True(moments.Count > 240 / Math.Round(hz / 2), $"{moments.Count} moments");
            Assert.Equal(moments.Select(m => m.Frame).OrderByDescending(f => f), moments.Select(m => m.Frame));
            Assert.Same(moments[window.ReelStops[1]], window.ReelFocused);
            Assert.All(moments.Skip(1), m => Assert.Equal(MainWindow.ReelMark.Ago((frameBefore - m.Frame) / hz), m.Label));

            pad.Right(1);
            Settle(window);
            MainWindow.ReelMark target = moments[window.ReelStops[2]];
            Assert.Same(target, window.ReelFocused);
            Assert.Equal(target.Label, PadMenu.Entries(window)[1].Value!());
            Assert.Equal($"{MainWindow.GoBackRow} to {target.Label}", PadMenu.Lines(window)[0]);
            UiTest.Dump("rewind-reel-page", UiTest.Capture(window));

            Assert.Equal(0, PadMenu.Selected(window));
            pad.A();
            WaitFor(() => Status(window).StartsWith("Rewound", StringComparison.Ordinal));
            Assert.False(PadMenu.IsOpen(window));
            Assert.True(window.IsPaused);
            WaitUntilParked(window);
            Assert.Equal(target.Frame, MistressGame.TotalFrames(window));
            Assert.Equal((byte)(countBefore - (frameBefore - target.Frame)), Count(window));
            _out.WriteLine($"{moments.Count} moments; back from frame {frameBefore} to {target.Frame} ({target.Label})");

            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task B_returns_to_the_card_leaving_the_machine_and_the_history_exactly_as_they_were() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 240);
            window.PauseEmulation();
            long frames = WaitUntilParked(window);
            byte count = Count(window);
            string history = Reel(window);

            OpenReel(window, pad);
            pad.Right(2);
            pad.L1();
            Settle(window);
            Assert.False(window.ReelFocused!.IsNow);

            pad.B();
            Settle(window);
            Assert.True(PadMenu.IsOpen(window));
            Assert.Equal(1, PadMenu.Depth(window));
            Assert.StartsWith("Rewind", PadMenu.SelectedLine(window));
            pad.B();
            Settle(window);
            WaitUntilParked(window);
            Assert.True(window.IsPaused);
            Assert.Equal(frames, MistressGame.TotalFrames(window));
            Assert.Equal(count, Count(window));
            Assert.Equal(history, Reel(window));

            Stop(window);
            window.Close();
        }, default);

        // The present is a moment on the strip, and A on it is no rewind: the card closes and nothing else changes.
        [Fact]
        public Task A_on_now_closes_the_card_and_changes_nothing() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 120);
            window.PauseEmulation();
            long frames = WaitUntilParked(window);
            string history = Reel(window);

            OpenReel(window, pad);
            pad.Right(2);
            pad.Left(3);
            Assert.True(window.ReelFocused!.IsNow);
            Assert.Equal("Stay at Now", PadMenu.Lines(window)[0]);
            pad.A();
            Settle(window);
            WaitUntilParked(window);

            Assert.False(PadMenu.IsOpen(window));
            Assert.Equal(frames, MistressGame.TotalFrames(window));
            Assert.Equal(history, Reel(window));
            Assert.True(window.IsPaused);
            Assert.False(Status(window).StartsWith("Rewound", StringComparison.Ordinal), $"choosing Now said '{Status(window)}'");

            Stop(window);
            window.Close();
        }, default);

        // The card paused the game, so after going back it plays on from the moment, which the reel still holds.
        [Fact]
        public Task A_from_the_card_goes_back_and_resumes_the_game_the_card_paused() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 240);

            OpenReel(window, pad);
            Assert.True(window.IsPaused);
            pad.Right(2);
            MainWindow.ReelMark target = window.ReelFocused!;
            pad.A();
            WaitFor(() => Status(window).StartsWith("Rewound", StringComparison.Ordinal));
            Assert.False(window.IsPaused);
            Assert.Contains($"\"frame\":{target.Frame}", Reel(window).Replace(" ", ""));

            Stop(window);
            window.Close();
        }, default);

        // As ingame-rewind-reel draws it: the strip is the present and moments sampled across the whole reel, a press steps between them, and Right held scrubs one held moment, half a second, at a time.
        [Fact]
        public Task A_press_steps_between_the_strip_s_stops_and_holding_scrubs_half_a_second_at_a_time() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 900);
            OpenReel(window, pad);
            IReadOnlyList<MainWindow.ReelMark> moments = window.ReelShown;
            int[] stops = window.ReelStops.ToArray();
            Assert.Equal(6, stops.Length);
            Assert.True(moments[stops[0]].IsNow);
            for (int i = 0; i < MainWindow.StopSeconds.Length; i++) Assert.InRange(moments[stops[i + 1]].SecondsBack, MainWindow.StopSeconds[i] - 0.3, MainWindow.StopSeconds[i] + 0.3);
            Assert.Equal(moments.Count - 1, stops[^1]);
            Assert.Equal(6, PadMenu.Pane(window).GetVisualDescendants().OfType<Border>().Count(b => b.Name?.StartsWith("PaneMoment", StringComparison.Ordinal) == true && b.Name != "PaneMoment" && b.Name != "PaneMoments"));

            pad.Right(4);
            Assert.Same(moments[stops[5]], window.ReelFocused);
            pad.Left(2);
            Assert.Same(moments[stops[3]], window.ReelFocused);

            TimeSpan now = TimeSpan.FromSeconds(100);
            typeof(MainWindow).GetProperty("UiClock", BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(window, (Func<TimeSpan>)(() => now));
            pad.Pad.Press(SDL.GamepadButton.DPadRight);
            pad.Tick();
            Assert.Same(moments[stops[4]], window.ReelFocused);
            for (int i = 1; i <= 3; i++)
            {
                now += TimeSpan.FromMilliseconds(i == 1 ? 450 : 90);
                pad.Tick();
                Assert.Same(moments[stops[4] + i], window.ReelFocused);
            }
            pad.Pad.Release(SDL.GamepadButton.DPadRight);
            pad.Tick();
            Settle(window);
            Assert.Equal(1, PadMenu.Pane(window).GetVisualDescendants().OfType<Border>().Count(b => b.Name?.StartsWith("PaneMoment", StringComparison.Ordinal) == true && b.BorderThickness.Left > 0));
            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task The_shoulders_stride_five_seconds_and_the_triggers_go_to_either_end() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            Play(window, 900);

            OpenReel(window, pad);
            IReadOnlyList<MainWindow.ReelMark> moments = window.ReelShown;
            Assert.True(moments[^1].SecondsBack > 12, $"{moments[^1].SecondsBack:F1} seconds held");

            pad.Press(UiButton.Last);
            Assert.Same(moments[0], window.ReelFocused);
            pad.L1();
            Settle(window);
            MainWindow.ReelMark once = window.ReelFocused!;
            Assert.InRange(once.SecondsBack, MainWindow.StrideSeconds, MainWindow.StrideSeconds + 0.5);
            Assert.True(moments[moments.ToList().IndexOf(once) - 1].SecondsBack < MainWindow.StrideSeconds);

            pad.L1();
            Settle(window);
            MainWindow.ReelMark twice = window.ReelFocused!;
            Assert.InRange(twice.SecondsBack - once.SecondsBack, MainWindow.StrideSeconds, MainWindow.StrideSeconds + 0.5);

            pad.R1();
            Settle(window);
            Assert.InRange(twice.SecondsBack - window.ReelFocused!.SecondsBack, MainWindow.StrideSeconds, MainWindow.StrideSeconds + 0.5);

            pad.Press(UiButton.First);
            Assert.Same(moments[^1], window.ReelFocused);
            pad.Right();
            Assert.Same(moments[^1], window.ReelFocused);
            pad.Press(UiButton.Last);
            Assert.Same(moments[0], window.ReelFocused);

            // Every row is reached by Down and drawn; the last a dimmed hint that does nothing.
            pad.Down(2);
            Assert.True(PadMenu.Entries(window)[2].Dimmed);
            pad.A();
            Assert.Equal(2, PadMenu.Depth(window));

            // B goes back to the card, and B again resumes the game the card paused.
            pad.B();
            pad.B();
            Settle(window);
            Assert.False(PadMenu.IsOpen(window));
            Assert.False(window.IsPaused);
            Stop(window);
            window.Close();
        }, default);

        // On the desktop the Game menu's Rewind opens the card on its reel page, and a click on Go back chooses it as A does.
        [Fact]
        public Task On_the_desktop_the_menu_s_rewind_opens_the_card_on_the_reel_and_a_click_goes_back() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: false);
            Play(window, 240);
            window.PauseEmulation();
            long frameBefore = WaitUntilParked(window);

            typeof(MainWindow).GetMethod("OpenRewindReel", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            Settle(window);
            Assert.True(PadMenu.IsOpen(window) && PadMenu.Card(window).IsVisible);
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal("moment", PadMenu.Pane(window).Showing);
            pad.Right(2);
            MainWindow.ReelMark target = window.ReelFocused!;
            Settle(window);
            UiTest.Dump("rewind-reel-desktop", UiTest.Capture(window));

            ListBoxItem row = PadMenu.List(window).GetVisualDescendants().OfType<ListBoxItem>().First(i => PadMenu.List(window).IndexFromContainer(i) == 0);
            Point at = row.TranslatePoint(new Point(row.Bounds.Width / 2, row.Bounds.Height / 2), window)!.Value;
            window.MouseDown(at, MouseButton.Left);
            window.MouseUp(at, MouseButton.Left);
            WaitFor(() => Status(window).StartsWith("Rewound", StringComparison.Ordinal));
            WaitUntilParked(window);
            Assert.Equal(target.Frame, MistressGame.TotalFrames(window));
            Assert.True(target.Frame < frameBefore);
            Assert.True(window.IsPaused);

            Stop(window);
            window.Close();
        }, default);

        [CSharpImplementationFact("clears the C# engine's rewind buffer behind the session")]
        public Task With_no_history_the_row_says_so_and_opens_nothing() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = WithAGame(bigScreen: true);
            WaitFor(() => MistressGame.Rewind(window).SnapshotBytes > 0);
            window.PauseEmulation();
            WaitUntilParked(window);
            MistressGame.Rewind(window).Clear();

            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            string[] lines = PadMenu.Lines(window);
            int at = Array.FindIndex(lines, l => l.StartsWith("Rewind", StringComparison.Ordinal));
            Assert.Equal("Rewind  (nothing to go back to yet)", lines[at]);
            pad.Down(at);
            pad.A();
            Settle(window);
            Assert.Equal(1, PadMenu.Depth(window));
            Assert.True(PadMenu.IsOpen(window));
            pad.B();

            Stop(window);
            window.Close();
        }, default);
    }
}

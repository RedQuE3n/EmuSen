using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Automation.Peers;
using Avalonia.Automation.Provider;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // The Pause Card's own pages of EmuSen_Interface.md §18.5: rows, panes and help as the mockups draw them, a screen opened over the card as a sheet, X on every page, and no frame run throughout.
    [Collection(TestCollections.ProcessGlobals)]
    public class PauseCardPagesTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PauseCardPagesTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPauseCardPages", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PauseCardPagesTests()
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

        private (MainWindow Window, PadDriver Pad) Playing(bool bigScreen = true)
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Cobalt Harbor (Synthetic).sfc"), PadRewindReelTests.ChangingBackdrop());
            var app = new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen, StateDirectory = Path.Combine(_root, "States") };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            app.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            if (bigScreen) OwnLook.Play(window, pad);
            else typeof(MainWindow).GetMethod("LaunchSelectedLibraryEntry", Hidden)!.Invoke(window, null);
            MistressGame.Play(window, 120, (until, what) => WaitFor(until, what));
            return (window, pad);
        }

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
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail($"timed out waiting for {what}");
                Thread.Sleep(2);
            }
        }

        private static void Controllers(MainWindow w, PadDriver pad)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            PadMenu.Choose(w, pad, MainWindow.GameOptionsMenu, exact: true);
            PadMenu.Choose(w, pad, MainWindow.ControllersMenu, exact: true);
            WindowFitAuditTests.Settle(w);
        }

        private static string Value(MainWindow w, int row) => PadMenu.Entries(w)[row].Value?.Invoke() ?? "";

        // ingame-options-controllers: each player's controller, the profile, Bindings and the swap, Choose and Value in the help.
        [Fact]
        public Task Controllers_is_each_player_s_controller_the_profile_the_bindings_and_the_swap() => Session.Dispatch(() =>
        {
            (MainWindow w, PadDriver pad) = Playing();
            Controllers(w, pad);
            string name = pad.Gamepad.Pads[0].Name;
            Assert.Equal(3, PadMenu.Depth(w));
            Assert.Equal(MainWindow.ControllersMenu, PadMenu.PageTitle(w));
            Assert.Equal("Cobalt Harbor (Synthetic) · Game Options", PadMenu.Big(w).Subtitle);
            Assert.Equal(["Player 1", "Player 2", "Profile", "Bindings", "Swap Players"], PadMenu.Lines(w));
            Assert.Equal((name, "None", "Default"), (Value(w, 0), Value(w, 1), Value(w, 2)));
            Assert.True(PadMenu.Entries(w)[4].Dimmed, "a swap with no player 2 is offered dimmed");
            Assert.Equal(["Choose", "Value", "Back", "Screenshot"], PadMenu.Big(w).Hints!.Select(h => h.Label));
            Assert.Equal("screen", PadMenu.Pane(w).Showing);
            Assert.Contains($"Player 1 plays on the {name}, with the Default profile.", PaneWords(w));

            // Right on Player 2 seats the pad there, as Settings' Players does, and the rows read it at once.
            pad.Down();
            pad.Right();
            Assert.Equal(2, pad.Gamepad.Players.PlayerOf(pad.Gamepad.Pads[0]));
            Assert.Equal(("None", name), (Value(w, 0), Value(w, 1)));
            pad.Left();
            Assert.Equal(0, pad.Gamepad.Players.PlayerOf(pad.Gamepad.Pads[0]));
            Done(w);
        }, default);

        // Bindings lays Settings' drawing page over the card as a sheet; B twice uncovers the card on Bindings, still paused, no frame run.
        [Fact]
        public Task Bindings_lays_the_drawing_over_the_card_and_b_uncovers_it_where_it_was() => Session.Dispatch(() =>
        {
            (MainWindow w, PadDriver pad) = Playing();
            Controllers(w, pad);
            long frames = MistressGame.TotalFrames(w);
            PadMenu.Choose(w, pad, "Bindings", exact: true);
            Assert.NotNull(w.SettingsShown?.Binding);
            Assert.True(PadMenu.IsOpen(w), "the card stays beneath the sheet");
            Assert.Equal(3, PadMenu.Depth(w));
            pad.B();
            pad.B();
            Dispatcher.UIThread.RunJobs();
            Assert.Null(w.SettingsShown);
            Assert.Equal(MainWindow.ControllersMenu, PadMenu.PageTitle(w));
            Assert.Equal("Bindings", PadMenu.SelectedLine(w));
            Assert.True(w.IsPaused);
            Assert.Equal(frames, MistressGame.TotalFrames(w));
            Done(w);
        }, default);

        // From the first page to the reel and back, into Controllers and back: one card, no frame run, every row a reader reaches reported selected, and X a screenshot on each page.
        [Fact]
        public Task Every_page_runs_no_frame_reports_its_row_selected_and_takes_a_screenshot_on_x() => Session.Dispatch(() =>
        {
            (MainWindow w, PadDriver pad) = Playing();
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            long frames = MistressGame.TotalFrames(w);
            string shots = Path.Combine(DataStore.Screenshots);
            int taken = 0;
            void Page(string what)
            {
                WindowFitAuditTests.Settle(w);
                Assert.True(w.IsPaused, what);
                Assert.Equal(frames, MistressGame.TotalFrames(w));
                for (int i = 0; i < PadMenu.Lines(w).Length; i++)
                {
                    ListBoxItem row = PadMenu.List(w).GetVisualDescendants().OfType<ListBoxItem>().Single(c => PadMenu.List(w).IndexFromContainer(c) == PadMenu.Selected(w));
                    Assert.True(ControlAutomationPeer.CreatePeerForElement(row).GetProvider<ISelectionItemProvider>()?.IsSelected, $"{what}: row {i} is not reported selected");
                    pad.Down();
                }
                pad.X();
                taken++;
                WaitFor(() => Directory.Exists(shots) && Directory.GetFiles(shots, "*.png").Length == taken, $"{what}: the screenshot");
            }
            PadMenu.Choose(w, pad, "Rewind");
            Page("the reel");
            pad.B();
            PadMenu.Choose(w, pad, MainWindow.GameOptionsMenu, exact: true);
            PadMenu.Choose(w, pad, MainWindow.ControllersMenu, exact: true);
            Page("Controllers");
            pad.B();
            pad.B();
            Assert.Equal(1, PadMenu.Depth(w));
            Assert.Equal(frames, MistressGame.TotalFrames(w));
            Done(w);
        }, default);

        // The card slides in on opening and out on closing, the game still and the scrim in place throughout, and on a page turn or a sheet's close it does not move; with motion reduced it comes and goes at once.
        [Fact]
        public Task The_card_slides_in_and_out_only_as_it_opens_and_closes() => Session.Dispatch(() =>
        {
            (MainWindow w, PadDriver pad) = Playing();
            ((AppSettings)typeof(MainWindow).GetField("_appSettings", Hidden)!.GetValue(w)!).BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningScaleUp;
            TimeSpan now = TimeSpan.FromSeconds(100);
            typeof(MainWindow).GetProperty("UiClock", Hidden)!.SetValue(w, (Func<TimeSpan>)(() => now));
            var slide = (EmuSen.Mistress.BigPicture.CardSlide)typeof(MainWindow).GetProperty("CardSlideNow", Hidden)!.GetValue(w)!;
            var card = w.GetControl<EmuSen.LunaP.Controls.AdaptiveLayout>("PadCard");
            EmuSen.LunaP.Controls.MenuPanel panel = card.CardPanel;
            Control scrim = card.GetVisualDescendants().OfType<Panel>().Single(p => p.Name == "AdaptiveCardScrim");
            void Run(int ms) { now += TimeSpan.FromMilliseconds(ms); slide.Advance(now); Dispatcher.UIThread.RunJobs(); }

            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            long frames = MistressGame.TotalFrames(w);
            Assert.Same(panel, slide.Moving);
            Assert.Equal(1.0, panel.SlideOut);
            Assert.Equal(1.0, panel.OpeningScale);
            Run(100);
            Assert.InRange(panel.SlideOut, 0.01, 0.99);
            Assert.Equal(1 - panel.SlideOut, panel.Pane!.Opacity, 6);
            Assert.True(scrim.IsEffectivelyVisible && scrim.Opacity == 1);
            Run(150);
            Assert.Null(slide.Moving);
            Assert.Equal(0.0, panel.SlideOut);
            Assert.Equal(1.0, panel.Pane!.Opacity);

            // A page turn and a sheet's close move nothing.
            PadMenu.Choose(w, pad, MainWindow.GameOptionsMenu, exact: true);
            Assert.Null(slide.Moving);
            PadMenu.Choose(w, pad, MainWindow.ControllersMenu, exact: true);
            PadMenu.Choose(w, pad, "Bindings", exact: true);
            Assert.NotNull(w.SettingsShown);
            w.CloseSettings();
            Dispatcher.UIThread.RunJobs();
            Assert.Null(slide.Moving);
            Assert.Equal(0.0, panel.SlideOut);

            // Closing: out over Exit, the game still until the card is gone, then resumed.
            pad.Start();
            Assert.True(slide.Leaving);
            Assert.True(w.GetControl<Control>("PadMenuPanel").IsVisible);
            Run(80);
            Assert.InRange(panel.Pane!.Opacity, 0.01, 0.99);
            Assert.True(w.IsPaused);
            Assert.Equal(frames, MistressGame.TotalFrames(w));
            Run(100);
            Assert.False(w.GetControl<Control>("PadMenuPanel").IsVisible);
            Assert.False(w.IsPaused);

            // With motion reduced the card comes and goes at once.
            EmuSen.LunaP.Theme.LunaMotion.Reduced = true;
            try
            {
                pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
                Assert.Null(slide.Moving);
                Assert.Equal(0.0, panel.SlideOut);
                pad.Start();
                Assert.False(w.GetControl<Control>("PadMenuPanel").IsVisible);
                Assert.False(w.IsPaused);
            }
            finally { EmuSen.LunaP.Theme.LunaMotion.Reduced = false; }
            Done(w);
        }, default);

        private static List<string> PaneWords(MainWindow w) =>
            PadMenu.Pane(w).GetVisualDescendants().OfType<TextBlock>().Select(t => t.Text ?? "").ToList();
    }
}

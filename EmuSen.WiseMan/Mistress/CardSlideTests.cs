using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using Xunit;

namespace EmuSen.WiseMan.Mistress
{
    // Every card on the Card form slides in and out as the Pause Card does: the List card, Start Console, Power, and the Home card over the desktop's list, its Home and a theme - see EmuSen_Interface.md §20.6.
    [Collection(TestCollections.ProcessGlobals)]
    public class CardSlideTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CardSlideTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCardSlide", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;
        private ThemedSession? _themed;

        public CardSlideTests()
        {
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            EmuSen.LunaP.Theme.LunaMotion.Reduced = false;
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // A window on the stepped clock with the menus' opening effect at its default, Scale-up, which the slide follows.
        private sealed class Rig(MainWindow window, PadDriver pad)
        {
            public MainWindow Window { get; } = window;
            public PadDriver Pad { get; } = pad;
            public TimeSpan Now = TimeSpan.FromSeconds(100);
            public CardSlide Slide => (CardSlide)typeof(MainWindow).GetProperty("CardSlideNow", Hidden)!.GetValue(Window)!;
            public MenuPanel Panel => PadMenu.Big(Window);
            public bool Showing => Window.GetControl<Control>("PadMenuPanel").IsVisible;

            public void Run(int ms)
            {
                Now += TimeSpan.FromMilliseconds(ms);
                Slide.Advance(Now);
                Dispatcher.UIThread.RunJobs();
            }
        }

        private Rig Open(bool bigScreen)
        {
            foreach (string g in new[] { "Aurora Drift (Synthetic)", "Brass Lantern (Synthetic)", "Cobalt Harbor (Synthetic)" })
                File.WriteAllBytes(Path.Combine(_romDir, g + ".sfc"), SyntheticRom.BuildBlank());
            var app = new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningScaleUp;
            app.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            return Clocked(window, new PadDriver(window));
        }

        private static Rig Clocked(MainWindow window, PadDriver pad)
        {
            var rig = new Rig(window, pad);
            typeof(MainWindow).GetProperty("UiClock", Hidden)!.SetValue(window, (Func<TimeSpan>)(() => rig.Now));
            Dispatcher.UIThread.RunJobs();
            return rig;
        }

        // Each card brought on screen as the player brings it there.
        private Rig Card(string card)
        {
            switch (card)
            {
                case "list":
                {
                    Rig r = Open(bigScreen: true);
                    OwnLook.OpenGames(r.Window, r.Pad);
                    r.Pad.Select();
                    return r;
                }
                case "start-console":
                {
                    Rig r = Card("list");
                    r.Run(400);
                    PadMenu.Choose(r.Window, r.Pad, "Start Console", exact: true);
                    Assert.Equal(MainWindow.StartConsoleMenu, PadMenu.PageTitle(r.Window));
                    return r;
                }
                case "power":
                {
                    Rig r = Open(bigScreen: true);
                    r.Window.HomeLayout!.FocusOn(HubRow.Places, HomeScreen.Power);
                    r.Pad.A();
                    Assert.Equal(MainWindow.PowerMenu, PadMenu.PageTitle(r.Window));
                    return r;
                }
                case "home-desktop":
                {
                    Rig r = Open(bigScreen: false);
                    r.Window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
                    r.Pad.Start();
                    return r;
                }
                case "home-desktop-home":
                {
                    Rig r = Open(bigScreen: false);
                    typeof(MainWindow).GetMethod("ShowHome", Hidden)!.Invoke(r.Window, null);
                    Dispatcher.UIThread.RunJobs();
                    r.Pad.Start();
                    return r;
                }
                default:
                {
                    _themed = new ThemedSession(settings: a => a.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningScaleUp);
                    ThemedLibraryPadTests.Enter(_themed, "snes");
                    Rig r = Clocked(_themed.Window, _themed.Pad);
                    r.Pad.Start();
                    return r;
                }
            }
        }

        private void Done(Rig r)
        {
            typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(r.Window, null);
            if (_themed is { } themed) themed.Dispose();
            else r.Window.Close();
            _themed = null;
        }

        public static TheoryData<string> Cards() => new() { "list", "start-console", "power", "home-desktop", "home-desktop-home", "home-theme" };

        // In from the left over Enter with the pane fading in, out over Exit with B; never grown, and gone from the screen only when the slide ends.
        [Theory]
        [MemberData(nameof(Cards))]
        public Task Each_card_slides_in_on_opening_and_out_on_closing(string card) => Session.Dispatch(() =>
        {
            Rig r = Card(card);
            MenuPanel panel = r.Panel;
            Assert.True(PadMenu.IsOpen(r.Window), card);
            Assert.Same(panel, r.Slide.Moving);
            Assert.Equal(1.0, panel.SlideOut);
            Assert.Equal(1.0, panel.OpeningScale);
            r.Run(100);
            Assert.InRange(panel.SlideOut, 0.01, 0.99);
            Assert.Equal(1 - panel.SlideOut, panel.Pane!.Opacity, 6);
            Assert.Equal(1.0, panel.OpeningScale);
            r.Run(150);
            Assert.Null(r.Slide.Moving);
            Assert.Equal(0.0, panel.SlideOut);

            r.Pad.B();
            Assert.False(PadMenu.IsOpen(r.Window));
            Assert.True(r.Slide.Leaving, card);
            Assert.True(r.Showing);
            r.Run(80);
            Assert.InRange(panel.SlideOut, 0.01, 0.99);
            r.Run(100);
            Assert.False(r.Showing);
            Assert.Equal(0.0, panel.SlideOut);
            Done(r);
        }, default);

        // Closing with Start slides out too, and the grid beneath takes no press until the card is gone.
        [Fact]
        public Task The_grid_beneath_takes_the_pad_only_once_the_list_card_has_slid_out() => Session.Dispatch(() =>
        {
            Rig r = Card("list");
            r.Run(400);
            GalleryLayout grid = r.Window.GamesLayout!;
            string at = grid.Focused.Key;
            r.Pad.Start();
            Assert.True(r.Slide.Leaving);
            r.Pad.Right();
            Assert.Equal(at, grid.Focused.Key);
            r.Run(200);
            Assert.False(r.Showing);
            r.Pad.Right();
            Assert.NotEqual(at, grid.Focused.Key);
            Done(r);
        }, default);

        // Into Filter and back: one card, still throughout.
        [Fact]
        public Task A_page_turn_inside_the_list_card_moves_nothing() => Session.Dispatch(() =>
        {
            Rig r = Card("list");
            r.Run(400);
            Assert.Null(r.Slide.Moving);
            PadMenu.Choose(r.Window, r.Pad, "Filter", exact: true);
            Assert.Equal(2, PadMenu.Depth(r.Window));
            Assert.Null(r.Slide.Moving);
            Assert.Equal(0.0, r.Panel.SlideOut);
            Assert.Equal(1.0, r.Panel.OpeningScale);
            r.Pad.B();
            Assert.Equal(1, PadMenu.Depth(r.Window));
            Assert.True(PadMenu.IsOpen(r.Window));
            Assert.Null(r.Slide.Moving);
            Assert.Equal(0.0, r.Panel.SlideOut);
            Done(r);
        }, default);

        // With motion reduced the List card comes and goes at once.
        [Fact]
        public Task With_motion_reduced_the_list_card_comes_and_goes_at_once() => Session.Dispatch(() =>
        {
            Rig r = Open(bigScreen: true);
            OwnLook.OpenGames(r.Window, r.Pad);
            EmuSen.LunaP.Theme.LunaMotion.Reduced = true;
            r.Pad.Select();
            Assert.True(PadMenu.IsOpen(r.Window));
            Assert.Null(r.Slide.Moving);
            Assert.Equal(0.0, r.Panel.SlideOut);
            Assert.Equal(1.0, r.Panel.OpeningScale);
            r.Pad.B();
            Assert.False(r.Slide.Leaving);
            Assert.False(r.Showing);
            Done(r);
        }, default);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "card-slide");

        // The List card at 1280 by 800 halfway in, settled, and halfway out, for the eye.
        [CardSlidePngFact]
        public Task The_list_card_halfway_through_its_slide_is_drawn_to_pictures() => Session.Dispatch(() =>
        {
            Rig r = Card("list");
            Directory.CreateDirectory(PngFolder);
            void Save(string name)
            {
                for (int i = 0; i < 3; i++) { Dispatcher.UIThread.RunJobs(); UiTest.Capture(r.Window); }
                UiTest.Capture(r.Window).SavePng(Path.Combine(PngFolder, name + ".png"));
            }
            r.Run(100);
            Save("list-card-entering-halfway");
            r.Run(300);
            Save("list-card-settled");
            r.Pad.B();
            r.Run(80);
            Save("list-card-leaving-halfway");
            Done(r);
        }, default);
    }

    public sealed class CardSlidePngFactAttribute : FactAttribute
    {
        public CardSlidePngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the List card's slide to ~/.cache/emusen/bigpicture/png/card-slide/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_Interface.md §20.6";
        }
    }
}

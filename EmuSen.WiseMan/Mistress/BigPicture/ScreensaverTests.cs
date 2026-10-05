using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Media;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Pass 10: ES-DE's screensaver over the big-screen library, and the frames it draws - see EmuSen_BigPicture.md §37.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScreensaverTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScreensaverTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly ITestOutputHelper _out;

        public ScreensaverTests(ITestOutputHelper output) => _out = output;

        internal const string Aurora = "Aurora Drift (Synthetic)", Brass = "Brass Lantern (Synthetic)", Cobalt = "Cobalt Harbor (Synthetic)",
            Dune = "Dune Relay (Synthetic)", Ember = "Ember Circuit (Synthetic)", Fable = "Fable of Tiles (Synthetic)";

        internal static Screensaver? Saver(ThemedSession s) => (Screensaver?)typeof(MainWindow).GetProperty("ScreensaverShown", Hidden)!.GetValue(s.Window);

        internal static int SaverFrames(ThemedSession s) => (int)typeof(MainWindow).GetProperty("ScreensaverFramesDrawn", Hidden)!.GetValue(s.Window)!;

        internal static TimeSpan? SaverWake(ThemedSession s) => (TimeSpan?)typeof(MainWindow).GetProperty("ScreensaverWakeAt", Hidden)!.GetValue(s.Window);

        private static void Set(ThemedSession s, string property, object value) => typeof(MainWindow).GetProperty(property, Hidden)!.SetValue(s.Window, value);

        private static string? Running(ThemedSession s) => LaunchScreenTests.Running(s.Window);

        // ES-DE's kinds for each game, drawn so a picture's two halves say whose and which it is (§37: miximage, screenshot, titlescreen, cover, in that order).
        internal static readonly Dictionary<string, string[]> Kinds = new()
        {
            [Aurora] = ["miximages", "screenshots", "titlescreens", "covers", "marquees"],
            [Brass] = ["screenshots", "titlescreens", "covers"],
            [Cobalt] = ["titlescreens", "covers"],
            [Dune] = ["covers"],
            [Ember] = ["marquees", "fanart", "3dboxes"],
            [Fable] = ["screenshots"],
        };

        internal static readonly Dictionary<string, string> Shown = new()
        {
            [Aurora] = "miximages", [Brass] = "screenshots", [Cobalt] = "titlescreens", [Dune] = "covers", [Fable] = "screenshots",
        };

        internal static string Media(string root)
        {
            string media = Path.Combine(root, "downloaded_media");
            var colours = new[] { Colors.Firebrick, Colors.SeaGreen, Colors.SteelBlue, Colors.Goldenrod, Colors.Orchid, Colors.Teal };
            int n = 0;
            foreach ((string game, string[] kinds) in Kinds)
                foreach (string kind in kinds)
                {
                    string system = game == Fable ? "nes" : "snes";
                    string path = Path.Combine(media, system, kind, game + ".png");
                    Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                    Color c = colours[n++ % colours.Length];
                    File.Copy(SceneAssets.Halves($"saver-{kind}-{n}", 640, 480, c, c), path, overwrite: true);
                }
            return media;
        }

        internal static ThemedSession Open(Action<BigPictureInterface>? saver = null, double width = 1280, double height = 800, Action<AppSettings>? more = null, string? extraGamelist = null)
        {
            string? media = null;
            var s = new ThemedSession(width, height, extraGamelist: extraGamelist, settings: a =>
            {
                a.EsdeMediaDirectory = media;
                a.BigPictureInterface.ScreensaverTimer = 300000;
                saver?.Invoke(a.BigPictureInterface);
                more?.Invoke(a);
            }, roms: r => media = Media(Path.GetDirectoryName(r)!));
            Set(s, "ScreensaverRandom", new Random(7));
            return s;
        }

        private static string PictureGame(Screensaver saver) => Path.GetFileNameWithoutExtension(saver.PicturePath!);

        private static string PictureKind(Screensaver saver) => Path.GetFileName(Path.GetDirectoryName(saver.PicturePath!)!);

        // The defaults a new appsettings.json gets, ES-DE's but for the type (Q34), and the values as es_settings.xml spells them.
        [Fact]
        public void The_defaults_are_ES_DE_s_but_Dim_and_the_values_are_its_own()
        {
            var i = new BigPictureInterface();
            Assert.Equal(300000, i.ScreensaverTimer);
            Assert.Equal("dim", i.ScreensaverType);
            Assert.True(i.ScreensaverControls);
            Assert.Equal(10000, i.ScreensaverSwapImageTimeout);
            Assert.False(i.ScreensaverSlideshowOnlyFavorites);
            Assert.False(i.ScreensaverStretchImages);
            Assert.True(i.ScreensaverSlideshowGameInfo);
            Assert.False(i.ScreensaverSlideshowCustomImages);
            Assert.False(i.ScreensaverSlideshowRecurse);
            Assert.Equal("", i.ScreensaverSlideshowCustomDir);
            Assert.True(i.ScreensaverInGameMode);
            Assert.Equal(["dim", "black", "slideshow", "video"], InterfaceSettingsPane.ScreensaverChoices.Select(c => c.Value));
        }

        [Fact]
        public Task It_starts_after_five_idle_minutes_and_not_a_poll_before() => Session.Dispatch(() =>
        {
            using var s = Open();
            ThemedLibraryPadTests.Enter(s, "snes");
            TimeSpan idle = s.Now;
            s.Run(300000 - 16 * 50, step: 16 * 50);
            s.Run(16 * 49);
            Assert.Null(Saver(s));
            s.Run(16);
            Screensaver saver = Assert.IsType<Screensaver>(Saver(s));
            Assert.Equal(BigPictureInterface.SaverDim, saver.Kind);
            _out.WriteLine($"idle from {idle}, opened at {saver.Opened}");
            Assert.InRange((saver.Opened - idle).TotalMilliseconds, 300000, 300000 + 16);
        }, default);

        [Fact]
        public Task A_press_starts_the_idle_time_again_and_0_is_never() => Session.Dispatch(() =>
        {
            using var s = Open();
            s.Run(200000, step: 1000);
            s.Pad.Down();
            s.Run(200000, step: 1000);
            Assert.Null(Saver(s));
            s.Run(100000, step: 1000);
            Assert.NotNull(Saver(s));

            using var never = Open(i => i.ScreensaverTimer = 0);
            never.Run(3600000, step: 5000);
            Assert.Null(Saver(never));
        }, default);

        // P114 for Dim and Black: frames only through the fade, then none while it holds, and the view beneath draws none either.
        [Theory]
        [InlineData(BigPictureInterface.SaverDim, 167)]
        [InlineData(BigPictureInterface.SaverBlack, 140)]
        public Task Dim_and_Black_draw_only_their_fade_and_nothing_while_they_hold(string type, int fadeMs) => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = type; });
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Run(59000, step: 1000);
            while (Saver(s) is null) s.Run(16);
            Screensaver saver = Saver(s)!;
            int themed = s.FramesDrawn, frames = SaverFrames(s);
            s.Run(fadeMs + 32);
            int fade = SaverFrames(s) - frames;
            Assert.Null(SaverWake(s));
            Assert.Null(s.WakeAt);
            frames = SaverFrames(s);
            s.Run(600000, step: 16 * 25);
            s.Run(10000);
            _out.WriteLine($"{type}: {fade} frames through the {fadeMs} ms fade; {SaverFrames(s) - frames} in the 610 s after; the view {s.FramesDrawn - themed}");
            Assert.InRange(fade, fadeMs / 16 - 1, fadeMs / 16 + 2);
            Assert.Equal(0, SaverFrames(s) - frames);
            Assert.Equal(0, s.FramesDrawn - themed);
            Assert.Equal(type == BigPictureInterface.SaverDim ? 0 : 1, saver.Saturation);
            Assert.Equal(type == BigPictureInterface.SaverDim ? 0.4 : 0, saver.Brightness, 3);
        }, default);

        [Fact]
        public Task Dim_fades_both_levels_together_over_its_time() => Session.Dispatch(() =>
        {
            using var s = Open(i => i.ScreensaverTimer = 60000);
            s.Run(60000, step: 1000);
            while (Saver(s) is null) s.Run(16);
            Screensaver saver = Saver(s)!;
            Assert.Equal(1, saver.Saturation);
            s.Run(80);
            double at = (s.Now - saver.Opened).TotalMilliseconds / 167;
            Assert.Equal(1 - at, saver.Saturation, 3);
            Assert.Equal(1 - 0.6 * at, saver.Brightness, 3);
        }, default);

        private static (byte R, byte G, byte B) At(RenderedFrame f, int x, int y)
        {
            int i = (y * f.Width + x) * 4;
            return (f.Rgba[i], f.Rgba[i + 1], f.Rgba[i + 2]);
        }

        // ES-DE's Dim measured: every pixel the grey of its luma at 0.4, the theme's help bar included; Black, every pixel black.
        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1200)]
        public Task Dim_is_the_luma_grey_at_0_4_and_Black_is_black_over_the_whole_window(int w, int h) => Session.Dispatch(() =>
        {
            using var s = Open(i => i.ScreensaverTimer = 60000, w, h);
            ThemedLibraryPadTests.Enter(s, "snes");
            RenderedFrame before = s.Capture();
            s.Run(61000, step: 500);
            s.Run(300);
            Assert.Equal(BigPictureInterface.SaverDim, Saver(s)?.Kind);
            RenderedFrame dim = s.Capture();
            int worst = 0, coloured = 0;
            for (int y = 0; y < before.Height; y += 7)
                for (int x = 0; x < before.Width; x += 7)
                {
                    (byte r, byte g, byte b) = At(before, x, y);
                    (byte r2, byte g2, byte b2) = At(dim, x, y);
                    double grey = 0.4 * (0.3 * r + 0.59 * g + 0.11 * b);
                    worst = Math.Max(worst, (int)Math.Ceiling(new[] { Math.Abs(r2 - grey), Math.Abs(g2 - grey), Math.Abs(b2 - grey) }.Max()));
                    if (r != g || g != b) coloured++;
                }
            _out.WriteLine($"{w}x{h}: {coloured} coloured samples; the largest distance from 0.4 x luma {worst}");
            Assert.True(coloured > 500);
            Assert.InRange(worst, 0, 2);

            s.Pad.B();
            Assert.Null(Saver(s));
            s.Settings.BigPictureInterface.ScreensaverType = BigPictureInterface.SaverBlack;
            s.Run(61000, step: 500);
            s.Run(300);
            RenderedFrame black = s.Capture();
            Assert.All(Enumerable.Range(0, black.Width * black.Height).Where(i => i % 97 == 0), i => Assert.Equal(0, black.Rgba[i * 4] + black.Rgba[i * 4 + 1] + black.Rgba[i * 4 + 2]));
        }, default);

        // The press that wakes it is its own, as ES-DE's is: the list does not move, even while the button stays down, and the next press moves it.
        [Fact]
        public Task Any_press_wakes_it_and_does_nothing_else() => Session.Dispatch(() =>
        {
            using var s = Open(i => i.ScreensaverTimer = 60000);
            ThemedLibraryPadTests.Enter(s, "snes");
            string game = s.Game!;
            s.Run(61000, step: 1000);
            Assert.NotNull(Saver(s));
            s.Hold(SDL3.SDL.GamepadButton.DPadDown, 1000);
            Assert.Null(Saver(s));
            Assert.Equal(game, s.Game);
            s.Pad.Down();
            Assert.NotEqual(game, s.Game);

            s.Run(61000, step: 1000);
            Assert.NotNull(Saver(s));
            s.Pad.A();
            Assert.Null(Saver(s));
            Assert.Null(Running(s));
            Assert.Equal("gamelist", s.View);
            s.Pad.Start();
            Assert.True(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
        }, default);

        [Fact]
        public Task A_key_wakes_it_and_does_nothing_else() => Session.Dispatch(() =>
        {
            using var s = Open(i => i.ScreensaverTimer = 60000);
            ThemedLibraryPadTests.Enter(s, "snes");
            string game = s.Game!;
            s.Run(61000, step: 1000);
            Assert.NotNull(Saver(s));
            BigPictureSwitchTests.Press(s, Key.Down);
            s.Run(32);
            Assert.Null(Saver(s));
            Assert.Equal(game, s.Game);
            s.Run(59000, step: 1000);
            Assert.Null(Saver(s));
            BigPictureSwitchTests.Press(s, Key.Q);
            s.Run(59000, step: 1000);
            Assert.Null(Saver(s));
            s.Run(2000, step: 1000);
            Assert.NotNull(Saver(s));
        }, default);

        // ES-DE's slideshow measured: a picture per game by miximage, screenshot, titlescreen, cover; games without one left out; never the same game twice running; every 10 s.
        [Fact]
        public Task The_slideshow_shows_each_game_s_first_picture_in_ES_DE_s_order_every_10_s() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; });
            ThemedCollectionsTests.Records(s).ToggleFavourite(ThemedCollectionsTests.Rom(s, Aurora));
            ThemedCollectionsTests.Refresh(s);
            s.Run(61000, step: 1000);
            Screensaver saver = Assert.IsType<Screensaver>(Saver(s));
            Assert.Equal(BigPictureInterface.SaverSlideshow, saver.Kind);
            var seen = new List<string>();
            TimeSpan last = saver.Changed;
            for (int i = 0; i < 40; i++)
            {
                string game = PictureGame(saver);
                seen.Add(game);
                Assert.Equal(Shown[game], PictureKind(saver));
                Assert.Equal(game, saver.GameName);
                Assert.Equal(game == Fable ? "Nintendo Entertainment System" : "Super Nintendo", saver.SystemName);
                Assert.Equal(game == Aurora, saver.StarShown);
                s.Run(10000, step: 250);
                Assert.Equal(TimeSpan.FromSeconds(10), saver.Changed - last);
                last = saver.Changed;
            }
            _out.WriteLine(string.Join(", ", seen));
            Assert.Equal(Shown.Keys.Order(), seen.Distinct().Order());
            Assert.DoesNotContain(seen.Zip(seen.Skip(1)), p => p.First == p.Second);
        }, default);

        // P114 for the slideshow: frames only through each change (the overlay's 117 ms, then the picture from 217 to 450 ms), none in the rest of the 10 s.
        [Fact]
        public Task The_slideshow_draws_only_its_changes() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; });
            s.Run(60000, step: 1000);
            while (Saver(s) is null) s.Run(16);
            Screensaver saver = Saver(s)!;
            int themed = s.FramesDrawn;
            for (int swap = 0; swap < 6; swap++)
            {
                TimeSpan changed = saver.Changed;
                int start = SaverFrames(s);
                while (s.Now < changed + TimeSpan.FromMilliseconds(460)) s.Run(16);
                int during = SaverFrames(s) - start;
                Assert.Equal(1, saver.PictureOpacity);
                Assert.Equal(1, saver.OverlayOpacity);
                Assert.Equal(changed + TimeSpan.FromSeconds(10), SaverWake(s));
                int after = SaverFrames(s);
                while (s.Now < changed + TimeSpan.FromMilliseconds(10000 - 16)) s.Run(16);
                int between = SaverFrames(s) - after;
                _out.WriteLine($"swap {swap}: {during} frames through the change, {between} in the {(changed + TimeSpan.FromSeconds(10) - s.Now).TotalMilliseconds + 9540 - 16:F0} ms after");
                Assert.InRange(during, 20, 32);
                Assert.Equal(0, between);
                s.Run(32);
                Assert.NotEqual(changed, saver.Changed);
            }
            Assert.Equal(0, s.FramesDrawn - themed);
        }, default);

        // ES-DE's swap measured: black at once, the overlay whole by 117 ms, the picture unseen until 217 ms and then at t / 450 ms.
        [Fact]
        public Task A_change_cuts_to_black_and_fades_the_picture_in_on_ES_DE_s_timing() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; });
            s.Run(60000, step: 1000);
            while (Saver(s) is null) s.Run(16);
            Screensaver saver = Saver(s)!;
            Assert.Equal(0, saver.PictureOpacity);
            Assert.Equal(0, saver.OverlayOpacity);
            s.Run(64);
            Assert.InRange(saver.OverlayOpacity, 0.5, 0.6);
            Assert.Equal(0, saver.PictureOpacity);
            s.Run(64);
            Assert.Equal(1, saver.OverlayOpacity);
            while (s.Now < saver.Changed + TimeSpan.FromMilliseconds(217)) s.Run(16);
            Assert.InRange(saver.PictureOpacity, 0.48, 0.53);
            s.Run(112);
            Assert.InRange(saver.PictureOpacity, 0.73, 0.78);
            s.Run(160);
            Assert.Equal(1, saver.PictureOpacity);
        }, default);

        [Fact]
        public Task Only_favourites_shows_no_star_and_the_overlay_can_be_turned_off() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; i.ScreensaverSlideshowOnlyFavorites = true; });
            ThemedCollectionsTests.Records(s).ToggleFavourite(ThemedCollectionsTests.Rom(s, Aurora));
            ThemedCollectionsTests.Records(s).ToggleFavourite(ThemedCollectionsTests.Rom(s, Dune));
            ThemedCollectionsTests.Refresh(s);
            s.Run(61000, step: 1000);
            Screensaver saver = Saver(s)!;
            var seen = new HashSet<string>();
            for (int i = 0; i < 12; i++)
            {
                seen.Add(PictureGame(saver));
                Assert.False(saver.StarShown);
                Assert.NotNull(saver.GameName);
                s.Run(10000, step: 500);
            }
            Assert.Equal([Aurora, Dune], seen.Order());

            s.Pad.B();
            s.Settings.BigPictureInterface.ScreensaverSlideshowGameInfo = false;
            s.Settings.BigPictureInterface.ScreensaverStretchImages = true;
            s.Run(61000, step: 1000);
            Assert.Null(Saver(s)!.GameName);
            Assert.Equal(0, Saver(s)!.OverlayOpacity);
            Assert.Equal(ImageFit.Fill, Saver(s)!.PictureFit);
        }, default);

        // ES-DE fits a 4:3 picture to the height with black at the sides, and with the stretch on fills the screen.
        [Fact]
        public Task A_picture_is_fitted_whole_and_stretched_only_when_asked() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; });
            s.Run(61000, step: 1000);
            s.Run(500);
            RenderedFrame fitted = s.Capture();
            Assert.Equal((0, 0, 0), ((int, int, int))At(fitted, 60, 600));
            Assert.NotEqual((0, 0, 0), ((int, int, int))At(fitted, 640, 600));
            s.Pad.B();
            s.Settings.BigPictureInterface.ScreensaverStretchImages = true;
            s.Run(61000, step: 1000);
            s.Run(500);
            RenderedFrame stretched = s.Capture();
            Assert.NotEqual((0, 0, 0), ((int, int, int))At(stretched, 60, 600));
        }, default);

        [Fact]
        public Task A_custom_folder_is_shown_without_an_overlay_and_its_subfolders_only_when_asked() => Session.Dispatch(() =>
        {
            string folder = Path.Combine(Path.GetTempPath(), "EmuSenSaverCustom", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(Path.Combine(folder, "sub"));
            foreach (string n in new[] { "one.png", "two.jpg", "sub/three.png", "notes.txt" }) File.Copy(SceneAssets.Halves("saver-custom", 64, 40, Colors.Tomato, Colors.Tomato), Path.Combine(folder, n), overwrite: true);
            try
            {
                using var s = Open(i =>
                {
                    i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; i.ScreensaverSlideshowCustomImages = true; i.ScreensaverSlideshowCustomDir = folder;
                });
                s.Run(61000, step: 1000);
                var seen = new HashSet<string>();
                for (int i = 0; i < 10; i++)
                {
                    seen.Add(Path.GetFileName(Saver(s)!.PicturePath!));
                    Assert.Null(Saver(s)!.GameName);
                    s.Run(10000, step: 500);
                }
                Assert.Equal(["one.png", "two.jpg"], seen.Order());
                Assert.Equal(3, Screensaver.CustomImages(folder, recurse: true, null).Count);
                Assert.Equal(2, Screensaver.CustomImages("%ROMPATH%/" + Path.GetFileName(folder), recurse: false, Path.GetDirectoryName(folder)).Count);
                Assert.Empty(Screensaver.CustomImages(Path.Combine(folder, "missing"), recurse: true, null));
            }
            finally
            {
                Directory.Delete(folder, recursive: true);
            }
        }, default);

        // Video waits for Pass 12 and a slideshow with no picture to show has nothing to show: both are Dim, as in ES-DE.
        [Fact]
        public Task Video_and_an_empty_slideshow_are_Dim() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverVideo; });
            s.Run(61000, step: 1000);
            Assert.Equal(BigPictureInterface.SaverDim, Saver(s)?.Kind);
            s.Pad.B();
            s.Settings.BigPictureInterface.ScreensaverType = BigPictureInterface.SaverSlideshow;
            s.Settings.EsdeMediaDirectory = Path.Combine(s.Root, "no-media");
            s.Run(61000, step: 1000);
            Assert.Equal(BigPictureInterface.SaverDim, Saver(s)?.Kind);
        }, default);

        // ES-DE's controls: right and left another game at once, Y to its gamelist, A through the launch screen to the game.
        [Fact]
        public Task Left_and_right_change_the_game_Y_goes_to_it_and_A_starts_it() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; },
                more: a => a.BigPictureInterface.LaunchScreenDuration = BigPictureInterface.LaunchNormal);
            s.Run(61000, step: 1000);
            Screensaver saver = Saver(s)!;
            string first = PictureGame(saver);
            s.Run(3000);
            s.Pad.Right();
            Assert.NotEqual(first, PictureGame(saver));
            Assert.InRange(s.Now - saver.Changed, TimeSpan.Zero, TimeSpan.FromMilliseconds(32));
            s.Run(500);
            Assert.Equal(saver.Changed + TimeSpan.FromSeconds(10), SaverWake(s));
            string second = PictureGame(saver);
            s.Pad.Left();
            Assert.NotEqual(second, PictureGame(saver));

            string shown = PictureGame(saver);
            s.Pad.Y();
            Assert.Null(Saver(s));
            Assert.Equal("gamelist", s.View);
            Assert.Equal(shown, s.Game);
            Assert.Equal(shown == Fable ? "nes" : "snes", s.System);

            s.Run(61000, step: 1000);
            saver = Saver(s)!;
            // The NES stand-ins are not ROMs that load, so an SNES game is the one started.
            while (PictureGame(saver) == Fable) s.Pad.Right();
            shown = PictureGame(saver);
            s.Pad.A();
            Assert.Null(Saver(s));
            Assert.Equal(shown, s.Game);
            Assert.Equal(shown, LaunchScreenTests.Screen(s)?.GameName);
            s.Run(3000);
            Assert.Equal(shown, Path.GetFileNameWithoutExtension(Running(s)));
            LaunchScreenTests.Stop(s.Window);
        }, default);

        [Fact]
        public Task With_the_controls_off_or_in_Dim_every_button_only_wakes_it() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; i.ScreensaverControls = false; });
            ThemedLibraryPadTests.Enter(s, "snes");
            string game = s.Game!;
            foreach (Action press in new Action[] { () => s.Pad.Right(), s.Pad.A, s.Pad.Y })
            {
                s.Run(61000, step: 1000);
                Assert.NotNull(Saver(s));
                press();
                Assert.Null(Saver(s));
                Assert.Equal(game, s.Game);
                Assert.Equal("snes", s.System);
                Assert.Null(Running(s));
            }

            s.Settings.BigPictureInterface.ScreensaverControls = true;
            s.Settings.BigPictureInterface.ScreensaverType = BigPictureInterface.SaverDim;
            foreach (Action press in new Action[] { () => s.Pad.Right(), s.Pad.A, s.Pad.Y })
            {
                s.Run(61000, step: 1000);
                Assert.Equal(BigPictureInterface.SaverDim, Saver(s)?.Kind);
                press();
                Assert.Null(Saver(s));
                Assert.Equal(game, s.Game);
                Assert.Null(Running(s));
            }
        }, default);

        // X from the system view starts it with the timer at never; in a gamelist it does not; while it shows, X wakes the view.
        [Fact]
        public Task X_in_the_system_view_starts_it_and_its_help_names_it() => Session.Dispatch(() =>
        {
            using var s = Open(i => i.ScreensaverTimer = 0);
            Assert.Equal("system", s.View);
            Assert.Contains("Screensaver", HelpLabels(s));
            s.Pad.X();
            Assert.NotNull(Saver(s));
            s.Run(5000);
            s.Pad.X();
            Assert.Null(Saver(s));
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.X();
            Assert.Null(Saver(s));
            Assert.DoesNotContain("Screensaver", HelpLabels(s));

            s.Settings.BigPictureInterface.ScreensaverControls = false;
            s.Pad.B();
            s.Refresh();
            Assert.Equal("system", s.View);
            Assert.DoesNotContain("Screensaver", HelpLabels(s));
            s.Pad.X();
            Assert.Null(Saver(s));
        }, default);

        private static string[] HelpLabels(ThemedSession s) =>
            s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().SelectMany(b => b.Entries ?? []).Select(e => e.Label).ToArray();

        // ES-DE starts none while a menu is open; Mistress none under a sheet, the launch screen or a game either.
        [Fact]
        public Task It_never_starts_under_a_menu_a_sheet_or_a_game() => Session.Dispatch(() =>
        {
            using var s = Open(i => i.ScreensaverTimer = 60000);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Start();
            s.Run(600000, step: 1000);
            Assert.Null(Saver(s));
            s.Pad.B();
            ThemedCollectionsTests.OpenMenu(s);
            s.Run(600000, step: 1000);
            Assert.Null(Saver(s));
            s.Pad.B();
            s.Settle();
            s.Pad.A();
            Assert.NotNull(Running(s));
            s.Run(600000, step: 1000);
            Assert.Null(Saver(s));
            ThemedLibraryFlowTests.Choose(s, "Back to Library");
            s.Run(61000, step: 1000);
            Assert.NotNull(Saver(s));
            LaunchScreenTests.Stop(s.Window);
        }, default);

        // Q34: the Game Mode switch, on by default; off, a gamescope session never starts it and a desktop session still does.
        [Fact]
        public Task The_Game_Mode_switch_keeps_it_off_in_Game_Mode_only() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverInGameMode = false; });
            Set(s, "ScreensaverEnvironment", (Func<string, string?>)(n => n == "XDG_CURRENT_DESKTOP" ? "gamescope" : null));
            s.Run(600000, step: 1000);
            Assert.Null(Saver(s));
            s.Settings.BigPictureInterface.ScreensaverInGameMode = true;
            s.Run(61000, step: 1000);
            Assert.NotNull(Saver(s));
            s.Pad.B();
            s.Settings.BigPictureInterface.ScreensaverInGameMode = false;
            Set(s, "ScreensaverEnvironment", (Func<string, string?>)(n => n == "XDG_CURRENT_DESKTOP" ? "KDE" : null));
            s.Run(61000, step: 1000);
            Assert.NotNull(Saver(s));
        }, default);

        // Desktop big picture (F10) has it; the desktop's own library does not.
        [Fact]
        public Task F10_big_picture_has_it_and_the_desktop_does_not() => Session.Dispatch(() =>
        {
            using var s = Open(i => i.ScreensaverTimer = 60000, more: a => a.BigScreen = false);
            Assert.False(s.Shown);
            s.Run(600000, step: 1000);
            Assert.Null(Saver(s));
            BigPictureSwitchTests.Press(s, Key.F10);
            Assert.True(s.Shown);
            s.Run(61000, step: 1000);
            Assert.NotNull(Saver(s));
            // The key that wakes it is only that, even F10, which would otherwise leave big picture.
            BigPictureSwitchTests.Press(s, Key.F10);
            Assert.Null(Saver(s));
            Assert.True(s.Shown);
        }, default);

        // A theme whose text scrolls asks for frames; under the screensaver it asks for none and draws none, and once woken it asks again.
        [Fact]
        public Task The_view_beneath_asks_for_no_frame_while_it_shows() => Session.Dispatch(() =>
        {
            const string ticker = "<text name=\"ticker\"><pos>0.55 0.85</pos><size>0.2 0.05</size><fontSize>0.04</fontSize>" +
                                  "<text>A literal line much too long for the small box it has been given in this theme</text>" +
                                  "<container>true</container><containerType>horizontal</containerType><containerStartDelay>2</containerStartDelay></text>";
            using var s = Open(i => i.ScreensaverTimer = 60000, extraGamelist: ticker);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Settle();
            Assert.NotNull(s.WakeAt);
            s.Run(59000, step: 1000);
            while (Saver(s) is null) s.Run(16);
            Assert.Null(s.WakeAt);
            int themed = s.FramesDrawn;
            Assert.Equal(0, s.Loop(30000));
            Assert.Null(s.WakeAt);
            Assert.Equal(themed, s.FramesDrawn);
            s.Pad.B();
            Assert.Null(Saver(s));
            Assert.NotNull(s.WakeAt);
            Assert.True(s.Loop(3000) > 0);
        }, default);

        // A window closed while it shows stops its timer, so nothing is drawn afterwards (§15.14's lesson).
        [Fact]
        public Task Closing_the_window_stops_it() => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; });
            s.Run(61000, step: 1000);
            Assert.NotNull(Saver(s));
            int frames = SaverFrames(s);
            s.Window.Close();
            Assert.Null(SaverWake(s));
            s.Now += TimeSpan.FromSeconds(30);
            typeof(MainWindow).GetMethod("ScreensaverFrame", Hidden)!.Invoke(s.Window, null);
            Assert.Equal(frames, SaverFrames(s));
        }, default);

        // The rows on Theme Settings' Interface tab write appsettings.json.
        [Fact]
        public Task The_Interface_tab_s_rows_store_the_settings() => Session.Dispatch(() =>
        {
            using var s = Open();
            ThemedSwitchesTests.Choose(s, "ScreensaverType", "Slideshow");
            Assert.Equal(BigPictureInterface.SaverSlideshow, AppSettings.Load().BigPictureInterface.ScreensaverType);
            ThemedSwitchesTests.Flip(s, "ScreensaverSlideshowOnlyFavorites");
            Assert.True(AppSettings.Load().BigPictureInterface.ScreensaverSlideshowOnlyFavorites);
            ThemedSwitchesTests.Flip(s, "ScreensaverInGameMode");
            Assert.False(AppSettings.Load().BigPictureInterface.ScreensaverInGameMode);
            ThemeSettingsWindow sheet = ThemedSwitchesTests.OpenInterface(s);
            Slider timer = ThemedSwitchesTests.Named<Slider>(sheet, "ScreensaverTimer");
            Assert.Equal(5, timer.Value);
            timer.Value = 2;
            ThemedSwitchesTests.Named<Slider>(sheet, "ScreensaverSwapImageTimeout").Value = 30;
            s.Pad.B();
            Assert.Equal(120000, AppSettings.Load().BigPictureInterface.ScreensaverTimer);
            Assert.Equal(30000, AppSettings.Load().BigPictureInterface.ScreensaverSwapImageTimeout);
        }, default);

        private static Rect InWindow(ThemedSession s, Rect r)
        {
            Point o = Saver(s)!.TranslatePoint(default, s.Window)!.Value;
            return r.Translate(new Vector(o.X, o.Y));
        }

        // The overlay where ES-DE draws it: its box 17 px in and 16 down at 800 lines, 84 high, #AA000000 over the picture, and white capitals inside.
        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1200)]
        public Task The_overlay_is_drawn_where_ES_DE_draws_it(int w, int h) => Session.Dispatch(() =>
        {
            using var s = Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; i.ScreensaverStretchImages = true; }, w, h);
            s.Run(61000, step: 1000);
            s.Run(500);
            Screensaver saver = Saver(s)!;
            RenderedFrame f = s.Capture();
            double k = h / 800.0;
            Rect box = InWindow(s, saver.OverlayBox);
            _out.WriteLine($"{w}x{h}: {PictureGame(saver)}, box {box}");
            Assert.InRange(box.X, 17 * k - 1, 17 * k + 1);
            Assert.InRange(box.Y, 16 * k - 1, 16 * k + 1);
            Assert.InRange(box.Height, 84 * k - 1, 84 * k + 1);
            (byte pr, byte pg, byte pb) = At(f, (int)(box.Right + 20), (int)(box.Y + 4 * k));
            (byte r, byte g, byte b) = At(f, (int)(box.X + 4 * k), (int)(box.Y + 4 * k));
            Assert.InRange(r, pr * 0.333 - 2, pr * 0.333 + 2);
            Assert.InRange(g, pg * 0.333 - 2, pg * 0.333 + 2);
            Assert.InRange(b, pb * 0.333 - 2, pb * 0.333 + 2);
            int white = 0;
            for (int y = (int)box.Y; y < box.Bottom; y++)
                for (int x = (int)box.X; x < box.Right; x++)
                    if (At(f, x, y) is (> 240, > 240, > 240)) white++;
            Assert.True(white > 100 * k * k, $"{white} white pixels in the box");
            (byte r2, byte g2, byte b2) = At(f, (int)(box.Right + 3), (int)(box.Y + 4 * k));
            Assert.Equal((pr, pg, pb), (r2, g2, b2));
        }, default);
    }
}

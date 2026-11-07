using EmuSen.Crystal.Systems.N64;
using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Crystal;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Mistress.Pages;

namespace EmuSen.WiseMan.Serenity
{
    // The screen's shape a core reports, drawn by default, and square pixels as before when asked for - see EmuSen_Serenity.md §2.9.
    [Collection(TestCollections.ProcessGlobals)]
    public class DisplayAspectTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DisplayAspectTests).GetTypeInfo().Assembly);

        [Fact]
        public void A_shape_given_fits_the_bounds_whatever_the_frame_s_pixel_count_and_none_given_is_the_frame_s()
        {
            Assert.Equal((240.0, 0.0, 1440.0, 1080.0), GameFrameControl.ComputeLetterboxRect(640, 576, 4.0 / 3.0, 1920, 1080));
            Assert.Equal((240.0, 0.0, 1440.0, 1080.0), GameFrameControl.ComputeLetterboxRect(640, 480, 4.0 / 3.0, 1920, 1080));
            Assert.Equal((240.0, 0.0, 1440.0, 1080.0), GameFrameControl.ComputeLetterboxRect(256, 224, 4.0 / 3.0, 1920, 1080));
            Assert.Equal(GameFrameControl.ComputeLetterboxRect(640, 576, 1920, 1080), GameFrameControl.ComputeLetterboxRect(640, 576, 0, 1920, 1080));
            Assert.Equal((0.0, 0.0, 0.0, 0.0), GameFrameControl.ComputeLetterboxRect(640, 576, 4.0 / 3.0, 0, 1080));
        }

        [Fact]
        public void Each_console_reports_its_screen_s_shape()
        {
            Assert.Equal(DisplayShape.Television, AspectOf(Fixtures.SyntheticNesRom.WriteTemp(Fixtures.SyntheticNesRom.Build())), 9);
            Assert.Equal(DisplayShape.Television, AspectOf(SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build())), 9);
            Assert.Equal(10.0 / 9.0, AspectOf(SyntheticGbRom.WriteTemp(SyntheticGbRom.Build())), 9);
            Assert.Equal(DisplayShape.Television, CrtFilter.Filter.Aspect);
        }

        // A game's shape as the session's first frame states it, read through the frontend API as a window reads it; the file is removed after.
        private static double AspectOf(string rom)
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenAspect_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = dir;
            DataStore.OverrideDirectory = Path.Combine(dir, "Home");
            try
            {
                using var runtime = new CrystalRuntime();
                using CrystalSession session = runtime.Open(new CrystalOpen { Path = rom, NoBattery = true });
                var info = new CrystalFrameInfo();
                session.Frame(ref info);
                return info.display_aspect;
            }
            finally
            {
                DataStore.OverrideDirectory = null;
                ConfigStore.OverrideDirectory = null;
                File.Delete(rom);
                try { Directory.Delete(dir, recursive: true); } catch (IOException) { }
            }
        }

        // The Genesis's engine gives its shape in its machine's information only, which the engine falls back to, in either region.
        [Theory]
        [InlineData("md.us")]
        [InlineData("md.eu")]
        public void A_v1_engine_s_shape_falls_back_to_its_machine_s(string region)
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenAspect_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            ConfigStore.OverrideDirectory = dir;
            DataStore.OverrideDirectory = Path.Combine(dir, "Home");
            try
            {
                var config = new ConsolesConfig();
                config.SetValue("Genesis", "region", region);
                config.Save();
                string rom = Path.Combine(dir, "game.md");
                File.WriteAllBytes(rom, SyntheticMdRom.Cartridge(region: "U"));
                using var runtime = new CrystalRuntime();
                using CrystalSession session = runtime.Open(new CrystalOpen { Path = rom, NoBattery = true });
                var info = new CrystalFrameInfo();
                session.Frame(ref info);
                Assert.Equal(4.0 / 3.0, info.display_aspect, 9);
                for (int i = 0; i < 3; i++) session.Frame(ref info);
                Assert.Equal(4.0 / 3.0, info.display_aspect, 9);
            }
            finally
            {
                DataStore.OverrideDirectory = null;
                ConfigStore.OverrideDirectory = null;
                try { Directory.Delete(dir, recursive: true); } catch { }
            }
        }

        private static byte[] White(int w, int h) => Enumerable.Repeat((byte)255, w * h * 4).ToArray();

        // What the control draws in a window: the lit rectangle, and a hash of every pixel.
        private static ((int Left, int Top, int Right, int Bottom) Lit, string Hash) Drawn(int width, int height, int rowRepeat, double aspect, bool square, int windowWidth = 400, int windowHeight = 300)
        {
            var control = new GameFrameControl { SquarePixels = square };
            var window = new Window { Width = windowWidth, Height = windowHeight, Content = control };
            window.Show();
            try
            {
                control.UpdateFrame(White(width, height), width, height, rowRepeat, aspect: aspect);
                using var captured = window.CaptureRenderedFrame()!;
                var capture = UiTest.Capture(captured);
                int left = int.MaxValue, top = int.MaxValue, right = -1, bottom = -1;
                for (int y = 0; y < capture.Height; y++) for (int x = 0; x < capture.Width; x++)
                    if (capture.Rgba[(y * capture.Width + x) * 4 + 1] > 128) { left = Math.Min(left, x); top = Math.Min(top, y); right = Math.Max(right, x); bottom = Math.Max(bottom, y); }
                return ((left, top, right + 1, bottom + 1), Convert.ToHexString(SHA256.HashData(capture.Rgba))[..16]);
            }
            finally
            {
                window.Close();
            }
        }

        [Fact]
        public Task A_pal_and_an_ntsc_n64_frame_fill_the_same_four_by_three_rectangle() => Session.Dispatch(() =>
        {
            var pal = Drawn(640, 576, 1, DisplayShape.Television, false).Lit;
            var ntsc = Drawn(640, 480, 1, DisplayShape.Television, false).Lit;
            var rows = Drawn(640, 240, 2, DisplayShape.Television, false).Lit;
            var low = Drawn(320, 240, 1, DisplayShape.Television, false).Lit;
            Assert.Equal((0, 0, 400, 300), ntsc);
            Assert.Equal(ntsc, pal);
            Assert.Equal(ntsc, rows);
            Assert.Equal(ntsc, low);
        }, default);

        // The requirement of EmuSen_CRT.md §14: within a core, a frame of any size or region lights the same rectangle, here with no filter.
        [Theory]
        [InlineData("NES", 4.0 / 3.0, 0, 400)]
        [InlineData("SNES", 4.0 / 3.0, 0, 400)]
        [InlineData("Genesis", 4.0 / 3.0, 0, 400)]
        [InlineData("N64", 4.0 / 3.0, 0, 400)]
        [InlineData("GB", 10.0 / 9.0, 33, 366)]
        public Task With_no_filter_every_frame_a_console_hands_over_lights_the_same_rectangle(string console, double shape, int left, int right) => Session.Dispatch(() =>
        {
            foreach (var (width, rows, repeat) in ScreenFilterRenderTests.FramesOf[console])
                Assert.Equal((left, 0, right, 300), Drawn(width, rows, repeat, shape, false).Lit);
        }, default);

        [Fact]
        public Task Square_pixels_draw_as_before_whatever_shape_is_reported() => Session.Dispatch(() =>
        {
            var before = Drawn(640, 576, 1, 0, false);
            var square = Drawn(640, 576, 1, DisplayShape.Television, true);
            Assert.Equal(before.Hash, square.Hash);
            Assert.InRange(square.Lit.Right - square.Lit.Left, 332, 334);
            Assert.Equal((0, 300), (square.Lit.Top, square.Lit.Bottom));
            var unknown = Drawn(256, 224, 1, 0, false);
            Assert.Equal(Drawn(256, 224, 1, 0, true).Hash, unknown.Hash);
        }, default);

        [Fact]
        public Task Settings_offer_the_picture_shape_on_every_console_s_page_tv_shape_first_and_store_its_value() => Session.Dispatch(() =>
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenAspectWindow_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            ConfigStore.OverrideDirectory = dir;
            DataStore.OverrideDirectory = Path.Combine(dir, "Home");
            try
            {
                MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:N64");
                Assert.Equal("TV shape", SettingsDesk.Value(window, "consoleshape"));
                Assert.Equal(new[] { "TV shape", "Square pixels" }, SettingsDesk.Row(window, "consoleshape").Choices);
                SettingsDesk.Pick(window, "consoleshape", "Square pixels");
                Assert.Equal(EmuSen.Mistress.Library.GraphicsKeys.SquarePixels, GraphicsConfig.Load().Value("N64", EmuSen.Mistress.Library.GraphicsKeys.PictureShape));
                foreach (var console in EmuSen.Mistress.Library.RuntimeCatalog.Systems.Where(s => s.Engines.Count > 0).Select(s => s.Console).Distinct())
                {
                    SettingsDesk.At(window, SettingsScreen.Consoles, "console:" + console);
                    Assert.Contains("consoleshape", SettingsDesk.Keys(window));
                }
                window.Close();
            }
            finally
            {
                ConfigStore.OverrideDirectory = null;
                DataStore.OverrideDirectory = null;
                try { Directory.Delete(dir, recursive: true); } catch { }
            }
        }, default);

        // A filter that states a shape keeps it under square pixels, so the CRT's tube does not change with the setting.
        [Fact]
        public Task A_filter_s_own_shape_holds_under_either_setting() => Session.Dispatch(() =>
        {
            if (ChainDevice.Missing) return;
            var tube = new ScreenFilter("tube", new[] { new FilterPass("vec4 filter_main(vec2 coord) { return vec4(1.0, 1.0, 1.0, 1.0); }", PassScale.Viewport) }, null, "") { Aspect = DisplayShape.Television };
            foreach (bool square in new[] { false, true })
            {
                var control = new GameFrameControl { SquarePixels = square, ActiveFilter = tube };
                control.WaitForFilter();
                var window = new Window { Width = 400, Height = 300, Content = control };
                window.Show();
                control.UpdateFrame(new byte[640 * 576 * 4], 640, 576, aspect: DisplayShape.Television);
                using var captured = window.CaptureRenderedFrame()!;
                var capture = UiTest.Capture(captured);
                Assert.Equal(255, capture.Rgba[(150 * capture.Width + 5) * 4 + 1]);
                window.Close();
            }
        }, default);
    }
}

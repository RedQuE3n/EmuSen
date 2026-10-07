using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using Avalonia.Threading;
using Avalonia.VisualTree;

namespace EmuSen.WiseMan.Serenity
{
    // The screen's shape a core reports, drawn by default, and square pixels as before when asked for - see EmuSen_Serenity.md §2.9.
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
            Assert.Equal(DisplayShape.Television, new MoonCore().DisplayAspect);
            Assert.Equal(DisplayShape.Television, new VenusCore().DisplayAspect);
            Assert.Equal(DisplayShape.Television, new MarsCore().DisplayAspect);
            Assert.Equal(DisplayShape.GameBoy, new MercuryCore().DisplayAspect);
            Assert.Equal(DisplayShape.Television, CrtFilter.Filter.Aspect);
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
            CoreDiscovery.UseDevelopment(false);
            try
            {
                var config = new GraphicsConfig();
                config.SetValue("Genesis", "region", region);
                config.Save();
                string rom = Path.Combine(dir, "game.md");
                File.WriteAllBytes(rom, SyntheticMdRom.Cartridge(region: "U"));
                var core = Assert.IsType<CoreEngine>(CoreFactory.Load(rom).Core);
                Assert.Equal(4.0 / 3.0, core.DisplayAspect, 9);
                for (int i = 0; i < 3; i++) core.RunFrame();
                Assert.Equal(4.0 / 3.0, core.DisplayAspect, 9);
                core.Dispose();
            }
            finally
            {
                CoreDiscovery.UseDevelopment(null);
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
        public Task Graphics_settings_offer_the_picture_shape_on_every_tab_tv_shape_first_and_store_its_value() => Session.Dispatch(() =>
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenAspectWindow_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            ConfigStore.OverrideDirectory = dir;
            try
            {
                var config = new GraphicsConfig();
                string? changed = null;
                var window = new GraphicsSettingsWindow(config, c => changed = c, "N64");
                window.Show();
                window.CaptureRenderedFrame();
                Dropdown shape = window.GetVisualDescendants().OfType<Dropdown>().First(d => d.Name == "N64.PictureShape");
                Assert.Equal("TV shape", shape.SelectedItem);
                Assert.Equal(new[] { "TV shape", "Square pixels" }, shape.Items.Cast<object>().Select(o => o.ToString()));
                shape.SelectedItem = "Square pixels";
                Dispatcher.UIThread.RunJobs();
                Assert.Equal((GraphicsSettingsWindow.SquarePixels, "N64"), (config.Value("N64", GraphicsSettingsWindow.PictureShapeKey), changed));
                foreach (var console in CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console))
                    Assert.Contains(Avalonia.LogicalTree.LogicalExtensions.GetLogicalDescendants(window).OfType<Control>(), c => c.Name == $"{console}.PictureShape");
                window.Close();
            }
            finally
            {
                ConfigStore.OverrideDirectory = null;
                try { Directory.Delete(dir, recursive: true); } catch { }
            }
        }, default);

        // A filter that states a shape keeps it under square pixels, so the CRT's tube does not change with the setting.
        [Fact]
        public Task A_filter_s_own_shape_holds_under_either_setting() => Session.Dispatch(() =>
        {
            var tube = new ScreenFilter("tube", new[] { new FilterPass("half4 main(float2 coord) { return half4(1.0, 1.0, 1.0, 1.0); }", PassScale.Viewport) }, null, "") { Aspect = DisplayShape.Television };
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

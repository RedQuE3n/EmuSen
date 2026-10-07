using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Serenity
{
    // What is hidden at the picture's edges, per console: its arithmetic, the picture drawn with it, and the setting - see EmuSen_Serenity.md §2.10.
    public class PictureCropTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PictureCropTests).GetTypeInfo().Assembly);

        [Fact]
        public void A_television_s_overscan_hides_the_same_share_at_every_edge()
        {
            PictureCrop tv = PictureCrop.Television;
            Assert.Equal(0.032710, tv.Left, 6);
            Assert.Equal((tv.Left, tv.Left, tv.Left), (tv.Top, tv.Right, tv.Bottom));
            Assert.Equal(0.934579, tv.Width, 6);
            Assert.Equal(0.934579, tv.Height, 6);

            // In a console's own pixels and lines: the N64's 640 by 240, its PAL field's 288, and the NES's 256 by 240.
            Assert.Equal(20.9, 640 * tv.Left, 1);
            Assert.Equal(7.9, 240 * tv.Top, 1);
            Assert.Equal(9.4, 288 * tv.Top, 1);
            Assert.Equal(8.4, 256 * tv.Left, 1);
        }

        [Fact]
        public void A_crop_is_a_share_of_the_picture_held_to_a_quarter_at_each_edge()
        {
            Assert.Equal(new PictureCrop(0.024, 0.087, 0.012, 0.09), PictureCrop.Percent(2.4, 8.7, 1.2, 9));
            Assert.Equal(new PictureCrop(0.25, 0, 0.25, 0), PictureCrop.Percent(60, -3, 25, double.NaN));
            Assert.Equal(0.5, PictureCrop.Percent(25, 25, 25, 25).Width);
            Assert.Equal(new PictureCrop(0.45, 0.45, 0.45, 0.45), PictureCrop.Enlarged(10));
            Assert.Equal(PictureCrop.None, PictureCrop.Enlarged(0.5));
        }

        [Fact]
        public void Nothing_hidden_and_a_crop_no_picture_could_have_are_both_none()
        {
            Assert.True(PictureCrop.None.IsNone);
            Assert.True(new PictureCrop(-0.1, 0, 0, 0).IsNone);
            Assert.True(new PictureCrop(0.5, 0, 0.5, 0).IsNone);
            Assert.True(new PictureCrop(double.NaN, 0, 0, 0).IsNone);
            Assert.False(new PictureCrop(0, 0, 0, 0.01).IsNone);
            Assert.Equal((1.0, 1.0), (PictureCrop.None.Width, PictureCrop.None.Height));
        }

        [Fact]
        public void The_whole_frame_is_placed_so_that_the_part_kept_fills_the_rectangle_shown()
        {
            Assert.Equal((107.0, 0.0, 1067.0, 800.0), PictureCrop.None.Whole(107, 0, 1067, 800));

            // A quarter hidden left and right, an eighth above and none below: 1067 is half of 2134 and 800 seven eighths of 914.
            var (x, y, w, h) = new PictureCrop(0.25, 0.125, 0.25, 0).Whole(107, 0, 1067, 800);
            Assert.Equal((2134.0, 914.0), (w, h));
            Assert.Equal((107.0 - 534, -114.0), (x, y));
        }

        // A frame white over one rectangle of it and black elsewhere, or the other way about.
        internal static byte[] Marked(int width, int height, PictureCrop kept, bool whiteInside)
        {
            var frame = new byte[width * height * 4];
            for (int y = 0; y < height; y++) for (int x = 0; x < width; x++)
            {
                bool inside = x >= kept.Left * width && x < (1 - kept.Right) * width && y >= kept.Top * height && y < (1 - kept.Bottom) * height;
                byte v = inside == whiteInside ? (byte)255 : (byte)0;
                int i = (y * width + x) * 4;
                (frame[i], frame[i + 1], frame[i + 2], frame[i + 3]) = (v, v, v, 255);
            }
            return frame;
        }

        // What the control draws in a 400 by 300 window: the rectangle lit above half, the share of the window lit, and a hash.
        private static ((int Left, int Top, int Right, int Bottom) Lit, double Share, string Hash) Drawn(byte[] frame, int width, int height, int rowRepeat, Action<GameFrameControl> set, int windowWidth = 400)
        {
            var control = new GameFrameControl();
            set(control);
            control.WaitForFilter();
            var window = new Window { Width = windowWidth, Height = 300, Content = control };
            window.Show();
            try
            {
                control.UpdateFrame(frame, width, height, rowRepeat, aspect: DisplayShape.Television);
                using var captured = window.CaptureRenderedFrame()!;
                var capture = UiTest.Capture(captured);
                int left = int.MaxValue, top = int.MaxValue, right = -1, bottom = -1, lit = 0;
                for (int y = 0; y < capture.Height; y++) for (int x = 0; x < capture.Width; x++)
                {
                    if (capture.Rgba[(y * capture.Width + x) * 4 + 1] <= 128) continue;
                    (left, top, right, bottom) = (Math.Min(left, x), Math.Min(top, y), Math.Max(right, x), Math.Max(bottom, y));
                    lit++;
                }
                return ((left, top, right + 1, bottom + 1), lit / (double)(capture.Width * capture.Height), Convert.ToHexString(SHA256.HashData(capture.Rgba))[..16]);
            }
            finally
            {
                window.Close();
            }
        }

        // 10% hidden at the left, 5% at the right, 12.5% above and 25% below: whole pixels of every frame drawn here.
        private static readonly PictureCrop Uneven = new(0.1, 0.125, 0.05, 0.25);

        // The N64's frames of §14.2 of EmuSen_CRT.md: an NTSC and a PAL field shown twice, and both as two fields.
        public static readonly (int Width, int Rows, int Repeat)[] N64Frames = { (640, 240, 2), (640, 288, 2), (640, 480, 1), (640, 576, 1) };

        [Fact]
        public Task A_cropped_pal_and_ntsc_n64_frame_show_the_same_part_filling_the_same_rectangle() => Session.Dispatch(() =>
        {
            foreach (var (width, rows, repeat) in N64Frames)
            {
                var kept = Drawn(Marked(width, rows, Uneven, whiteInside: true), width, rows, repeat, c => c.Crop = Uneven);
                Assert.Equal((0, 0, 400, 300), kept.Lit);
                Assert.Equal(1.0, kept.Share, 3);
                var hidden = Drawn(Marked(width, rows, Uneven, whiteInside: false), width, rows, repeat, c => c.Crop = Uneven);
                Assert.True(hidden.Share < 0.012, $"{width}x{rows}: {hidden.Share:P1} of the window shows what was to be hidden");

                // Uncropped, the same frame lights only the part marked: 85% of the width from 10% in, 62.5% of the height from 12.5% down, which is row 37.5.
                Assert.Equal((40, 38, 380, 225), Drawn(Marked(width, rows, Uneven, whiteInside: true), width, rows, repeat, _ => { }).Lit);
            }
        }, default);

        // In a window wider than the picture, the frame enlarged for the crop is cut at the picture's rectangle and leaves the bars beside it black.
        [Fact]
        public Task What_is_hidden_is_not_drawn_in_the_bars_beside_the_picture() => Session.Dispatch(() =>
        {
            byte[] white = Marked(640, 480, PictureCrop.None, whiteInside: true);
            Assert.Equal((100, 0, 500, 300), Drawn(white, 640, 480, 1, _ => { }, windowWidth: 600).Lit);
            Assert.Equal((100, 0, 500, 300), Drawn(white, 640, 480, 1, c => c.Crop = Uneven, windowWidth: 600).Lit);
        }, default);

        [Fact]
        public Task Under_square_pixels_the_part_kept_decides_the_shape() => Session.Dispatch(() =>
        {
            // A quarter hidden at each side leaves 320 by 480, two to three: 200 by 300 in the window.
            var sides = new PictureCrop(0.25, 0, 0.25, 0);
            var drawn = Drawn(Marked(640, 480, sides, whiteInside: true), 640, 480, 1, c => { c.Crop = sides; c.SquarePixels = true; });
            Assert.Equal((100, 0, 300, 300), drawn.Lit);
        }, default);

        [Fact]
        public Task A_simple_effect_and_a_filter_of_passes_are_cropped_as_the_plain_picture_is() => Session.Dispatch(() =>
        {
            const string through = "uniform shader source; uniform float2 inputSize; uniform float2 outputSize; half4 main(float2 coord) { return source.eval(coord * inputSize / outputSize); }";
            var passes = new ScreenFilter("through", new[] { new FilterPass(through, PassScale.Source), new FilterPass(through, PassScale.Viewport, LinearSource: true) }, null, "");
            byte[] frame = Marked(640, 480, Uneven, whiteInside: true);
            Assert.Equal((0, 0, 400, 300), Drawn(frame, 640, 480, 1, c => { c.Crop = Uneven; c.ActiveFilter = passes; }).Lit);
            Assert.Equal((40, 38, 380, 225), Drawn(frame, 640, 480, 1, c => c.ActiveFilter = passes).Lit);
            var scanlines = Drawn(frame, 640, 480, 1, c => { c.Crop = Uneven; c.ActiveEffect = ShaderEffect.Scanlines; });
            Assert.Equal((0, 400), (scanlines.Lit.Left, scanlines.Lit.Right));
            Assert.True(scanlines.Lit.Top <= 1 && scanlines.Lit.Bottom >= 299, $"scanlines lit rows {scanlines.Lit.Top} to {scanlines.Lit.Bottom}");
        }, default);

        [Fact]
        public Task With_nothing_hidden_the_picture_is_the_one_drawn_before_there_was_a_crop() => Session.Dispatch(() =>
        {
            byte[] frame = Marked(640, 576, Uneven, whiteInside: true);
            string before = Drawn(frame, 640, 576, 1, _ => { }).Hash;
            Assert.Equal(before, Drawn(frame, 640, 576, 1, c => c.Crop = PictureCrop.None).Hash);
            Assert.Equal(before, Drawn(frame, 640, 576, 1, c => c.Crop = new PictureCrop(0.6, 0, 0.6, 0)).Hash);
            Assert.NotEqual(before, Drawn(frame, 640, 576, 1, c => c.Crop = Uneven).Hash);
        }, default);

        private static T Named<T>(Window window, string name) where T : Control =>
            Avalonia.LogicalTree.LogicalExtensions.GetLogicalDescendants(window).OfType<T>().First(c => c.Name == name);

        [Fact]
        public Task Graphics_settings_offer_overscan_on_every_console_that_had_a_television_and_store_it_per_console() => Session.Dispatch(() =>
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenCropWindow_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            ConfigStore.OverrideDirectory = dir;
            try
            {
                var config = new GraphicsConfig();
                string? changed = null;
                var window = new GraphicsSettingsWindow(config, c => changed = c, "N64");
                window.Show();
                window.CaptureRenderedFrame();

                foreach (string console in new[] { "NES", "SNES", "N64", "Genesis" })
                {
                    Assert.True(GraphicsSettingsWindow.HasOverscan(console));
                    Assert.Equal(PictureCrop.None, GraphicsSettingsWindow.CropFor(config, console));
                }
                Assert.False(GraphicsSettingsWindow.HasOverscan("GB"));
                Assert.DoesNotContain(Avalonia.LogicalTree.LogicalExtensions.GetLogicalDescendants(window).OfType<Control>(), c => c.Name is "GB.Overscan" or "GB.Crop");

                Dropdown overscan = Named<Dropdown>(window, "N64.Overscan");
                Assert.Equal("Whole picture", overscan.SelectedItem);
                Assert.Equal(new[] { "Whole picture", "TV overscan", "Custom crop" }, overscan.Items.Cast<object>().Select(o => o.ToString()));
                FieldRow row = Named<FieldRow>(window, "N64.Crop");
                Assert.False(row.IsVisible);

                overscan.SelectedItem = "TV overscan";
                Dispatcher.UIThread.RunJobs();
                Assert.Equal((GraphicsSettingsWindow.TvOverscan, "N64"), (config.Value("N64", GraphicsSettingsWindow.OverscanKey), changed));
                Assert.Equal(PictureCrop.Television, GraphicsSettingsWindow.CropFor(config, "N64"));
                Assert.False(row.IsVisible);

                overscan.SelectedItem = "Custom crop";
                Dispatcher.UIThread.RunJobs();
                Assert.True(row.IsVisible);
                Assert.Equal(PictureCrop.None, GraphicsSettingsWindow.CropFor(config, "N64"));
                Named<TextBox>(window, "N64.CropLeft").Text = "2.4";
                Named<TextBox>(window, "N64.CropRight").Text = "1,2";
                Named<TextBox>(window, "N64.CropTop").Text = "8.68";
                Named<TextBox>(window, "N64.CropBottom").Text = "99";
                Dispatcher.UIThread.RunJobs();
                Assert.Equal(("2.4", "1.2", "8.7", "25"), (config.Value("N64", "CropLeft"), config.Value("N64", "CropRight"), config.Value("N64", "CropTop"), config.Value("N64", "CropBottom")));
                Assert.Equal(new PictureCrop(0.024, 0.087, 0.012, 0.25), GraphicsSettingsWindow.CropFor(config, "N64"));

                // What is no number leaves the stored value alone.
                Named<TextBox>(window, "N64.CropLeft").Text = "wide";
                Dispatcher.UIThread.RunJobs();
                Assert.Equal("2.4", config.Value("N64", "CropLeft"));

                // Another console's is its own, and the saved file holds both.
                Assert.Equal(PictureCrop.None, GraphicsSettingsWindow.CropFor(config, "SNES"));
                Named<Dropdown>(window, "SNES.Overscan").SelectedItem = "TV overscan";
                Dispatcher.UIThread.RunJobs();
                GraphicsConfig saved = GraphicsConfig.Load();
                Assert.Equal(PictureCrop.Television, GraphicsSettingsWindow.CropFor(saved, "SNES"));
                Assert.Equal(new PictureCrop(0.024, 0.087, 0.012, 0.25), GraphicsSettingsWindow.CropFor(saved, "N64"));

                // The values are kept while another choice is made, and Reset This Console clears them all.
                overscan.SelectedItem = "Whole picture";
                Dispatcher.UIThread.RunJobs();
                Assert.Equal(PictureCrop.None, GraphicsSettingsWindow.CropFor(config, "N64"));
                Assert.Equal("8.7", config.Value("N64", "CropTop"));
                window.GetVisualDescendants().OfType<Button>().First(b => b.Content as string == "Reset This Console").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
                Dispatcher.UIThread.RunJobs();
                Assert.Null(config.Value("N64", "CropTop"));
                Assert.Equal("Whole picture", overscan.SelectedItem);
                Assert.Equal(PictureCrop.Television, GraphicsSettingsWindow.CropFor(config, "SNES"));
                window.Close();
            }
            finally
            {
                ConfigStore.OverrideDirectory = null;
                try { Directory.Delete(dir, recursive: true); } catch { }
            }
        }, default);
    }
}

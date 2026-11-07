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
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Mistress.Pages;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Serenity
{
    // What is hidden at the picture's edges, per console: its arithmetic, the picture drawn with it, and the setting - see EmuSen_Serenity.md §2.10.
    [Collection(TestCollections.ProcessGlobals)]
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
            if (ChainDevice.Missing) return;
            const string through = "uniform sampler2D source; uniform vec2 inputSize; uniform vec2 outputSize; vec4 filter_main(vec2 coord) { return source_eval(coord * inputSize / outputSize); }";
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

        [Fact]
        public Task Settings_offer_overscan_on_every_console_that_had_a_television_and_store_it_per_console() => Session.Dispatch(() =>
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenCropWindow_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            ConfigStore.OverrideDirectory = dir;
            DataStore.OverrideDirectory = Path.Combine(dir, "Home");
            try
            {
                static GraphicsConfig Saved() => GraphicsConfig.Load();
                foreach (string console in new[] { "NES", "SNES", "N64", "Genesis" })
                {
                    Assert.True(EmuSen.Mistress.Library.ConsoleCrop.HasOverscan(console));
                    Assert.Equal(PictureCrop.None, EmuSen.Mistress.Library.ConsoleCrop.CropFor(Saved(), console));
                }
                Assert.False(EmuSen.Mistress.Library.ConsoleCrop.HasOverscan("GB"));
                MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:GB");
                Assert.DoesNotContain("consolecrop", SettingsDesk.Keys(window));

                SettingsDesk.At(window, SettingsScreen.Consoles, "console:N64");
                Assert.Equal("Whole picture", SettingsDesk.Value(window, "consolecrop"));
                Assert.Equal(new[] { "Whole picture", "TV overscan", "Custom crop" }, SettingsDesk.Row(window, "consolecrop").Choices);
                Assert.DoesNotContain("consolecropedges", SettingsDesk.Keys(window));

                SettingsDesk.Pick(window, "consolecrop", "TV overscan");
                Assert.Equal(EmuSen.Mistress.Library.GraphicsKeys.TvOverscan, Saved().Value("N64", EmuSen.Mistress.Library.GraphicsKeys.Overscan));
                Assert.Equal(PictureCrop.Television, EmuSen.Mistress.Library.ConsoleCrop.CropFor(Saved(), "N64"));
                Assert.DoesNotContain("consolecropedges", SettingsDesk.Keys(window));

                SettingsDesk.Pick(window, "consolecrop", "Custom crop");
                Assert.Contains("consolecropedges", SettingsDesk.Keys(window));
                Assert.Equal(PictureCrop.None, EmuSen.Mistress.Library.ConsoleCrop.CropFor(Saved(), "N64"));
                SettingsDesk.Choose(window, "consolecropedges");
                foreach ((string key, string typed) in new[] { ("CropLeft", "2.4"), ("CropRight", "1,2"), ("CropTop", "8.68"), ("CropBottom", "99") })
                {
                    SettingsDesk.Choose(window, "crop:" + key);
                    SettingsDesk.Answer(window, typed);
                }
                GraphicsConfig now = Saved();
                Assert.Equal(("2.4", "1.2", "8.7", "25"), (now.Value("N64", "CropLeft"), now.Value("N64", "CropRight"), now.Value("N64", "CropTop"), now.Value("N64", "CropBottom")));
                Assert.Equal(new PictureCrop(0.024, 0.087, 0.012, 0.25), EmuSen.Mistress.Library.ConsoleCrop.CropFor(now, "N64"));
                Assert.Equal("8.7%", SettingsDesk.Value(window, "crop:CropTop"));

                // What is no number leaves the stored value alone; a step moves a tenth.
                SettingsDesk.Choose(window, "crop:CropLeft");
                SettingsDesk.Answer(window, "wide");
                Assert.Equal("2.4", Saved().Value("N64", "CropLeft"));
                SettingsDesk.Step(window, "crop:CropLeft", 1);
                Assert.Equal("2.5", Saved().Value("N64", "CropLeft"));
                SettingsDesk.Step(window, "crop:CropLeft", -1);
                Assert.Equal("2.4", Saved().Value("N64", "CropLeft"));

                // Another console's is its own, and the saved file holds both.
                Assert.Equal(PictureCrop.None, EmuSen.Mistress.Library.ConsoleCrop.CropFor(Saved(), "SNES"));
                SettingsDesk.At(window, SettingsScreen.Consoles, "console:SNES");
                SettingsDesk.Pick(window, "consolecrop", "TV overscan");
                Assert.Equal(PictureCrop.Television, EmuSen.Mistress.Library.ConsoleCrop.CropFor(Saved(), "SNES"));
                Assert.Equal(new PictureCrop(0.024, 0.087, 0.012, 0.25), EmuSen.Mistress.Library.ConsoleCrop.CropFor(Saved(), "N64"));

                // The values are kept while another choice is made, and Reset this console clears them all.
                SettingsDesk.At(window, SettingsScreen.Consoles, "console:N64");
                SettingsDesk.Pick(window, "consolecrop", "Whole picture");
                Assert.Equal(PictureCrop.None, EmuSen.Mistress.Library.ConsoleCrop.CropFor(Saved(), "N64"));
                Assert.Equal("8.7", Saved().Value("N64", "CropTop"));
                SettingsDesk.Choose(window, "reset");
                Assert.Null(Saved().Value("N64", "CropTop"));
                Assert.Equal("Whole picture", SettingsDesk.Value(window, "consolecrop"));
                Assert.Equal(PictureCrop.Television, EmuSen.Mistress.Library.ConsoleCrop.CropFor(Saved(), "SNES"));
                window.Close();
            }
            finally
            {
                ConfigStore.OverrideDirectory = null;
                DataStore.OverrideDirectory = null;
                try { Directory.Delete(dir, recursive: true); } catch { }
            }
        }, default);
    }
}

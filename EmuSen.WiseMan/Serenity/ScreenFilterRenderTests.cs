using System;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media.Imaging;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // The accurate filters drawn through GameFrameControl's real render pass, and their pixels checked against the measured constants - see EmuSen_Serenity.md §3.4 and §3.5.
    public class ScreenFilterRenderTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScreenFilterRenderTests).GetTypeInfo().Assembly);

        private sealed class Picture
        {
            public required byte[] Rgba;
            public required int Width;
            public (int R, int G, int B) At(int x, int y) { int i = (y * Width + x) * 4; return (Rgba[i], Rgba[i + 1], Rgba[i + 2]); }
            public double Luma(int x, int y) { var (r, g, b) = At(x, y); return 0.299 * r + 0.587 * g + 0.114 * b; }
        }

        private static byte[] Solid(int width, int height, byte r, byte g, byte b)
        {
            var frame = new byte[width * height * 4];
            for (int i = 0; i < frame.Length; i += 4) (frame[i], frame[i + 1], frame[i + 2], frame[i + 3]) = (r, g, b, 255);
            return frame;
        }

        // Frames shown in order, each drawn once, and the last one captured.
        private static Picture Show(ScreenFilter filter, int width, int height, int windowWidth, int windowHeight, params byte[][] frames)
        {
            var control = new GameFrameControl { ActiveFilter = filter };
            var window = new Window { Width = windowWidth, Height = windowHeight, Content = control };
            window.Show();
            try
            {
                WriteableBitmap? captured = null;
                foreach (byte[] frame in frames)
                {
                    control.UpdateFrame(frame, width, height);
                    captured?.Dispose();
                    captured = window.CaptureRenderedFrame()!;
                }
                var capture = EmuSen.WiseMan.Fixtures.UiTest.Capture(captured!);
                captured!.Dispose();
                return new Picture { Rgba = capture.Rgba, Width = capture.Width };
            }
            finally
            {
                window.Close();
            }
        }

        private static void Near(int expected, int actual, int tolerance, string what) =>
            Assert.True(Math.Abs(expected - actual) <= tolerance, $"{what}: expected {expected} within {tolerance}, got {actual}");

        private static void NearRgb(int rgb, (int R, int G, int B) actual, int tolerance, string what)
        {
            Near((rgb >> 16) & 0xFF, actual.R, tolerance, what + " red");
            Near((rgb >> 8) & 0xFF, actual.G, tolerance, what + " green");
            Near(rgb & 0xFF, actual.B, tolerance, what + " blue");
        }

        [Fact]
        public void Every_built_in_filter_compiles()
        {
            foreach (ScreenFilterChoice choice in ScreenFilters.All.Where(c => c.Filter is not null))
            {
                using var chain = (IDisposable)Activator.CreateInstance(typeof(GameFrameControl).Assembly.GetType("EmuSen.Serenity.Shaders.FilterChain")!, choice.Filter)!;
            }
        }

        [Fact]
        public void Each_console_is_offered_the_filters_that_suit_its_screen()
        {
            Assert.Contains(CrtFilters.Lottes.Name, ScreenFilters.NamesFor("SNES"));
            Assert.Contains(CrtFilters.Lottes.Name, ScreenFilters.NamesFor("N64"));
            Assert.DoesNotContain(HandheldFilters.DmgLcd.Name, ScreenFilters.NamesFor("SNES"));
            Assert.Contains(HandheldFilters.DmgLcd.Name, ScreenFilters.NamesFor("GB"));
            Assert.Contains(HandheldFilters.GbcLcd.Name, ScreenFilters.NamesFor("GB"));
            Assert.DoesNotContain(CrtFilters.Lottes.Name, ScreenFilters.NamesFor("GB"));
            Assert.DoesNotContain(HandheldFilters.AgbLcd.Name, ScreenFilters.NamesFor("GB"));
            Assert.Contains(HandheldFilters.AgbLcd.Name, ScreenFilters.NamesFor("GBA"));
            Assert.Equal(ScreenFilters.None, ScreenFilters.NamesFor("GB")[0]);
        }

        // Mercury's four greys at one pixel each become SameBoy's four DMG shades; at this size there is no grid to draw.
        [Theory]
        [InlineData(0xFF, HandheldFilters.DmgLightest)]
        [InlineData(0xAA, HandheldFilters.DmgLight)]
        [InlineData(0x55, HandheldFilters.DmgDark)]
        [InlineData(0x00, HandheldFilters.DmgDarkest)]
        public Task A_dmg_grey_becomes_the_panel_s_measured_shade(byte grey, int shade) => Session.Dispatch(() =>
        {
            Picture picture = Show(HandheldFilters.DmgLcd, 8, 8, 8, 8, Solid(8, 8, grey, grey, grey));
            NearRgb(shade, picture.At(4, 4), 2, $"grey {grey:X2}");
        }, default);

        // The DMG's panel is slow: one black frame after white ones is not yet black, and several are.
        [Fact]
        public Task A_dmg_pixel_turned_dark_takes_frames_to_get_there() => Session.Dispatch(() =>
        {
            byte[] white = Solid(8, 8, 0xFF, 0xFF, 0xFF), black = Solid(8, 8, 0, 0, 0);
            Picture once = Show(HandheldFilters.DmgLcd, 8, 8, 8, 8, white, white, white, white, black);
            Picture settled = Show(HandheldFilters.DmgLcd, 8, 8, 8, 8, white, black, black, black, black);

            Assert.True(once.Luma(4, 4) > settled.Luma(4, 4) + 20, $"one frame after white {once.Luma(4, 4)}, settled {settled.Luma(4, 4)}");
            NearRgb(HandheldFilters.DmgDarkest, settled.At(4, 4), 2, "settled");
        }, default);

        // Magnified, a dot's centre is its shade and the gap beside it the lighter reflector, and a dark dot's shadow falls below and to the right.
        [Fact]
        public Task A_magnified_dmg_screen_shows_dots_gaps_and_the_shadow_they_cast() => Session.Dispatch(() =>
        {
            byte[] frame = Solid(4, 4, 0xFF, 0xFF, 0xFF);
            int i = (1 * 4 + 1) * 4;
            (frame[i], frame[i + 1], frame[i + 2]) = (0, 0, 0);
            Picture picture = Show(HandheldFilters.DmgLcd, 4, 4, 64, 64, frame, frame, frame, frame);

            NearRgb(HandheldFilters.DmgDarkest, picture.At(24, 24), 4, "the dark dot's centre");
            NearRgb(HandheldFilters.DmgLightest, picture.At(40, 8), 4, "a light dot's centre");
            Assert.True(picture.Luma(32, 8) > picture.Luma(40, 8), "the gap between two light dots is the lighter reflector");
            Assert.True(picture.Luma(40, 38) < picture.Luma(8, 56) - 8, $"shadow {picture.Luma(40, 38)} against open reflector {picture.Luma(8, 56)}");
        }, default);

        // Pokefan531's matrix in linear light: pure red on a GBC panel is a washed-out pink-red.
        [Fact]
        public Task Gbc_red_is_the_measured_panel_s_red() => Session.Dispatch(() =>
        {
            Picture picture = Show(HandheldFilters.GbcLcd, 8, 8, 8, 8, Solid(8, 8, 0xFF, 0, 0));
            double r = Math.Pow(0.905 * 0.91, 1 / 2.2) * 255, g = Math.Pow(0.10 * 0.91, 1 / 2.2) * 255, b = Math.Pow(0.1575 * 0.91, 1 / 2.2) * 255;
            var (ar, ag, ab) = picture.At(4, 4);
            Near((int)Math.Round(r), ar, 3, "red");
            Near((int)Math.Round(g), ag, 3, "green");
            Near((int)Math.Round(b), ab, 3, "blue");
        }, default);

        // Magnified, a white GBC pixel shows its three stripes, red at the left and blue at the right.
        [Fact]
        public Task A_magnified_gbc_pixel_shows_red_green_and_blue_stripes() => Session.Dispatch(() =>
        {
            Picture picture = Show(HandheldFilters.GbcLcd, 2, 2, 24, 24, Solid(2, 2, 0xFF, 0xFF, 0xFF), Solid(2, 2, 0xFF, 0xFF, 0xFF));
            var left = picture.At(2, 4); var middle = picture.At(6, 4); var right = picture.At(10, 4);
            Assert.True(left.R > left.G && left.R > left.B, $"left stripe {left}");
            Assert.True(middle.G > middle.R && middle.G > middle.B, $"middle stripe {middle}");
            Assert.True(right.B > right.R && right.B > right.G, $"right stripe {right}");
        }, default);

        // Lottes' scanlines: a flat grey picture magnified is brighter on a line's centre than between lines, and its curved corners are black.
        [Fact]
        public Task The_lottes_crt_draws_scanlines_and_a_curved_edge() => Session.Dispatch(() =>
        {
            Picture picture = Show(CrtFilters.Lottes, 16, 16, 128, 128, Solid(16, 16, 0xA0, 0xA0, 0xA0));
            double onLine = Enumerable.Range(58, 12).Average(x => picture.Luma(x, 68));
            double between = Enumerable.Range(58, 12).Average(x => picture.Luma(x, 64));
            Assert.True(onLine > between + 15, $"on a scanline {onLine}, between {between}");
            Assert.Equal((0, 0, 0), picture.At(0, 0));
        }, default);

        // The chain keeps as many earlier frames as its passes read, and no more.
        [Fact]
        public Task A_filter_keeps_the_frames_it_looks_back_at() => Session.Dispatch(() =>
        {
            var control = new GameFrameControl { ActiveFilter = HandheldFilters.DmgLcd };
            var window = new Window { Width = 8, Height = 8, Content = control };
            window.Show();
            for (int i = 0; i < 6; i++)
            {
                control.UpdateFrame(Solid(8, 8, (byte)(i * 40), 0, 0), 8, 8);
                window.CaptureRenderedFrame()!.Dispose();
            }
            int held = (int)typeof(GameFrameControl).GetProperty("FilterHistoryHeld", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(control)!;
            Assert.Equal(HandheldFilters.DmgLcd.History, held);
            window.Close();
        }, default);
    }
}

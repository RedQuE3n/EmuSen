using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // Within a core every frame is drawn in the same area of the screen, whatever its size, region or filter - see EmuSen_CRT.md §14.
    public partial class ScreenFilterRenderTests
    {
        // Every size of frame each console's cores hand over: width, rows, and how many times a row is shown - see EmuSen_CRT.md §14.2.
        public static readonly IReadOnlyDictionary<string, (int Width, int Rows, int Repeat)[]> FramesOf = new Dictionary<string, (int, int, int)[]>
        {
            ["NES"] = new[] { (256, 240, 1) },
            ["SNES"] = new[] { (256, 224, 1), (512, 224, 1), (256, 239, 1), (512, 448, 1), (512, 478, 1) },
            ["Genesis"] = new[] { (320, 224, 1), (256, 224, 1), (320, 240, 1), (256, 240, 1), (320, 448, 1), (320, 480, 1) },
            ["N64"] = new[] { (640, 240, 2), (640, 288, 2), (640, 480, 1), (640, 576, 1), (1280, 480, 2), (1280, 576, 2) },
            ["GB"] = new[] { (160, 144, 1) },
        };

        // The rectangle holding every pixel with a channel at 32 of 255 or more.
        private static (int Left, int Top, int Right, int Bottom) LitOf(Crt picture)
        {
            int left = int.MaxValue, top = int.MaxValue, right = -1, bottom = -1;
            for (int y = 0; y < picture.Height; y++) for (int x = 0; x < picture.Width; x++)
            {
                if (Math.Max(picture.Code(x, y, 0), Math.Max(picture.Code(x, y, 1), picture.Code(x, y, 2))) < 32) continue;
                (left, top, right, bottom) = (Math.Min(left, x), Math.Min(top, y), Math.Max(right, x), Math.Max(bottom, y));
            }
            return (left, top, right + 1, bottom + 1);
        }

        private static void SameArea((int Left, int Top, int Right, int Bottom) expected, (int Left, int Top, int Right, int Bottom) actual, int within, string what) =>
            Assert.True(Math.Abs(expected.Left - actual.Left) <= within && Math.Abs(expected.Top - actual.Top) <= within
                && Math.Abs(expected.Right - actual.Right) <= within && Math.Abs(expected.Bottom - actual.Bottom) <= within,
                $"{what}: lit {actual}, expected {expected} within {within}");

        // The defect of §14.1: a 288-line PAL field was placed in the 525-line raster and drawn 902 wide where an NTSC one was 1067.
        [Theory]
        [InlineData(0)]
        [InlineData(1)]
        [InlineData(2)]
        public Task Under_the_crt_filter_a_pal_and_an_ntsc_n64_frame_occupy_the_same_tube(int quality) => Session.Dispatch(() =>
        {
            (string, float)[] set = { ("quality", quality), ("level", 0f) };
            if (Draw("N64", 640, 240, 2, 1280, 800, Twice(Grey(640, 240, 1)), set) is not { } ntsc) return;
            if (Draw("N64", 640, 288, 2, 1280, 800, Twice(Grey(640, 288, 1)), set) is not { } pal) return;
            SameArea((107, 0, 1173, 800), LitOf(ntsc), 2, "an NTSC field fills the 4:3 tube of a 1280 by 800 screen");
            SameArea(LitOf(ntsc), LitOf(pal), 2, "and a PAL field the same");
        }, default);

        public static IEnumerable<object[]> ConsolesAndQualities() =>
            from console in EmuSen.Serenity.Shaders.CrtFilter.Filter.Consoles! from quality in new[] { 0, 1, 2 } select new object[] { console, quality };

        [Theory]
        [MemberData(nameof(ConsolesAndQualities))]
        public Task Under_the_crt_filter_every_frame_a_console_hands_over_occupies_the_same_tube(string console, int quality) => Session.Dispatch(() =>
        {
            (string, float)[] set = { ("quality", quality), ("level", 0f) };
            foreach (var (width, rows, repeat) in FramesOf[console])
            {
                if (Draw(console, width, rows, repeat, 1280, 800, Twice(Grey(width, rows, 1)), set) is not { } picture) return;
                SameArea((107, 0, 1173, 800), LitOf(picture), 2, $"{console} {width}x{rows}, rows shown {repeat}x, quality {quality}");
            }
        }, default);

        // A lit column's width at half its height, in pixels, magnified ten times: the spot along the line, with the signal's own blur.
        private static double? ColumnWidth(int rows, int repeat)
        {
            byte[] frame = Paint(640, rows, (x, _) => x == 320 ? (1.0, 1.0, 1.0) : (0.0, 0.0, 0.0));
            if (Draw("N64", 640, rows, repeat, 640, 480, Twice(frame), With(Plain, ("signal", 0f), ("mask", 1f), ("subpixels", 0f), ("overscan", 900f), ("displayNits", 300f))) is not { } picture) return null;
            double[] row = picture.Across(1, 120, 240, 200, 440);
            double floor = row.Take(40).Average();
            row = row.Select(v => v - floor).ToArray();
            double peak = row.Max(), half = peak / 2;
            int top = Array.IndexOf(row, peak), a = top, b = top;
            while (row[a] > half) a--;
            while (row[b] > half) b++;
            return (b - 1 + (row[b - 1] - half) / (row[b - 1] - row[b])) - (a + (half - row[a]) / (row[a + 1] - row[a]));
        }

        // The spot is as wide along a line as across it, so a field of 288 lines, drawn closer together, has a narrower one than a field of 240.
        [Fact]
        public Task The_spot_along_a_line_narrows_with_the_lines_of_a_field() => Session.Dispatch(() =>
        {
            if (ColumnWidth(240, 2) is not { } ntsc || ColumnWidth(288, 2) is not { } pal || ColumnWidth(480, 1) is not { } woven) return;
            Assert.InRange(pal / ntsc, 0.91, 0.96);
            Within(ntsc, woven, 0.02, "two fields of 240 lines have one field's spot");
        }, default);

        // One setting, two paths: under the filter the part kept fills the tube as it fills the screen with none, for a PAL field and an NTSC one - see EmuSen_CRT.md §15.
        [Theory]
        [InlineData(0)]
        [InlineData(1)]
        [InlineData(2)]
        public Task Under_the_crt_filter_a_crop_shows_the_part_the_plain_picture_shows(int quality) => Session.Dispatch(() =>
        {
            (string, float)[] set = { ("quality", quality), ("level", 0f) };
            var uneven = new PictureCrop(0.1, 0.125, 0.05, 0.25);
            foreach (PictureCrop crop in new[] { uneven, PictureCrop.Television })
                foreach (int rows in new[] { 240, 288 })
                {
                    string what = $"{rows} lines, quality {quality}, crop {crop}";
                    if (Cropped(crop, "N64", 640, rows, 2, 1280, 800, Twice(PictureCropTests.Marked(640, rows, crop, whiteInside: true)), set) is not { } kept) return;
                    SameArea((107, 0, 1173, 800), LitOf(kept), 2, what);
                    if (Cropped(crop, "N64", 640, rows, 2, 1280, 800, Twice(PictureCropTests.Marked(640, rows, crop, whiteInside: false)), set) is not { } hidden) return;
                    Assert.True(hidden.Luminance(160, 60, 960, 680) < 0.004, $"{what}: what was to be hidden lights the tube, {hidden.Luminance(160, 60, 960, 680):F4}");
                    if (Cropped(PictureCrop.None, "N64", 640, rows, 2, 1280, 800, Twice(PictureCropTests.Marked(640, rows, crop, whiteInside: false)), set) is not { } whole) return;
                    Assert.True(LitOf(whole) is (<= 109, <= 2, >= 1171, >= 798), $"{what}: uncropped, the same frame's margin is lit to the glass's edges, {LitOf(whole)}");
                }
        }, default);

        // The glass does not move: cropped, the tube's rounded corner is where it was and as dark, with the picture lit beside it.
        [Fact]
        public Task Under_the_crt_filter_a_crop_moves_the_picture_and_not_the_glass() => Session.Dispatch(() =>
        {
            (string, float)[] set = With(Plain, ("signal", 0f), ("mask", 1f));
            byte[][] white = Twice(Grey(640, 240, 1));
            if (Cropped(PictureCrop.None, "N64", 640, 240, 2, 1280, 800, white, set) is not { } whole) return;
            if (Cropped(PictureCrop.Enlarged(1.5), "N64", 640, 240, 2, 1280, 800, white, set) is not { } cropped) return;
            foreach (Crt picture in new[] { whole, cropped })
            {
                Assert.True(picture.Luminance(108, 1, 4, 4) < 0.01, $"the tube's corner is dark: {picture.Luminance(108, 1, 4, 4):F4}");
                Assert.True(picture.Luminance(150, 40, 8, 40) > 0.2, $"and the picture inside it lit: {picture.Luminance(150, 40, 8, 40):F4}");
            }
        }, default);

        // Each pixel of a frame drawn again as a square of them: what a core drawing at a multiple of the console's picture hands over, for a picture with no finer detail.
        private static byte[] Enlarged(byte[] frame, int width, int height, int by)
        {
            var large = new byte[frame.Length * by * by];
            for (int y = 0; y < height * by; y++) for (int x = 0; x < width * by; x++)
                Buffer.BlockCopy(frame, (y / by * width + x / by) * 4, large, (y * width * by + x) * 4, 4);
            return large;
        }

        // A picture with an edge every way, so that a line, a column or a colour misplaced by the multiple shows.
        private static byte[] Pattern(int width, int height) => Paint(width, height, (x, y) =>
            ((x / 40 + y / 12) % 2 == 0 ? 0.9 : 0.1, x * 1.0 / width, y % 2 == 0 ? 0.8 : 0.2));

        // The defect of §15.1: at an internal resolution of n the tube drew n times the console's lines, and from two up took them for woven fields.
        [Theory]
        [InlineData(0, 240)]
        [InlineData(1, 240)]
        [InlineData(2, 240)]
        [InlineData(0, 288)]
        [InlineData(1, 288)]
        [InlineData(2, 288)]
        public Task Under_the_crt_filter_a_frame_at_any_internal_resolution_is_drawn_as_the_console_s_own(int quality, int lines) => Session.Dispatch(() =>
        {
            (string, float)[] set = { ("quality", quality), ("level", 0f) };
            byte[] frame = Pattern(640, lines);
            if (Scaled(0, PictureCrop.None, "N64", 640, lines, 2, 1280, 800, Twice(frame), set) is not { } own) return;
            Assert.True(own.Luminance(400, 300, 480, 200) > 0.02, "it is a picture");
            foreach (int by in new[] { 1, 2, 3, 4 })
            {
                if (Scaled(lines, PictureCrop.None, "N64", 640 * by, lines * by, 2, 1280, 800, Twice(Enlarged(frame, 640, lines, by)), set) is not { } scaled) return;
                // The mean of a square of equal values is that value to a float's last place, and a last place can turn a rounding: one code value, in a few values of a thousand.
                int differing = 0, worst = 0;
                for (int y = 0; y < 800; y++) for (int x = 0; x < 1280; x++) for (int c = 0; c < 3; c++)
                {
                    int d = Math.Abs(own.Code(x, y, c) - scaled.Code(x, y, c));
                    if (d > 0) differing++;
                    worst = Math.Max(worst, d);
                }
                Assert.True(worst <= 1 && differing < 1280 * 800 * 3 / 200, $"{lines} lines at {by} times, quality {quality}: {differing} values differ from the console's own frame, by at most {worst}");
            }
        }, default);

        // The scanlines themselves, counted: a white field's lines down a quarter of the tube, and how far the light swings across them.
        private static (int Lines, double Low, double High) Scanlines(Crt picture)
        {
            double[] column = Enumerable.Range(0, 800).Select(y => picture.Mean(600, y, 80, 1)[1]).ToArray();
            double mean = column.Average();
            int peaks = 0;
            for (int y = 1; y < 799; y++) if (column[y] > mean && column[y] >= column[y - 1] && column[y] > column[y + 1]) peaks++;
            return (peaks, column.Min(), column.Max());
        }

        [Theory]
        [InlineData(240, 60)]
        [InlineData(288, 72)]
        public Task The_tube_draws_the_console_s_lines_at_every_internal_resolution(int lines, int inAQuarter) => Session.Dispatch(() =>
        {
            (string, float)[] set = With(Plain, ("signal", 0f), ("mask", 1f), ("subpixels", 0f));
            var seen = new List<(int Lines, double Low, double High)>();
            foreach (int by in new[] { 1, 2, 3, 4 })
            {
                if (Scaled(lines, PictureCrop.Enlarged(4), "N64", 640 * by, lines * by, 2, 1280, 800, Twice(Grey(640 * by, lines * by, 1)), set) is not { } picture) return;
                seen.Add(Scanlines(picture));
            }
            Assert.All(seen, s => Assert.Equal(inAQuarter, s.Lines));
            Assert.All(seen, s => Assert.Equal(seen[0], s));
            Assert.True(seen[0].High > 2 * seen[0].Low, $"and they are lines, dark between: {seen[0].Low:F3} to {seen[0].High:F3}");

            // Without its lines told, the same frame at four times is the defect: four times the lines, and no dark between them.
            if (Scaled(0, PictureCrop.Enlarged(4), "N64", 2560, lines * 4, 2, 1280, 800, Twice(Grey(2560, lines * 4, 1)), set) is not { } untold) return;
            Assert.True(Scanlines(untold).High < 1.2 * Scanlines(untold).Low, "a frame that does not say its lines is drawn row for line, as before");
        }, default);

        [Fact]
        public void The_n64_s_cores_report_the_console_s_lines_from_the_frame_s_width()
        {
            Assert.Equal(240, EmuSen.Cores.Nintendo.Mars.MarsCore.LinesOf(640, 240));
            Assert.Equal(240, EmuSen.Cores.Nintendo.Mars.MarsCore.LinesOf(2560, 960));
            Assert.Equal(288, EmuSen.Cores.Nintendo.Mars.MarsCore.LinesOf(1920, 864));
            Assert.Equal(480, EmuSen.Cores.Nintendo.Mars.MarsCore.LinesOf(1280, 960));
            Assert.Equal(0, EmuSen.Cores.Nintendo.Mars.MarsCore.LinesOf(320, 240));
            Assert.Equal(0, ((EmuSen.Cores.ICore)new EmuSen.Cores.Nintendo.Moon.MoonCore()).DisplayLines);

            // Before a game is loaded each engine's frame is 640 by 480, both fields of a picture at the console's own size.
            Assert.Equal(480, ((EmuSen.Cores.ICore)new EmuSen.Cores.Nintendo.Mars.MarsCore()).DisplayLines);
            if (!EmuSen.Cores.Nintendo.MarsRT.MarsRtCore.Available) return;
            using var native = new EmuSen.Cores.Nintendo.MarsRT.MarsRtCore();
            Assert.Equal(480, ((EmuSen.Cores.ICore)native).DisplayLines);
        }

        // A frame at twice the console's picture whose pixels alternate white and black: every square of four is half white, so each line is drawn as a grey one.
        [Fact]
        public Task Each_of_the_console_s_pixels_is_the_mean_of_the_square_drawn_for_it() => Session.Dispatch(() =>
        {
            (string, float)[] set = With(Plain, ("signal", 0f), ("mask", 1f));
            byte[] checker = Paint(1280, 480, (x, y) => (x + y) % 2 == 0 ? (1.0, 1.0, 1.0) : (0.0, 0.0, 0.0));
            if (Scaled(240, PictureCrop.None, "N64", 1280, 480, 2, 1280, 800, Twice(checker), set) is not { } mixed) return;
            if (Scaled(0, PictureCrop.None, "N64", 640, 240, 2, 1280, 800, Twice(Grey(640, 240, 0.5)), set) is not { } grey) return;
            foreach (var (x, y) in new[] { (200, 100), (640, 400), (1000, 650) })
                Within(grey.Luminance(x, y, 80, 80), mixed.Luminance(x, y, 80, 80), 0.02, $"the light at ({x}, {y})");
            Assert.Equal(Scanlines(grey).Lines, Scanlines(mixed).Lines);
        }, default);

        // The console's lines are for a filter that draws a screen; any other filter is given the frame as it is.
        [Fact]
        public Task A_filter_that_draws_no_screen_takes_the_frame_at_its_internal_resolution() => Session.Dispatch(() =>
        {
            if (CrtDevice.Value is not { } device) return;
            const string through = "uniform shader source; uniform float2 inputSize; uniform float2 outputSize; half4 main(float2 coord) { return source.eval(coord * inputSize / outputSize); }";
            byte[] columns = Paint(1280, 480, (x, _) => x % 2 == 0 ? (1.0, 1.0, 1.0) : (0.0, 0.0, 0.0));
            byte[] Drawn(int lines)
            {
                var filter = new ScreenFilter("through", new[] { new FilterPass(through, PassScale.Viewport) }, null, "");
                return ShaderBench.Picture(new GameFrameControl { ActiveFilter = filter }, new byte[]?[] { columns }, 1280, 480, 1, 1280, 480, device, lines);
            }
            byte[] told = Drawn(240), untold = Drawn(0);
            Assert.Equal(untold, told);
            Assert.Equal((255, 0), (told[1], told[5]));
        }, default);
    }
}

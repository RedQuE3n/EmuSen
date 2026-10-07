using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Serenity;

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
    }
}

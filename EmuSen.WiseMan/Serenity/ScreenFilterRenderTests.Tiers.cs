using System;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // Balanced and Performance measured as reductions of Accurate, and the bright picture the filter draws by default - see EmuSen_CRT.md §12.
    public partial class ScreenFilterRenderTests
    {
        private static readonly (string, float)[] Balanced = { ("quality", 1f) }, Performance = { ("quality", 0f) };

        // Linear light averaged over blocks of a picture, then CIELAB against the reference's brightest block: the mean colour difference a person would see from a step back.
        private static double MeanDeltaE(Crt reference, Crt other, int block)
        {
            double[] Xyz(double[] c) => new[] { 0.4124 * c[0] + 0.3576 * c[1] + 0.1805 * c[2], 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2], 0.0193 * c[0] + 0.1192 * c[1] + 0.9505 * c[2] };
            var blocks = Enumerable.Range(0, reference.Height / block).SelectMany(by => Enumerable.Range(0, reference.Width / block).Select(bx =>
                (A: reference.Mean(bx * block, by * block, block, block), B: other.Mean(bx * block, by * block, block, block)))).ToArray();
            double white = blocks.Max(p => Xyz(p.A)[1]);
            double F(double t) => t > 0.008856 ? Math.Cbrt(t) : 7.787 * t + 16.0 / 116;
            double[] Lab(double[] c)
            {
                double[] x = Xyz(c);
                double fx = F(x[0] / (0.9505 * white)), fy = F(x[1] / white), fz = F(x[2] / (1.089 * white));
                return new[] { 116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz) };
            }
            var lit = blocks.Where(p => p.A.Max() + p.B.Max() > 1e-6).ToArray();
            return lit.Average(p => { double[] a = Lab(p.A), b = Lab(p.B); return Math.Sqrt(Enumerable.Range(0, 3).Sum(i => (a[i] - b[i]) * (a[i] - b[i]))); });
        }

        private static byte[] Scene(int width, int height) => Paint(width, height, (x, y) =>
        {
            int band = y * 4 / height;
            double t = (double)x / (width - 1);
            return band switch
            {
                0 => (x * 8 / width) switch { 0 => (0.75, 0.75, 0.75), 1 => (0.75, 0.75, 0), 2 => (0, 0.75, 0.75), 3 => (0, 0.75, 0), 4 => (0.75, 0, 0.75), 5 => (0.75, 0, 0), 6 => (0, 0, 0.75), _ => (0, 0, 0) },
                1 => (t, t, t),
                2 => ((x / 8 + y / 8) % 2 == 0 ? (0.9, 0.8, 0.3) : (0.1, 0.2, 0.5)),
                _ => (0.5 + 0.4 * Math.Sin(t * 12), 0.4, 0.5 - 0.4 * Math.Sin(t * 12)),
            };
        });

        [Fact]
        public Task Balanced_and_performance_keep_a_flat_field_s_light_and_colour() => Session.Dispatch(() =>
        {
            foreach (var (tier, label) in new[] { (Balanced, "Balanced"), (Performance, "Performance") })
                foreach (var (width, height, spot) in new[] { (1440, 1080, 1f), (2880, 2160, 1f), (1440, 1080, 2f) })
                    foreach (double level in new[] { 1.0, 0.5 })
                    {
                        if (Draw("SNES", 256, 224, width, height, Twice(Grey(256, 224, level)), With(With(Plain, ("signal", 0f), ("spotSize", spot)), tier)) is not { } picture) return;
                        double[] mean = picture.Mean(width / 4.0, height / 4.0, width / 2.0, height / 2.0);
                        string what = $"{label} at {width}x{height}, spot {spot}, level {level}";
                        foreach (double channel in mean) Within(Tube(level) / 3, channel, 0.03, what);
                        Assert.True(mean.Max() - mean.Min() < 0.015 * mean.Average(), $"{what} is tinted: {string.Join(", ", mean.Select(m => m.ToString("F4")))}");
                    }
        }, default);

        // Measured on 2026-10-07 at 1440x1080 with the glass flat: Balanced 0.7 to 1.1 and Performance 3.5 to 7.8 on real frames; the bounds hold those with room - see EmuSen_CRT.md §12.4.
        [Fact]
        public Task Each_lower_tier_stays_within_its_measured_distance_of_accurate() => Session.Dispatch(() =>
        {
            byte[][] scene = Enumerable.Repeat(Scene(256, 224), 6).ToArray();
            (string, float)[] flat = { ("curvature", 0f) };
            if (Draw("SNES", 256, 224, 1440, 1080, scene, flat) is not { } accurate) return;
            if (Draw("SNES", 256, 224, 1440, 1080, scene, With(flat, Balanced)) is not { } balanced) return;
            if (Draw("SNES", 256, 224, 1440, 1080, scene, With(flat, Performance)) is not { } performance) return;
            double b = MeanDeltaE(accurate, balanced, 8), p = MeanDeltaE(accurate, performance, 8);
            Assert.True(b < 2, $"Balanced is {b:F2} from Accurate");
            Assert.True(p < 8 && p > b, $"Performance is {p:F2} from Accurate, Balanced {b:F2}");
        }, default);

        // Balanced keeps the real subcarrier, so the Genesis's columns still turn into its rainbow; Performance only blurs, so they blend without one.
        [Fact]
        public Task Balanced_keeps_the_genesis_rainbow_and_performance_blends_the_columns_without_one() => Session.Dispatch(() =>
        {
            byte[] columns = Paint(320, 224, (x, _) => x % 2 == 0 ? (0.6, 0.6, 0.6) : (0.2, 0.2, 0.2));
            (string, float)[] set = With(Plain, ("mask", 1f), ("spotGrowth", 0f), ("signal", 2f));
            if (Draw("Genesis", 320, 224, 1440, 1080, Twice(columns), With(set, Balanced)) is not { } balanced) return;
            if (Draw("Genesis", 320, 224, 1440, 1080, Twice(columns), With(set, Performance)) is not { } performance) return;
            if (Draw("Genesis", 320, 224, 1440, 1080, Twice(columns), With(set, ("signal", 0f))) is not { } rgb) return;
            var (left, top, width, height) = PictureIn(1440, 1080);
            int x0 = (int)(left + width * 0.2), x1 = (int)(left + width * 0.8);
            double Swing(Crt picture, int channel) { double[] row = picture.Across(channel, top + height / 2 - 20, 40, x0, x1); return (row.Max() - row.Min()) / row.Average(); }

            Within(30 * width / 320, Period(balanced.Across(2, top + height / 2 - 20, 40, x0, x1), 60, 220), 0.04, "Balanced's rainbow, in output pixels");
            Assert.True(Swing(balanced, 2) > 1, $"Balanced's blue swings {Swing(balanced, 2):F2}");
            Assert.True(Swing(performance, 2) < 0.2 && Swing(performance, 1) < 0.2, $"Performance has no rainbow: blue {Swing(performance, 2):F2}, green {Swing(performance, 1):F2}");
            Assert.True(Swing(rgb, 1) > 5 * Swing(performance, 1), $"and its columns blend: {Swing(performance, 1):F2} against {Swing(rgb, 1):F2} by RGB");
        }, default);

        // The default: white at three fifths of the display's peak, nothing clipped, the mask at full depth in the darks and eased at white.
        [Fact]
        public Task By_default_the_picture_is_bright_and_the_mask_eases_off_only_toward_white() => Session.Dispatch(() =>
        {
            Assert.Equal((float)CrtFilter.Quality.Balanced, CrtFilter.Parameters.Single(p => p.Id == "quality").Initial);
            Assert.Equal(0f, CrtFilter.Parameters.Single(p => p.Id == "level").Initial);
            (string, float)[] set = With(Plain, ("signal", 0f), ("level", 0f), ("spotSize", 2f), ("spotGrowth", 0f));
            double Depth(Crt picture) { double[] row = picture.Across(1, 520, 40, 400, 1040); return (row.Max() - row.Min()) / row.Average(); }
            Crt? white = Draw("SNES", 256, 224, 1440, 1080, Twice(Grey(256, 224, 1)), set);
            Crt? dark = Draw("SNES", 256, 224, 1440, 1080, Twice(Grey(256, 224, 0.25)), set);
            Crt? darkFull = Draw("SNES", 256, 224, 1440, 1080, Twice(Grey(256, 224, 0.25)), With(set, ("level", 1f), ("displayNits", 1000f)));
            if (white is null || dark is null || darkFull is null) return;
            Within(0.6, white.Mean(360, 270, 720, 540).Average(), 0.03, "white's light as a share of the display's peak, three fifths as decided");
            Assert.True(white.Largest < 255, "nothing clips");
            Within(Depth(darkFull), Depth(dark), 0.05, "a dark grey's mask against the same grey with all the headroom it needs");
            Assert.True(Depth(white) < 0.5 * Depth(dark), $"white's mask is eased: {Depth(white):F2} against {Depth(dark):F2} at a dark grey");
        }, default);
    }
}

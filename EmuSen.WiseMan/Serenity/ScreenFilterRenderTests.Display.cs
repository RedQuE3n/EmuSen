using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Crystal.Systems;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using SkiaSharp;

namespace EmuSen.WiseMan.Serenity
{
    // The tube on displays of every shape, size and scale: its rectangle, its light, its scanlines and its mask - see EmuSen_CRT.md §20.
    public unsafe partial class ScreenFilterRenderTests
    {
        // A frame drawn by the filter on a display of W by H pixels whose windowing scales the picture by the given factor.
        private static Crt? OnDisplay(string console, byte[] frame, int width, int rows, int W, int H, double scale, params (string Id, float Value)[] set)
        {
            if (CrtDevice.Value is not { } device) return null;
            using var gl = ShaderBench.GlContext.Create(device);
            using GRGlInterface glInterface = GRGlInterface.CreateOpenGl(name => ShaderBench.GlContext.GetProc(name))!;
            using GRContext context = GRContext.CreateGl(glInterface)!;
            using SKSurface target = SKSurface.Create(context, true, new SKImageInfo(W, H, SKColorType.Rgba8888, SKAlphaType.Premul))!;
            var control = new GameFrameControl { FilterConsole = console, ShaderParameters = Values(Reference.Concat(set)), ActiveFilter = CrtFilter.Filter };
            control.WaitForFilter();
            for (int i = 0; i < 2; i++)
            {
                control.UpdateFrame(frame, width, rows, aspect: DisplayShape.Television);
                target.Canvas.Clear(SKColors.Black);
                target.Canvas.ResetMatrix();
                target.Canvas.Scale((float)scale);
                using (var op = control.CaptureDrawOp(new Avalonia.Size(W / scale, H / scale))!) op.RenderTo(target.Canvas, context);
                context.Flush();
                gl.Finish();
            }
            var pixels = new byte[W * H * 4];
            fixed (byte* p = pixels) target.ReadPixels(new SKImageInfo(W, H, SKColorType.Rgba8888, SKAlphaType.Premul), (nint)p, W * 4, 0, 0);
            control.ActiveFilter = null;
            return new Crt(pixels, W, H);
        }

        // The tube's rectangle in display pixels, rounded as the frame control rounds it.
        private static (double X, double Y, double W, double H) TubeOn(int W, int H, double scale)
        {
            var (x, y, w, h) = GameFrameControl.ComputeLetterboxRect(4.0 / 3.0, 1, W / scale, H / scale);
            return (x * scale, y * scale, Math.Max(1, Math.Round(w)) * scale, Math.Max(1, Math.Round(h)) * scale);
        }

        // Each line's light over the middle three fifths of the tube, integrated across the line's height: the mean, and the spread from line to line as a share of it.
        private static (double Mean, double Spread) LineLight(Crt p, (double X, double Y, double W, double H) tube, int lines)
        {
            int x0 = (int)(tube.X + tube.W * 0.25), x1 = (int)(tube.X + tube.W * 0.75);
            var profile = new double[p.Height];
            for (int y = Math.Max(0, (int)tube.Y); y < Math.Min(p.Height, (int)(tube.Y + tube.H)); y++)
                profile[y] = Enumerable.Range(x0, x1 - x0).Average(x => p.Light(x, y, 1));
            double pitch = tube.H / lines;
            var light = new List<double>();
            for (int k = 0; k < lines; k++)
            {
                double top = tube.Y + k * pitch, bottom = top + pitch;
                if (top < tube.Y + tube.H * 0.2 || bottom > tube.Y + tube.H * 0.8) continue;
                double sum = 0;
                for (int y = (int)Math.Floor(top); y < Math.Ceiling(bottom); y++) sum += Math.Max(0, Math.Min(bottom, y + 1) - Math.Max(top, y)) * profile[y];
                light.Add(sum / pitch);
            }
            return (light.Average(), (light.Max() - light.Min()) / light.Average());
        }

        // The picture blurred by a Gaussian of ten pixels, which takes out a mask of up to six and the scanlines: what colour and light still vary by across the tube is moire.
        private static (double Chroma, double Light) Moire(Crt p, (double X, double Y, double W, double H) tube)
        {
            const int step = 8, reach = 30;
            const double sigma = 10;
            double[] k = Enumerable.Range(-reach, 2 * reach + 1).Select(i => Math.Exp(-0.5 * i * i / (sigma * sigma))).ToArray();
            double total = k.Sum();
            int x0 = (int)(tube.X + tube.W * 0.1), x1 = (int)(tube.X + tube.W * 0.9), y0 = (int)(tube.Y + tube.H * 0.1), y1 = (int)(tube.Y + tube.H * 0.9);
            int[] xs = Enumerable.Range(0, (x1 - x0) / step).Select(i => x0 + i * step).ToArray();
            var across = new double[3][,];
            for (int c = 0; c < 3; c++) across[c] = new double[p.Height, xs.Length];
            for (int y = y0 - reach; y < y1 + reach; y++)
                for (int i = 0; i < xs.Length; i++)
                    for (int c = 0; c < 3; c++)
                    {
                        double s = 0;
                        for (int d = -reach; d <= reach; d++) s += k[d + reach] * p.Light(xs[i] + d, y, c);
                        across[c][y, i] = s / total;
                    }
            var red = new List<double>(); var blue = new List<double>(); var luma = new List<double>();
            for (int y = y0; y < y1; y += step)
                for (int i = 0; i < xs.Length; i++)
                {
                    var m = new double[3];
                    for (int c = 0; c < 3; c++)
                    {
                        double s = 0;
                        for (int d = -reach; d <= reach; d++) s += k[d + reach] * across[c][y + d, i];
                        m[c] = s / total;
                    }
                    red.Add(m[0] / m[1]); blue.Add(m[2] / m[1]); luma.Add(0.2126 * m[0] + 0.7152 * m[1] + 0.0722 * m[2]);
                }
            static double Deviation(List<double> v) { double a = v.Average(); return Math.Sqrt(v.Select(x => (x - a) * (x - a)).Average()) / a; }
            return (Math.Max(Deviation(red), Deviation(blue)), Deviation(luma));
        }

        // The displays of §20.1: each shape a player might draw on, odd and very small windows, and windowing scaled by half and by two.
        public static IEnumerable<object[]> Displays() => new (int, int, double)[]
        {
            (1920, 1080, 1), (1280, 800, 1), (3440, 1440, 1), (5120, 1440, 1), (1080, 2340, 1), (999, 777, 1), (97, 61, 1), (2880, 1620, 1.5), (2560, 1600, 2),
        }.Select(d => new object[] { d.Item1, d.Item2, d.Item3 });

        [Theory]
        [MemberData(nameof(Displays))]
        public Task On_any_display_the_tube_is_four_by_three_centred_and_its_light_and_colour_hold_at_every_quality(int W, int H, double scale) => Session.Dispatch(() =>
        {
            var tube = TubeOn(W, H, scale);
            (string, float)[] flat = With(Plain, ("signal", 0f), ("mask", 1f), ("subpixels", 0f));
            foreach (int quality in new[] { 0, 1, 2 })
            {
                string what = $"{W}x{H} at {scale:P0}, quality {quality}";
                if (OnDisplay("SNES", Grey(256, 224, 1), 256, 224, W, H, scale, With(flat, ("quality", quality))) is not { } picture) return;
                SameArea(((int)Math.Round(tube.X), (int)Math.Round(tube.Y), (int)Math.Round(tube.X + tube.W), (int)Math.Round(tube.Y + tube.H)), LitOf(picture), 1, what);
                double[] mean = picture.Mean(tube.X + tube.W * 0.25, tube.Y + tube.H * 0.2, tube.W * 0.5, tube.H * 0.6);
                Within(Tube(1) / 3, mean.Average(), 0.01, $"{what}: a white field's light");
                Assert.True(mean.Max() - mean.Min() < 0.003 * mean.Average(), $"{what}: tinted {string.Join(", ", mean.Select(m => m.ToString("F4")))}");
            }
        }, default);

        // The glass's curve and corners are the tube's, not the display's: the share of the tube they leave lit is the same at every size.
        [Fact]
        public Task Curvature_and_corners_scale_with_the_tube() => Session.Dispatch(() =>
        {
            var shares = new List<double>();
            foreach (var (W, H) in new[] { (640, 480), (1280, 800), (1920, 1080), (1080, 2340), (3840, 2160) })
            {
                var tube = TubeOn(W, H, 1);
                if (OnDisplay("SNES", Grey(256, 224, 1), 256, 224, W, H, 1, With(Plain, ("signal", 0f), ("mask", 1f), ("curvature", 1f))) is not { } picture) return;
                int lit = 0;
                for (int y = 0; y < H; y++) for (int x = 0; x < W; x++) if (picture.Code(x, y, 1) >= 32) lit++;
                shares.Add(lit / (tube.W * tube.H));
            }
            Assert.True(shares.Max() - shares.Min() < 0.002, $"the share of the tube lit with the glass curved: {string.Join(", ", shares.Select(s => s.ToString("F4")))}");
        }, default);

        // The defect of §20.2: a pixel taller than the lines a tier reads lost their light, and the lower tiers, sampling the spot at the pixel's centre, banded.
        [Fact]
        public Task In_a_small_window_every_line_s_light_is_kept_at_every_quality() => Session.Dispatch(() =>
        {
            (string, float)[] flat = With(Plain, ("signal", 0f), ("mask", 1f), ("subpixels", 0f));
            foreach (var (W, H) in new[] { (333, 250), (160, 120), (33, 25) })
                foreach (int quality in new[] { 0, 1, 2 })
                {
                    var tube = TubeOn(W, H, 1);
                    if (OnDisplay("SNES", Grey(256, 224, 1), 256, 224, W, H, 1, With(flat, ("quality", quality))) is not { } picture) return;
                    string what = $"{W}x{H}, quality {quality}";
                    Within(Tube(1) / 3, picture.Mean(tube.X + tube.W * 0.25, tube.Y + tube.H * 0.2, tube.W * 0.5, tube.H * 0.6)[1], 0.015, $"{what}: a white field's light");
                    if (H >= 250) Assert.True(LineLight(picture, tube, 224).Spread < 0.25, $"{what}: lines' light spread by {LineLight(picture, tube, 224).Spread:F3}");
                }
        }, default);

        // The defect of §20.3: up to its limit the mask beat against the display's pixels and, curved, drew rings of colour; drawn in whole display pixels it leaves none.
        [Fact]
        public Task Up_to_its_limit_the_mask_is_drawn_in_whole_display_pixels_and_leaves_no_moire() => Session.Dispatch(() =>
        {
            foreach (var (W, H, qualities) in new[] { (1280, 800, new[] { 0, 1, 2 }), (1920, 1200, new[] { 0, 1, 2 }), (3840, 2160, new[] { 1 }) })
                foreach (int quality in qualities)
                    foreach (int screen in new[] { 1, 2 })
                    {
                        var tube = TubeOn(W, H, 1);
                        if (OnDisplay("SNES", Grey(256, 224, 0.5), 256, 224, W, H, 1, ("level", 0f), ("maskLimit", 1f), ("quality", quality), ("screen", screen), ("curvature", 1f)) is not { } picture) return;
                        var (chroma, light) = Moire(picture, tube);
                        string what = $"{W}x{H}, quality {quality}, {CrtFilter.Screens[screen].Label}";
                        Assert.True(chroma < 0.0012 && light < 0.0018, $"{what}: colour varies by {chroma:P3} and light by {light:P3} across the tube");
                    }

            // On flat glass the period is whole: two pixels a triad on the handheld's panel, three at 1080p, where the set's own are 2.07 and 2.80.
            foreach (var (W, H, pixels) in new[] { (1280, 800, 2), (1920, 1080, 3) })
            {
                if (OnDisplay("SNES", Grey(256, 224, 0.5), 256, 224, W, H, 1, With(Plain, ("signal", 0f), ("subpixels", 0f), ("maskLimit", 1f))) is not { } flat) return;
                double[] green = flat.Across(1, H / 2.0 - 20, 40, W / 4, 3 * W / 4);
                double mean = green.Average();
                // Two triads, since a slot mask's columns alternate their slots' height.
                double drift = Enumerable.Range(0, green.Length - 2 * pixels).Max(i => Math.Abs(green[i] - green[i + 2 * pixels])) / mean;
                Assert.True(drift < 0.02, $"{W}x{H}: the mask repeats every {2 * pixels} pixels to within {drift:P2}");
            }
        }, default);

        // Below one and a half pixels a triad the display cannot draw the mask at all, and it is let go rather than drawn as an alias; at one and a half it is drawn across two.
        [Fact]
        public Task A_mask_the_display_cannot_draw_is_let_go() => Session.Dispatch(() =>
        {
            double Modulation(Crt picture, int W, int H)
            {
                double[] green = picture.Across(1, H / 2.0 - 20, 40, W / 4, 3 * W / 4);
                double mean = green.Average();
                return Math.Sqrt(green.Average(g => (g - mean) * (g - mean))) / mean;
            }
            (string, float)[] set = With(Plain, ("signal", 0f), ("subpixels", 0f), ("maskLimit", 1f));
            if (OnDisplay("SNES", Grey(256, 224, 0.5), 256, 224, 640, 480, 1, set) is not { } gone) return;
            if (OnDisplay("SNES", Grey(256, 224, 0.5), 256, 224, 1060, 580, 1, set) is not { } drawn) return;
            Assert.True(Modulation(gone, 640, 480) < 0.01, $"at 1.24 pixels a triad the mask is gone: {Modulation(gone, 640, 480):F3}");
            Assert.True(Modulation(drawn, 1060, 580) > 0.3, $"at 1.50 it is drawn: {Modulation(drawn, 1060, 580):F3}");
            Within(Tube(0.5) / 3, gone.Mean(160, 120, 320, 240)[1], 0.015, "and the light is kept");
        }, default);
    }
}

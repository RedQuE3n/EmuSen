using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // The CRT filter's pictures measured against its own physics, the signal's arithmetic and the published measurements, on a GL device - see EmuSen_CRT.md §11.9.
    public partial class ScreenFilterRenderTests
    {
        private static readonly double[] ToLight = Enumerable.Range(0, 256).Select(i => i / 255.0)
            .Select(c => c <= 0.04045 ? c / 12.92 : Math.Pow((c + 0.055) / 1.055, 2.4)).ToArray();

        // A drawn picture, read in linear light as a share of the display's peak.
        private sealed class Crt(byte[] rgba, int width, int height)
        {
            public int Width => width;
            public int Height => height;

            public double Light(int x, int y, int channel) => ToLight[rgba[(y * width + x) * 4 + channel]];

            public int Code(int x, int y, int channel) => rgba[(y * width + x) * 4 + channel];

            public double[] Mean(double x, double y, double w, double h)
            {
                var sum = new double[3];
                int x0 = (int)Math.Round(x), y0 = (int)Math.Round(y), x1 = (int)Math.Round(x + w), y1 = (int)Math.Round(y + h);
                for (int j = y0; j < y1; j++) for (int i = x0; i < x1; i++) for (int c = 0; c < 3; c++) sum[c] += Light(i, j, c);
                return sum.Select(s => s / ((x1 - x0) * (y1 - y0))).ToArray();
            }

            public double Luminance(double x, double y, double w, double h) { double[] m = Mean(x, y, w, h); return 0.2126 * m[0] + 0.7152 * m[1] + 0.0722 * m[2]; }

            // One channel along a row, each sample the mean of a band of rows, so that scanlines average out.
            public double[] Across(int channel, double y, double h, int x0, int x1) =>
                Enumerable.Range(x0, x1 - x0).Select(x => Mean(x, y, 1, h)[channel]).ToArray();

            public int Largest => rgba.Where((_, i) => i % 4 != 3).Max();
        }

        // The device the CRT's pictures are drawn on: the one named, the RX 6800 otherwise, or none, when the cases below return without drawing.
        private static readonly Lazy<string?> CrtDevice = new(() =>
        {
            string name = Environment.GetEnvironmentVariable(ShaderBenchTests.GlVariable) ?? "6800";
            try { using var gl = ShaderBench.GlContext.Create(name); return name; }
            catch (Exception) { return null; }
        });

        // Glass, persistence, convergence and curvature off, so a case measures the one thing it turns back on.
        private static readonly (string, float)[] Plain = { ("persistence", 0f), ("glare", 0f), ("convergence", 0f), ("curvature", 0f) };

        private static Crt? Draw(string console, int sourceWidth, int sourceHeight, int windowWidth, int windowHeight, IReadOnlyList<byte[]?> frames, params (string Id, float Value)[] set) =>
            Draw(console, sourceWidth, sourceHeight, 1, windowWidth, windowHeight, frames, set);

        private static Crt? Draw(string console, int sourceWidth, int sourceHeight, int rowRepeat, int windowWidth, int windowHeight, IReadOnlyList<byte[]?> frames, params (string Id, float Value)[] set)
        {
            if (CrtDevice.Value is not { } device) return null;
            var control = new GameFrameControl { FilterConsole = console, ShaderParameters = Values(set), ActiveFilter = CrtFilter.Filter };
            return new Crt(ShaderBench.Picture(control, frames, sourceWidth, sourceHeight, rowRepeat, windowWidth, windowHeight, device), windowWidth, windowHeight);
        }

        private static (string, float)[] With((string, float)[] set, params (string, float)[] more) => set.Concat(more).ToArray();

        // A later value for an id replaces an earlier one, so a case can start from another's settings.
        private static Dictionary<string, float> Values(IEnumerable<(string Id, float Value)> set)
        {
            var values = new Dictionary<string, float>();
            foreach (var (id, value) in set) values[id] = value;
            return values;
        }

        private static byte[] Paint(int width, int height, Func<int, int, (double R, double G, double B)> colour)
        {
            var frame = new byte[width * height * 4];
            for (int y = 0; y < height; y++) for (int x = 0; x < width; x++)
            {
                var (r, g, b) = colour(x, y);
                int i = (y * width + x) * 4;
                (frame[i], frame[i + 1], frame[i + 2], frame[i + 3]) = ((byte)Math.Round(255 * r), (byte)Math.Round(255 * g), (byte)Math.Round(255 * b), 255);
            }
            return frame;
        }

        private static byte[] Grey(int width, int height, double v) => Paint(width, height, (_, _) => (v, v, v));

        private static byte[][] Twice(byte[] frame) => new[] { frame, frame };

        // The tube's light for a drive voltage, the transfer the filter is built on.
        private static double Tube(double v)
        {
            double lb = Math.Pow(CrtFilter.BlackLevel, 1 / 2.4), b = lb / (1 - lb);
            return Math.Pow((v + b) / (1 + b), 2.4);
        }

        // Where a console's picture lies in a window with the glass flat: the standard raster zoomed to show the whole picture, as the filter places it.
        private static (double Left, double Top, double Width, double Height) PictureIn(string console, int rows, int windowWidth, int windowHeight, double overscan = 0)
        {
            double fillX = CrtFilter.ConsoleTiming[console]["activeUs"] / CrtFilter.StandardActiveMicroseconds;
            double fillY = (rows > 300 ? rows / 2.0 : rows) / CrtFilter.StandardFieldLines;
            double zoom = overscan > 0 ? 1 + overscan / 100 : Math.Min(1 / fillX, 1 / fillY);
            var (x, y, w, h) = GameFrameControl.ComputeLetterboxRect(4, 3, windowWidth, windowHeight);
            return (x + w * (0.5 - 0.5 * fillX * zoom), y + h * (0.5 - 0.5 * fillY * zoom), w * fillX * zoom, h * fillY * zoom);
        }

        private static void Within(double expected, double actual, double share, string what) =>
            Assert.True(Math.Abs(actual - expected) <= share * Math.Abs(expected), $"{what}: expected {expected:G5} within {share:P0}, got {actual:G5}");

        [Fact]
        public void Every_variant_of_the_crt_compiles_for_every_console()
        {
            Type chainType = typeof(GameFrameControl).Assembly.GetType("EmuSen.Serenity.Shaders.FilterChain")!;
            foreach (string? console in new[] { "NES", "SNES", "Genesis", "N64", null })
                for (int signal = 0; signal < 3; signal++) for (int mask = 0; mask < 5; mask++) for (int decoder = 0; decoder < 3; decoder++)
                    foreach (int screen in signal == 2 && mask == 0 ? Enumerable.Range(0, CrtFilter.Screens.Count) : new[] { CrtFilter.DefaultScreen })
                    {
                        using var chain = (IDisposable)Activator.CreateInstance(chainType, CrtFilter.Filter, console)!;
                        chainType.GetMethod("SetParameters")!.Invoke(chain, new object[] { new Dictionary<string, float> { ["signal"] = signal, ["mask"] = mask, ["decoder"] = decoder, ["screen"] = screen } });
                        Assert.True((int)chainType.GetProperty("PassCount")!.GetValue(chain)! >= 8);
                    }
        }

        [Fact]
        public void The_crt_suits_the_consoles_that_had_a_television_draws_a_4_to_3_tube_and_counts_scanlines()
        {
            Assert.Equal(new[] { "NES", "SNES", "Genesis", "N64" }, CrtFilter.Filter.Consoles);
            Assert.Equal(4.0 / 3.0, CrtFilter.Filter.Aspect);
            Assert.True(CrtFilter.Filter.RowsOnce);
            foreach (string console in CrtFilter.Filter.Consoles!) Assert.True(CrtFilter.ConsoleTiming.ContainsKey(console), $"{console} has no timing");
        }

        // Through the headless raster path, on a frame small enough for it: a white field's light is the tube's white over the display's peak, and neutral.
        [Fact]
        public Task Drawn_in_software_a_white_field_keeps_its_light_and_its_colour() => Session.Dispatch(() =>
        {
            var control = new GameFrameControl { FilterConsole = "SNES", ActiveFilter = CrtFilter.Filter };
            control.ShaderParameters = Values(With(Plain, ("signal", 0f), ("overscan", 900f)));
            var window = new Avalonia.Controls.Window { Width = 240, Height = 180, Content = control };
            window.Show();
            try
            {
                byte[] frame = Grey(32, 12, 1);
                control.UpdateFrame(frame, 32, 12);
                using (window.CaptureRenderedFrame()) { }
                control.UpdateFrame(frame, 32, 12);
                using var captured = window.CaptureRenderedFrame()!;
                var capture = EmuSen.WiseMan.Fixtures.UiTest.Capture(captured);
                var picture = new Crt(capture.Rgba, capture.Width, capture.Height);

                // 241.5 lines over ten times the window's height: a whole number of scanlines is 74.5 pixels for ten of them.
                double[] mean = picture.Mean(60, 53, 120, 74.5);
                foreach (double channel in mean) Within(1.0 / 3.0, channel, 0.03, "a white field's light in software");
                Assert.True(mean.Max() - mean.Min() < 0.01, $"neutral: {string.Join(", ", mean)}");
            }
            finally
            {
                window.Close();
            }
        }, default);

        [Fact]
        public Task A_flat_field_keeps_its_light_and_its_colour_at_every_window_size_on_every_mask() => Session.Dispatch(() =>
        {
            foreach (var (width, height) in new[] { (1067, 800), (1440, 1080), (1600, 1200), (2880, 2160) })
                foreach (int screen in new[] { 0, 1, 2, 4, 5 })
                    foreach (double level in new[] { 1.0, 0.5 })
                    {
                        if (level < 1 && (screen != 1 || width != 1440)) continue;
                        if (Draw("SNES", 256, 224, width, height, Twice(Grey(256, 224, level)), With(Plain, ("signal", 0f), ("screen", screen))) is not { } picture) return;
                        double[] mean = picture.Mean(width / 4.0, height / 4.0, width / 2.0, height / 2.0);
                        string what = $"{CrtFilter.Screens[screen].Label} at {width}x{height}, level {level}";
                        foreach (double channel in mean) Within(Tube(level) / 3, channel, 0.02, what);
                        Assert.True(mean.Max() - mean.Min() < 0.015 * mean.Average(), $"{what} is tinted: {string.Join(", ", mean.Select(m => m.ToString("F4")))}");
                    }
        }, default);

        // Magnified ten times by the overscan, a single lit line's width at half its height is the spot's at that drive, and its light is the line's.
        [Fact]
        public Task A_scanline_is_as_wide_as_its_drive_makes_the_spot_and_carries_the_same_light_at_any_width() => Session.Dispatch(() =>
        {
            CrtFilter.Screen screen = CrtFilter.Screens[CrtFilter.DefaultScreen];
            double pixelsPerLine = 480 * 10 / CrtFilter.StandardFieldLines;
            var widths = new List<double>();
            foreach (double level in new[] { 0.25, 0.5, 1.0 })
            {
                byte[] frame = Paint(256, 224, (_, y) => y == 112 ? (level, level, level) : (0, 0, 0));
                float headroom = level < 1 ? 1f : 3f;
                if (Draw("SNES", 256, 224, 640, 480, Twice(frame), With(Plain, ("signal", 0f), ("mask", 1f), ("subpixels", 0f), ("overscan", 900f), ("displayNits", 100f * headroom))) is not { } picture) return;
                double[] column = Enumerable.Range(0, 480).Select(y => picture.Mean(316, y, 8, 1)[1]).ToArray();

                // The unlit lines glow at the tube's black, which is not this line's light.
                double floor = column.Take(120).Average();
                column = column.Select(v => v - floor).ToArray();
                double peak = column.Max(), half = peak / 2;
                int top = Array.IndexOf(column, peak), a = top, b = top;
                while (column[a] > half) a--;
                while (column[b] > half) b++;
                double width = (b - 1 + (column[b - 1] - half) / (column[b - 1] - column[b])) - (a + (half - column[a]) / (column[a + 1] - column[a]));
                double light = Tube(level), sigma = Math.Sqrt(screen.SpotMinimum * screen.SpotMinimum + (screen.SpotMaximum * screen.SpotMaximum - screen.SpotMinimum * screen.SpotMinimum) * light);
                Within(sigma, width / pixelsPerLine / 2.3548, 0.05, $"the spot at drive {level}");
                Within(light / headroom, column.Sum() / pixelsPerLine, 0.03, $"the line's light at drive {level}");
                widths.Add(width);
            }
            Assert.True(widths[2] > 1.7 * widths[0], $"a bright line is fatter than a dim one: {widths[0]:F2} and {widths[2]:F2} pixels");
        }, default);

        // Counted along a scanline's centre, the mask's triads are the screen's own number across the picture, whatever the window; the stripes run red, green, blue.
        [Fact]
        public Task The_mask_s_pitch_is_the_screen_s_at_any_window_size_and_its_stripes_run_red_green_blue() => Session.Dispatch(() =>
        {
            foreach (var (width, height, pitch) in new[] { (2880, 2160, 1f), (1440, 1080, 2f), (2000, 1500, 1.5f) })
            {
                if (Draw("SNES", 256, 224, width, height, Twice(Grey(256, 224, 0.6)), With(Plain, ("signal", 0f), ("screen", 2f), ("maskPitch", pitch), ("displayNits", 1000f))) is not { } picture) return;
                double[] green = picture.Across(1, height / 2.0 - 20, 40, width / 4, 3 * width / 4);
                double mean = green.Average();
                int crossings = Enumerable.Range(1, green.Length - 1).Count(i => green[i - 1] < mean && green[i] >= mean);
                Within(CrtFilter.Screens[2].TriadsAcross / pitch / 2, crossings, 0.02, $"triads across half of {width} pixels at pitch {pitch}");
            }

            if (Draw("SNES", 256, 224, 2880, 2160, Twice(Grey(256, 224, 0.6)), With(Plain, ("signal", 0f), ("screen", 2f), ("maskPitch", 3f), ("subpixels", 0f), ("displayNits", 1000f))) is not { } coarse) return;
            double period = 2880 / (CrtFilter.Screens[2].TriadsAcross / 3);
            double[][] rows = Enumerable.Range(0, 3).Select(c => coarse.Across(c, 1060, 40, 1000, 1000 + (int)(period * 6))).ToArray();
            double[] peaks = rows.Select(row => Enumerable.Range(0, (int)period).OrderByDescending(i => row[i] + row[i + (int)Math.Round(period)]).First() / period).ToArray();
            double redToGreen = (peaks[1] - peaks[0] + 1) % 1, greenToBlue = (peaks[2] - peaks[1] + 1) % 1;
            Assert.True(Math.Abs(redToGreen - 1.0 / 3) < 0.1 && Math.Abs(greenToBlue - 1.0 / 3) < 0.1, $"red, green, blue a third of a triad apart: {redToGreen:F2}, {greenToBlue:F2}");

            // The photographed grille's stripes are lit for 0.199 of a triad.
            double[] stripes = coarse.Across(1, 1060, 40, 1000, 1000 + (int)(period * 20));
            double half = (stripes.Max() + stripes.Min()) / 2;
            Within(0.199, stripes.Count(v => v > half) / (double)stripes.Length, 0.15, "the share of a triad that one colour's stripe is lit for");
        }, default);

        // Drawn three times its size so the display resolves it, the slot mask has the photographed one's proportions: slots 0.81 of a triad apart, lit for 0.76 of that, the next triad's half a pitch down.
        [Fact]
        public Task A_slot_mask_s_slots_are_as_long_and_as_staggered_as_the_photographed_tube_s() => Session.Dispatch(() =>
        {
            // A woven 448-row picture has lines at half the pitch, so the beam is nearly even down the screen and what is measured is the mask.
            (string, float)[] set = With(Plain, ("signal", 0f), ("screen", 1f), ("maskPitch", 3f), ("subpixels", 0f), ("spotSize", 2f), ("displayNits", 1000f));
            if (Draw("SNES", 256, 448, 2880, 2160, Twice(Grey(256, 448, 0.6)), set) is not { } picture) return;
            double triad = 2880 / (CrtFilter.Screens[1].TriadsAcross / 3), slot = triad * 0.81;

            // Green's stripe in the triad nearest the centre, and in the one beside it.
            double[] across = picture.Across(1, 900, 360, 1400, 1400 + (int)(2 * triad) + 2);
            int first = Enumerable.Range(0, (int)triad).OrderByDescending(i => across[i]).First();
            double[] Down(double x) => Enumerable.Range(900, 360).Select(y => picture.Mean(x - 1, y, 3, 1)[1]).ToArray();
            double[] here = Down(1400 + first), beside = Down(1400 + first + triad);
            Within(slot, Period(here, (int)(slot * 0.6), (int)(slot * 1.6)), 0.06, "the slots' pitch in pixels");
            double half = (here.Max() + here.Min()) / 2;
            Within(0.76, here.Count(v => v > half) / (double)here.Length, 0.1, "the share of a slot's pitch that is lit");
            double mean = here.Average(), shifted = Enumerable.Range(0, here.Length - (int)slot).Sum(i => (here[i] - mean) * (beside[i + (int)Math.Round(slot / 2)] - mean)), level = Enumerable.Range(0, here.Length - (int)slot).Sum(i => (here[i] - mean) * (beside[i] - mean));
            Assert.True(shifted > 0 && level < 0, $"the next triad's slots are half a pitch down: correlation {shifted:F3} when shifted, {level:F3} when not");
        }, default);

        // A professional monitor's grille is finer than a 1280x800 display's pixels: it must fade to its mean, leaving the scanlines and no tint.
        [Fact]
        public Task A_mask_too_fine_for_the_display_fades_out_without_a_tint() => Session.Dispatch(() =>
        {
            if (Draw("SNES", 256, 224, 1067, 800, Twice(Grey(256, 224, 0.6)), With(Plain, ("signal", 0f), ("screen", 4f), ("displayNits", 1000f))) is not { } fine) return;
            if (Draw("SNES", 256, 224, 1067, 800, Twice(Grey(256, 224, 0.6)), With(Plain, ("signal", 0f), ("screen", 4f), ("maskPitch", 3f), ("displayNits", 1000f))) is not { } coarse) return;
            double Ripple(Crt picture) { double[] row = picture.Across(1, 380, 40, 300, 760); double mean = row.Average(); return Math.Sqrt(row.Average(v => (v - mean) * (v - mean))) / mean; }
            Assert.True(Ripple(fine) < 0.06 && Ripple(coarse) > 5 * Ripple(fine), $"ripple along a line: {Ripple(fine):F3} at the screen's pitch, {Ripple(coarse):F3} at three times it");
            double[] mean = fine.Mean(266, 200, 534, 400);
            Assert.True(mean.Max() - mean.Min() < 0.015 * mean.Average(), $"tinted: {string.Join(", ", mean.Select(m => m.ToString("F4")))}");
        }, default);

        // A display with no headroom cannot draw a mask that needs seven times the mean: the mask is flattened, nothing clips, and the light is unchanged.
        [Fact]
        public Task Without_headroom_the_mask_gives_way_before_the_picture_dims_or_clips() => Session.Dispatch(() =>
        {
            byte[][] grey = Twice(Grey(256, 224, 0.75));
            if (Draw("SNES", 256, 224, 1440, 1080, grey, With(Plain, ("signal", 0f), ("spotSize", 2f), ("spotGrowth", 0f), ("displayNits", 100f), ("tubeNits", 100f))) is not { } tight) return;
            if (Draw("SNES", 256, 224, 1440, 1080, grey, With(Plain, ("signal", 0f), ("spotSize", 2f), ("spotGrowth", 0f), ("displayNits", 1000f), ("tubeNits", 100f))) is not { } roomy) return;
            double Depth(Crt picture) { double[] row = picture.Across(1, 520, 40, 400, 1040); return (row.Max() - row.Min()) / row.Average(); }
            Within(Tube(0.75), tight.Mean(360, 270, 720, 540).Average(), 0.02, "the light with no headroom");
            Within(Tube(0.75) / 10, roomy.Mean(360, 270, 720, 540).Average(), 0.02, "the light with ten times the headroom");
            Assert.True(tight.Largest < 255, "nothing clips");
            Assert.True(Depth(roomy) > 3 * Depth(tight), $"the mask is deeper with headroom: {Depth(roomy):F2} against {Depth(tight):F2}");
        }, default);

        private static byte[] Bars(int width, int height) => Paint(width, height, (x, _) =>
        {
            int bar = x * 8 / width;
            return (bar is 0 or 1 or 4 or 5 ? 0.75 : 0, bar <= 3 ? 0.75 : 0, bar % 2 == 0 && bar < 7 ? 0.75 : 0);
        });

        private static double[][] BarColours(Crt picture, string console)
        {
            var (left, top, width, height) = PictureIn(console, 224, picture.Width, picture.Height);
            double line = height / 224;
            return Enumerable.Range(0, 8).Select(bar => picture.Mean(left + width * (bar + 0.5) / 8 - 20, top + height / 2 - 20 * line, 40, 40 * line)).ToArray();
        }

        // Colour bars come back from composite and S-Video as they went in by RGB, and from a comb on a console whose phase is not the standard's.
        [Fact]
        public Task Colour_bars_survive_composite_s_video_and_a_comb_on_each_console_s_phase() => Session.Dispatch(() =>
        {
            foreach (string console in new[] { "SNES", "Genesis", "N64" })
            {
                (string, float)[] flat = With(Plain, ("mask", 1f), ("spotGrowth", 0f));
                if (Draw(console, 256, 224, 1440, 1080, Twice(Bars(256, 224)), With(flat, ("signal", 0f))) is not { } rgb) return;
                double[][] wanted = BarColours(rgb, console);
                Within(Tube(0.75) / 3, wanted[0][1], 0.03, $"{console}: white's green by RGB");
                Assert.True(wanted[5][0] > 20 * wanted[5][2] && wanted[6][2] > 20 * wanted[6][0], "red is red and blue is blue");

                foreach (var (label, set) in new[] { ("S-Video", With(flat, ("signal", 1f))), ("composite, notch", With(flat, ("signal", 2f), ("decoder", 1f))), ("composite, comb", With(flat, ("signal", 2f), ("decoder", 2f))) })
                {
                    double[][] got = BarColours(Draw(console, 256, 224, 1440, 1080, Twice(Bars(256, 224)), set)!, console);
                    for (int bar = 0; bar < 8; bar++) for (int c = 0; c < 3; c++)
                        Assert.True(Math.Abs(got[bar][c] - wanted[bar][c]) < 0.004, $"{console}, {label}, bar {bar}, channel {c}: {got[bar][c]:F4} against {wanted[bar][c]:F4} by RGB");
                }
            }
        }, default);

        // Across a step between two colours of one luma, chroma rises in 0.35 over its bandwidth, and four times as fast on a professional monitor's wider channel.
        [Fact]
        public Task Chroma_rises_as_slowly_as_its_bandwidth_says() => Session.Dispatch(() =>
        {
            byte[] step = Paint(256, 224, (x, _) => x < 128 ? (0.2, 0.5, 0.2) : (0.714, 0.2, 0.714));
            double Rise(Crt picture)
            {
                var (left, top, width, height) = PictureIn("SNES", 224, 1440, 1080);
                double[] red = picture.Across(0, top + height / 2 - 20, 40, (int)(left + width * 0.3), (int)(left + width * 0.7));
                double low = red.Take(40).Average(), high = red.Skip(red.Length - 40).Average();
                double At(double share) { double level = low + (high - low) * share; int i = Array.FindIndex(red, v => v >= level); return i - 1 + (level - red[i - 1]) / (red[i] - red[i - 1]); }
                return (At(0.9) - At(0.1)) / width * CrtFilter.ConsoleTiming["SNES"]["activeUs"];
            }
            (string, float)[] flat = With(Plain, ("mask", 1f), ("spotGrowth", 0f), ("signal", 2f), ("gamma", 1f));
            if (Draw("SNES", 256, 224, 1440, 1080, Twice(step), With(flat, ("screen", 1f))) is not { } consumer) return;
            if (Draw("SNES", 256, 224, 1440, 1080, Twice(step), With(flat, ("screen", 4f))) is not { } professional) return;
            Within(0.35 / 0.5, Rise(consumer), 0.2, "microseconds from 10% to 90% through a 0.5 MHz channel");
            Assert.True(Rise(professional) < 0.6 * Rise(consumer), $"a wider channel rises faster: {Rise(professional):F2} against {Rise(consumer):F2} microseconds");
        }, default);

        // Thin vertical stripes turn to colour that follows the subcarrier: the lines and frames after which it repeats are each console's own.
        [Theory]
        [InlineData("NES", 3, 2)]
        [InlineData("SNES", 3, 2)]
        [InlineData("Genesis", 1, 1)]
        [InlineData("N64", 2, 2)]
        public Task A_console_s_artifacts_repeat_after_its_own_number_of_lines_and_frames(string console, int lines, int frames) => Session.Dispatch(() =>
        {
            // Five display pixels to a line, so a line's mean is taken over exactly its own pitch.
            int width = console == "N64" ? 640 : 256, stripe = console == "N64" ? 2 : 1, rows = console == "N64" ? 240 : 224, tall = console == "N64" ? 1206 : 1120;
            byte[] columns = Paint(width, rows, (x, _) => x % (stripe + 1) == 0 ? (0.8, 0.8, 0.8) : (0.1, 0.1, 0.1));
            (string, float)[] set = With(Plain, ("mask", 1f), ("spotGrowth", 0f), ("signal", 2f), ("decoder", 1f));
            double[][] Lines(int shown)
            {
                Crt picture = Draw(console, width, rows, 1600, tall, Enumerable.Repeat(columns, shown).ToArray(), set)!;
                var (left, top, w, h) = PictureIn(console, rows, 1600, tall);
                return Enumerable.Range(100, 6).Select(row => picture.Mean(left + w * 0.5 - 2, Math.Round(top + h * row / rows), 4, 5)).ToArray();
            }
            static double Apart(double[] a, double[] b) => Enumerable.Range(0, 3).Max(c => Math.Abs(a[c] - b[c]));
            if (CrtDevice.Value is null) return;

            double[][] first = Lines(4), next = Lines(5), after = Lines(6);
            double noise = 0.004;
            for (int row = 0; row + lines < 6; row++) Assert.True(Apart(first[row], first[row + lines]) < noise, $"{console}: line {row} and the one {lines} below differ by {Apart(first[row], first[row + lines]):F4}");
            for (int gap = 1; gap < lines; gap++) Assert.True(Apart(first[0], first[gap]) > 5 * noise, $"{console}: lines {gap} apart are the same");
            double[][] repeat = frames == 1 ? next : after;
            for (int row = 0; row < 6; row++) Assert.True(Apart(first[row], repeat[row]) < noise, $"{console}: the frame {frames} later differs by {Apart(first[row], repeat[row]):F4}");
            if (frames > 1) Assert.True(Apart(first[0], next[0]) > 5 * noise, $"{console}: the next frame is the same");
        }, default);

        private static double Period(double[] row, int from, int to)
        {
            double mean = row.Average();
            return Enumerable.Range(from, to - from).OrderByDescending(lag => Enumerable.Range(0, row.Length - lag).Sum(i => (row[i] - mean) * (row[i + lag] - mean)) / (row.Length - lag)).First();
        }

        // Alternate columns at 320 wide sit 0.22 MHz under the subcarrier: the receiver blends their luma and paints a rainbow thirty pixels long, a third as strong at 256 wide.
        [Fact]
        public Task The_genesis_s_dithered_columns_blend_into_a_static_rainbow_thirty_pixels_long() => Session.Dispatch(() =>
        {
            (string, float)[] set = With(Plain, ("mask", 1f), ("spotGrowth", 0f), ("signal", 2f));
            byte[] Columns(int width) => Paint(width, 224, (x, _) => x % 2 == 0 ? (0.6, 0.6, 0.6) : (0.2, 0.2, 0.2));
            if (Draw("Genesis", 320, 224, 1440, 1080, Twice(Columns(320)), set) is not { } wide) return;
            if (Draw("Genesis", 320, 224, 1440, 1080, Twice(Columns(320)), With(set, ("signal", 0f))) is not { } rgb) return;
            var (left, top, width, height) = PictureIn("Genesis", 224, 1440, 1080);
            int x0 = (int)(left + width * 0.2), x1 = (int)(left + width * 0.8);
            static double Luma(double[] m) => 0.2126 * m[0] + 0.7152 * m[1] + 0.0722 * m[2];

            // The columns' own swing, taken inside windows two source pixels long, which the rainbow's slow turn does not enter.
            double Columns2(Crt picture)
            {
                double[] row = Enumerable.Range(x0, x1 - x0).Select(x => Luma(picture.Mean(x, top + height / 2 - 20, 1, 40))).ToArray();
                int window = (int)Math.Round(2 * width / 320);
                return Enumerable.Range(0, row.Length / window).Average(k => (row.Skip(k * window).Take(window).Max() - row.Skip(k * window).Take(window).Min()) / row.Skip(k * window).Take(window).Average());
            }

            Within(30 * width / 320, Period(wide.Across(2, top + height / 2 - 20, 40, x0, x1), 60, 220), 0.04, "the rainbow's length in output pixels");
            Assert.True(Columns2(wide) < 0.1 && Columns2(rgb) > 8 * Columns2(wide), $"the columns blend through composite: swing {Columns2(wide):F3} against {Columns2(rgb):F3} by RGB");

            // The strength is compared on faint columns and a straight-line tube, where chroma stays inside the gamut and adds to blue in proportion.
            byte[] Faint(int across) => Paint(across, 224, (x, _) => x % 2 == 0 ? (0.45, 0.45, 0.45) : (0.35, 0.35, 0.35));
            double Blue(int across)
            {
                double[] row = Draw("Genesis", across, 224, 1440, 1080, Twice(Faint(across)), With(set, ("gamma", 1f)))!.Across(2, top + height / 2 - 20, 40, x0, x1);
                double mean = row.Average();
                return Math.Sqrt(row.Average(v => (v - mean) * (v - mean)));
            }
            double ratio = Blue(320) / Blue(256);
            Assert.True(ratio > 2.2 && ratio < 3.8, $"the rainbow at 320 wide is {ratio:F2} times the one at 256; the chroma filter's response at the two beats gives 2.9");
        }, default);

        // The SNES's 512-wide alternate columns, its way of drawing transparency: a consumer set by composite blends most of the way, a professional monitor by RGB does not.
        [Fact]
        public Task Pseudo_hi_res_columns_blend_by_composite_on_a_consumer_set_and_stay_apart_by_rgb_on_a_monitor() => Session.Dispatch(() =>
        {
            byte[] columns = Paint(512, 224, (x, _) => x % 2 == 0 ? (0.7, 0.7, 0.7) : (0.3, 0.3, 0.3));
            (string, float)[] flat = With(Plain, ("mask", 1f));
            if (Draw("SNES", 512, 224, 2880, 2160, Twice(columns), With(flat, ("signal", 2f), ("screen", 1f))) is not { } composite) return;
            if (Draw("SNES", 512, 224, 2880, 2160, Twice(columns), With(flat, ("signal", 0f), ("screen", 4f))) is not { } rgb) return;
            var (left, top, width, height) = PictureIn("SNES", 224, 2880, 2160);
            double Swing(Crt picture) { double[] row = picture.Across(1, top + height / 2 - 40, 80, (int)(left + width * 0.3), (int)(left + width * 0.7)); return (row.Max() - row.Min()) / row.Average(); }
            Assert.True(Swing(composite) < 0.35 && Swing(rgb) > 3 * Swing(composite), $"swing between columns: {Swing(composite):F2} by composite on the consumer set, {Swing(rgb):F2} by RGB on the monitor");
        }, default);

        private static double Contrast(Crt lit, Crt dark, string console, int squares, int rows)
        {
            var (left, top, width, height) = PictureIn(console, rows, lit.Width, lit.Height);
            double line = height / rows, white = 0, black = 0;
            for (int j = 0; j < squares; j++) for (int i = 0; i < squares; i++)
            {
                double tall = Math.Max(1, Math.Floor(height / squares * 0.2 / line)) * line;
                double x = left + width * (i + 0.4) / squares, y = Math.Round(top + height * (j + 0.5) / squares - tall / 2);
                if ((i + j) % 2 == 0) white += lit.Luminance(x, y, width * 0.2 / squares, tall);
                else black += dark.Luminance(x, y, width * 0.2 / squares, tall);
            }
            return white / black;
        }

        // DisplayMate measured a PVM-20L5 at 219:1 on a 4x4 checkerboard and 75:1 on a 9x9; the glare's one Gaussian is solved from those two and must give them back.
        [Fact]
        public Task A_checkerboard_s_black_squares_are_lit_through_the_glass_as_the_measured_monitor_s_were() => Session.Dispatch(() =>
        {
            foreach (var (squares, width, height, measured) in new[] { (4, 256, 224, 219.0), (9, 288, 216, 75.0) })
            {
                byte[][] board = Twice(Paint(width, height, (x, y) => (x * squares / width + y * squares / height) % 2 == 0 ? (1.0, 1.0, 1.0) : (0.0, 0.0, 0.0)));
                (string, float)[] set = With(Plain.Where(p => p.Item1 != "glare").ToArray(), ("mask", 1f), ("signal", 0f), ("screen", 4f));
                if (Draw("SNES", width, height, 1440, 1080, board, set) is not { } lit) return;

                // The black squares are read from a picture three times as bright, where 8 bits resolve them; the white ones clip there.
                if (Draw("SNES", width, height, 1440, 1080, board, With(set, ("displayNits", 100f))) is not { } dark) return;
                Within(measured, 3 * Contrast(lit, dark, "SNES", squares, height), 0.08, $"contrast on a {squares}x{squares} checkerboard");
            }
        }, default);

        // After a long white, the first dark frame keeps 4% of the green, a little less of the blue and none of the red; and the tail falls slowly, not by half each frame.
        [Fact]
        public Task A_white_that_goes_dark_leaves_green_and_blue_behind_for_frames_and_no_red() => Session.Dispatch(() =>
        {
            byte[] white = Grey(256, 224, 1), black = Grey(256, 224, 0);
            (string, float)[] set = With(Plain.Where(p => p.Item1 != "persistence").ToArray(), ("mask", 1f), ("signal", 0f), ("subpixels", 0f), ("gamut", 0f), ("displayNits", 100f));
            double[] After(int dark)
            {
                Crt picture = Draw("SNES", 256, 224, 640, 480, Enumerable.Repeat(white, 150).Concat(Enumerable.Repeat(black, dark)).ToArray(), set)!;
                var (left, top, width, height) = PictureIn("SNES", 224, 640, 480);
                return picture.Mean(left + width / 2 - 20, Math.Round(top + height * 0.4), 40, Math.Round(height / 224 * 20));
            }
            if (CrtDevice.Value is null) return;
            double[] first = After(1), third = After(3), tenth = After(10);
            Within(0.04, first[1], 0.15, "green in the first dark frame");
            Within(0.04 * CrtFilter.BlueSlow / CrtFilter.GreenSlow, first[2], 0.2, "blue in the first dark frame");
            Assert.True(first[0] < 0.004, $"red is gone: {first[0]:F4}");
            Assert.True(third[1] > 0.5 * first[1] && tenth[1] > 0.25 * first[1] && tenth[1] < third[1], $"green falls slowly: {first[1]:F4}, {third[1]:F4}, {tenth[1]:F4}");
        }, default);

        // A one-pixel white line's red and blue are drawn either side of its green, by the screen's stated error at the centre and by more toward the edge.
        [Fact]
        public Task Red_and_blue_land_either_side_of_green_by_the_screen_s_convergence_error() => Session.Dispatch(() =>
        {
            byte[] lines = Paint(256, 224, (x, _) => x is 128 or 240 ? (1.0, 1.0, 1.0) : (0.0, 0.0, 0.0));
            (string, float)[] set = With(Plain.Where(p => p.Item1 != "convergence").ToArray(), ("mask", 1f), ("signal", 0f), ("subpixels", 0f), ("screen", 1f));
            if (Draw("SNES", 256, 224, 2880, 2160, Twice(lines), set) is not { } picture) return;
            var (left, top, width, height) = PictureIn("SNES", 224, 2880, 2160);
            double Centre(int channel, int column)
            {
                int x0 = (int)(left + width * (column + 0.5) / 256) - 40;
                double[] row = picture.Across(channel, top + height / 2 - 20, 40, x0, x0 + 80);
                return x0 + Enumerable.Range(0, 80).Sum(i => i * row[i]) / row.Sum();
            }
            CrtFilter.Screen screen = CrtFilter.Screens[1];
            double perMillimetre = 2880 / screen.WidthMillimetres;
            double centre = Centre(2, 128) - Centre(0, 128), edge = Centre(2, 240) - Centre(0, 240);
            Within(screen.ConvergenceCentre * perMillimetre, Math.Abs(centre), 0.15, "red to blue at the centre, in pixels");
            Assert.True(Math.Abs(edge) > 1.25 * Math.Abs(centre), $"the error grows toward the edge: {centre:F2} and {edge:F2} pixels");
            Assert.True(Math.Abs(Centre(1, 128) - (Centre(0, 128) + Centre(2, 128)) / 2) < 0.3, "green lies between");
        }, default);

        // Curved, the glass leaves the corners outside the picture and the centre where it was; flat, the corners are lit.
        [Fact]
        public Task The_curved_glass_darkens_the_corners_and_leaves_the_centre_alone() => Session.Dispatch(() =>
        {
            byte[][] white = Twice(Grey(640, 240, 1));
            (string, float)[] set = With(Plain.Where(p => p.Item1 != "curvature").ToArray(), ("mask", 1f), ("signal", 0f), ("spotSize", 2f));
            if (Draw("N64", 640, 240, 1440, 1080, white, With(set, ("curvature", 0f))) is not { } flat) return;
            if (Draw("N64", 640, 240, 1440, 1080, white, With(set, ("curvature", 1f))) is not { } curved) return;
            Assert.True(flat.Luminance(20, 20, 12, 12) > 0.2 && curved.Luminance(4, 4, 12, 12) < 0.01, $"the corner: {flat.Luminance(20, 20, 12, 12):F3} flat, {curved.Luminance(4, 4, 12, 12):F3} curved");
            Within(flat.Luminance(700, 520, 40, 40), curved.Luminance(700, 520, 40, 40), 0.02, "the centre");
            Assert.True(curved.Luminance(700, 6, 40, 18) > 0.2 && curved.Luminance(6, 520, 18, 40) > 0.2, "the middle of each edge is still picture");
        }, default);

        // A Japanese set's white is 9300 K: bluer than the display's own, by the ratio the two chromaticities give, and it is shown so.
        [Fact]
        public Task A_japanese_white_is_drawn_bluer_and_a_north_american_one_neutral() => Session.Dispatch(() =>
        {
            byte[][] white = Twice(Grey(256, 224, 1));
            (string, float)[] set = With(Plain, ("mask", 1f), ("signal", 0f));
            if (Draw("SNES", 256, 224, 640, 480, white, With(set, ("colour", 0f))) is not { } america) return;
            if (Draw("SNES", 256, 224, 640, 480, white, With(set, ("colour", 1f))) is not { } japan) return;
            double[] neutral = america.Mean(160, 120, 320, 240), blue = japan.Mean(160, 120, 320, 240);
            double[][] matrix = CrtFilter.GunsToDisplay(CrtFilter.Colours[1], CrtFilter.Displays[0]);
            Assert.True(neutral.Max() - neutral.Min() < 0.01 * neutral.Average(), "North America's white is the display's");
            Within(matrix[2].Sum() / matrix[0].Sum(), blue[2] / blue[0], 0.03, "Japan's blue over red");
            Assert.True(blue[2] / blue[0] > 1.3, $"and it is visibly blue: {blue[2] / blue[0]:F2}");
        }, default);

        // A 448-row picture is two fields: woven, every row is lit in every frame; as fields, alternate rows, and the other ones in the next frame.
        [Fact]
        public Task An_interlaced_picture_is_woven_or_drawn_a_field_at_a_time() => Session.Dispatch(() =>
        {
            byte[] white = Grey(256, 448, 1);
            (string, float)[] set = With(Plain, ("mask", 1f), ("signal", 0f), ("overscan", 900f), ("spotSize", 0.5f), ("displayNits", 1000f));
            double[] Rows(Crt picture) => Enumerable.Range(0, 6).Select(k => picture.Mean(300, 240 + (k - 2.5) * 480 * 10 / (2 * CrtFilter.StandardFieldLines) - 2, 40, 4)[1]).ToArray();
            if (Draw("SNES", 256, 448, 640, 480, new[] { white, white }, With(set, ("interlace", 0f))) is not { } woven) return;
            if (Draw("SNES", 256, 448, 640, 480, new[] { white, white }, With(set, ("interlace", 1f))) is not { } even) return;
            if (Draw("SNES", 256, 448, 640, 480, new[] { white, white, white }, With(set, ("interlace", 1f))) is not { } odd) return;
            double[] w = Rows(woven), a = Rows(even), b = Rows(odd);
            Assert.True(w.Min() > 0.8 * w.Max(), $"woven rows are alike: {string.Join(", ", w.Select(v => v.ToString("F3")))}");
            for (int k = 0; k < 5; k++) Assert.True(Math.Min(a[k], a[k + 1]) < 0.2 * Math.Max(a[k], a[k + 1]), $"a field lights alternate rows: {string.Join(", ", a.Select(v => v.ToString("F3")))}");
            for (int k = 0; k < 6; k++) Assert.True((a[k] > b[k]) != (a[(k + 1) % 6] > b[(k + 1) % 6]), "and the next frame lights the others");
        }, default);

        // The whole picture by default, with the blank either side of it; with a television's overscan the picture's edge is off the glass.
        [Fact]
        public Task The_whole_picture_is_shown_until_overscan_is_asked_for() => Session.Dispatch(() =>
        {
            byte[][] white = Twice(Grey(256, 224, 1));
            (string, float)[] set = With(Plain, ("mask", 1f), ("signal", 0f));
            if (Draw("SNES", 256, 224, 1440, 1080, white, set) is not { } whole) return;
            if (Draw("SNES", 256, 224, 1440, 1080, white, With(set, ("overscan", 12f))) is not { } cropped) return;
            var (left, _, _, _) = PictureIn("SNES", 224, 1440, 1080);
            Assert.True(left > 10 && whole.Luminance(2, 500, 6, 80) < 0.01 && whole.Luminance(left + 6, 500, 20, 80) > 0.2, $"the whole picture starts {left:F0} pixels in");
            Assert.True(cropped.Luminance(2, 500, 6, 80) > 0.2, "overscanned, it reaches the edge");
        }, default);

        // The N64's cores send 240 rows to be shown twice each; a tube draws them as 240 scanlines, so the repeat changes nothing.
        [Fact]
        public Task Rows_sent_to_be_repeated_are_drawn_as_the_same_scanlines() => Session.Dispatch(() =>
        {
            byte[] frame = Paint(640, 240, (x, y) => (x / 639.0, y / 239.0, 0.5));
            if (Draw("N64", 640, 240, 1, 960, 720, Twice(frame)) is not { } once) return;
            if (Draw("N64", 640, 240, 2, 960, 720, Twice(frame)) is not { } twice) return;
            int differing = 0;
            for (int y = 0; y < 720; y++) for (int x = 0; x < 960; x++) for (int c = 0; c < 3; c++) if (once.Code(x, y, c) != twice.Code(x, y, c)) differing++;
            Assert.Equal(0, differing);
            Assert.True(once.Luminance(400, 300, 160, 120) > 0.02, "and it is a picture");
        }, default);

        // The same picture through Skia's raster code and through the device: one shader, so one picture, to within a code value.
        [Fact]
        public Task The_device_and_the_software_path_draw_the_same_picture() => Session.Dispatch(() =>
        {
            byte[] frame = Paint(48, 16, (x, y) => (x / 47.0, y / 15.0, (x * 7 + y * 3) % 16 / 15.0));
            (string, float)[] set = { ("overscan", 300f) };
            if (Draw("Genesis", 48, 16, 240, 180, Twice(frame), set) is not { } device) return;
            var control = new GameFrameControl { FilterConsole = "Genesis", ShaderParameters = Values(set), ActiveFilter = CrtFilter.Filter };
            var window = new Avalonia.Controls.Window { Width = 240, Height = 180, Content = control };
            window.Show();
            try
            {
                control.UpdateFrame(frame, 48, 16);
                using (window.CaptureRenderedFrame()) { }
                control.UpdateFrame(frame, 48, 16);
                using var captured = window.CaptureRenderedFrame()!;
                var capture = EmuSen.WiseMan.Fixtures.UiTest.Capture(captured);
                int worst = 0, differing = 0;
                for (int y = 0; y < 180; y++) for (int x = 0; x < 240; x++) for (int c = 0; c < 3; c++)
                {
                    int d = Math.Abs(capture.Rgba[(y * capture.Width + x) * 4 + c] - device.Code(x, y, c));
                    worst = Math.Max(worst, d);
                    if (d > 0) differing++;
                }
                Assert.True(worst <= 2 && differing < 240 * 180 * 3 / 10, $"worst difference {worst}, {differing} of {240 * 180 * 3} values differ");
            }
            finally
            {
                window.Close();
            }
        }, default);
    }
}

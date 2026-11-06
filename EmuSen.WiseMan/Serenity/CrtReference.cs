using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // The CRT's physics as the tests derive their expected pictures from it, independent of the library that writes the filter - see EmuSen_Backend.md §40.6 and EmuSen_CRT.md.
    public static class CrtReference
    {
        // The signal's samples across the active picture, 12 to a subcarrier cycle on the 21.48 MHz consoles - see EmuSen_CRT.md §5.1.
        public const int SignalSamples = 2048;

        // The glare map's size, in cells across the tube's face - see EmuSen_CRT.md §11.6.
        public const int GlareWidth = 128, GlareHeight = 96;

        public enum Signal { Rgb, SVideo, Composite }
        public enum Mask { None, ApertureGrille, Slot, Dot }
        public enum Decoder { Notch, Comb }
        public enum Quality { Performance, Balanced, Accurate }

        // One class of set: its picture, its mask, its spot, its receiver and its glass, each sourced or marked a choice in EmuSen_CRT.md §11.2.
        public sealed record Screen(string Label, double WidthMillimetres, Mask Mask, double TriadsAcross, double SpotMinimum, double SpotMaximum,
            Decoder Decoder, double ChromaMegahertz, double VideoMegahertz, bool Cylinder, double RadiusOverWidth, double ConvergenceCentre, double ConvergenceEdge);

        public static IReadOnlyList<Screen> Screens { get; } = new Screen[]
        {
            new("Consumer TV, 14-inch", 264, Mask.Slot, 440, 0.158, 0.316, Decoder.Notch, 0.5, 4.2, false, 1.9, 0.6, 1.0),
            new("Consumer TV, 20-inch", 386, Mask.Slot, 515, 0.108, 0.217, Decoder.Notch, 0.5, 4.2, false, 1.9, 0.6, 1.0),
            new("Consumer Trinitron, 27-inch", 549, Mask.ApertureGrille, 732, 0.076, 0.153, Decoder.Comb, 0.5, 4.2, true, 2.5, 0.6, 1.0),
            new("Professional monitor, 14-inch", 266, Mask.ApertureGrille, 1064, 0.126, 0.252, Decoder.Comb, 1.3, 10, true, 2.5, 0.4, 0.5),
            new("Professional monitor, 20-inch", 386, Mask.ApertureGrille, 1245, 0.087, 0.173, Decoder.Comb, 1.3, 10, true, 2.5, 0.5, 0.7),
            new("PC monitor, 15-inch", 280, Mask.Dot, 1167, 0.09, 0.18, Decoder.Comb, 1.3, 30, false, 3.3, 0.3, 0.4),
        };

        public const int DefaultScreen = 1;

        // A phosphor set and its white, as chromaticities - see EmuSen_CRT.md §3.4.
        public sealed record Colour(string Label, double Rx, double Ry, double Gx, double Gy, double Bx, double By, double Wx, double Wy);

        public static IReadOnlyList<Colour> Colours { get; } = new Colour[]
        {
            new("North America", 0.630, 0.340, 0.310, 0.595, 0.155, 0.070, 0.3127, 0.3290),
            new("Japan", 0.618, 0.350, 0.280, 0.605, 0.152, 0.063, 0.281, 0.311),
            new("Europe", 0.640, 0.330, 0.290, 0.600, 0.150, 0.060, 0.3127, 0.3290),
        };

        // The display's own primaries, both with a D65 white.
        public static IReadOnlyList<Colour> Displays { get; } = new Colour[]
        {
            new("sRGB", 0.640, 0.330, 0.300, 0.600, 0.150, 0.060, 0.3127, 0.3290),
            new("Display P3", 0.680, 0.320, 0.265, 0.690, 0.150, 0.060, 0.3127, 0.3290),
        };

        // Two exponentials fitted to Kuhn's measured P22 tails, per frame ratio and weight relative to the frame struck - see EmuSen_CRT.md §11.4.
        public const double TailFast = 0.590, TailSlow = 0.945, GreenFast = 0.0679, GreenSlow = 0.0094, BlueFast = 0.0601, BlueSlow = 0.0081;

        // The two decays' ratios a frame and their weights, on a standard's field: fitted at NTSC's and carried to PAL's longer one by the exponentials' own form - see EmuSen_CRT.md §23.
        public sealed record Tails(double Fast, double Slow, double GreenFast, double GreenSlow, double BlueFast, double BlueSlow)
        {
            public static Tails For(bool pal)
            {
                if (!pal) return new(TailFast, TailSlow, CrtReference.GreenFast, CrtReference.GreenSlow, CrtReference.BlueFast, CrtReference.BlueSlow);
                double t = PalFieldMilliseconds / NtscFieldMilliseconds, fast = Math.Pow(TailFast, t), slow = Math.Pow(TailSlow, t);
                double kf = (1 - fast) / (1 - TailFast), ks = (1 - slow) / (1 - TailSlow);
                return new(fast, slow, CrtReference.GreenFast * kf, CrtReference.GreenSlow * ks, CrtReference.BlueFast * kf, CrtReference.BlueSlow * ks);
            }
        }

        // The tails' strength at the setting's 1: a sustained white leaves 4% of its green in the first dark frame, the figure the vision literature gives - see EmuSen_CRT.md §11.4.
        public const double TailScale = 0.1606;

        // The wide glare Gaussian solved from DisplayMate's two checkerboard contrasts, as a share of the picture's width and of the light - see EmuSen_CRT.md §11.6.
        public const double GlareSigma = 0.0668, GlareWeight = 0.0299;

        // A black of 0.01 cd/m² under a white of 176, the PVM-20L5's - see EmuSen_CRT.md §3.3.
        public const double BlackLevel = 0.01 / 176.0;

        // The standard's active line in microseconds, for a console the timing table does not name, and its subcarrier - see EmuSen_CRT.md §3.1.
        public const double StandardActiveMicroseconds = 52.66, SubcarrierMegahertz = 3.579545;


        // Where white sits on the display when the picture is drawn bright, as a share of its peak - see EmuSen_CRT.md §12.3.
        public const double BrightWhite = 0.6;

        // The most display pixels a triad is drawn across when the mask is kept fine on a large display - see EmuSen_CRT.md §18.
        public const double FinestTriadPixels = 3;

        // The fewest display pixels a mask is drawn across before it is let go, since below two the display's own grid beats against it - see EmuSen_CRT.md §20.
        public const double LeastTriadPixels = 1.5;

        // A demodulator's R-Y and G-Y axes, as angles from B-Y and gains against B-Y's, from the CXA2025AS datasheet's AXIS register; the first is the standard's matrix - see EmuSen_CRT.md §24.
        public sealed record Demodulator(string Label, double RedDegrees, double RedGain, double GreenDegrees, double GreenGain);

        public static IReadOnlyList<Demodulator> Demodulators { get; } = new Demodulator[]
        {
            new("The standard's", 90, 0.492 / 0.877, Math.Atan2(-0.299 / 0.587 * 0.492 / 0.877, -0.114 / 0.587) * 180 / Math.PI, Math.Sqrt(Math.Pow(0.114 / 0.587, 2) + Math.Pow(0.299 / 0.587 * 0.492 / 0.877, 2))),
            new("Sony CXA2025AS, US axis", 112, 0.83, 252, 0.30),
            new("Sony CXA2025AS, Japan axis", 95, 0.78, 240, 0.30),
        };

        // Rows of R-Y, G-Y and B-Y over B-Y and R-Y as the standard's axes give them: an axis at angle a with gain g reads g (B-Y cos a + R-Y sin a x 0.877 / 0.492).
        public static double[][] AxesMatrix(Demodulator d)
        {
            double k = 0.877 / 0.492, r = d.RedDegrees * Math.PI / 180, g = d.GreenDegrees * Math.PI / 180;
            return new[] { new[] { d.RedGain * Math.Cos(r), d.RedGain * k * Math.Sin(r) }, new[] { d.GreenGain * Math.Cos(g), d.GreenGain * k * Math.Sin(g) }, new[] { 1.0, 0.0 } };
        }

        // One standard's line on a console: its active picture, cycles across, phase moved a line, alternating and crawling between frames, and whether a frame turns PAL's switch over - see EmuSen_CRT.md §23.
        public sealed record LineTiming(double ActiveMicroseconds, double CyclesAcross, double PhaseLine, double PhaseFrame, double PhaseCrawl = 0, bool SwitchFrame = false);

        // A console's line in the standard chosen, from its constants.
        public static LineTiming LineOf(IReadOnlyDictionary<string, float> values, bool pal)
        {
            string prefix = pal ? "pal." : "";
            float Of(string key) => values.TryGetValue(prefix + key, out float v) ? v : 0;
            return new LineTiming(Of("activeUs"), Of("cyclesAcross"), Of("phaseLine"), Of("phaseFrame"), Of("phaseCrawl"), Of("switchFrame") > 0.5);
        }

        // PAL's subcarrier, its active line, and its line of 283.7516 cycles - see EmuSen_CRT.md §23.
        public const double PalSubcarrierMegahertz = 4.43361875, PalActiveMicroseconds = 52.0, PalCyclesPerLine = 283.7516;

        // The Sony CXA2025AS's luma trap: its lower -3 dB edge read at 2.70 MHz from the datasheet's CTRAP curve, a second-order notch at the subcarrier with that edge - see EmuSen_CRT.md §24.
        public const double TrapLowerEdgeMegahertz = 2.70;
        public static readonly double TrapQ = SubcarrierMegahertz * TrapLowerEdgeMegahertz / (SubcarrierMegahertz * SubcarrierMegahertz - TrapLowerEdgeMegahertz * TrapLowerEdgeMegahertz);
        private static readonly double TrapStepScale = 1 / Math.Sqrt(TrapQ * TrapQ - 0.25);

        // The analogue trap's gain at a frequency, for a centre on either standard with the same Q.
        public static double TrapGain(double megahertz, double centre) =>
            Math.Abs(centre * centre - megahertz * megahertz) / Math.Sqrt(Math.Pow(centre * centre - megahertz * megahertz, 2) + Math.Pow(megahertz * centre / TrapQ, 2));

        // The band the trap takes away, as taps back along a line: a bilinear notch with the analogue trap's centre and lower edge - see EmuSen_CRT.md §24.
        public static double[] TrapTaps(double samplesPerMicrosecond, double centre)
        {
            double c0 = Math.Tan(Math.PI * centre / samplesPerMicrosecond), c1 = Math.Tan(Math.PI * centre * TrapLowerEdgeMegahertz / SubcarrierMegahertz / samplesPerMicrosecond);
            double q = c0 * c1 / (c0 * c0 - c1 * c1), w0 = 2 * Math.PI * centre / samplesPerMicrosecond, alpha = Math.Sin(w0) / (2 * q), a0 = 1 + alpha;
            double b0 = 1 / a0, b1 = -2 * Math.Cos(w0) / a0, a1 = b1, a2 = (1 - alpha) / a0;
            var taps = new List<double>();
            double x1 = 0, x2 = 0, y1 = 0, y2 = 0;
            for (int n = 0; n < 256; n++)
            {
                double x = n == 0 ? 1 : 0, y = b0 * x + b1 * x1 + b0 * x2 - a1 * y1 - a2 * y2;
                (x2, x1, y2, y1) = (x1, x, y1, y);
                taps.Add(x - y);
                if (n > 2 && Math.Abs(y) < 1e-5 && Math.Abs(y2) < 1e-5) break;
            }
            return taps.ToArray();
        }

        // A field's length in milliseconds on each standard, for the phosphors' decay between frames - see EmuSen_CRT.md §23.
        public const double NtscFieldMilliseconds = 1000 / 59.94, PalFieldMilliseconds = 1000 / 50.0;

        // The TMS9929A's pixels against a PAL encoder's own crystal, the two clocks unrelated: subcarrier cycles a pixel of 10.738635 MHz over two - see EmuSen_CRT.md §31.3.
        public const double Sg1000PalCycles = PalSubcarrierMegahertz / (10.738635 / 2);

        // The PlayStation 2's PCRTC clock, four of BT.601's 13.5 MHz pixels, whose 858- and 864-pixel lines are the standards' own - see EmuSen_CRT.md §31.9.
        public const double Ps2VideoMegahertz = 54.0;

        private static int At(IReadOnlyDictionary<string, float> values, string id, int count) => Math.Clamp((int)MathF.Round(values[id]), 0, count - 1);

        // What the screen and the player's overrides come to: the mask drawn, the decoder used and the chroma channel's width in subcarrier cycles.
        public sealed record Resolved(Screen Screen, Signal Signal, Mask Mask, Decoder Decoder, int ChromaCycles, Colour Colour, Colour Display, Quality Quality, int Samples)
        {
            // The standard's line on this console, and on PAL whether the receiver averages each line's colour with the line before - see EmuSen_CRT.md §23.
            public LineTiming Line { get; init; } = new(StandardActiveMicroseconds, StandardActiveMicroseconds * SubcarrierMegahertz, 0.5, 0.5);
            public bool Pal { get; init; }
            public bool DelayLine { get; init; }
            public double LumaMegahertz { get; init; } = 5.0;
            public Demodulator Demodulator { get; init; } = Demodulators[0];
            public double EncoderChromaMegahertz { get; init; } = 1.3;
            public double Subcarrier => Pal ? PalSubcarrierMegahertz : SubcarrierMegahertz;
            public Tails Tail => Tails.For(Pal);
        }

        public static Resolved Resolve(IReadOnlyDictionary<string, float> values)
        {
            Screen screen = Screens[At(values, "screen", Screens.Count)];
            var quality = (Quality)At(values, "quality", 3);
            int mask = At(values, "mask", 5), decoder = At(values, "decoder", 3);
            double megahertz = screen.ChromaMegahertz * values["chroma"];
            bool pal = values.TryGetValue(TelevisionStandards.Parameter, out float standard) && TelevisionStandards.IsPal(standard);
            LineTiming line = LineOf(values, pal);

            // A two-line comb has nothing to subtract when a console repeats its phase every line, and a set with one falls back to its notch; PAL's comb is not modelled - see EmuSen_CRT.md §11.3 and §23.
            double phase = line.PhaseLine * 2 * Math.PI, combGain = (1 - Math.Cos(phase)) / 2;
            Decoder wanted = decoder == 0 ? screen.Decoder : (Decoder)(decoder - 1);
            return new Resolved(screen, (Signal)At(values, "signal", 3), mask == 0 ? screen.Mask : (Mask)(mask - 1),
                wanted == Decoder.Comb && (combGain < 0.25 || pal) ? Decoder.Notch : wanted,
                Math.Clamp((int)Math.Round(0.72 * (pal ? PalSubcarrierMegahertz : SubcarrierMegahertz) / megahertz), 1, 6), Colours[At(values, "colour", Colours.Count)], Displays[At(values, "gamut", Displays.Count)],
                quality, quality == Quality.Accurate ? SignalSamples : (int)Math.Round(4 * line.CyclesAcross))
            {
                Line = line,
                Pal = pal,
                DelayLine = pal && !(values.TryGetValue("palDecoder", out float simple) && simple > 0.5),
                LumaMegahertz = values.TryGetValue("lumaMHz", out float luma) ? luma : 5.0,
                Demodulator = pal ? Demodulators[0] : Demodulators[values.TryGetValue("axes", out float axes) ? Math.Clamp((int)MathF.Round(axes), 0, Demodulators.Count - 1) : 0],
                EncoderChromaMegahertz = values.TryGetValue("chromaMHz", out float chroma) ? chroma : 1.3,
            };
        }

        // The NES PPU's terminated levels, low and high for each row of the palette, its black and its 100 IRE, from nesdev's NTSC video page - see EmuSen_CRT.md §22.
        public static readonly double[] PpuLow = { 0.228, 0.312, 0.552, 0.880 }, PpuHigh = { 0.616, 0.840, 1.100, 1.100 };
        public const double PpuBlack = 0.312, PpuIre100 = (1.100 - 0.312) / 1.10, PpuBurstLow = 0.148, PpuBurstHigh = 0.524, PpuAttenuation = 0.816328, PpuRowSkewDegrees = -5;

        // A receiver's colour control holds the burst at 20 IRE: the PPU's square burst has a fundamental of 2/pi of its swing - see EmuSen_CRT.md §22.
        public static double PpuChromaGain => 0.2 / (2 / Math.PI * (PpuBurstHigh - PpuBurstLow) / PpuIre100);

        // A PAL receiver holds the burst at 3/7 of white peak to peak (ITU-R BT.470-6, Table 2 item 2.15), the 2C07's square burst the 2C02's - see EmuSen_CRT.md §27.
        public static double PpuChromaGainPal => 3.0 / 14.0 / (2 / Math.PI * (PpuBurstHigh - PpuBurstLow) / PpuIre100);

        // The 2C07's oscillators where -U lies, half a step from the 2C02's 8, so that its burst pair 7 and A sits at 135 and 225 degrees - see EmuSen_CRT.md §27.
        public const double PpuPalAxisStep = 9.5;

        // The masks' proportions, in triads - see EmuSen_CRT.md §11.7.
        public const double StripeWidth = 0.199, SlotWidth = 0.19, SlotPitch = 0.81, SlotLit = 0.76, DotWidth = 0.35;

        // Rows of the matrix from the tube's guns to the display's channels, in linear light, with no adaptation between their whites.
        public static double[][] GunsToDisplay(Colour tube, Colour display)
        {
            double[][] toXyz = ToXyz(tube), fromXyz = Invert(ToXyz(display));
            double[][] m = Multiply(fromXyz, toXyz);

            // A white outside the display's range is brought inside it whole, so it keeps its colour and gives up brightness.
            double peak = m.Max(row => row.Sum());
            if (peak > 1) foreach (double[] row in m) for (int i = 0; i < 3; i++) row[i] /= peak;
            return m;
        }

        private static double[][] ToXyz(Colour c)
        {
            double[][] p =
            {
                new[] { c.Rx / c.Ry, c.Gx / c.Gy, c.Bx / c.By },
                new[] { 1.0, 1.0, 1.0 },
                new[] { (1 - c.Rx - c.Ry) / c.Ry, (1 - c.Gx - c.Gy) / c.Gy, (1 - c.Bx - c.By) / c.By },
            };
            double[] white = { c.Wx / c.Wy, 1.0, (1 - c.Wx - c.Wy) / c.Wy };
            double[][] inverse = Invert(p);
            double[] scale = inverse.Select(row => row[0] * white[0] + row[1] * white[1] + row[2] * white[2]).ToArray();
            return p.Select(row => new[] { row[0] * scale[0], row[1] * scale[1], row[2] * scale[2] }).ToArray();
        }

        private static double[][] Multiply(double[][] a, double[][] b) =>
            Enumerable.Range(0, 3).Select(i => Enumerable.Range(0, 3).Select(j => a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j]).ToArray()).ToArray();

        private static double[][] Invert(double[][] m)
        {
            double a = m[0][0], b = m[0][1], c = m[0][2], d = m[1][0], e = m[1][1], f = m[1][2], g = m[2][0], h = m[2][1], i = m[2][2];
            double det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
            return new[]
            {
                new[] { (e * i - f * h) / det, (c * h - b * i) / det, (b * f - c * e) / det },
                new[] { (f * g - d * i) / det, (a * i - c * g) / det, (c * d - a * f) / det },
                new[] { (d * h - e * g) / det, (b * g - a * h) / det, (a * e - b * d) / det },
            };
        }

        public const double CornerRadius = 0.04;
    }
}
namespace EmuSen.WiseMan.Serenity
{
    // SameBoy's measured Game Boy shades, which the panel filter's pictures are held to - see EmuSen_Serenity.md §3.5.
    public static class PanelReference
    {
        public const int DmgLightest = 0xC6DE8C, DmgLight = 0x84A563, DmgDark = 0x396139, DmgDarkest = 0x081810;
    }
}

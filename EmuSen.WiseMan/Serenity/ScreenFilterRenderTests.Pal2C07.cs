using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // The PAL NES's 2C07, read from nesdev's PAL video page and checked against its PAL capture - see EmuSen_CRT.md §27.
    public partial class ScreenFilterRenderTests
    {
        private static int Osc(int k, bool switched) => switched ? ((5 - k) % 12 + 12) % 12 : k;

        // The 2C07's level at a phase step: the 2C02's levels, its oscillators paired k with 5 - k on a switched line, and its red and green bits changed over.
        internal static double PpuAtPal(int colour, int emphasis, int p, bool switched)
        {
            int hue = colour & 15, row = colour >> 4;
            double lo = (CrtFilter.PpuLow[row] - CrtFilter.PpuBlack) / CrtFilter.PpuIre100, hi = (CrtFilter.PpuHigh[row] - CrtFilter.PpuBlack) / CrtFilter.PpuIre100;
            if (hue >= 14) return 0;
            if (hue == 0) lo = hi;
            if (hue >= 13) hi = lo;
            bool In(int h) => ((h + p) % 12 + 12) % 12 < 6;
            double v = In(hue is >= 1 and <= 12 ? Osc(hue, switched) : hue) ? hi : lo;
            bool dim = ((emphasis & 2) != 0 && In(Osc(0, switched))) || ((emphasis & 1) != 0 && In(Osc(4, switched))) || ((emphasis & 4) != 0 && In(Osc(8, switched)));
            return dim ? (CrtFilter.PpuAttenuation * (v * CrtFilter.PpuIre100 + CrtFilter.PpuBlack) - CrtFilter.PpuBlack) / CrtFilter.PpuIre100 : v;
        }

        // What a PAL receiver locked to the burst's mean decodes from one line of the 2C07's wave, V turned back on a switched line, the row's skew a delay of the wave.
        internal static (double Y, double U, double V) PpuDecodedPal(int colour, int emphasis, bool switched)
        {
            const int fine = 1200;
            double y = 0, u = 0, v = 0, delay = (colour >> 4) * CrtFilter.PpuRowSkewDegrees / 30;
            for (int i = 0; i < fine; i++)
            {
                double q = (i + 0.5) * 12.0 / fine;
                double level = PpuAtPal(colour, emphasis, (int)Math.Floor(((q + delay) % 12 + 12) % 12), switched);
                double angle = 2 * Math.PI * (q - CrtFilter.PpuPalAxisStep) / 12;
                y += level / fine; u += 2 * level * Math.Sin(angle) / fine; v += 2 * level * Math.Cos(angle) / fine;
            }
            return (y, CrtFilter.PpuChromaGainPal * u, CrtFilter.PpuChromaGainPal * v * (switched ? -1 : 1));
        }

        private static double Degrees(double u, double v) => (Math.Atan2(v, u) * 180 / Math.PI + 360) % 360;

        private static double Turn(double a, double b) => ((a - b) % 360 + 540) % 360 - 180;

        [Fact]
        public void The_2c07_s_burst_pair_sits_at_135_and_225_degrees_and_its_paired_hues_are_pal_s_switch()
        {
            // The burst is oscillator 7 on a line and A on the next, on the burst's levels; undecoded, they must sit at -U+V and -U-V.
            (double U, double V) Burst(bool switched)
            {
                double u = 0, v = 0;
                for (int i = 0; i < 1200; i++)
                {
                    double q = (i + 0.5) / 100.0;
                    double level = ((Osc(7, switched) + (int)q) % 12 < 6 ? CrtFilter.PpuBurstHigh : CrtFilter.PpuBurstLow) / CrtFilter.PpuIre100;
                    double angle = 2 * Math.PI * (q - CrtFilter.PpuPalAxisStep) / 12;
                    u += 2 * level * Math.Sin(angle) / 1200; v += 2 * level * Math.Cos(angle) / 1200;
                }
                return (u, v);
            }
            var (u0, v0) = Burst(false);
            var (u1, v1) = Burst(true);
            Within(135, Degrees(u0, v0), 0.001, "the burst on a line");
            Within(225, Degrees(u1, v1), 0.001, "the burst on the next");
            Within(3.0 / 14, Math.Sqrt(u0 * u0 + v0 * v0) * CrtFilter.PpuChromaGainPal, 0.001, "the burst's size after the colour control");

            // Every hue of row 0, which has no skew, decodes the same on both lines once V is turned back, 15 degrees short of the 2C02's.
            for (int hue = 1; hue <= 12; hue++)
            {
                var a = PpuDecodedPal(hue, 0, false);
                var b = PpuDecodedPal(hue, 0, true);
                var ntsc = PpuDecoded(hue, 0);
                Assert.True(Math.Abs(Turn(Degrees(a.U, a.V), Degrees(b.U, b.V))) < 0.01, $"hue {hue}: {Degrees(a.U, a.V):F2} and {Degrees(b.U, b.V):F2} on the two lines");
                Within(-15, Turn(Degrees(a.U, a.V), Degrees(ntsc.U, ntsc.V)), 0.001, $"hue {hue} against the 2C02's");
                Within(Math.Sqrt(ntsc.U * ntsc.U + ntsc.V * ntsc.V) * CrtFilter.PpuChromaGainPal / CrtFilter.PpuChromaGain, Math.Sqrt(a.U * a.U + a.V * a.V), 0.001, $"hue {hue}'s saturation");
            }

            // A row's skew turns the two lines opposite ways, Hanover's bars, whose mean is the hue.
            var even = PpuDecodedPal(0x36, 0, false);
            var odd = PpuDecodedPal(0x36, 0, true);
            var flat = PpuDecodedPal(0x06, 0, false);
            Within(-6 * CrtFilter.PpuRowSkewDegrees, Turn(Degrees(odd.U, odd.V), Degrees(even.U, even.V)), 0.01, "row 3's two lines apart");
            Assert.True(Math.Abs(Turn((Degrees(odd.U, odd.V) + Degrees(even.U, even.V)) / 2, Degrees(flat.U, flat.V))) < 0.01, "their mean is the hue");

            // The red and green bits change places: the 2C07's second bit dims where the 2C02's first does.
            for (int colour = 0; colour < 64; colour++)
                for (int p = 0; p < 12; p++)
                {
                    Assert.Equal(PpuAt(colour, 1, p), PpuAtPal(colour, 2, p, false));
                    Assert.Equal(PpuAt(colour, 2, p), PpuAtPal(colour, 1, p, false));
                }
        }

        // The composite path generates the 2C07's wave and a PAL receiver decodes it; the closed form gives each pixel's colour; through the delay line they agree for every entry.
        [Fact]
        public Task On_pal_the_2c07_s_wave_decoded_by_the_receiver_is_the_closed_form_s_colour() => Session.Dispatch(() =>
        {
            byte[] blank = Grey(256, 240, 0);
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("displayNits", 1000f), ("standard", 1f));
            var worst = (Difference: 0.0, Colour: 0, Emphasis: 0);
            foreach (int emphasis in new[] { 0, 1, 2, 4, 7 })
            {
                RawPicture raw = RawField(256, 240, (x, y) => (y / 30 * 8 + x / 32, emphasis));
                if (DrawNes(blank, raw, 1440, 1080, With(set, ("signal", 2f))) is not { } composite) return;
                Crt svideo = DrawNes(blank, raw, 1440, 1080, With(set, ("signal", 1f)))!;
                for (int band = 0; band < 64; band++)
                {
                    double x = (band % 8 + 0.5) * 180 - 15, y = (band / 8 + 0.5) * 135 - 20;
                    double[] a = composite.Mean(x, y, 30, 40), b = svideo.Mean(x, y, 30, 40);
                    double d = Enumerable.Range(0, 3).Max(c => Math.Abs(a[c] - b[c]));
                    if (d > worst.Difference) worst = (d, band, emphasis);
                }
            }
            Console.WriteLine($"PAL NES, composite against the closed form: worst {worst.Difference:F4} at {worst.Colour:X2} with emphasis {worst.Emphasis}");
            Assert.True(worst.Difference < 0.006, $"colour {worst.Colour:X2} with emphasis {worst.Emphasis}: composite and closed form differ by {worst.Difference:F4} of the peak");
        }, default);

        // A bright row's skew alternates line by line on PAL: the simple decoder shows it as bars, the delay line averages it away, at every tier; by S-Video, so that composite's six-line artifacts do not stand in for them.
        [Fact]
        public Task On_pal_the_nes_s_skew_makes_bars_the_delay_line_takes_away() => Session.Dispatch(() =>
        {
            byte[] blank = Grey(256, 240, 0);
            RawPicture raw = RawField(256, 240, (_, _) => (0x36, 0));
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("signal", 1f), ("standard", 1f));
            // The red over green of one line against the next, over the picture's middle, one line being 4.5 rows of 1080.
            double Bars(Crt c) => Enumerable.Range(100, 40).Average(line =>
            {
                double[] a = c.Mean(500, line * 4.5, 440, 4), b = c.Mean(500, (line + 1) * 4.5, 440, 4);
                return Math.Abs((a[0] - a[1]) - (b[0] - b[1]));
            });
            foreach (int quality in new[] { 0, 1, 2 })
            {
                if (DrawNes(blank, raw, 1440, 1080, With(set, ("quality", quality), ("palDecoder", 1f))) is not { } simple) return;
                Crt delay = DrawNes(blank, raw, 1440, 1080, With(set, ("quality", quality), ("palDecoder", 0f)))!;
                Console.WriteLine($"PAL NES $36 at quality {quality}: line to line {Bars(simple):F4} simple, {Bars(delay):F4} through the delay line");
                Assert.True(Bars(simple) > 0.01 && Bars(delay) < 0.25 * Bars(simple), $"quality {quality}: {Bars(simple):F4} simple, {Bars(delay):F4} delay line");
            }
        }, default);

        // With a raw frame the PAL NES is drawn from the 2C07's signal, not its RGB picture; without one, from RGB as before.
        [Fact]
        public Task On_pal_the_nes_is_drawn_from_the_2c07_s_signal_when_its_raw_picture_comes() => Session.Dispatch(() =>
        {
            byte[] frame = Paint(256, 240, (x, y) => (x / 255.0, y / 239.0, (x ^ y) % 7 / 6.0));
            RawPicture raw = RawField(256, 240, (x, y) => ((x + y) % 64, 0));
            (string, float)[] pal = { ("standard", 1f) };
            if (DrawNes(frame, raw, 640, 480, pal) is not { } with) return;
            Crt without = DrawNes(frame, null, 640, 480, pal)!;
            Crt snes = new(ShaderBench.Picture(new GameFrameControl { FilterConsole = "SNES", ShaderParameters = Values(Reference.Concat(pal)), ActiveFilter = CrtFilter.Filter }, Twice(frame), 256, 240, 1, 640, 480, CrtDevice.Value), 640, 480);
            int changed = 0, apart = 0;
            for (int y = 0; y < 480; y++) for (int x = 0; x < 640; x++) for (int c = 0; c < 3; c++)
            {
                if (with.Code(x, y, c) != without.Code(x, y, c)) changed++;
                apart = Math.Max(apart, Math.Abs(without.Code(x, y, c) - snes.Code(x, y, c)));
            }
            Assert.True(changed > 640 * 480, $"the raw frame changed {changed} values of the PAL picture");
            Assert.True(apart <= 1, $"without it the PAL NES is the PAL SNES's picture, within {apart}");
        }, default);
    }
}

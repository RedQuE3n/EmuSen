using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // The NES's own composite signal, from the PPU's levels and phases rather than from a palette - see EmuSen_CRT.md §22.
    public partial class ScreenFilterRenderTests
    {
        // The PPU's level at a phase step for a colour number and emphasis bits, as nesdev's NTSC video page describes it, in IRE over 100.
        internal static double PpuAt(int colour, int emphasis, int p)
        {
            int hue = colour & 15, row = colour >> 4;
            double lo = (CrtFilter.PpuLow[row] - CrtFilter.PpuBlack) / CrtFilter.PpuIre100, hi = (CrtFilter.PpuHigh[row] - CrtFilter.PpuBlack) / CrtFilter.PpuIre100;
            if (hue >= 14) return 0;
            if (hue == 0) lo = hi;
            if (hue >= 13) hi = lo;
            bool In(int h) => ((h + p) % 12 + 12) % 12 < 6;
            double v = In(hue) ? hi : lo;
            bool dim = ((emphasis & 1) != 0 && In(0)) || ((emphasis & 2) != 0 && In(4)) || ((emphasis & 4) != 0 && In(8));
            return dim ? (CrtFilter.PpuAttenuation * (v * CrtFilter.PpuIre100 + CrtFilter.PpuBlack) - CrtFilter.PpuBlack) / CrtFilter.PpuIre100 : v;
        }

        // The colour a receiver locked to the burst decodes from the PPU's wave, found by demodulating the wave itself finely, with the row's skew as a delay of the wave.
        internal static (double Y, double U, double V) PpuDecoded(int colour, int emphasis)
        {
            const int fine = 1200;
            double y = 0, u = 0, v = 0, delay = (colour >> 4) * CrtFilter.PpuRowSkewDegrees / 30;
            for (int i = 0; i < fine; i++)
            {
                double q = (i + 0.5) * 12.0 / fine;
                double level = PpuAt(colour, emphasis, (int)Math.Floor(((q + delay) % 12 + 12) % 12));
                double angle = 2 * Math.PI * (q - 10) / 12;
                y += level / fine; u += 2 * level * Math.Sin(angle) / fine; v += 2 * level * Math.Cos(angle) / fine;
            }
            return (y, CrtFilter.PpuChromaGain * u, CrtFilter.PpuChromaGain * v);
        }

        // A picture before the palette in the core ABI's nes.ppu: the colour number in bits 0-5 and the emphasis bits in 6-8 - see EmuSen_CoreAPI.md §6.30.
        internal static RawPicture RawField(int width, int height, Func<int, int, (int Colour, int Emphasis)> at)
        {
            var values = new byte[width * height * 2];
            for (int yy = 0; yy < height; yy++) for (int xx = 0; xx < width; xx++)
            {
                var (c, e) = at(xx, yy);
                int i = (yy * width + xx) * 2, v = c | e << 6;
                (values[i], values[i + 1]) = ((byte)v, (byte)(v >> 8));
            }
            return new RawPicture(RawEncodings.NesPpu, width, height, 2, width * 2, values);
        }

        private static Crt? DrawNes(byte[] rgb, RawPicture? raw, int windowWidth, int windowHeight, params (string Id, float Value)[] set)
        {
            if (CrtDevice.Value is not { } device) return null;
            var control = new GameFrameControl { FilterConsole = "NES", ShaderParameters = Values(Reference.Concat(set)), ActiveFilter = CrtFilter.Filter };
            return new Crt(ShaderBench.Picture(control, Twice(rgb), 256, 240, 1, windowWidth, windowHeight, device, raw: raw), windowWidth, windowHeight);
        }

        [Fact]
        public void The_burst_decodes_to_minus_u_and_a_colour_s_hue_is_its_phase_from_the_burst()
        {
            // The burst is colour 8's wave on the burst's levels: it must decode on the -U axis.
            double u = 0, v = 0;
            for (int i = 0; i < 1200; i++)
            {
                double q = (i + 0.5) / 100.0;
                double level = ((8 + (int)q) % 12 < 6 ? CrtFilter.PpuBurstHigh : CrtFilter.PpuBurstLow) / CrtFilter.PpuIre100;
                double angle = 2 * Math.PI * (q - 10) / 12;
                u += 2 * level * Math.Sin(angle) / 1200; v += 2 * level * Math.Cos(angle) / 1200;
            }
            Assert.True(u < 0 && Math.Abs(v) < 1e-3 * Math.Abs(u), $"burst at U {u:F4}, V {v:F4}");
            Within(0.2, -u * CrtFilter.PpuChromaGain, 0.001, "the burst's size after the colour control");

            // Hues 1 to 12 sit 30 degrees apart, hue 8 on the burst's own axis, and each row turns the next by the skew.
            double Angle(int colour) { var d = PpuDecoded(colour, 0); return Math.Atan2(d.V, d.U) * 180 / Math.PI; }
            Within(180, Math.Abs(Angle(0x08)), 0.001, "hue 8 on -U");
            for (int hue = 1; hue <= 11; hue++)
            {
                double step = ((Angle(0x00 + hue + 1) - Angle(0x00 + hue)) % 360 + 540) % 360 - 180;
                Assert.True(Math.Abs(Math.Abs(step) - 30) < 0.01, $"hue {hue} to {hue + 1}: {step:F3} degrees");
            }
            double skew = ((Angle(0x38) - Angle(0x08)) % 360 + 540) % 360 - 180;
            Within(3 * CrtFilter.PpuRowSkewDegrees, skew, 0.01, "row 3 turned from row 0 by three rows' skew");
        }

        // The S-Video path takes each pixel's colour from the PPU's levels in closed form; the composite path generates the wave and the receiver decodes it. A flat field must come out the same both ways.
        [Fact]
        public Task The_ppu_s_wave_decoded_by_the_receiver_is_the_colour_the_closed_form_gives_for_every_entry() => Session.Dispatch(() =>
        {
            byte[] blank = Grey(256, 240, 0);
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("decoder", 1f), ("displayNits", 1000f));
            var worst = (Difference: 0.0, Colour: 0, Emphasis: 0);
            foreach (int emphasis in new[] { 0, 1, 2, 4, 7 })
            {
                // 64 bands across, one colour each, wide enough that the chroma channel settles in each.
                RawPicture raw = RawField(256, 240, (x, y) => (y / 30 * 8 + x / 32, emphasis));
                if (DrawNes(blank, raw, 1440, 1080, With(set, ("signal", 2f))) is not { } composite) return;
                if (DrawNes(blank, raw, 1440, 1080, With(set, ("signal", 1f))) is not { } svideo) return;
                for (int band = 0; band < 64; band++)
                {
                    double x = (band % 8 + 0.5) * 180 - 15, y = (band / 8 + 0.5) * 135 - 20;
                    double[] a = composite.Mean(x, y, 30, 40), b = svideo.Mean(x, y, 30, 40);
                    double d = Enumerable.Range(0, 3).Max(c => Math.Abs(a[c] - b[c]));
                    if (d > worst.Difference) worst = (d, band, emphasis);
                }
            }
            // At a display ten times the tube's white the residue is 0.0044, in the saturated blues whose red and green sit near nothing (EmuSen_CRT.md §22.4).
            Assert.True(worst.Difference < 0.006, $"colour {worst.Colour:X2} with emphasis {worst.Emphasis}: composite and closed form differ by {worst.Difference:F4} of the peak");
        }, default);

        // The NES's artifacts from its own signal, read as red over green: a column pattern's colour repeats every three lines and turns over every frame, as the RGB-encoded one did (section 21.2).
        [Fact]
        public Task From_its_own_signal_the_nes_s_artifacts_keep_their_three_line_and_two_frame_pattern() => Session.Dispatch(() =>
        {
            byte[] blank = Grey(256, 240, 0);
            RawPicture raw = RawField(256, 240, (x, _) => (x % 2 == 0 ? 0x20 : 0x0F, 0));
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("signal", 2f));
            if (CrtDevice.Value is not { } device) return;
            double[] Line(byte[] picture, int line) => Enumerable.Range(480, 480).Select(x => { double s = 0; for (int yy = (int)(line * 4.5); yy < (int)((line + 1) * 4.5); yy++) s += picture[(yy * 1440 + x) * 4] - picture[(yy * 1440 + x) * 4 + 1]; return s; }).ToArray();
            double Corr(double[] p, double[] q) { double pm = p.Average(), qm = q.Average(), n = 0, a = 0, b = 0; for (int i = 0; i < p.Length; i++) { n += (p[i] - pm) * (q[i] - qm); a += (p[i] - pm) * (p[i] - pm); b += (q[i] - qm) * (q[i] - qm); } return n / Math.Sqrt(a * b); }
            var control = new GameFrameControl { FilterConsole = "NES", ShaderParameters = Values(Reference.Concat(set)), ActiveFilter = CrtFilter.Filter };
            byte[] first = ShaderBench.Picture(control, Twice(blank), 256, 240, 1, 1440, 1080, device, raw: raw);
            control = new GameFrameControl { FilterConsole = "NES", ShaderParameters = Values(Reference.Concat(set)), ActiveFilter = CrtFilter.Filter };
            byte[] next = ShaderBench.Picture(control, new[] { blank, blank, blank }, 256, 240, 1, 1440, 1080, device, raw: raw);
            Assert.True(Corr(Line(first, 120), Line(first, 123)) > 0.95, "three lines on, the same");
            Assert.True(Corr(Line(first, 120), Line(first, 121)) < 0.0, "the next line, different");
            Assert.True(Corr(Line(first, 120), Line(next, 120)) < 0.0, "the next frame, different");
        }, default);

        // MoonRT's picture before the palette, through the core ABI and the host to the filter: an emphasis write in the middle of the frame darkens the lines after it, which the RGB frame cannot show - see EmuSen_CoreAPI.md §6.30.
        [Fact]
        public Task MoonRt_s_emphasis_written_in_the_middle_of_a_frame_reaches_the_screen_through_the_raw_picture() => Session.Dispatch(() =>
        {
            if (CrtDevice.Value is not { } device) return;
            Assert.True(Fixtures.MoonEngines.Available, Fixtures.MoonEngines.Report);
            string path = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"emusen-raw-crt-{Environment.ProcessId}.nes");
            System.IO.File.WriteAllBytes(path, EmphasisMidFrame());
            using var core = Fixtures.MoonEngines.Create();
            try { core.LoadRom(path); } finally { System.IO.File.Delete(path); }
            core.RawEncoding = RawEncodings.NesPpu;
            for (int f = 0; f < 8; f++) core.RunFrame();
            RawPicture raw = core.RawPicture is { } r ? new RawPicture(r.Encoding, r.Width, r.Height, r.Element, r.Stride, r.Values) : throw new Xunit.Sdk.XunitException("no raw picture");
            byte[] rgb = core.GetFrameBufferRgba();
            int first = Enumerable.Range(0, 240).First(y => (raw.Values[y * 512] | raw.Values[y * 512 + 1] << 8) >> 6 == 7);
            Assert.InRange(first, 60, 180);
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("signal", 2f));
            Crt viaRaw = DrawNes(rgb, raw, 1440, 1080, set)!, viaRgb = DrawNes(rgb, null, 1440, 1080, set)!;
            double Light(Crt c, int fromLine, int toLine) => c.Mean(480, fromLine * 4.5, 480, (toLine - fromLine) * 4.5).Sum();
            (double above, double below) = (Light(viaRaw, first - 50, first - 10), Light(viaRaw, first + 10, first + 50));
            (double rgbAbove, double rgbBelow) = (Light(viaRgb, first - 50, first - 10), Light(viaRgb, first + 10, first + 50));
            Assert.True(below < 0.9 * above, $"from its raw picture, the lines after the write at {below:F3} against {above:F3} before it");
            Assert.True(Math.Abs(rgbBelow - rgbAbove) < 0.02 * rgbAbove, $"from its RGB frame, {rgbBelow:F3} against {rgbAbove:F3}: the same colour either side");
            Console.WriteLine($"emphasis from line {first}: through the raw picture {above:F3} above and {below:F3} below, through RGB {rgbAbove:F3} and {rgbBelow:F3}");
        }, default);

        // Colour $16 at the top of each frame and all three emphasis bits written from the NMI, about 124 lines later: MoonRT's own test program (Moon_Native.md §9.7).
        private static byte[] EmphasisMidFrame()
        {
            byte[] reset =
            {
                0x78, 0xD8, 0xA2, 0xFF, 0x9A, 0x2C, 0x02, 0x20, 0x10, 0xFB, 0x2C, 0x02, 0x20, 0x10, 0xFB,
                0xA9, 0x3F, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20, 0xA9, 0x16, 0x8D, 0x07, 0x20,
                0xA9, 0x00, 0x8D, 0x06, 0x20, 0x8D, 0x06, 0x20, 0x8D, 0x05, 0x20, 0x8D, 0x05, 0x20,
                0xA9, 0x00, 0x8D, 0x01, 0x20, 0xA9, 0x80, 0x8D, 0x00, 0x20, 0x4C, 0x36, 0xC0,
            };
            byte[] nmi =
            {
                0x48, 0x8A, 0x48, 0x98, 0x48, 0xA9, 0x00, 0x8D, 0x01, 0x20,
                0xA0, 0x0B, 0xA2, 0xFF, 0xCA, 0xD0, 0xFD, 0x88, 0xD0, 0xF8,
                0xA9, 0xE0, 0x8D, 0x01, 0x20, 0x68, 0xA8, 0x68, 0xAA, 0x68, 0x40,
            };
            byte[] rom = EmuSen.WiseMan.Fixtures.SyntheticNesRom.Build(prgBanks: 1, chrBanks: 1, patches: new[] { (0, reset), (0x100, nmi) });
            rom[16 + 0x3FFA] = 0x00; rom[16 + 0x3FFB] = 0xC1;
            return rom;
        }

        // Without a raw frame the NES is encoded from its RGB picture as before: drawn as the SNES is, whose timing it shares, to a code value where the shader compiles differently.
        [Fact]
        public Task Without_a_raw_frame_the_nes_is_drawn_from_its_rgb_picture_as_before() => Session.Dispatch(() =>
        {
            if (CrtDevice.Value is not { } device) return;
            byte[] frame = Paint(256, 240, (x, y) => (x / 255.0, y / 239.0, (x ^ y) % 7 / 6.0));
            foreach (int quality in new[] { 0, 1, 2 })
            {
                byte[] Picture(string console) => ShaderBench.Picture(new GameFrameControl { FilterConsole = console, ShaderParameters = Values(Reference.Concat(new[] { ("quality", (float)quality) })), ActiveFilter = CrtFilter.Filter }, Twice(frame), 256, 240, 1, 640, 480, device);
                byte[] snes = Picture("SNES"), nes = Picture("NES");
                int worst = 0, differing = 0;
                for (int i = 0; i < snes.Length; i++) { int d = Math.Abs(snes[i] - nes[i]); worst = Math.Max(worst, d); if (d > 0) differing++; }
                Assert.True(worst <= 1 && differing < snes.Length / 100, $"quality {quality}: {differing} values differ, by at most {worst}");
            }
        }, default);

        // Performance has no subcarrier: its colours are the closed form's, the same as S-Video's within the blur of its single pass.
        [Fact]
        public Task At_performance_the_nes_s_colours_are_the_ppu_s_closed_form() => Session.Dispatch(() =>
        {
            byte[] blank = Grey(256, 240, 0);
            RawPicture raw = RawField(256, 240, (x, y) => (y / 30 * 8 + x / 32, 0));
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("displayNits", 1000f), ("signal", 1f));
            if (DrawNes(blank, raw, 1440, 1080, With(set, ("quality", 0f))) is not { } performance) return;
            if (DrawNes(blank, raw, 1440, 1080, With(set, ("quality", 1f))) is not { } balanced) return;
            for (int band = 0; band < 64; band++)
            {
                double x = (band % 8 + 0.5) * 180 - 15, y = (band / 8 + 0.5) * 135 - 20;
                double[] a = performance.Mean(x, y, 30, 40), b = balanced.Mean(x, y, 30, 40);
                double d = Enumerable.Range(0, 3).Max(c => Math.Abs(a[c] - b[c]));
                Assert.True(d < 0.006, $"colour {band:X2}: Performance and Balanced differ by {d:F4}");
            }
        }, default);

        // Hues land where the burst puts them, and each emphasis bit dims the colours away from its own: red's dims blue's hue C and spares red's 6, blue's dims 8 and spares 2.
        [Fact]
        public Task Each_hue_decodes_to_its_colour_and_each_emphasis_bit_spares_its_own() => Session.Dispatch(() =>
        {
            byte[] blank = Grey(256, 240, 0);
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("displayNits", 1000f), ("signal", 2f));
            double[] At(Crt picture) => picture.Mean(560, 380, 320, 320);
            Crt Field(int colour, int emphasis) => DrawNes(blank, RawField(256, 240, (_, _) => (colour, emphasis)), 1440, 1080, set)!;
            if (CrtDevice.Value is null) return;
            double[] red = At(Field(0x16, 0)), green = At(Field(0x1A, 0)), blue = At(Field(0x12, 0)), white = At(Field(0x30, 0));
            Assert.True(red[0] > 2 * red[1] && red[0] > 2 * red[2], $"$16 is red: {string.Join(", ", red.Select(v => v.ToString("F3")))}");
            Assert.True(green[1] > 1.5 * green[0] && green[1] > 1.5 * green[2], $"$1A is green: {string.Join(", ", green.Select(v => v.ToString("F3")))}");
            Assert.True(blue[2] > 2 * blue[0] && blue[2] > 1.5 * blue[1], $"$12 is blue: {string.Join(", ", blue.Select(v => v.ToString("F3")))}");
            Assert.True(white.Max() - white.Min() < 0.03 * white.Average(), $"$30 is neutral: {string.Join(", ", white.Select(v => v.ToString("F3")))}");
            double Kept(int colour, int emphasis) => At(Field(colour, emphasis)).Sum() / At(Field(colour, 0)).Sum();
            Assert.True(Kept(0x1C, 1) < Kept(0x16, 1) - 0.1, $"red's bit: $1C kept {Kept(0x1C, 1):F2}, $16 kept {Kept(0x16, 1):F2}");
            Assert.True(Kept(0x14, 2) < Kept(0x1A, 2) - 0.1, $"green's bit: $14 kept {Kept(0x14, 2):F2}, $1A kept {Kept(0x1A, 2):F2}");
            Assert.True(Kept(0x18, 4) < Kept(0x12, 4) - 0.1, $"blue's bit: $18 kept {Kept(0x18, 4):F2}, $12 kept {Kept(0x12, 4):F2}");
        }, default);
    }
}

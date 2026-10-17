using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // PAL as a model of its own: V's switch, the delay line, 4.43 MHz and each console's PAL clocks - see EmuSen_CRT.md §23.
    public partial class ScreenFilterRenderTests
    {
        private static readonly (string, float)[] Pal = { ("standard", 1f) };

        // Each console's PAL line follows from its clocks as the documentation gives them; the table must say the same.
        [Fact]
        public void Every_console_s_pal_line_is_the_arithmetic_of_its_clocks()
        {
            const double fsc = CrtFilter.PalSubcarrierMegahertz;
            // Master clock in MHz, master clocks a pixel, a line and a subcarrier cycle, lines a frame, and any extra cycles a frame (the N64's leap lines, averaged).
            var clocks = new Dictionary<string, (double Master, double Pixel, double Line, double Cycle, int Lines, double Extra, int Width)>
            {
                ["NES"] = (26.601712, 5, 341 * 5, 6, 312, 0, 256),
                ["SNES"] = (21.28137, 4, 1364, 4.8, 312, 0, 256),
                ["Genesis"] = (53.203424, 10, 3420, 12, 313, 0, 256),
                ["N64"] = (49.65653, 4, 3178, 11.2, 313, (6 + 5 + 6 + 5 + 6) / 5.0 / 11.2, 640),
                ["PlayStation"] = (53.203425, 1, 3405, 12, 314, 0, 2560),
            };
            foreach (string console in CrtFilter.Filter.Consoles!)
            {
                var (master, pixel, line, cycle, lines, extra, width) = clocks[console];
                Within(fsc, master / cycle, 1e-5, $"{console}'s subcarrier from its master clock");
                CrtFilter.LineTiming pal = CrtFilter.LineOf(CrtFilter.ConsoleTiming[console], pal: true);
                double perLine = line / cycle, perFrame = lines * perLine + extra;
                Within(width * pixel / master, pal.ActiveMicroseconds, 1e-4, $"{console}'s active picture, µs");
                Within(width * pixel / cycle, pal.CyclesAcross, 1e-4, $"{console}'s cycles across");
                Within(perLine - Math.Floor(perLine), pal.PhaseLine, 1e-6, $"{console}'s phase a line");
                Within(perFrame - Math.Floor(perFrame), pal.PhaseFrame + pal.PhaseCrawl, 1e-6, $"{console}'s phase a frame");
                Assert.True(pal.SwitchFrame == (lines % 2 == 1), $"{console}: {lines} lines turn the switch over each frame only when odd");
            }
        }

        // The tails on PAL's longer field are the same exponentials: what follows the struck frame is the NTSC model's light, decayed over the 3.3 ms more the field lasts.
        [Fact]
        public void The_phosphor_tails_on_a_pal_field_are_the_ntsc_fit_s_exponentials()
        {
            CrtFilter.Tails ntsc = CrtFilter.Tails.For(pal: false), pal = CrtFilter.Tails.For(pal: true);
            double longer = (CrtFilter.PalFieldMilliseconds - CrtFilter.NtscFieldMilliseconds) / CrtFilter.NtscFieldMilliseconds;
            foreach (var (name, c60, r60, c50, r50) in new[] { ("green fast", ntsc.GreenFast, ntsc.Fast, pal.GreenFast, pal.Fast), ("green slow", ntsc.GreenSlow, ntsc.Slow, pal.GreenSlow, pal.Slow), ("blue fast", ntsc.BlueFast, ntsc.Fast, pal.BlueFast, pal.Fast), ("blue slow", ntsc.BlueSlow, ntsc.Slow, pal.BlueSlow, pal.Slow) })
            {
                double after60 = c60 * r60 / (1 - r60), after50 = c50 * r50 / (1 - r50);
                Within(after60 * Math.Pow(r60, longer), after50, 1e-9 + 1e-9 * after60, $"{name}: light after the struck field");
                Assert.True(r50 < r60, $"{name}: a longer field decays further");
            }
        }

        // A comb on PAL needs two lines of delay, which is not modelled: a screen with a comb decodes PAL through its notch.
        [Fact]
        public void A_pal_picture_is_separated_by_the_notch_and_decoded_through_the_delay_line_unless_the_player_chooses_otherwise()
        {
            Dictionary<string, float> Of(params (string, float)[] set)
            {
                var values = CrtFilter.Parameters.ToDictionary(p => p.Id, p => p.Initial);
                foreach (var (id, value) in CrtFilter.ConsoleTiming["N64"]) values[id] = value;
                foreach (var (id, value) in set) values[id] = value;
                return values;
            }
            Assert.Equal(CrtFilter.Decoder.Comb, CrtFilter.Resolve(Of(("decoder", 2f))).Decoder);
            CrtFilter.Resolved pal = CrtFilter.Resolve(Of(("decoder", 2f), ("standard", 1f)));
            Assert.Equal(CrtFilter.Decoder.Notch, pal.Decoder);
            Assert.True(pal.DelayLine && pal.Pal);
            Assert.False(CrtFilter.Resolve(Of(("standard", 1f), ("palDecoder", 1f))).DelayLine);
            Assert.Equal(CrtFilter.PalSubcarrierMegahertz, pal.Subcarrier);
        }

        // The encoder turns V over each line and the decoder turns it back: a flat colour comes out of PAL as out of NTSC, at every tier.
        [Fact]
        public Task A_flat_colour_decodes_the_same_on_pal_as_on_ntsc() => Session.Dispatch(() =>
        {
            byte[] bands = Paint(256, 224, (x, _) => new[] { (0.8, 0.2, 0.2), (0.2, 0.7, 0.2), (0.2, 0.3, 0.8), (0.7, 0.6, 0.1), (0.6, 0.2, 0.7), (0.1, 0.6, 0.6), (0.5, 0.5, 0.5), (0.9, 0.4, 0.3) }[x / 32]);
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("decoder", 1f));
            var worst = (Difference: 0.0, Where: "");
            foreach (int quality in new[] { 0, 1, 2 })
                foreach (int signal in new[] { 1, 2 })
                {
                    (string, float)[] at = With(set, ("quality", quality), ("signal", signal));
                    if (Draw("SNES", 256, 224, 1440, 1080, Twice(bands), at) is not { } ntsc) return;
                    Crt pal = Draw("SNES", 256, 224, 1440, 1080, Twice(bands), With(at, Pal))!;
                    for (int band = 0; band < 8; band++)
                    {
                        double x = (band + 0.5) * 180 - 30;
                        double[] a = ntsc.Mean(x, 400, 60, 280), b = pal.Mean(x, 400, 60, 280);
                        double d = Enumerable.Range(0, 3).Max(c => Math.Abs(a[c] - b[c]));
                        if (d > worst.Difference) worst = (d, $"band {band} at quality {quality}, signal {signal}");
                    }
                }
            Assert.True(worst.Difference < 0.01, $"{worst.Where}: PAL and NTSC differ by {worst.Difference:F4} of the peak");
        }, default);

        // The delay line averages each line's colour with the line's above it, so colour that alternates line by line is blended; the simple decoder keeps it apart.
        [Fact]
        public Task The_delay_line_averages_each_line_s_colour_with_the_line_before_and_the_simple_decoder_does_not() => Session.Dispatch(() =>
        {
            byte[] stripes = Paint(256, 224, (_, y) => y % 2 == 0 ? (0.7, 0.2, 0.3) : (0.2, 0.35, 0.6));
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("signal", 1f));
            foreach (int quality in new[] { 0, 1, 2 })
            {
                double Spread((string, float)[] at)
                {
                    Crt picture = Draw("SNES", 256, 224, 1440, 1080, Twice(stripes), With(at, ("quality", quality)))!;
                    var rows = Enumerable.Range(300, 480).Select(y => picture.Mean(500, y, 440, 1)).Where(m => m.Sum() > 0.02).Select(m => m[0] / m.Sum()).ToArray();
                    return rows.Max() - rows.Min();
                }
                if (CrtDevice.Value is null) return;
                double ntsc = Spread(set), delay = Spread(With(set, Pal)), simple = Spread(With(set, Pal).Append(("palDecoder", 1f)).ToArray());
                Assert.True(delay < 0.25 * simple, $"quality {quality}: red's share swings by {delay:F3} through the delay line, {simple:F3} without");
                Assert.True(Math.Abs(simple - ntsc) < 0.15 * ntsc, $"quality {quality}: the simple decoder keeps the lines apart as NTSC does, {simple:F3} against {ntsc:F3}");
            }
        }, default);

        // A thin line's fringes follow each console's PAL phase: six lines on the SNES and still between frames, four frames on the N64, and the Genesis's turned by the switch each frame.
        [Fact]
        public Task On_pal_the_fringes_repeat_every_six_lines_on_the_snes_and_crawl_through_four_frames_on_the_n64() => Session.Dispatch(() =>
        {
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("signal", 2f), ("palDecoder", 1f));
            set = With(set, Pal);
            if (CrtDevice.Value is null) return;

            // Each scanline's fringe: the colour of the light beside a one-pixel white line, as shares of red and blue, read at the scanline's centre.
            List<(double R, double B)> Fringes(string console, int width, int height, int frames)
            {
                byte[] line = Paint(width, height, (x, _) => x == width / 2 ? (1.0, 1.0, 1.0) : (0.0, 0.0, 0.0));
                Crt picture = Draw(console, width, height, 1440, 1080, Enumerable.Repeat(line, frames).ToArray(), set)!;
                int xc = Enumerable.Range(600, 240).OrderByDescending(x => picture.Mean(x, 540, 1, 1).Sum()).First();
                double[] column = Enumerable.Range(0, 1080).Select(y => picture.Mean(xc, y, 1, 1).Sum()).ToArray();
                var fringes = new List<(double, double)>();
                for (int y = 400; y < 700; y++)
                    if (column[y] > column[y - 1] && column[y] >= column[y + 1] && column[y] > 0.3 * column.Max())
                    {
                        double[] m = picture.Mean(xc - 6, y, 13, 1);
                        fringes.Add((m[0] / m.Sum(), m[2] / m.Sum()));
                    }
                return fringes;
            }
            // Scanlines fall 4 or 5 display rows apart, so a line's centre is read up to half a row off and the same phase on another line reads up to 0.03 apart.
            double Apart((double R, double B) p, (double R, double B) q) => Math.Max(Math.Abs(p.R - q.R), Math.Abs(p.B - q.B));

            List<(double R, double B)> snes = Fringes("SNES", 256, 224, 2), snesNext = Fringes("SNES", 256, 224, 3);
            for (int i = 2; i < 8; i++)
            {
                Assert.True(Apart(snes[i], snes[i + 6]) < 0.03, $"SNES, six lines on from line {i}: {Apart(snes[i], snes[i + 6]):F3} apart");
                Assert.True(Apart(snes[i], snes[i + 3]) > 0.05, $"SNES, three lines on from line {i}: {Apart(snes[i], snes[i + 3]):F3} apart");
                Assert.True(Apart(snes[i], snesNext[i]) < 0.01, $"SNES, the next frame at line {i}: {Apart(snes[i], snesNext[i]):F3} apart");
            }

            List<(double R, double B)> n64 = Fringes("N64", 320, 240, 4), n64Next = Fringes("N64", 320, 240, 5), n64Four = Fringes("N64", 320, 240, 8);
            for (int i = 2; i < 8; i++) Assert.True(Apart(n64[i], n64Four[i]) < 0.01, $"N64, four frames on at line {i}: {Apart(n64[i], n64Four[i]):F3} apart");
            List<(double R, double B)> n64Two = Fringes("N64", 320, 240, 6);
            double moved = Enumerable.Range(2, 8).Average(i => Apart(n64[i], n64Next[i])), crawled = Enumerable.Range(2, 8).Average(i => Apart(n64[i], n64Two[i]));
            Assert.True(moved > 0.05, $"N64, the next frame: {moved:F3} apart on average");
            Assert.True(crawled > 0.05, $"N64, two frames on: {crawled:F3} apart on average");

            // The Genesis's phase stands still, but its 313 lines turn the switch over each frame, so a line's fringe alternates between frames without the delay line.
            List<(double R, double B)> genesis = Fringes("Genesis", 256, 224, 2), genesisNext = Fringes("Genesis", 256, 224, 3), genesisTwo = Fringes("Genesis", 256, 224, 4);
            double turned = Enumerable.Range(2, 8).Average(i => Apart(genesis[i], genesisNext[i])), back = Enumerable.Range(2, 8).Max(i => Apart(genesis[i], genesisTwo[i]));
            Assert.True(turned > 0.02 && back < 0.01, $"Genesis: {turned:F3} apart the next frame, {back:F3} two frames on");
        }, default);

        // The receiver's chroma channel keeps its width in MHz on PAL, so a colour edge takes as long in microseconds on either standard.
        [Fact]
        public Task A_colour_edge_takes_as_long_on_pal_as_on_ntsc() => Session.Dispatch(() =>
        {
            byte[] edge = Paint(256, 224, (x, _) => x < 128 ? (0.6, 0.3, 0.6) : (0.3, 0.5, 0.3));
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("signal", 1f));
            double Microseconds((string, float)[] at, double activeUs)
            {
                Crt picture = Draw("SNES", 256, 224, 1440, 1080, Twice(edge), at)!;
                double[] red = Enumerable.Range(0, 1440).Select(x => picture.Mean(x, 440, 1, 200)[0]).ToArray();
                double high = red.Skip(400).Take(200).Average(), low = red.Skip(840).Take(200).Average();
                double At(double share) { double level = high - share * (high - low); for (int x = 600; x < 840; x++) if (red[x] <= level) return x - (red[x] - level) / (red[x] - red[x - 1]); return double.NaN; }
                int left = Enumerable.Range(0, 1440).First(x => picture.Mean(x, 440, 1, 200).Sum() > 0.02), right = Enumerable.Range(0, 1440).Last(x => picture.Mean(x, 440, 1, 200).Sum() > 0.02);
                return (At(0.9) - At(0.1)) / (right - left + 1) * activeUs;
            }
            if (CrtDevice.Value is null) return;
            double ntsc = Microseconds(set, CrtFilter.ConsoleTiming["SNES"]["activeUs"]), pal = Microseconds(With(set, Pal), CrtFilter.ConsoleTiming["SNES"]["pal.activeUs"]);
            Assert.True(Math.Abs(pal - ntsc) < 0.08 * ntsc, $"a colour edge takes {pal:F3} µs on PAL and {ntsc:F3} µs on NTSC");
        }, default);
    }
}

using System;
using System.Linq;
using System.Numerics;
using System.Threading.Tasks;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // A receiver's own decoder: the Sony CXA2025AS's luma trap and its demodulator's axes, from its datasheet - see EmuSen_CRT.md §24.
    public partial class ScreenFilterRenderTests
    {
        // The trap's response at a frequency, from its taps on a line of samples.
        private static double TrapResponse(double[] band, double samplesPerMicrosecond, double megahertz)
        {
            Complex sum = 1;
            for (int j = 0; j < band.Length; j++) sum -= band[j] * Complex.FromPolarCoordinates(1, -2 * Math.PI * megahertz / samplesPerMicrosecond * j);
            return sum.Magnitude;
        }

        // The taps are a notch at the subcarrier whose lower -3 dB edge is the datasheet's, at either signal tier's rate and on either standard.
        [Fact]
        public void The_trap_s_taps_are_a_notch_at_the_subcarrier_with_the_datasheet_s_lower_edge()
        {
            double active = CrtFilter.ConsoleTiming["SNES"]["activeUs"];
            foreach (var (rate, what) in new[] { (CrtFilter.SignalSamples / active, "Accurate"), (Math.Round(4 * CrtFilter.ConsoleTiming["SNES"]["cyclesAcross"]) / active, "Balanced") })
                foreach (double centre in new[] { CrtFilter.SubcarrierMegahertz, CrtFilter.PalSubcarrierMegahertz })
                {
                    double[] band = CrtFilter.TrapTaps(rate, centre);
                    double edge = centre * CrtFilter.TrapLowerEdgeMegahertz / CrtFilter.SubcarrierMegahertz;
                    Assert.True(TrapResponse(band, rate, centre) < 0.01, $"{what} at {centre} MHz: {TrapResponse(band, rate, centre):F4} left at the centre");
                    Assert.True(Math.Abs(TrapResponse(band, rate, edge) - Math.Sqrt(0.5)) < 0.005, $"{what} at {centre} MHz: {TrapResponse(band, rate, edge):F4} at the lower edge");
                    Assert.True(TrapResponse(band, rate, 0) > 0.999, $"{what} at {centre} MHz: {TrapResponse(band, rate, 0):F4} kept at 0 MHz");
                    foreach (double f in new[] { 1.0, 1.5 })
                        Assert.True(Math.Abs(TrapResponse(band, rate, f * centre / CrtFilter.SubcarrierMegahertz) - CrtFilter.TrapGain(f * centre / CrtFilter.SubcarrierMegahertz, centre)) < 0.02,
                            $"{what} at {centre} MHz: {TrapResponse(band, rate, f):F3} at {f} MHz where the analogue trap keeps {CrtFilter.TrapGain(f, centre):F3}");
                }
            Assert.True(Math.Abs(CrtFilter.TrapGain(CrtFilter.TrapLowerEdgeMegahertz, CrtFilter.SubcarrierMegahertz) - Math.Sqrt(0.5)) < 1e-9, "the analogue trap's lower edge");
        }

        // The demodulator's axes as the datasheet gives them: the standard's angles and gains rebuild the standard's matrix, and the CXA2025AS's give its rows.
        [Fact]
        public void The_standard_s_axes_rebuild_its_matrix_and_the_datasheet_s_axes_give_their_own_rows()
        {
            double[][] standard = CrtFilter.AxesMatrix(CrtFilter.Demodulators[0]);
            Assert.Equal(new[] { 0.0, 1.0 }, standard[0].Select(v => Math.Round(v, 9)));
            Assert.Equal(new[] { Math.Round(-0.114 / 0.587, 9), Math.Round(-0.299 / 0.587, 9) }, standard[1].Select(v => Math.Round(v, 9)));
            Assert.Equal(new[] { 1.0, 0.0 }, standard[2].Select(v => Math.Round(v, 9)));
            double k = 0.877 / 0.492;
            foreach (var (index, red, redGain, green, greenGain) in new[] { (1, 112.0, 0.83, 252.0, 0.30), (2, 95.0, 0.78, 240.0, 0.30) })
            {
                double[][] m = CrtFilter.AxesMatrix(CrtFilter.Demodulators[index]);
                double Rad(double d) => d * Math.PI / 180;
                Within(redGain * Math.Cos(Rad(red)), m[0][0], 1e-9, $"{CrtFilter.Demodulators[index].Label}: R-Y from B-Y");
                Within(redGain * k * Math.Sin(Rad(red)), m[0][1], 1e-9, $"{CrtFilter.Demodulators[index].Label}: R-Y from R-Y");
                Within(greenGain * Math.Cos(Rad(green)), m[1][0], 1e-9, $"{CrtFilter.Demodulators[index].Label}: G-Y from B-Y");
                Within(greenGain * k * Math.Sin(Rad(green)), m[1][1], 1e-9, $"{CrtFilter.Demodulators[index].Label}: G-Y from R-Y");
                Assert.Equal(new[] { 1.0, 0.0 }, m[2]);
            }
        }

        // A composite grating's luma, over the S-Video one's, is what the trap keeps at that frequency, at every tier and on either standard; the 512-wide frame carries gratings up to the subcarrier.
        [Fact]
        public Task Composite_luma_keeps_what_the_trap_passes_at_every_tier() => Session.Dispatch(() =>
        {
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("decoder", 1f), ("displayNits", 1000f));
            foreach (var (quality, pal) in new[] { (0, false), (1, false), (2, false), (2, true) })
                foreach (double megahertz in new[] { 1.0, 2.0, 2.8 })
                {
                    double clock = pal ? 10.640685 : 10.738635, centre = pal ? CrtFilter.PalSubcarrierMegahertz : CrtFilter.SubcarrierMegahertz;
                    byte[] grating = Paint(512, 224, (x, _) => { double v = 0.45 + 0.25 * Math.Cos(2 * Math.PI * megahertz * x / clock); return (v, v, v); });
                    // The picture's swing at the grating's own frequency, fitted along a band of lines, so that the frame's pixel images are not counted.
                    double Swing(float signal)
                    {
                        Crt picture = Draw("SNES", 512, 224, 1440, 1080, Twice(grating), With(set, ("quality", quality), ("signal", signal), ("standard", pal ? 1f : 0f)))!;
                        double[] across = Enumerable.Range(0, 1440).Select(x => picture.Luminance(x, 440, 1, 200)).ToArray();
                        int left = Array.FindIndex(across, v => v > 0.01), right = Array.FindLastIndex(across, v => v > 0.01);
                        double mean = across.Skip(360).Take(720).Average(), cc = 0, ss = 0, cs = 0, yc = 0, ys = 0;
                        for (int x = 360; x < 1080; x++)
                        {
                            double phase = 2 * Math.PI * megahertz * ((x + 0.5 - left) / (right - left + 1) * 512) / clock, c = Math.Cos(phase), sn = Math.Sin(phase), v = across[x] - mean;
                            cc += c * c; ss += sn * sn; cs += c * sn; yc += v * c; ys += v * sn;
                        }
                        double det = cc * ss - cs * cs, a = (yc * ss - ys * cs) / det, b = (ys * cc - yc * cs) / det;
                        return Math.Sqrt(a * a + b * b);
                    }
                    if (CrtDevice.Value is null) return;
                    double kept = Swing(2f) / Swing(1f), trap = CrtFilter.TrapGain(megahertz, centre);
                    Assert.True(Math.Abs(kept - trap) < 0.04, $"quality {quality}{(pal ? " on PAL" : "")} at {megahertz} MHz: composite keeps {kept:F3} of S-Video's swing, the trap {trap:F3}");
                }
        }, default);

        // The CXA2025AS decodes NTSC only: on PAL the demodulator is the standard's whatever is chosen.
        [Fact]
        public void On_pal_the_demodulator_is_the_standard_s()
        {
            var values = CrtFilter.Parameters.ToDictionary(p => p.Id, p => p.Initial);
            foreach (var (id, value) in CrtFilter.ConsoleTiming["SNES"]) values[id] = value;
            values["axes"] = 1;
            Assert.Equal(CrtFilter.Demodulators[1], CrtFilter.Resolve(values).Demodulator);
            values["standard"] = 1;
            Assert.Equal(CrtFilter.Demodulators[0], CrtFilter.Resolve(values).Demodulator);
        }

        // Grey has no colour difference, so every demodulator draws it alike; a colour through the CXA2025AS's axes is the colour its rows make, drawn through the standard's.
        [Fact]
        public Task A_colour_through_the_datasheet_s_axes_is_the_colour_its_rows_make() => Session.Dispatch(() =>
        {
            (string, float)[] set = With(Plain, ("mask", 1f), ("subpixels", 0f), ("spotGrowth", 0f), ("signal", 1f), ("displayNits", 1000f));
            (double R, double G, double B)[] colours = { (0.5, 0.5, 0.5), (0.55, 0.4, 0.35), (0.35, 0.45, 0.55), (0.45, 0.5, 0.4) };
            foreach (int quality in new[] { 0, 2 })
                foreach (int axes in new[] { 1, 2 })
                {
                    double[][] m = CrtFilter.AxesMatrix(CrtFilter.Demodulators[axes]);
                    byte[] source = Paint(256, 224, (x, _) => colours[x / 64]);
                    byte[] made = Paint(256, 224, (x, _) =>
                    {
                        var (r, g, b) = colours[x / 64];
                        (r, g, b) = (Math.Round(255 * r) / 255, Math.Round(255 * g) / 255, Math.Round(255 * b) / 255);
                        double y = 0.299 * r + 0.587 * g + 0.114 * b, by = b - y, ry = r - y;
                        return (y + m[0][0] * by + m[0][1] * ry, y + m[1][0] * by + m[1][1] * ry, y + by);
                    });
                    if (Draw("SNES", 256, 224, 1440, 1080, Twice(source), With(set, ("quality", quality), ("axes", axes))) is not { } through) return;
                    Crt standard = Draw("SNES", 256, 224, 1440, 1080, Twice(made), With(set, ("quality", quality)))!;
                    for (int band = 0; band < 4; band++)
                    {
                        double[] a = through.Mean(band * 360 + 120, 400, 120, 280), b = standard.Mean(band * 360 + 120, 400, 120, 280);
                        double d = Enumerable.Range(0, 3).Max(c => Math.Abs(a[c] - b[c]));
                        Assert.True(d < 0.006, $"quality {quality}, axes {axes}, colour {band}: {d:F4} apart");
                    }
                }
        }, default);
    }
}

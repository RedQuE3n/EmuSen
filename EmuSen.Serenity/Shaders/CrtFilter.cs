using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity.Shaders
{
    // A CRT computed from its display chain: the console's encoder, the receiver's decoder, the beam, the phosphors, the mask and the glass - see EmuSen_CRT.md §5 and §11.
    public static class CrtFilter
    {
        public const string Name = "CRT";

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

        // The tails' strength at the setting's 1: a sustained white leaves 4% of its green in the first dark frame, the figure the vision literature gives - see EmuSen_CRT.md §11.4.
        public const double TailScale = 0.1606;

        // The wide glare Gaussian solved from DisplayMate's two checkerboard contrasts, as a share of the picture's width and of the light - see EmuSen_CRT.md §11.6.
        public const double GlareSigma = 0.0668, GlareWeight = 0.0299;

        // A black of 0.01 cd/m² under a white of 176, the PVM-20L5's - see EmuSen_CRT.md §3.3.
        public const double BlackLevel = 0.01 / 176.0;

        // The standard's active line in microseconds and a field's active lines, which a console's picture is placed inside - see EmuSen_CRT.md §11.1.
        public const double StandardActiveMicroseconds = 52.66, StandardFieldLines = 241.5, SubcarrierMegahertz = 3.579545;

        private static SlangParameter Choice(string id, string label, int initial, params string[] choices) =>
            new(id, label, initial, 0, choices.Length - 1, 1) { Choices = choices };

        // Where white sits on the display when the picture is drawn bright, as a share of its peak - see EmuSen_CRT.md §12.3.
        public const double BrightWhite = 0.75;

        public static IReadOnlyList<SlangParameter> Parameters { get; } = new SlangParameter[]
        {
            Choice("quality", "Quality", (int)Quality.Balanced, "Performance", "Balanced", "Accurate"),
            Choice("signal", "Signal", (int)Signal.Composite, "RGB", "S-Video", "Composite"),
            Choice("screen", "Screen", DefaultScreen, Screens.Select(s => s.Label).ToArray()),
            Choice("colour", "Colour", 0, Colours.Select(c => c.Label).ToArray()),
            new("curvature", "Curvature", 1f, 0f, 2f, 0.05f),
            new("overscan", "Overscan (0 shows the whole picture)", 0f, 0f, 10f, 0.5f),
            Choice("level", "Picture brightness", 0, "Bright", "The tube's own"),
            new("displayNits", "Display brightness, cd/m²", 300f, 100f, 1000f, 10f),
            Choice("subpixels", "Display subpixels", 1, "None", "RGB", "BGR"),
            Choice("gamut", "Display colours", 0, Displays.Select(d => d.Label).ToArray()),
            Choice("mask", "Mask", 0, "The screen's own", "None", "Aperture grille", "Slot mask", "Dot mask"),
            new("maskPitch", "Mask pitch, relative to the screen's", 1f, 0.5f, 3f, 0.05f),
            new("maskDepth", "Mask depth", 1f, 0f, 1f, 0.05f),
            new("spotSize", "Beam spot size, relative to the screen's", 1f, 0.5f, 2f, 0.05f),
            new("spotGrowth", "Beam spot growth with brightness", 1f, 0f, 2f, 0.05f),
            Choice("decoder", "Luma and chroma separation", 0, "The screen's own", "Notch", "Comb"),
            new("chroma", "Chroma bandwidth, relative to the screen's", 1f, 0.5f, 3f, 0.1f),
            new("glare", "Glare in the glass", 1f, 0f, 3f, 0.1f),
            new("persistence", "Phosphor persistence", 1f, 0f, 6f, 0.1f),
            new("convergence", "Convergence error", 1f, 0f, 3f, 0.1f),
            new("tubeNits", "Tube white, cd/m²", 100f, 50f, 300f, 5f),
            new("gamma", "Tube gamma", 2.4f, 2.0f, 2.8f, 0.05f),
            new("contrast", "Contrast", 1f, 0.5f, 1.5f, 0.01f),
            new("brightness", "Brightness", 0f, -0.1f, 0.1f, 0.005f),
            Choice("interlace", "Interlaced pictures", 0, "Woven", "Fields"),
        };

        // The parameters that decide which passes exist or what is written into them as constants.
        public static IReadOnlyList<string> Structural { get; } = new[] { "quality", "signal", "screen", "colour", "gamut", "mask", "decoder", "chroma" };

        private static Dictionary<string, float> Timing(double activeMicroseconds, double cyclesAcross, double phaseLine, double phaseFrame) => new()
        {
            ["activeUs"] = (float)activeMicroseconds, ["cyclesAcross"] = (float)cyclesAcross, ["phaseLine"] = (float)phaseLine, ["phaseFrame"] = (float)phaseFrame,
            ["lumaMHz"] = 5.0f, ["chromaMHz"] = 1.3f,
        };

        // Each console's subcarrier against its pixels, derived from its clocks; the last entry is a standard line - see EmuSen_CRT.md §3.2.
        public static IReadOnlyDictionary<string, IReadOnlyDictionary<string, float>> ConsoleTiming { get; } = new Dictionary<string, IReadOnlyDictionary<string, float>>
        {
            ["NES"] = Timing(256 / 5.3693175, 256 * 2.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
            ["SNES"] = Timing(256 / 5.3693175, 256 * 2.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
            ["Genesis"] = Timing(256 / 5.3693175, 256 * 2.0 / 3.0, 0, 0),
            ["N64"] = Timing(640 / 12.1704545, 640 * 4 / 13.6, 0.5, 0.5),
            [ScreenFilter.AnyConsole] = Timing(StandardActiveMicroseconds, StandardActiveMicroseconds * SubcarrierMegahertz, 0.5, 0.5),
        };

        // Not yet one of ScreenFilters' choices: a frontend offers it when its settings window can draw its choices - see EmuSen_CRT.md §11.11.
        public static ScreenFilter Filter { get; } = new(Name, Array.Empty<FilterPass>(), new[] { "NES", "SNES", "Genesis", "N64" },
            "EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md", Parameters)
        {
            Build = Passes,
            Structural = Structural,
            ConsoleDefaults = ConsoleTiming,
            Aspect = 4.0 / 3.0,
            RowsOnce = true,
        };

        private static string N(double value) => value.ToString("0.0#######", CultureInfo.InvariantCulture);

        private static int At(IReadOnlyDictionary<string, float> values, string id, int count) => Math.Clamp((int)MathF.Round(values[id]), 0, count - 1);

        // What the screen and the player's overrides come to: the mask drawn, the decoder used and the chroma channel's width in subcarrier cycles.
        public sealed record Resolved(Screen Screen, Signal Signal, Mask Mask, Decoder Decoder, int ChromaCycles, Colour Colour, Colour Display, Quality Quality, int Samples);

        public static Resolved Resolve(IReadOnlyDictionary<string, float> values)
        {
            Screen screen = Screens[At(values, "screen", Screens.Count)];
            var quality = (Quality)At(values, "quality", 3);
            int mask = At(values, "mask", 5), decoder = At(values, "decoder", 3);
            double megahertz = screen.ChromaMegahertz * values["chroma"];

            // A two-line comb has nothing to subtract when a console repeats its phase every line, and a set with one falls back to its notch - see EmuSen_CRT.md §11.3.
            double phase = values["phaseLine"] * 2 * Math.PI, combGain = (1 - Math.Cos(phase)) / 2;
            Decoder wanted = decoder == 0 ? screen.Decoder : (Decoder)(decoder - 1);
            return new Resolved(screen, (Signal)At(values, "signal", 3), mask == 0 ? screen.Mask : (Mask)(mask - 1),
                wanted == Decoder.Comb && combGain < 0.25 ? Decoder.Notch : wanted,
                Math.Clamp((int)Math.Round(0.72 * SubcarrierMegahertz / megahertz), 1, 6), Colours[At(values, "colour", Colours.Count)], Displays[At(values, "gamut", Displays.Count)],
                quality, quality == Quality.Accurate ? SignalSamples : (int)Math.Round(4 * values["cyclesAcross"]));
        }

        public static IReadOnlyList<FilterPass> Passes(IReadOnlyDictionary<string, float> values)
        {
            Resolved r = Resolve(values);
            if (r.Quality == Quality.Performance)
                return new[]
                {
                    new FilterPass(Blend(r, values), PassScale.Source) { Name = "gun", Float = true, Width = new PassSize(PassAxis.Source, 2) },
                    new FilterPass(Face(r), PassScale.Viewport) { Linear = new[] { "gun" } },
                };
            var signal = new PassSize(PassAxis.Fixed, r.Samples);
            FilterPass Line(string sksl, string name, bool feedback = false, params string[] linear) =>
                new(sksl, PassScale.Source) { Width = signal, Float = true, Name = name, Feedback = feedback, Linear = linear };
            FilterPass Small(string sksl, string name, params string[] linear) =>
                new(sksl, PassScale.Source) { Width = PassSize.Fixed(GlareWidth), Height = PassSize.Fixed(GlareHeight), Float = true, Name = name, Linear = linear };

            var passes = new List<FilterPass>();
            if (r.Signal == Signal.Rgb) passes.Add(Line(Encode(r), "gun"));
            else
            {
                passes.Add(Line(Encode(r), "signal"));
                passes.Add(Line(Separate(r, values), "parts"));
                passes.Add(Line(Drive(r), "gun"));
            }
            passes.Add(Line(State, "state", feedback: true));
            bool accurate = r.Quality == Quality.Accurate;
            if (accurate)
            {
                passes.Add(Line(Beam(r, spot: false), "beam"));
                passes.Add(Line(Beam(r, spot: true), "spot"));
            }
            string lit = accurate ? "beam" : "gun";
            passes.Add(Small(GlareGather(lit), "small", lit));
            passes.Add(Small(GlareBlur(horizontal: true), "wide"));
            passes.Add(Small(GlareBlur(horizontal: false), "haze"));
            passes.Add(new FilterPass(Face(r), PassScale.Viewport) { Linear = accurate ? new[] { "beam", "spot", "haze" } : new[] { "gun", "state", "haze" } });
            return passes;
        }

        // Abramowitz and Stegun 7.1.26, and the normal distribution's integral from it.
        private const string Functions = @"
float erf1(float x) {
    float t = 1.0 / (1.0 + 0.3275911 * abs(x));
    float y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * exp(-x * x);
    return x < 0.0 ? -y : y;
}
float cdf(float x) { return 0.5 + 0.5 * erf1(x * 0.70710678); }
float3 cdf3(float3 x) { return float3(cdf(x.r), cdf(x.g), cdf(x.b)); }
";

        // The tube's transfer in BT.1886's form, with the white at 1 and the black at the measured ratio.
        private static string Transfer => @"
uniform float gamma;
uniform float contrast;
uniform float brightness;
float3 light(float3 v) {
    float lb = pow(" + N(BlackLevel) + @", 1.0 / gamma);
    float b = lb / (1.0 - lb);
    float a = pow(1.0 - lb, gamma);
    return a * pow(max(contrast * v + brightness + b, float3(0.0)), float3(gamma));
}
";

        // The subcarrier's phase, in cycles, at a sample of a line of a frame.
        private const string Phase = @"
uniform float frameCount;
uniform float cyclesAcross;
uniform float phaseLine;
uniform float phaseFrame;
float phaseAt(float index, float row, float samples) {
    return fract((index + 0.5) / samples * cyclesAcross + fract(row * phaseLine) + mod(frameCount, 2.0) * phaseFrame);
}
";

        // The console's picture through its encoder's band limits, each pixel a box in time under a Gaussian - see EmuSen_CRT.md §11.3.
        private static string Encode(Resolved r) => @"
uniform shader original;
uniform float2 originalSize;
uniform float2 outputSize;
uniform float activeUs;
uniform float lumaMHz;
uniform float chromaMHz;
" + Functions + (r.Signal == Signal.Rgb ? Transfer : Phase) + @"
half4 main(float2 coord) {
    float width = originalSize.x;
    float x = coord.x / outputSize.x * width;
    float row = floor(coord.y) + 0.5;
    float rate = width / activeUs;
    float sy = 0.132532 * rate * " + (r.Signal == Signal.Rgb ? "sqrt(1.0 / (lumaMHz * lumaMHz) + " + N(1 / (r.Screen.VideoMegahertz * r.Screen.VideoMegahertz)) + ")" : "(1.0 / lumaMHz)") + @";
    float sc = 0.132532 / chromaMHz * rate;
    float first = floor(x);
    float3 wide = float3(0.0);
    float3 narrow = float3(0.0);
    for (int i = -4; i <= 4; i++) {
        float k = first + float(i);
        if (k < 0.0 || k >= width) continue;
        float3 rgb = original.eval(float2(k + 0.5, row)).rgb;
        float y = dot(rgb, float3(0.299, 0.587, 0.114));
        float wy = cdf((k + 1.0 - x) / sy) - cdf((k - x) / sy);
        float wc = cdf((k + 1.0 - x) / sc) - cdf((k - x) / sc);
" + (r.Signal == Signal.Rgb ? @"
        wide += wy * rgb;
    }
    return half4(light(wide), 1.0);
}" : @"
        wide.x += wy * y;
        narrow += wc * float3(0.0, 0.492 * (rgb.b - y), 0.877 * (rgb.r - y));
    }
    float angle = 6.2831853 * phaseAt(floor(coord.x), floor(coord.y), outputSize.x);
    float c = narrow.y * sin(angle) + narrow.z * cos(angle);
" + (r.Signal == Signal.Composite ? "    return half4(wide.x + c, 0.0, 0.0, 1.0);\n}" : "    return half4(wide.x, c, 0.0, 1.0);\n}"));

        // The receiver's first half: luma and chroma separated by a notch or a two-line comb, and chroma demodulated - see EmuSen_CRT.md §11.3.
        private static string Separate(Resolved r, IReadOnlyDictionary<string, float> values)
        {
            double theta = values["phaseLine"] * 2 * Math.PI, fr = (1 - Math.Cos(theta)) / 2, fi = Math.Sin(theta) / 2, gain = fr * fr + fi * fi;
            bool comb = r.Signal == Signal.Composite && r.Decoder == Decoder.Comb;
            string read = r.Signal == Signal.SVideo ? "source.eval(float2(x + 0.5, row + 0.5)).g" : "source.eval(float2(x + 0.5, row + 0.5)).r";
            string above = "(row < 1.0 ? 0.0 : source.eval(float2(x + 0.5, row - 0.5)).r)";

            // The loop reaches as far as the longer window does on this console, and no further.
            int reach = (int)Math.Ceiling(Math.Max(r.ChromaCycles, 2) * r.Samples / values["cyclesAcross"] / 2);
            return @"
uniform shader source;
uniform float2 inputSize;
" + Phase + @"
half4 main(float2 coord) {
    float s = floor(coord.x);
    float row = floor(coord.y);
    float perCycle = inputSize.x / cyclesAcross;
    float chromaHalf = " + N(r.ChromaCycles) + @" * perCycle * 0.5;
    float notchHalf = perCycle;
    float2 chroma = float2(0.0);
    float2 notch = float2(0.0);
    float chromaSum = 0.0;
    float notchSum = 0.0;
    for (int j = -" + reach + "; j <= " + reach + @"; j++) {
        float x = s + float(j);
        float t = abs(float(j));
        if (x < 0.0 || x >= inputSize.x || t >= max(chromaHalf, notchHalf)) continue;
        float v = " + (comb ? $"0.5 * ({read} - {above})" : read) + @";
        float angle = 6.2831853 * phaseAt(x, row, inputSize.x);
        float2 axes = float2(sin(angle), cos(angle));
        float wc = t < chromaHalf ? 0.5 + 0.5 * cos(3.14159265 * t / chromaHalf) : 0.0;
        float wn = t < notchHalf ? 0.5 + 0.5 * cos(3.14159265 * t / notchHalf) : 0.0;
        chroma += wc * v * axes;
        notch += wn * v * axes;
        chromaSum += wc;
        notchSum += wn;
    }
    chroma *= 2.0 / max(chromaSum, 0.0001);
    notch *= 2.0 / max(notchSum, 0.0001);
" + (comb ? @"
    chroma = float2(" + N(fr / gain) + " * chroma.x + " + N(fi / gain) + " * chroma.y, " + N(fr / gain) + " * chroma.y - " + N(fi / gain) + @" * chroma.x);
    notch = float2(" + N(fr / gain) + " * notch.x + " + N(fi / gain) + " * notch.y, " + N(fr / gain) + " * notch.y - " + N(fi / gain) + @" * notch.x);
" : "") + @"
    float here = 6.2831853 * phaseAt(s, row, inputSize.x);
    float y = source.eval(float2(s + 0.5, row + 0.5)).r" + (r.Signal == Signal.SVideo ? "" : " - (notch.x * sin(here) + notch.y * cos(here))") + @";
    return half4(y, chroma.x, chroma.y, 1.0);
}";
        }

        // The receiver's second half: its video bandwidth on the luma, the standard's matrix, and the tube's transfer - see EmuSen_CRT.md §11.3.
        private static string Drive(Resolved r) => @"
uniform shader source;
uniform float2 inputSize;
uniform float activeUs;
" + Transfer + @"
half4 main(float2 coord) {
    float s = floor(coord.x);
    float row = floor(coord.y) + 0.5;
    float sigma = 0.132532 / " + N(r.Screen.VideoMegahertz) + @" * inputSize.x / activeUs;
    float y = 0.0;
    float sum = 0.0;
    for (int j = -6; j <= 6; j++) {
        float x = s + float(j);
        if (x < 0.0 || x >= inputSize.x) continue;
        float w = exp(-0.5 * float(j * j) / (sigma * sigma));
        y += w * source.eval(float2(x + 0.5, row)).r;
        sum += w;
    }
    y /= sum;
    half4 here = source.eval(float2(s + 0.5, row));
    float ry = here.b / 0.877;
    float by = here.g / 0.492;
    float gy = -(0.299 * ry + 0.114 * by) / 0.587;
    return half4(light(float3(y + ry, y + gy, y + by)), 1.0);
}";

        // The Performance tier's signal: the frame's own pixels, luma and chroma blurred at the bandwidths the cable and the receiver leave them, no subcarrier - see EmuSen_CRT.md §12.1.
        private static string Blend(Resolved r, IReadOnlyDictionary<string, float> values)
        {
            double video = r.Screen.VideoMegahertz, chroma = 0.72 * SubcarrierMegahertz / r.ChromaCycles;
            bool rgb = r.Signal == Signal.Rgb, notch = r.Signal == Signal.Composite && r.Decoder == Decoder.Notch;

            // The notch acts on the encoder's output and the video stage after it, whose gains at the subcarrier scale the band it takes away.
            double encoder = values["lumaMHz"], through = Math.Exp(-0.5 * Math.Log(2) * SubcarrierMegahertz * SubcarrierMegahertz * (1 / (encoder * encoder) + 1 / (video * video)));
            return @"
uniform shader original;
uniform float2 originalSize;
uniform float activeUs;
uniform float lumaMHz;
uniform float chromaMHz;
uniform float cyclesAcross;
" + Functions + Transfer + @"

// The integral of the receiver's notch band, a Hann window two subcarrier cycles long times the subcarrier, from the window's centre.
float band(float u, float reach, float w) {
    u = clamp(u, -reach, reach);
    float a = 3.14159265 / reach;
    return (2.0 / reach) * (0.5 * sin(w * u) / w + 0.25 * (sin((w + a) * u) / (w + a) + sin((w - a) * u) / (w - a)));
}

half4 main(float2 coord) {
    float width = originalSize.x;
    float x = coord.x * 0.5;
    float row = floor(coord.y) + 0.5;
    float rate = width / activeUs;
    float reach = width / cyclesAcross;
    float w = 6.2831853 / reach;
    float sy = 0.132532 * rate * sqrt(1.0 / (lumaMHz * lumaMHz) + " + N(1 / (video * video)) + @");
    float sc = 0.132532 * rate * sqrt(1.0 / (chromaMHz * chromaMHz) + " + N(1 / (chroma * chroma)) + @");
    float first = floor(x);
    float3 wide = float3(0.0);
    float2 narrow = float2(0.0);
    for (int i = -4; i <= 4; i++) {
        float k = first + float(i);
        if (k < 0.0 || k >= width) continue;
        float3 rgb = original.eval(float2(k + 0.5, row)).rgb;
        float wy = cdf((k + 1.0 - x) / sy) - cdf((k - x) / sy)" + (notch ? " - " + N(through) + " * (band(x - k, reach, w) - band(x - k - 1.0, reach, w))" : "") + @";
" + (rgb ? @"        wide += wy * rgb;
    }
    return half4(light(wide), 1.0);
}" : @"        float wc = cdf((k + 1.0 - x) / sc) - cdf((k - x) / sc);
        float y = dot(rgb, float3(0.299, 0.587, 0.114));
        wide.x += wy * y;
        narrow += wc * float2(rgb.b - y, rgb.r - y);
    }
    float gy = -(0.299 * narrow.y + 0.114 * narrow.x) / 0.587;
    return half4(light(float3(wide.x + narrow.y, wide.x + gy, wide.x + narrow.x)), 1.0);
}");
        }

        // Green's and blue's slow light, two sums each over the frames before; red's is gone inside the frame - see EmuSen_CRT.md §11.4.
        private static string State => @"
uniform shader source;
uniform shader feedback;
half4 main(float2 coord) {
    float3 e = source.eval(coord).rgb;
    half4 f = feedback.eval(coord);
    return half4(e.g + " + N(TailFast) + " * f.r, e.g + " + N(TailSlow) + " * f.g, e.b + " + N(TailFast) + " * f.b, e.b + " + N(TailSlow) + @" * f.a);
}";

        // Each sample's light with its tail, and the spot it is drawn with: wider as the gun is driven harder - see EmuSen_CRT.md §11.5.
        private static string Spot(Resolved r) => @"
uniform shader gun;
uniform shader state;
uniform float persistence;
uniform float spotSize;
uniform float spotGrowth;
float3 lightAt(float2 p) {
    float3 e = gun.eval(p).rgb;
    half4 st = state.eval(p);
    float2 fast = persistence * float2(" + N(TailScale * GreenFast) + ", " + N(TailScale * BlueFast) + @");
    float2 slow = persistence * float2(" + N(TailScale * GreenSlow) + ", " + N(TailScale * BlueSlow) + @");
    float2 kept = 1.0 / (1.0 + fast * " + N(TailFast / (1 - TailFast)) + " + slow * " + N(TailSlow / (1 - TailSlow)) + @");
    float2 now = float2(e.g, e.b);
    float2 lit = kept * (now + fast * (float2(st.r, st.b) - now) + slow * (float2(st.g, st.a) - now));
    return float3(e.r, lit.x, lit.y);
}
float3 variance(float3 l) {
    float low = " + N(r.Screen.SpotMinimum * r.Screen.SpotMinimum) + @";
    float high = " + N(r.Screen.SpotMaximum * r.Screen.SpotMaximum) + @";
    return spotSize * spotSize * (low + (high - low) * spotGrowth * clamp(l, 0.0, 1.5));
}
";

        // The spot's own width along the line, scattered from each sample; the second pass keeps the energy-weighted variance for the width across lines - see EmuSen_CRT.md §11.5.
        private static string Beam(Resolved r, bool spot) => @"
uniform float2 outputSize;
uniform float activeUs;
" + Functions + Spot(r) + @"
half4 main(float2 coord) {
    float s = floor(coord.x);
    float row = floor(coord.y) + 0.5;
    float perLine = outputSize.x * 0.75 / (" + N(StandardFieldLines) + " * activeUs / " + N(StandardActiveMicroseconds) + @");
    float3 sum = float3(0.0);
    float3 weighted = float3(0.0);
    for (int t = -8; t <= 8; t++) {
        float x = s + float(t);
        if (x < 0.0 || x >= outputSize.x) continue;
        float3 l = lightAt(float2(x + 0.5, row));
        float3 v = variance(l);
        float3 sigma = sqrt(v) * perLine;
        float3 w = cdf3((0.5 - float(t)) / sigma) - cdf3((-0.5 - float(t)) / sigma);
        sum += l * w;
        weighted += l * w * v;
    }
" + (spot ? "    return half4(weighted / max(sum, float3(0.000001)) + step(sum, float3(0.000001)) * variance(float3(0.0)), 1.0);\n}" : "    return half4(sum, 1.0);\n}");

        // Where a point of the tube's face falls in the console's picture: the standard raster, zoomed by the overscan, with the picture inside it - see EmuSen_CRT.md §11.1.
        private static string Raster => @"
uniform float activeUs;
uniform float overscan;
float2 pictureOf(float2 face, float rows) {
    float fieldRows = rows > 300.0 ? rows * 0.5 : rows;
    float2 fill = float2(activeUs / " + N(StandardActiveMicroseconds) + ", fieldRows / " + N(StandardFieldLines) + @");
    float zoom = overscan > 0.0 ? 1.0 + overscan * 0.01 : min(1.0 / fill.x, 1.0 / fill.y);
    return face / (fill * zoom * float2(1.0, 0.75)) * 0.5 + 0.5;
}
";

        // The picture's light gathered into cells of the face, for the glass to spread - see EmuSen_CRT.md §11.6.
        private static string GlareGather(string lit) => (@"
uniform shader beam;
uniform float2 beamSize;
uniform float2 outputSize;
" + Raster + @"
half4 main(float2 coord) {
    float3 sum = float3(0.0);
    for (int i = 0; i < 6; i++) {
        for (int j = 0; j < 3; j++) {
            float2 cell = (floor(coord) + float2((float(i) + 0.5) / 6.0, (float(j) + 0.5) / 3.0)) / outputSize;
            float2 p = pictureOf((cell * 2.0 - 1.0) * float2(1.0, 0.75), beamSize.y);
            if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0) continue;
            sum += beam.eval(p * beamSize).rgb;
        }
    }
    return half4(sum / 18.0, 1.0);
}").Replace("beam", lit);

        // One axis of the glare's Gaussian; light that would land outside the face is lost.
        private static string GlareBlur(bool horizontal) => @"
uniform shader source;
uniform float2 inputSize;
half4 main(float2 coord) {
    float sigma = " + N(GlareSigma) + @" * inputSize.x;
    float3 sum = float3(0.0);
    for (int t = -24; t <= 24; t++) {
        float2 p = floor(coord) + 0.5 + " + (horizontal ? "float2(float(t), 0.0)" : "float2(0.0, float(t))") + @";
        if (p.x < 0.0 || p.x > inputSize.x || p.y < 0.0 || p.y > inputSize.y) continue;
        sum += source.eval(p).rgb * exp(-0.5 * float(t * t) / (sigma * sigma));
    }
    return half4(sum / " + N(Enumerable.Range(-24, 49).Sum(t => Math.Exp(-0.5 * t * t / (GlareSigma * GlareWidth * GlareSigma * GlareWidth)))) + @", 1.0);
}";

        // One colour's stripes or dots: where its lit run starts in the triad, how wide it is, and whether its columns sit half a pitch down.
        private sealed record Layout(double Width, double[] Start, bool[] Shifted, double VerticalPitch, double VerticalLit)
        {
            public double LitFraction => Width * (VerticalPitch > 0 ? VerticalLit : 1);
        }

        // The masks' proportions, in triads; fitted to photographs where EmuSen_CRT.md §11.7 says so and choices elsewhere.
        private static Layout LayoutOf(Mask mask) => mask switch
        {
            Mask.ApertureGrille => new Layout(StripeWidth, Thirds(StripeWidth), new[] { false, false, false }, 0, 1),
            Mask.Slot => new Layout(SlotWidth, new[] { 0.15 - SlotWidth / 2, 0.45 - SlotWidth / 2, 0.75 - SlotWidth / 2 }, new[] { false, false, false }, SlotPitch, SlotLit),
            Mask.Dot => new Layout(DotWidth, new[] { (1 / 3.0 - DotWidth) / 2, 2 / 3.0 + (1 / 3.0 - DotWidth) / 2, 1 / 3.0 + (1 / 3.0 - DotWidth) / 2 }, new[] { false, false, true }, 2 / Math.Sqrt(3), DotWidth * Math.Sqrt(3) / 2),
            _ => new Layout(1, new[] { 0.0, 0.0, 0.0 }, new[] { false, false, false }, 0, 1),
        };

        public const double StripeWidth = 0.199, SlotWidth = 0.19, SlotPitch = 0.81, SlotLit = 0.76, DotWidth = 0.35;

        private static double[] Thirds(double width) => new[] { (1 / 3.0 - width) / 2, 1 / 3.0 + (1 / 3.0 - width) / 2, 2 / 3.0 + (1 / 3.0 - width) / 2 };

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

        private static string Row(double[] row) => $"float3({N(row[0])}, {N(row[1])}, {N(row[2])})";

        // The share of a tent-shaped footprint that a periodic lit run covers, in closed form - see EmuSen_CRT.md §11.7.
        private const string Coverage = @"
float runIntegral(float x, float lit, float period) {
    float n = floor(x / period);
    float f = x - n * period;
    float whole = lit * period * n * (n - 1.0) * 0.5 + n * (lit * period - 0.5 * lit * lit);
    float part = n * lit * f + (f < lit ? 0.5 * f * f : lit * (f - 0.5 * lit));
    return whole + part;
}
float runLength(float x, float lit, float period) {
    float n = floor(x / period);
    return n * lit + min(x - n * period, lit);
}
float boxed(float x, float reach, float start, float lit, float period) {
    float base = floor((x - reach - start) / period) * period + start;
    float u = x - base;
    float h = max(reach, 0.0001);
    return (runLength(u + h, lit, period) - runLength(u - h, lit, period)) / (2.0 * h);
}
float covered(float x, float reach, float start, float lit, float period) {
    float base = floor((x - reach - start) / period) * period + start;
    float u = x - base;
    float h = max(reach, 0.0001);
    return (runIntegral(u + h, lit, period) - 2.0 * runIntegral(u, lit, period) + runIntegral(u - h, lit, period)) / (h * h);
}
";

        // The tube's face seen from the front: the glass's curve, each gun's beam, the mask per subpixel, the glare, and the display's limits; the lower tiers draw less of it - see EmuSen_CRT.md §11 and §12.
        private static string Face(Resolved r)
        {
            Layout layout = LayoutOf(r.Mask);
            double[][] m = GunsToDisplay(r.Colour, r.Display);
            double radius = r.Screen.RadiusOverWidth * 2, distance = 6;
            double halfWidth = r.Screen.WidthMillimetres / 2;
            bool accurate = r.Quality == Quality.Accurate, performance = r.Quality == Quality.Performance;

            // Accurate weighs the mask over a tent two pixels wide, the others over the pixel's box.
            string cover = accurate ? "covered" : "boxed";
            // The rows of a slot or dot mask are the same two for every colour, level and half a pitch down, so they are weighed once.
            string rowsOf = layout.VerticalPitch <= 0 ? "" :
                $"    float rowLevel = {cover}(triad.y, footprint.y, 0.0, {N(layout.VerticalPitch * layout.VerticalLit)}, {N(layout.VerticalPitch)});\n"
                + $"    float rowDown = {cover}(triad.y, footprint.y, {N(layout.VerticalPitch * 0.5)}, {N(layout.VerticalPitch * layout.VerticalLit)}, {N(layout.VerticalPitch)});\n";
            string mask = r.Mask == Mask.None ? "    cover = float3(1.0);\n" : rowsOf + string.Concat(Enumerable.Range(0, 3).Select(c =>
            {
                string channel = "rgb"[c].ToString();
                string phase = $"(triad.x + shift.{channel} * grain.x * across)";
                if (layout.VerticalPitch <= 0) return $"    cover.{channel} = {cover}({phase}, footprint.x, {N(layout.Start[c])}, {N(layout.Width)}, 1.0);\n";
                string even = $"{cover}({phase}, footprint.x, {N(layout.Start[c])}, {N(layout.Width)}, 2.0)";
                string odd = $"{cover}({phase}, footprint.x, {N(layout.Start[c] + 1)}, {N(layout.Width)}, 2.0)";
                return layout.Shifted[c] ? $"    cover.{channel} = {even} * rowDown + {odd} * rowLevel;\n" : $"    cover.{channel} = {even} * rowLevel + {odd} * rowDown;\n";
            }));

            string beams = accurate ? @"
uniform shader beam;
uniform shader spot;
uniform float2 beamSize;
float beamOf(float2 picture, float tall, float channel, float field, inout float lineMean, inout float linePeak) {
    float2 p = picture * beamSize;
    if (picture.x < 0.0 || picture.x > 1.0) return 0.0;
    float scale = beamSize.y > 300.0 ? 2.0 : 1.0;
    float first = floor(p.y);
    float sum = 0.0;
    for (int j = -2; j <= 2; j++) {
        float row = first + float(j);
        if (row < 0.0 || row >= beamSize.y) continue;
        float2 at = float2(p.x, row + 0.5);
        half4 l = beam.eval(at);
        half4 v = spot.eval(at);
        float light = channel < 0.5 ? l.r : channel < 1.5 ? l.g : l.b;
        float sigma = scale * sqrt(max(channel < 0.5 ? v.r : channel < 1.5 ? v.g : v.b, 0.000001));
        float d = p.y - (row + 0.5);
        float lit = scale > 1.5 && interlace > 0.5 ? (mod(row + field, 2.0) < 0.5 ? 2.0 : 0.0) : 1.0;
        sum += lit * light * (cdf((d + 0.5 * tall) / sigma) - cdf((d - 0.5 * tall) / sigma)) / tall;
        float near = max(1.0 - abs(d) / scale, 0.0);
        lineMean += lit * light * near / scale;
        if (near > 0.0) linePeak = max(linePeak, lit * light / (sigma * 2.5066283));
    }
    return sum;
}
" : @"
uniform shader gun;
uniform float2 gunSize;
uniform float spotSize;
uniform float spotGrowth;
" + (performance ? @"
float3 lightAt(float2 p) { return gun.eval(p).rgb; }
" : @"
uniform shader state;
uniform float persistence;
float3 lightAt(float2 p) {
    float3 e = gun.eval(p).rgb;
    half4 st = state.eval(p);
    float2 fast = persistence * float2(" + N(TailScale * GreenFast) + ", " + N(TailScale * BlueFast) + @");
    float2 slow = persistence * float2(" + N(TailScale * GreenSlow) + ", " + N(TailScale * BlueSlow) + @");
    float2 kept = 1.0 / (1.0 + fast * " + N(TailFast / (1 - TailFast)) + " + slow * " + N(TailSlow / (1 - TailSlow)) + @");
    float2 now = float2(e.g, e.b);
    float2 lit = kept * (now + fast * (float2(st.r, st.b) - now) + slow * (float2(st.g, st.a) - now));
    return float3(e.r, lit.x, lit.y);
}
") + @"
float3 beamsOf(float2 picture, float3 across, float field, inout float3 lineMean, inout float3 linePeak) {
    float2 p = picture * gunSize;
    if (picture.x < 0.0 || picture.x > 1.0) return float3(0.0);
    float scale = gunSize.y > 300.0 ? 2.0 : 1.0;
    float first = floor(p.y - 0.5);
    float3 sum = float3(0.0);
    for (int j = " + (performance ? "0" : "-1") + @"; j <= " + (performance ? "1" : "1") + @"; j++) {
        float row = first + float(j)" + (performance ? "" : " + 0.0") + @";
        if (row < 0.0 || row >= gunSize.y) continue;
        float3 l = " + (performance ? "lightAt(float2(p.x, row + 0.5))" : "float3(lightAt(float2(across.r * gunSize.x, row + 0.5)).r, lightAt(float2(across.g * gunSize.x, row + 0.5)).g, lightAt(float2(across.b * gunSize.x, row + 0.5)).b)") + @";
        " + (performance ? "float" : "float3") + @" v = spotSize * spotSize * (" + N(r.Screen.SpotMinimum * r.Screen.SpotMinimum) + " + " + N(r.Screen.SpotMaximum * r.Screen.SpotMaximum - r.Screen.SpotMinimum * r.Screen.SpotMinimum) + @" * spotGrowth * clamp(" + (performance ? "dot(l, float3(0.299, 0.587, 0.114))" : "l") + @", 0.0, 1.5));
        " + (performance ? "float" : "float3") + @" sigma = scale * sqrt(v);
        float d = p.y - (row + 0.5);
        float lit = scale > 1.5 && interlace > 0.5 ? (mod(row + field, 2.0) < 0.5 ? 2.0 : 0.0) : 1.0;
        sum += lit * l * exp(-0.5 * d * d / (sigma * sigma)) / (sigma * 2.5066283);
        float near = max(1.0 - abs(d) / scale, 0.0);
        lineMean += lit * l * near / scale;
        if (near > 0.0) linePeak = max(linePeak, lit * l / (sigma * 2.5066283));
    }
    return sum;
}
";
            if (!accurate && !performance) beams = beams.Replace("float first = floor(p.y - 0.5);", "float first = floor(p.y);");

            string guns = accurate ? @"
    float spread = convergence * (" + N(r.Screen.ConvergenceCentre / halfWidth / 2) + " + " + N((r.Screen.ConvergenceEdge - r.Screen.ConvergenceCentre) / halfWidth / 2) + @" * dot(face, face) / 1.5625);
    float2 miss = float2(pictureOf(face + float2(spread, 0.0), beamSize.y).x - before.x, 0.0);
    float3 lineMean = float3(0.0);
    float3 linePeak = float3(0.0);
    float3 guns = float3(
        beamOf(before + float2(shift.r * perPixel.x, 0.0) - miss, tall, 0.0, frameCount, lineMean.r, linePeak.r),
        beamOf(before + float2(shift.g * perPixel.x, 0.0), tall, 1.0, frameCount, lineMean.g, linePeak.g),
        beamOf(before + float2(shift.b * perPixel.x, 0.0) + miss, tall, 2.0, frameCount, lineMean.b, linePeak.b));
" : performance ? @"
    float3 lineMean = float3(0.0);
    float3 linePeak = float3(0.0);
    float3 guns = beamsOf(before, float3(before.x), frameCount, lineMean, linePeak);
" : @"
    float spread = convergence * (" + N(r.Screen.ConvergenceCentre / halfWidth / 2) + " + " + N((r.Screen.ConvergenceEdge - r.Screen.ConvergenceCentre) / halfWidth / 2) + @" * dot(face, face) / 1.5625);
    float miss = pictureOf(face + float2(spread, 0.0), gunSize.y).x - before.x;
    float3 lineMean = float3(0.0);
    float3 linePeak = float3(0.0);
    float3 guns = beamsOf(before, before.x + shift * perPixel.x + float3(-miss, 0.0, miss), frameCount, lineMean, linePeak);
";
            string rows = accurate ? "beamSize.y" : "gunSize.y";
            string glass = performance ? @"
    float3 direct = float3(dot(" + Row(m[0]) + ", guns), dot(" + Row(m[1]) + ", guns), dot(" + Row(m[2]) + @", guns));
    float3 even = float3(dot(" + Row(m[0]) + ", lineMean), dot(" + Row(m[1]) + ", lineMean), dot(" + Row(m[2]) + @", lineMean));
    float3 peak = float3(dot(" + Row(m[0]) + ", linePeak), dot(" + Row(m[1]) + ", linePeak), dot(" + Row(m[2]) + @", linePeak));
    float3 veil = float3(0.0);
" : @"
    float spill = " + N(GlareWeight) + @" * glare;
    float3 scattered = haze.eval((face / float2(1.0, 0.75) * 0.5 + 0.5) * hazeSize).rgb;
    float3 direct = (1.0 - spill) * float3(dot(" + Row(m[0]) + ", guns), dot(" + Row(m[1]) + ", guns), dot(" + Row(m[2]) + @", guns));
    float3 even = (1.0 - spill) * float3(dot(" + Row(m[0]) + ", lineMean), dot(" + Row(m[1]) + ", lineMean), dot(" + Row(m[2]) + @", lineMean));
    float3 peak = (1.0 - spill) * float3(dot(" + Row(m[0]) + ", linePeak), dot(" + Row(m[1]) + ", linePeak), dot(" + Row(m[2]) + @", linePeak));
    float3 veil = spill * float3(dot(" + Row(m[0]) + ", scattered), dot(" + Row(m[1]) + ", scattered), dot(" + Row(m[2]) + @", scattered));
";

            return @"
uniform float2 outputSize;
uniform float pixelScale;
uniform float2 pixelOrigin;
uniform float frameCount;
uniform float displayNits;
uniform float tubeNits;
uniform float level;
uniform float subpixels;
uniform float maskPitch;
uniform float maskDepth;
uniform float interlace;
" + (performance ? "uniform float curvature;\n" : @"uniform shader haze;
uniform float2 hazeSize;
uniform float curvature;
uniform float glare;
") + (performance ? "" : "uniform float convergence;\n") + Functions + Raster + Coverage + beams + @"

float2 faceOf(float2 screen) {
" + (@"    if (curvature <= 0.0) return screen;
    float radius = " + N(radius) + @" / curvature;
    float eye = " + N(distance) + @";
    float edgeX = radius - sqrt(radius * radius - 1.0);
    float edgeY = " + (r.Screen.Cylinder ? "0.0" : "radius - sqrt(radius * radius - 0.5625)") + @";
    float2 s = screen * float2(eye / (eye + edgeX), eye / (eye + edgeY));
    float a = " + (r.Screen.Cylinder ? "s.x * s.x" : "dot(s, s)") + @" + eye * eye;
    float b = eye * (eye + radius);
    float c = eye * eye + 2.0 * eye * radius;
    float t = (b - sqrt(max(b * b - a * c, 0.0))) / a;
    return s * t;
") + @"}

float3 encode(float3 c) {
    return float3(c.r <= 0.0031308 ? c.r * 12.92 : 1.055 * pow(c.r, 0.41666667) - 0.055,
                  c.g <= 0.0031308 ? c.g * 12.92 : 1.055 * pow(c.g, 0.41666667) - 0.055,
                  c.b <= 0.0031308 ? c.b * 12.92 : 1.055 * pow(c.b, 0.41666667) - 0.055);
}

half4 main(float2 coord) {
    float2 screen = (coord / outputSize * 2.0 - 1.0) * float2(1.0, 0.75);
    float2 pixel = 2.0 / (outputSize * pixelScale) * float2(1.0, 0.75);
    float2 face = faceOf(screen);
    float2 grain = float2(faceOf(screen + float2(pixel.x, 0.0)).x - face.x, faceOf(screen + float2(0.0, pixel.y)).y - face.y);

    float2 corner = max(abs(face) - float2(1.0, 0.75) + " + N(CornerRadius) + @", 0.0);
    float inside = clamp((" + N(CornerRadius) + @" - length(corner)) / max(grain.x, 0.000001) + 0.5, 0.0, 1.0);
    if (inside <= 0.0) return half4(0.0, 0.0, 0.0, 1.0);

    float3 shift = subpixels < 0.5 ? float3(0.0) : subpixels < 1.5 ? float3(-1.0, 0.0, 1.0) / 3.0 : float3(1.0, 0.0, -1.0) / 3.0;
    float across = " + N(r.Screen.TriadsAcross) + @" * 0.5 / maskPitch;
    float2 triad = face * across;
    float2 footprint = grain * across" + (accurate ? "" : " * 0.5") + @";
    float3 cover;
" + mask + @"
    float litFraction = " + N(layout.LitFraction) + @";

    float2 before = pictureOf(face, " + rows + @");
    float2 perPixel = float2(pictureOf(face + float2(grain.x, 0.0), " + rows + ").x - before.x, pictureOf(face + float2(0.0, grain.y), " + rows + @").y - before.y);
    float tall = perPixel.y * " + rows + @";
" + guns + glass + @"
    float luminance = dot(direct, float3(0.2126, 0.7152, 0.0722));
    float lowest = min(direct.r, min(direct.g, direct.b));
    if (lowest < 0.0) direct = luminance + (direct - luminance) * (luminance / max(luminance - lowest, 0.000001));

    float headroom = level < 0.5 ? " + N(1 / BrightWhite) + @" : displayNits / tubeNits;

    // A scanline brighter at its centre than the display can go is drawn flatter, toward its line's even light, by just enough for its peak.
    direct = even + (direct - even) * clamp((headroom - even) / max(peak - even, float3(0.000001)), 0.0, 1.0);
    float3 depth = maskDepth * clamp((headroom / max(direct, float3(0.000001)) - 1.0) / (1.0 / litFraction - 1.0 + 0.000001), 0.0, 1.0);
    float3 shown = (direct * (1.0 + depth * (cover / litFraction - 1.0)) + veil) / headroom;
" + (performance ? @"    return half4(encode(clamp(shown * inside, 0.0, 1.0)), 1.0);
}" : @"
    float2 device = coord * pixelScale + pixelOrigin;
    float noise = fract(52.9829189 * fract(dot(floor(device), float2(0.06711056, 0.00583715)))) - 0.5;
    return half4(clamp(encode(clamp(shown * inside, 0.0, 1.0)) + noise / 255.0, 0.0, 1.0), 1.0);
}");
        }

        public const double CornerRadius = 0.04;
    }
}

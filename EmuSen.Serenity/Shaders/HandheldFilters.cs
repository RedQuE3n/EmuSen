using System.Globalization;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity.Shaders
{
    // Handheld LCD filters: the panel's colour and response at the game's size, then its pixel grid at the screen's - see EmuSen_Serenity.md §3.5.
    public static class HandheldFilters
    {
        private static string F(double value) => value.ToString("0.0#####", CultureInfo.InvariantCulture);

        private static string Rgb(int rgb) => $"vec3({F(((rgb >> 16) & 0xFF) / 255.0)}, {F(((rgb >> 8) & 0xFF) / 255.0)}, {F((rgb & 0xFF) / 255.0)})";

        // The four greys Mercury draws become the panel's four shades, with the panel's slow response over the last three frames.
        public static string MonochromePanelGlsl(int lightest, int light, int dark, int darkest) => @"
uniform sampler2D original;
uniform sampler2D history1;
uniform sampler2D history2;
uniform sampler2D history3;

uniform float response;
const vec3 p0 = P0;
const vec3 p1 = P1;
const vec3 p2 = P2;
const vec3 p3 = P3;

float Shade(vec3 c) { return dot(c, vec3(0.299, 0.587, 0.114)); }

vec3 Palette(float v) {
    float i = clamp((1.0 - v) * 3.0, 0.0, 3.0);
    if (i < 1.0) return mix(p0, p1, i);
    if (i < 2.0) return mix(p1, p2, i - 1.0);
    return mix(p2, p3, i - 2.0);
}

vec4 filter_main(vec2 coord) {
    float v = Shade(original_eval(coord).rgb);
    v += (Shade(history1_eval(coord).rgb) - v) * response;
    v += (Shade(history2_eval(coord).rgb) - v) * response * response;
    v += (Shade(history3_eval(coord).rgb) - v) * response * response * response;
    return vec4(Palette(v), 1.0);
}
".Replace("P0", Rgb(lightest)).Replace("P1", Rgb(light)).Replace("P2", Rgb(dark)).Replace("P3", Rgb(darkest));

        // Dots with gaps showing the reflector behind them, and the offset shadow each dot casts onto it.
        public static string DotMatrixGlsl(int background, double coverage) => @"
uniform sampler2D source;
uniform vec2 inputSize;
uniform vec2 outputSize;

const vec3 background = BG;
const float coverage = COVER;
uniform float shadowOpacity;
const vec2 shadowOffset = vec2(0.6, 0.8);

float Cover(vec2 t, float aa) {
    vec2 f = t - floor(t);
    vec2 d = abs(f - 0.5) * 2.0;
    vec2 c = 1.0 - smoothstep(coverage - aa, coverage + aa, d);
    return c.x * c.y;
}

float Luma(vec3 c) { return dot(c, vec3(0.299, 0.587, 0.114)); }

vec4 filter_main(vec2 coord) {
    vec2 t = coord / outputSize * inputSize;
    float perTexel = outputSize.x / inputSize.x;
    float grid = smoothstep(1.5, 3.0, perTexel);
    float aa = 2.0 / perTexel;
    vec3 lit = source_eval(floor(t) + 0.5).rgb;
    float cover = mix(1.0, Cover(t, aa), grid);
    vec3 color = mix(background, lit, cover);
    vec2 s = t - shadowOffset;
    vec3 caster = source_eval(floor(s) + 0.5).rgb;
    float darkness = clamp(1.0 - Luma(caster) / Luma(background), 0.0, 1.0);
    float shadow = darkness * Cover(s, aa) * shadowOpacity * grid;
    return vec4(color * (1.0 - shadow), 1.0);
}
".Replace("BG", Rgb(background)).Replace("COVER", F(coverage));

        // A colour panel: blended toward the last frame by the panel's response, then Pokefan531's measured matrix in linear light.
        public static string ColourPanelGlsl(double[] matrix, double luminance) => @"
uniform sampler2D original;
uniform sampler2D history1;

uniform float response;
const float lum = LUM;

vec4 filter_main(vec2 coord) {
    vec3 c = mix(original_eval(coord).rgb, history1_eval(coord).rgb, response);
    vec3 l = pow(c, vec3(2.2, 2.2, 2.2));
    vec3 o = vec3(M00 * l.r + M01 * l.g + M02 * l.b,
                      M10 * l.r + M11 * l.g + M12 * l.b,
                      M20 * l.r + M21 * l.g + M22 * l.b) * lum;
    return vec4(pow(clamp(o, 0.0, 1.0), vec3(1.0 / 2.2, 1.0 / 2.2, 1.0 / 2.2)), 1.0);
}
".Replace("LUM", F(luminance))
 .Replace("M00", F(matrix[0])).Replace("M01", F(matrix[1])).Replace("M02", F(matrix[2]))
 .Replace("M10", F(matrix[3])).Replace("M11", F(matrix[4])).Replace("M12", F(matrix[5]))
 .Replace("M20", F(matrix[6])).Replace("M21", F(matrix[7])).Replace("M22", F(matrix[8]));

        // Vertical red, green and blue stripes inside each pixel, with a dimmer row at its foot; gap is the share given to a dark fourth column.
        public static string StripeGridGlsl(double floor, double gain, double gap, double rowDim) => @"
uniform sampler2D source;
uniform vec2 inputSize;
uniform vec2 outputSize;

const float lowest = FLOOR;
const float gain = GAIN;
const float gap = GAP;
const float rowDim = ROWDIM;

float Box(float x, float a, float b, float aa) { return smoothstep(a - aa, a + aa, x) - smoothstep(b - aa, b + aa, x); }

vec4 filter_main(vec2 coord) {
    vec2 t = coord / outputSize * inputSize;
    float perTexel = outputSize.x / inputSize.x;
    float grid = smoothstep(2.0, 4.0, perTexel);
    float aa = 1.0 / perTexel;
    vec2 f = t - floor(t);
    float w = (1.0 - gap) / 3.0;
    vec3 stripe = vec3(Box(f.x, 0.0, w, aa), Box(f.x, w, 2.0 * w, aa), Box(f.x, 2.0 * w, 3.0 * w, aa));
    vec3 mask = lowest + (1.0 - lowest) * stripe;
    mask *= mix(1.0, rowDim, smoothstep(0.88 - aa, 0.88 + aa, f.y));
    vec3 lit = source_eval(floor(t) + 0.5).rgb;
    return vec4(clamp(lit * mix(vec3(1.0, 1.0, 1.0), mask * gain, grid), 0.0, 1.0), 1.0);
}
".Replace("FLOOR", F(floor)).Replace("GAIN", F(gain)).Replace("GAP", F(gap)).Replace("ROWDIM", F(rowDim));

        // SameBoy's measured palettes: the four shades and the colour of a switched-off panel - see EmuSen_Serenity.md §3.5.
        public const int DmgLightest = 0xC6DE8C, DmgLight = 0x84A563, DmgDark = 0x396139, DmgDarkest = 0x081810, DmgOff = 0xD2E6A6;
        public const int PocketLightest = 0xC2CE93, PocketLight = 0x818D66, PocketDark = 0x3A4C3A, PocketDarkest = 0x07100E, PocketOff = 0xCFDAAC;
        public const int LightLightest = 0x7FE2C3, LightLight = 0x56B495, LightDark = 0x357862, LightDarkest = 0x0A1C15, LightOff = 0x91EAD0;

        // Pokefan531's sRGB matrices, rows of output from linear input, and the luminance each is scaled by.
        public static readonly double[] GbcGbaMatrix = { 0.905, 0.195, -0.10, 0.10, 0.65, 0.25, 0.1575, 0.1425, 0.70 };
        public const double GbcGbaLuminance = 0.91;
        public static readonly double[] Sp101Matrix = { 0.96, 0.11, -0.07, 0.0325, 0.89, 0.0775, 0.001, -0.03, 1.029 };
        public const double Sp101Luminance = 0.935;

        private static readonly string[] GameBoy = { "GB" };
        private static readonly string[] GameBoyAdvance = { "GBA" };

        // The two numbers a player may change on a panel: how far it lags, and how dark a dot's shadow is - see EmuSen_Serenity.md §3.7.
        public static SlangParameter Response(double initial) => new("response", "LCD response (ghosting)", (float)initial, 0f, 0.9f, 0.01f);

        public static SlangParameter Shadow(double initial) => new("shadowOpacity", "Dot shadow", (float)initial, 0f, 1f, 0.05f);

        private static ScreenFilter Monochrome(string name, int p0, int p1, int p2, int p3, int off, double response, double shadow) => new(name,
            new[]
            {
                new FilterPass(MonochromePanelGlsl(p0, p1, p2, p3), PassScale.Source, History: 3),
                new FilterPass(DotMatrixGlsl(off, 0.86), PassScale.Viewport),
            },
            GameBoy, "palettes measured by SameBoy (LIJI32, MIT); response after Harlequin's gameboy shader; shadow after SameBoy's MonoLCD",
            new[] { Response(response), Shadow(shadow) });

        public static ScreenFilter DmgLcd { get; } = Monochrome("Game Boy LCD", DmgLightest, DmgLight, DmgDark, DmgDarkest, DmgOff, 0.33, 0.25);
        public static ScreenFilter PocketLcd { get; } = Monochrome("Game Boy Pocket LCD", PocketLightest, PocketLight, PocketDark, PocketDarkest, PocketOff, 0.12, 0.2);
        public static ScreenFilter LightLcd { get; } = Monochrome("Game Boy Light LCD", LightLightest, LightLight, LightDark, LightDarkest, LightOff, 0.12, 0.1);

        public static ScreenFilter GbcLcd { get; } = new("Game Boy Color LCD",
            new[]
            {
                new FilterPass(ColourPanelGlsl(GbcGbaMatrix, GbcGbaLuminance), PassScale.Source, History: 1),
                new FilterPass(StripeGridGlsl(0.5, 1.25, 0.0, 0.85), PassScale.Viewport),
            },
            GameBoy, "colour matrix measured by Pokefan531 (public domain); stripes after fishku's authentic_gbc (CC0)",
            new[] { Response(0.33) });

        public static ScreenFilter AgbLcd { get; } = new("Game Boy Advance LCD",
            new[]
            {
                new FilterPass(ColourPanelGlsl(GbcGbaMatrix, GbcGbaLuminance), PassScale.Source, History: 1),
                new FilterPass(StripeGridGlsl(0.45, 1.2, 0.25, 0.85), PassScale.Viewport),
            },
            GameBoyAdvance, "colour matrix measured by Pokefan531 (public domain); stripes and gap after mGBA's agb001 layout",
            new[] { Response(0.25) });

        public static ScreenFilter Ags101Lcd { get; } = new("Game Boy Advance SP (AGS-101) LCD",
            new[]
            {
                new FilterPass(ColourPanelGlsl(Sp101Matrix, Sp101Luminance), PassScale.Source, History: 1),
                new FilterPass(StripeGridGlsl(0.6, 1.15, 0.0, 0.9), PassScale.Viewport),
            },
            GameBoyAdvance, "colour matrix measured by Pokefan531 (public domain)",
            new[] { Response(0.0) });
    }
}

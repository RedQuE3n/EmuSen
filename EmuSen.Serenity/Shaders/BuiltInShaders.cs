namespace EmuSen.Serenity.Shaders
{
    // The simple effects' passes and the plain picture's, in Vulkan GLSL - see EmuSen_Serenity.md §3 and EmuSen_DevicePicture.md §20.
    internal static class BuiltInShaders
    {
        public const string ScanlinesGlsl = @"
uniform sampler2D image;
uniform vec2 outputSize;

vec4 filter_main(vec2 coord) {
    vec4 texColor = image_eval(coord);

    // Darkens every other output-resolution row, so it stays crisp at any window size.
    float row = floor(coord.y);
    float scanline = mod(row, 2.0) < 1.0 ? 1.0 : 0.78;

    return vec4(texColor.rgb * scanline, texColor.a);
}
";

        public const string CrtGlsl = @"
uniform sampler2D image;
uniform vec2 outputSize;

vec4 filter_main(vec2 coord) {
    vec4 texColor = image_eval(coord);

    float row = floor(coord.y);
    float scanline = mod(row, 2.0) < 1.0 ? 1.0 : 0.72;

    // Soft vignette - darkens toward the corners like a curved CRT tube.
    vec2 centered = (coord / outputSize) - 0.5;
    float vignette = 1.0 - dot(centered, centered) * 0.55;

    return vec4(texColor.rgb * scanline * vignette, texColor.a);
}
";

        // The picture drawn into its rectangle with nothing over it, as Skia's DrawImage draws it.
        public const string PlainGlsl = @"
uniform sampler2D image;

vec4 filter_main(vec2 coord) {
    return image_eval(coord);
}
";

        // A simple effect, or the plain picture, as a one-pass filter whose image is the frame stretched to the rectangle, sampled as the setting says - see EmuSen_DevicePicture.md §3.2.
        public static ScreenFilter AsFilter(ShaderEffect effect, bool bilinear) =>
            (bilinear ? Linear : Nearest)[(int)effect];

        private static ScreenFilter Make(ShaderEffect effect, bool bilinear) => new(effect.ToString(),
            new[] { new FilterPass(effect switch { ShaderEffect.Scanlines => ScanlinesGlsl, ShaderEffect.Crt => CrtGlsl, _ => PlainGlsl }, PassScale.Viewport, LinearSource: bilinear) },
            null, "EmuSen's own") { RowsOnce = true };

        private static readonly ScreenFilter[] Nearest = { Make(ShaderEffect.None, false), Make(ShaderEffect.Scanlines, false), Make(ShaderEffect.Crt, false) };
        private static readonly ScreenFilter[] Linear = { Make(ShaderEffect.None, true), Make(ShaderEffect.Scanlines, true), Make(ShaderEffect.Crt, true) };
    }
}

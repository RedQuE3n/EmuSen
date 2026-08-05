namespace EmuSen.Serenity.Shaders
{
    // Embedded SkSL for the built-in shader passes - see EmuSen_Serenity.md §3.
    internal static class BuiltInShaders
    {
        public const string ScanlinesSksl = @"
uniform shader image;
uniform float2 outputSize;

half4 main(float2 coord) {
    half4 texColor = image.eval(coord);

    // Darkens every other output-resolution row, so it stays crisp at any window size.
    float row = floor(coord.y);
    float scanline = mod(row, 2.0) < 1.0 ? 1.0 : 0.78;

    return half4(texColor.rgb * scanline, texColor.a);
}
";

        public const string CrtSksl = @"
uniform shader image;
uniform float2 outputSize;

half4 main(float2 coord) {
    half4 texColor = image.eval(coord);

    float row = floor(coord.y);
    float scanline = mod(row, 2.0) < 1.0 ? 1.0 : 0.72;

    // Soft vignette - darkens toward the corners like a curved CRT tube.
    float2 centered = (coord / outputSize) - 0.5;
    float vignette = 1.0 - dot(centered, centered) * 0.55;

    return half4(texColor.rgb * scanline * vignette, texColor.a);
}
";
    }
}

namespace EmuSen.Serenity.Shaders
{
    // Embedded Skia SkSL source for the built-in shader passes - a faithful
    // port of the original Raylib/GLSL versions (see git history), not a
    // redesign, done as part of moving EmuSen.Hotaru off Raylib onto
    // Avalonia/Serenity's own Skia-backed GameFrameControl. Embedded as
    // strings for the same reason the GLSL predecessor was: no "did the
    // build actually copy this file" failure mode while the pipeline
    // itself is still being proven out.
    //
    // SkSL's `shader` uniform type is how one shader samples another
    // (here: the frame image) rather than GLSL's `sampler2D`/`texture()`
    // pair - `image.eval(coord)` is the direct equivalent of
    // `texture(texture0, fragTexCoord)`. `coord` is already in the
    // destination's local pixel space (not a normalized 0-1 UV the way
    // GLSL's fragTexCoord was), so the scanline row check uses it
    // directly; the vignette's centering still divides by outputSize
    // first to recover the same 0-1 UV space the original GLSL vignette
    // math was written against.
    internal static class BuiltInShaders
    {
        public const string ScanlinesSksl = @"
uniform shader image;
uniform float2 outputSize;

half4 main(float2 coord) {
    half4 texColor = image.eval(coord);

    // Darkens every other output-resolution row - the classic simple
    // scanline look. Runs at outputSize (the upscaled draw target), not
    // the console's native resolution, so it stays crisp regardless of
    // window size.
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

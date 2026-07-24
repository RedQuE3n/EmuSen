namespace EmuSen.Presentation.Shaders
{
    // Embedded GLSL source for the first working shader passes - prototype
    // for RetroArch-style post-processing (see the shader game-plan
    // discussion). Embedded as strings rather than loose .glsl files
    // specifically to avoid "did the build actually copy this to the
    // output directory" as a whole extra failure mode while the pipeline
    // itself is still being proven out; that can change once there's a
    // real preset/pass-chain format to load from disk anyway.
    //
    // Every fragment shader here follows Raylib's default texture-shader
    // contract: sampler2D texture0 + vec4 colDiffuse are what
    // BeginShaderMode()/DrawTexturePro() feed in automatically, so no
    // custom vertex shader is needed - FramePresenter loads these with a
    // null vsCode, which makes Raylib fall back to its own default.
    internal static class BuiltInShaders
    {
        public const string ScanlinesFs = @"
#version 330

in vec2 fragTexCoord;
in vec4 fragColor;
out vec4 finalColor;

uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec2 outputSize;

void main()
{
    vec4 texColor = texture(texture0, fragTexCoord);

    // Darkens every other output-resolution row - the classic simple
    // scanline look. Runs at outputSize (the upscaled draw target), not
    // the console's native resolution, so it stays crisp regardless of
    // window size.
    float row = floor(fragTexCoord.y * outputSize.y);
    float scanline = mod(row, 2.0) < 1.0 ? 1.0 : 0.78;

    finalColor = vec4(texColor.rgb * scanline, texColor.a) * colDiffuse;
}
";

        public const string CrtFs = @"
#version 330

in vec2 fragTexCoord;
in vec4 fragColor;
out vec4 finalColor;

uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec2 outputSize;

void main()
{
    vec4 texColor = texture(texture0, fragTexCoord);

    float row = floor(fragTexCoord.y * outputSize.y);
    float scanline = mod(row, 2.0) < 1.0 ? 1.0 : 0.72;

    // Soft vignette - darkens toward the corners like a curved CRT tube.
    vec2 centered = fragTexCoord - 0.5;
    float vignette = 1.0 - dot(centered, centered) * 0.55;

    finalColor = vec4(texColor.rgb * scanline * vignette, texColor.a) * colDiffuse;
}
";
    }
}

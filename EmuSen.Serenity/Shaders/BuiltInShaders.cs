namespace EmuSen.Serenity.Shaders
{
    // The simple effects and the plain picture as one-pass filters, as the platform library's catalog gives them - see EmuSen_Serenity.md §3 and EmuSen_DevicePicture.md §20.
    internal static class BuiltInShaders
    {
        public static string ScanlinesGlsl => AsFilter(ShaderEffect.Scanlines, false).Passes[0].Glsl;

        public static string CrtGlsl => AsFilter(ShaderEffect.Crt, false).Passes[0].Glsl;

        // A simple effect, or the plain picture, whose image is the frame stretched to the rectangle, sampled as the setting says - see EmuSen_DevicePicture.md §3.2.
        public static ScreenFilter AsFilter(ShaderEffect effect, bool bilinear) =>
            FilterCatalog.Keyed($"effect.{effect}.{(bilinear ? "linear" : "nearest")}");
    }
}

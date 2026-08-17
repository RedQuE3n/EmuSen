using SkiaSharp;

namespace EmuSen.Common.Imaging
{
    // The one RGBA8888-to-PNG encoder, shared by the screenshot hotkey and `record` - see EmuSen_Debugging_Tools_Reference_v5.md §3.15b.
    public static class PngFile
    {
        public static void Write(string path, byte[] rgba, int width, int height)
        {
            var info = new SKImageInfo(width, height, SKColorType.Rgba8888, SKAlphaType.Unpremul);
            using SKImage image = SKImage.FromPixelCopy(info, rgba);
            using SKData data = image.Encode(SKEncodedImageFormat.Png, 100);
            using FileStream stream = File.Create(path);
            data.SaveTo(stream);
        }
    }
}

using System.IO;
using SkiaSharp;

namespace EmuSen.Serenity.Imaging
{
    // An RGBA8888 picture copied from the API to a PNG, for a frontend's screenshot and state picture; SkiaSharp directly, testable with no display - see EmuSen_Serenity.md §2.11.
    public static class FrameImageWriter
    {
        public static void SavePng(byte[] rgba, int width, int height, string path)
        {
            var info = new SKImageInfo(width, height, SKColorType.Rgba8888, SKAlphaType.Unpremul);
            using SKImage image = SKImage.FromPixelCopy(info, rgba);
            using SKData data = image.Encode(SKEncodedImageFormat.Png, 100);
            using FileStream stream = File.Create(path);
            data.SaveTo(stream);
        }
    }
}

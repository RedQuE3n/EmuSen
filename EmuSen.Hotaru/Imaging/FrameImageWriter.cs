using System.IO;
using SkiaSharp;

namespace EmuSen.Hotaru.Imaging
{
    // An RGBA8888 picture copied from the API to a PNG, for the F3 screenshot; SkiaSharp directly, testable with no display - see EmuSen_Backend.md §14.11.
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

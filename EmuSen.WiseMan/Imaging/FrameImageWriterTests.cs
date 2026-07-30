using EmuSen.Hotaru.Imaging;
using SkiaSharp;

namespace EmuSen.WiseMan.Imaging
{
    // FrameImageWriter.SavePng - a synthetic-RGBA-buffer round trip: write
    // a small frame to a temp PNG, decode it back via SkiaSharp, and
    // confirm the pixels and dimensions match. Runs with no display and no
    // Avalonia App ever started, since FrameImageWriter deliberately uses
    // SkiaSharp directly rather than Avalonia's WriteableBitmap - see that
    // file's own comment for why this exact test is possible at all.
    public class FrameImageWriterTests
    {
        private static string TempPngPath() => Path.Combine(Path.GetTempPath(), $"wiseman_png_{Guid.NewGuid():N}.png");

        [Fact]
        public void Round_trip_preserves_pixels_and_dimensions()
        {
            const int width = 4, height = 3;
            byte[] rgba = new byte[width * height * 4];
            var rng = new Random(7);
            rng.NextBytes(rgba);
            for (int i = 3; i < rgba.Length; i += 4) rgba[i] = 255; // fully opaque, matching real frame buffers

            string path = TempPngPath();
            try
            {
                FrameImageWriter.SavePng(rgba, width, height, path);

                using SKBitmap decoded = SKBitmap.Decode(path);
                Assert.Equal(width, decoded.Width);
                Assert.Equal(height, decoded.Height);

                for (int y = 0; y < height; y++)
                {
                    for (int x = 0; x < width; x++)
                    {
                        int i = (y * width + x) * 4;
                        SKColor pixel = decoded.GetPixel(x, y);
                        Assert.Equal(rgba[i], pixel.Red);
                        Assert.Equal(rgba[i + 1], pixel.Green);
                        Assert.Equal(rgba[i + 2], pixel.Blue);
                        Assert.Equal(rgba[i + 3], pixel.Alpha);
                    }
                }
            }
            finally
            {
                File.Delete(path);
            }
        }
    }
}

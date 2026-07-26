using EmuSen.Common.Imaging;

namespace EmuSen.WiseMan.Imaging
{
    // BmpFile.Write/Read - a write/read round trip should reproduce the
    // exact pixels and dimensions given to it, since Read only ever needs
    // to load this toolkit's own screenshots back (see BmpFile.cs's own
    // comment on why it doesn't handle arbitrary externally-authored BMPs).
    public class BmpFileTests
    {
        private static string TempBmpPath() => Path.Combine(Path.GetTempPath(), $"wiseman_bmp_{Guid.NewGuid():N}.bmp");

        [Fact]
        public void Round_trip_preserves_pixels_and_dimensions()
        {
            const int width = 4, height = 3;
            byte[] rgba = new byte[width * height * 4];
            var rng = new Random(42);
            rng.NextBytes(rgba);
            // Alpha fully opaque, matching what real frame buffers use -
            // not load-bearing for the round trip, just realistic.
            for (int i = 3; i < rgba.Length; i += 4) rgba[i] = 255;

            string path = TempBmpPath();
            try
            {
                BmpFile.Write(path, rgba, width, height);
                var (readRgba, readWidth, readHeight) = BmpFile.Read(path);

                Assert.Equal(width, readWidth);
                Assert.Equal(height, readHeight);
                Assert.Equal(rgba, readRgba);
            }
            finally
            {
                File.Delete(path);
            }
        }

        [Fact]
        public void Single_pixel_image_round_trips()
        {
            byte[] rgba = { 10, 20, 30, 255 };
            string path = TempBmpPath();
            try
            {
                BmpFile.Write(path, rgba, 1, 1);
                var (readRgba, readWidth, readHeight) = BmpFile.Read(path);

                Assert.Equal(1, readWidth);
                Assert.Equal(1, readHeight);
                Assert.Equal(rgba, readRgba);
            }
            finally
            {
                File.Delete(path);
            }
        }
    }
}

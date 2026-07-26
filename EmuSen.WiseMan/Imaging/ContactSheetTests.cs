using EmuSen.Common.Imaging;

namespace EmuSen.WiseMan.Imaging
{
    // ContactSheet's grid/tiling dimension math - the actual pixel content
    // is exercised transitively via BmpFileTests; what's worth locking
    // down here is that Downsample/WriteContactSheet compute the sheet
    // size correctly for a few count/cols combinations, since a wrong
    // count here would corrupt or truncate the tiled image silently.
    public class ContactSheetTests
    {
        [Fact]
        public void Downsample_divides_dimensions_by_scale()
        {
            byte[] src = new byte[16 * 16 * 4];
            byte[] dst = ContactSheet.Downsample(src, 16, 16, scale: 4, out int dstWidth, out int dstHeight);

            Assert.Equal(4, dstWidth);
            Assert.Equal(4, dstHeight);
            Assert.Equal(dstWidth * dstHeight * 4, dst.Length);
        }

        [Fact]
        public void Downsample_never_produces_a_zero_sized_dimension()
        {
            byte[] src = new byte[2 * 2 * 4];
            byte[] dst = ContactSheet.Downsample(src, 2, 2, scale: 100, out int dstWidth, out int dstHeight);

            Assert.Equal(1, dstWidth);
            Assert.Equal(1, dstHeight);
            Assert.Equal(4, dst.Length);
        }

        [Theory]
        [InlineData(8, 4, 2)]  // exact fit: 8 thumbs, 4 cols -> 2 rows
        [InlineData(9, 4, 3)]  // 9 thumbs, 4 cols -> needs a 3rd, mostly-empty row
        [InlineData(3, 8, 1)]  // fewer thumbs than one row's worth
        public void WriteContactSheet_sizes_the_sheet_for_a_full_grid(int count, int cols, int expectedRows)
        {
            const int thumbW = 4, thumbH = 3;
            var thumbs = new List<byte[]>();
            for (int i = 0; i < count; i++) thumbs.Add(new byte[thumbW * thumbH * 4]);

            string path = Path.Combine(Path.GetTempPath(), $"wiseman_sheet_{Guid.NewGuid():N}.bmp");
            try
            {
                ContactSheet.WriteContactSheet(path, thumbs, thumbW, thumbH, cols);
                var (_, width, height) = BmpFile.Read(path);

                Assert.Equal(cols * thumbW, width);
                Assert.Equal(expectedRows * thumbH, height);
            }
            finally
            {
                File.Delete(path);
            }
        }
    }
}

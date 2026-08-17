namespace EmuSen.Common.Imaging
{
    // Tiles frames into one grid, for "is this actually moving" across a span without N screenshots.
    public static class ContactSheet
    {
        // Nearest-neighbor downsample - a debugging contact sheet needs "can I tell this changed.
        public static byte[] Downsample(byte[] src, int srcWidth, int srcHeight, int scale, out int dstWidth, out int dstHeight)
        {
            dstWidth = Math.Max(1, srcWidth / scale);
            dstHeight = Math.Max(1, srcHeight / scale);
            byte[] dst = new byte[dstWidth * dstHeight * 4];

            for (int y = 0; y < dstHeight; y++)
            {
                int srcY = Math.Min(srcHeight - 1, y * scale);
                for (int x = 0; x < dstWidth; x++)
                {
                    int srcX = Math.Min(srcWidth - 1, x * scale);
                    int srcIdx = (srcY * srcWidth + srcX) * 4;
                    int dstIdx = (y * dstWidth + x) * 4;
                    Array.Copy(src, srcIdx, dst, dstIdx, 4);
                }
            }
            return dst;
        }

        // Tiles a list of equally-sized RGBA thumbnails into one grid image, <cols> per row - empty trailing.
        public static void WriteContactSheet(string path, List<byte[]> thumbs, int thumbWidth, int thumbHeight, int cols)
        {
            int count = thumbs.Count;
            int rows = (count + cols - 1) / cols;
            int sheetWidth = cols * thumbWidth;
            int sheetHeight = rows * thumbHeight;
            byte[] sheet = new byte[sheetWidth * sheetHeight * 4];

            for (int i = 0; i < count; i++)
            {
                int originX = (i % cols) * thumbWidth;
                int originY = (i / cols) * thumbHeight;
                byte[] thumb = thumbs[i];
                for (int y = 0; y < thumbHeight; y++)
                {
                    int srcRowStart = y * thumbWidth * 4;
                    int dstRowStart = ((originY + y) * sheetWidth + originX) * 4;
                    Array.Copy(thumb, srcRowStart, sheet, dstRowStart, thumbWidth * 4);
                }
            }

            BmpFile.Write(path, sheet, sheetWidth, sheetHeight);
        }
    }
}

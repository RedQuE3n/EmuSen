using System;

namespace EmuSen.Common
{
    // One rewind moment's picture, downscaled and held as RGB565 - see EmuSen_Rewind_And_FastForward.md §5.3.
    public sealed class RewindThumbnail
    {
        private readonly byte[] _pixels;

        private RewindThumbnail(int width, int height, byte[] pixels)
        {
            Width = width;
            Height = height;
            _pixels = pixels;
        }

        public int Width { get; }
        public int Height { get; }

        // What it holds in memory, two bytes a pixel.
        public int Bytes => _pixels.Length;

        // A frame of `rows` rows each shown `rowRepeat` times, sampled four times a pixel whatever its size - see §5.3.
        public static RewindThumbnail? From(ReadOnlySpan<byte> rgba, int width, int rows, int rowRepeat, int targetWidth)
        {
            if (width <= 0 || rows <= 0 || targetWidth <= 0 || rgba.Length < width * rows * 4) return null;

            int repeat = Math.Max(1, rowRepeat);
            int shownHeight = rows * repeat;
            int tw = Math.Min(targetWidth, width);
            int th = Math.Max(1, (int)Math.Round((double)tw * shownHeight / width));
            var pixels = new byte[tw * th * 2];

            int o = 0;
            for (int y = 0; y < th; y++)
            {
                long y0 = (long)y * shownHeight / th, y1 = Math.Max(y0 + 1, (long)(y + 1) * shownHeight / th);
                int rowA = (int)((y0 + (y1 - y0) / 4) / repeat), rowB = (int)((y0 + 3 * (y1 - y0) / 4) / repeat);
                for (int x = 0; x < tw; x++)
                {
                    long x0 = (long)x * width / tw, x1 = Math.Max(x0 + 1, (long)(x + 1) * width / tw);
                    int colA = (int)(x0 + (x1 - x0) / 4), colB = (int)(x0 + 3 * (x1 - x0) / 4);
                    int a = (rowA * width + colA) * 4, b = (rowA * width + colB) * 4, c = (rowB * width + colA) * 4, d = (rowB * width + colB) * 4;
                    int r = (rgba[a] + rgba[b] + rgba[c] + rgba[d] + 2) >> 2;
                    int g = (rgba[a + 1] + rgba[b + 1] + rgba[c + 1] + rgba[d + 1] + 2) >> 2;
                    int bl = (rgba[a + 2] + rgba[b + 2] + rgba[c + 2] + rgba[d + 2] + 2) >> 2;
                    ushort packed = (ushort)(((r >> 3) << 11) | ((g >> 2) << 5) | (bl >> 3));
                    pixels[o++] = (byte)packed;
                    pixels[o++] = (byte)(packed >> 8);
                }
            }

            return new RewindThumbnail(tw, th, pixels);
        }

        // Opaque RGBA8888, row by row, for a surface that shows it.
        public byte[] ToRgba()
        {
            var rgba = new byte[Width * Height * 4];
            for (int i = 0, o = 0; i < _pixels.Length; i += 2, o += 4)
            {
                int v = _pixels[i] | (_pixels[i + 1] << 8);
                int r = (v >> 11) & 31, g = (v >> 5) & 63, b = v & 31;
                rgba[o] = (byte)((r << 3) | (r >> 2));
                rgba[o + 1] = (byte)((g << 2) | (g >> 4));
                rgba[o + 2] = (byte)((b << 3) | (b >> 2));
                rgba[o + 3] = 255;
            }
            return rgba;
        }
    }
}

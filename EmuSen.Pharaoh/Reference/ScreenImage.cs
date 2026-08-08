using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Pharaoh.Reference
{
    // A framebuffer normalised to RGB triples, because two emulators describe the
    // same picture in different formats and raw bytes never compare - see §3.48.
    public sealed class ScreenImage
    {
        public int Width { get; }
        public int Height { get; }

        // Three bytes per pixel, row-major.
        public byte[] Rgb { get; }

        private ScreenImage(int width, int height, byte[] rgb)
        {
            Width = width;
            Height = height;
            Rgb = rgb;
        }

        public static ScreenImage? Read(byte[] raw, string format)
        {
            return format switch
            {
                // The reference's NES buffer: one 16-bit palette index per pixel, of
                // which the low six bits are the colour and the rest is emphasis.
                "PaletteIndex16" => FromPaletteIndex16(raw, 256, 240),
                "Rgba8888" => FromRgba(raw, 256, 240),
                "Xrgb8888" => FromXrgb(raw, 256, 240),
                _ => null,
            };
        }

        // The core's own table, not a copy of it: a second palette here would be one
        // more constant that has to be kept in step with the thing it describes.
        private static ReadOnlySpan<byte> Palette => EmuSen.Cores.Nintendo.Moon.Video.Ppu.NesPalette;

        private static ScreenImage? FromPaletteIndex16(byte[] raw, int width, int height)
        {
            if (raw.Length < width * height * 2) return null;

            var rgb = new byte[width * height * 3];
            for (int i = 0; i < width * height; i++)
            {
                int index = (raw[i * 2] | (raw[(i * 2) + 1] << 8)) & 0x3F;
                rgb[i * 3] = Palette[index * 3];
                rgb[(i * 3) + 1] = Palette[(index * 3) + 1];
                rgb[(i * 3) + 2] = Palette[(index * 3) + 2];
            }
            return new ScreenImage(width, height, rgb);
        }

        private static ScreenImage? FromRgba(byte[] raw, int width, int height)
        {
            if (raw.Length < width * height * 4) return null;

            var rgb = new byte[width * height * 3];
            for (int i = 0; i < width * height; i++)
            {
                rgb[i * 3] = raw[i * 4];
                rgb[(i * 3) + 1] = raw[(i * 4) + 1];
                rgb[(i * 3) + 2] = raw[(i * 4) + 2];
            }
            return new ScreenImage(width, height, rgb);
        }

        private static ScreenImage? FromXrgb(byte[] raw, int width, int height)
        {
            if (raw.Length < width * height * 4) return null;

            var rgb = new byte[width * height * 3];
            for (int i = 0; i < width * height; i++)
            {
                rgb[i * 3] = raw[(i * 4) + 2];
                rgb[(i * 3) + 1] = raw[(i * 4) + 1];
                rgb[(i * 3) + 2] = raw[i * 4];
            }
            return new ScreenImage(width, height, rgb);
        }

        public int DifferingPixels(ScreenImage other)
        {
            if (Width != other.Width || Height != other.Height) return Width * Height;

            int differing = 0;
            for (int i = 0; i < Width * Height; i++)
            {
                if (Rgb[i * 3] != other.Rgb[i * 3] ||
                    Rgb[(i * 3) + 1] != other.Rgb[(i * 3) + 1] ||
                    Rgb[(i * 3) + 2] != other.Rgb[(i * 3) + 2])
                {
                    differing++;
                }
            }
            return differing;
        }

        // How much a frame can possibly prove: a near-uniform screen agrees with
        // anything, and reporting that as a pass is the false positive §3.48 exists
        // to stop.
        public (int Colours, double DominantFraction) Information()
        {
            var counts = new Dictionary<int, int>();
            for (int i = 0; i < Width * Height; i++)
            {
                int key = (Rgb[i * 3] << 16) | (Rgb[(i * 3) + 1] << 8) | Rgb[(i * 3) + 2];
                counts[key] = counts.TryGetValue(key, out int n) ? n + 1 : 1;
            }
            return (counts.Count, counts.Values.Max() / (double)(Width * Height));
        }

        // Whole-image vertical offset that explains the difference, if one does.
        public int? VerticalShift(ScreenImage other, int limit = 16)
        {
            for (int dy = -limit; dy <= limit; dy++)
            {
                if (dy == 0) continue;
                int matched = 0;
                for (int y = 0; y < Height; y++)
                {
                    int source = y + dy;
                    if (source < 0 || source >= Height) continue;
                    if (RowEquals(source, other, y)) matched++;
                }
                if (matched >= Height - 6) return dy;
            }
            return null;
        }

        private bool RowEquals(int myRow, ScreenImage other, int theirRow)
        {
            int a = myRow * Width * 3;
            int b = theirRow * other.Width * 3;
            for (int i = 0; i < Width * 3; i++)
            {
                if (Rgb[a + i] != other.Rgb[b + i]) return false;
            }
            return true;
        }
    }
}

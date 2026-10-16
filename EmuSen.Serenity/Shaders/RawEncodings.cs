using System;

namespace EmuSen.Serenity.Shaders
{
    // The encodings of a picture before the palette that the built-in filters read, each turned into the image their passes sample - see EmuSen_CRT.md §22.1.
    public static class RawEncodings
    {
        // The 2C02's colour number in bits 0-5 and PPUMASK's emphasis bits in 6-8 - see EmuSen_CoreAPI.md §6.30's register.
        public const string NesPpu = "nes.ppu";

        // The picture as an RGBA image of its own size in into, which holds at least Width x Height x 4 bytes; false for an encoding no filter reads, or a malformed picture.
        public static bool ToImage(in RawPicture raw, byte[] into)
        {
            if (raw.Encoding != NesPpu || raw.Element != 2 || raw.Width <= 0 || raw.Height <= 0 || raw.Stride < raw.Width * 2) return false;
            if (into.Length < raw.Width * raw.Height * 4 || raw.Values.Length < raw.Stride * (raw.Height - 1) + raw.Width * 2) return false;
            ReadOnlySpan<byte> values = raw.Values;
            for (int y = 0, o = 0; y < raw.Height; y++)
            {
                ReadOnlySpan<byte> row = values.Slice(y * raw.Stride, raw.Width * 2);
                for (int x = 0; x < raw.Width; x++, o += 4)
                {
                    int v = row[2 * x] | row[2 * x + 1] << 8;
                    (into[o], into[o + 1], into[o + 2], into[o + 3]) = ((byte)(v & 0x3F), (byte)(v >> 6 & 7), 0, 255);
                }
            }
            return true;
        }
    }
}

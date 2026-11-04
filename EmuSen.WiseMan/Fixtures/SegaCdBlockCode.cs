using System;
using System.Buffers.Binary;

namespace EmuSen.WiseMan.Fixtures
{
    // The Sega CD backup RAM's block code, kept to write and read test volumes by hand since the C# format module was deleted on 2026-11-04 - see EmuSen_MediumFormats.md, EmuSen_Backend.md §40.1.
    public static class SegaCdBlockCode
    {
        private static readonly int[][] Order = BuildOrder();
        private static readonly ushort[] T = { 0x000E, 0x003E, 0x00FE, 0x03FE, 0x0FFE, 0x3FFE };
        private const int Inverse3 = 0x3E;

        private static int[][] BuildOrder()
        {
            var order = new int[8][];
            for (int k = 0; k < 8; k++)
                order[k] = new[] { k, k + 9, k + 18, k + 27, k + 36, k < 3 ? 45 + k : 9 * k - 19, 48 + k, 56 + k };
            return order;
        }

        private static int Alpha(int x) => (x << 1 & 0x40) != 0 ? (x << 1) ^ 0x43 : x << 1;

        private static int Multiply(int x, int y)
        {
            int r = 0;
            for (int b = 5; b >= 0; b--)
            {
                r = Alpha(r);
                if ((y >> b & 1) != 0) r ^= x;
            }
            return r;
        }

        public static ushort Crc(ReadOnlySpan<byte> data)
        {
            int c = 0;
            foreach (byte b in data)
            {
                c ^= b << 8;
                for (int k = 0; k < 8; k++) c = (c & 0x8000) != 0 ? (c << 1 ^ 0x1021) & 0xFFFF : c << 1 & 0xFFFF;
            }
            return (ushort)c;
        }

        public static void Encode(ReadOnlySpan<byte> u, Span<byte> stored)
        {
            Span<byte> m = stackalloc byte[36];
            ushort crc = Crc(u[..32]);
            BinaryPrimitives.WriteUInt16BigEndian(m, crc);
            u[..32].CopyTo(m[2..]);
            BinaryPrimitives.WriteUInt16BigEndian(m[34..], (ushort)~crc);
            Span<int> s = stackalloc int[64];
            for (int g = 0; g < 12; g++)
            {
                int bits = m[3 * g] << 16 | m[3 * g + 1] << 8 | m[3 * g + 2];
                for (int k = 0; k < 4; k++) s[4 * g + k] = bits >> (18 - 6 * k) & 63;
            }
            foreach (int[] o in Order)
            {
                int s0 = 0, h = 0;
                for (int k = 0; k < 6; k++) { s0 ^= s[o[k]]; h = Alpha(h) ^ s[o[k]]; }
                int p0 = Multiply(s0 ^ Alpha(Alpha(h)), Inverse3);
                s[o[6]] = p0;
                s[o[7]] = s0 ^ p0;
            }
            for (int c = 0; c < 8; c++)
            {
                int r = 0;
                for (int row = 0; row < 8; row++)
                {
                    int top = r >> 14;
                    r = (r << 2 & 0xFFFF) ^ top << 8 ^ top << 6 ^ top << 4 ^ top;
                    for (int b = 0; b < 6; b++) if ((s[8 * row + c] >> b & 1) != 0) r ^= T[b];
                }
                for (int row = 0; row < 8; row++) stored[8 * row + c] = (byte)(s[8 * row + c] << 2 | r >> (14 - 2 * row) & 3);
            }
        }

        // The 32 data bytes into u, one symbol error a codeword corrected, the low bits and the CRC not used (§6.7): 0 whole, 1 corrected, -1 a codeword past correcting.
        public static int Decode(ReadOnlySpan<byte> stored, Span<byte> u)
        {
            Span<int> s = stackalloc int[64];
            for (int i = 0; i < 64; i++) s[i] = stored[i] >> 2;
            int result = 0;
            foreach (int[] o in Order)
            {
                int s0 = 0, s1 = 0;
                foreach (int i in o) { s0 ^= s[i]; s1 = Alpha(s1) ^ s[i]; }
                if ((s0 | s1) == 0) continue;
                int at = -1;
                for (int p = 0, t = s0; s0 != 0 && s1 != 0 && p < 8; p++, t = Alpha(t))
                    if (t == s1) { at = 7 - p; break; }
                if (at < 0) { result = -1; continue; }
                s[o[at]] ^= s0;
                if (result == 0) result = 1;
            }
            Span<byte> m = stackalloc byte[36];
            for (int g = 0; g < 12; g++)
            {
                int bits = s[4 * g] << 18 | s[4 * g + 1] << 12 | s[4 * g + 2] << 6 | s[4 * g + 3];
                m[3 * g] = (byte)(bits >> 16);
                m[3 * g + 1] = (byte)(bits >> 8);
                m[3 * g + 2] = (byte)bits;
            }
            m.Slice(2, 32).CopyTo(u);
            return result;
        }
    }
}

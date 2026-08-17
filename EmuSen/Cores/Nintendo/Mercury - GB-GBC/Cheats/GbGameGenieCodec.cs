using System;
using System.Linq;

namespace EmuSen.Cores.Nintendo.Mercury.Cheats
{
    // The Game Boy Game Genie: a ROM-read intercept, nine hex digits with a compare byte or six without - see Mercury_Cheats.md §2.
    public static class GbGameGenieCodec
    {
        public static bool CanDecode(string code) => TryNormalize(code, out _);

        public static (int Address, byte Value) Decode(string code)
        {
            string hex = Normalize(code);

            byte value = (byte)((Digit(hex[0]) << 4) | Digit(hex[1]));

            // The address is stored with its top nibble last and inverted, which is the whole cipher.
            int address = ((Digit(hex[5]) ^ 0x0F) << 12)
                        | (Digit(hex[2]) << 8)
                        | (Digit(hex[3]) << 4)
                        | Digit(hex[4]);

            return (address, value);
        }

        // Six-digit codes patch every bank; nine-digit ones name the byte they expect to replace.
        public static byte? DecodeCompare(string code)
        {
            string hex = Normalize(code);
            if (hex.Length != 9) return null;

            // The seventh digit is unused; the sixth and ninth carry the compare, rotated and masked.
            byte packed = (byte)((Digit(hex[6]) << 4) | Digit(hex[8]));
            byte rotated = (byte)((packed >> 2) | (packed << 6));

            return (byte)(rotated ^ 0xBA);
        }

        private static string Normalize(string code)
        {
            if (!TryNormalize(code, out string hex))
            {
                throw new FormatException(
                    $"'{code}' isn't a valid Game Boy Game Genie code - expected 6 or 9 hex digits (AAA-AAA-AAA).");
            }

            return hex;
        }

        private static bool TryNormalize(string code, out string hex)
        {
            hex = new string((code ?? "").Where(Uri.IsHexDigit).ToArray()).ToUpperInvariant();
            return hex.Length is 6 or 9;
        }

        private static int Digit(char c) => Convert.ToInt32(c.ToString(), 16);
    }
}

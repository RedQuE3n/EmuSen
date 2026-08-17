using System;
using System.Linq;

namespace EmuSen.Cores.Nintendo.Mercury.Cheats
{
    // The Game Boy GameShark: eight hex digits, and a RAM poke rather than a ROM patch - see Mercury_Cheats.md §3.
    public static class GbGameSharkCodec
    {
        public static bool CanDecode(string code) => TryNormalize(code, out _);

        public static (int Address, byte Value) Decode(string code)
        {
            if (!TryNormalize(code, out string hex))
            {
                throw new FormatException(
                    $"'{code}' isn't a valid Game Boy GameShark code - expected 8 hex digits (TTVVAAAA).");
            }

            byte value = Convert.ToByte(hex.Substring(2, 2), 16);

            // The address is little-endian in the code, so the halves come out swapped.
            int low = Convert.ToInt32(hex.Substring(4, 2), 16);
            int high = Convert.ToInt32(hex.Substring(6, 2), 16);

            return ((high << 8) | low, value);
        }

        // The first byte is the external RAM bank a code applies to; Mercury pokes the live bus instead.
        public static byte BankOf(string code) =>
            TryNormalize(code, out string hex) ? Convert.ToByte(hex.Substring(0, 2), 16) : (byte)0;

        private static bool TryNormalize(string code, out string hex)
        {
            hex = new string((code ?? "").Where(Uri.IsHexDigit).ToArray()).ToUpperInvariant();
            return hex.Length == 8;
        }
    }
}

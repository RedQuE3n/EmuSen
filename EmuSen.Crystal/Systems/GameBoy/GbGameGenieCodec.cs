using System;
using System.Linq;

namespace EmuSen.Crystal.Systems.GameBoy
{
    // The Game Boy Game Genie: a ROM-read intercept, nine hex digits with a compare byte or six without - see Mercury_Cheats.md §2.
    public static class GbGameGenieCodec
    {
        public static bool CanDecode(string code) => TryNormalize(code, out _);

        // The address is stored with its top nibble last and inverted, which is the whole cipher.
        public static (int Address, byte Value) Decode(string code) => Cheats.GenieCipher.Decode(Normalize(code));

        // Six-digit codes patch every bank; nine-digit ones name the byte they expect to replace.
        public static byte? DecodeCompare(string code) => Cheats.GenieCipher.Compare(Normalize(code));

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
    }
}

using System;
using System.Linq;

namespace EmuSen.Cores.Nintendo.Moon.Cheats
{
    // The plain "address:value" form NES cheats are shared in when they did not come off a Game Genie - see Moon_Cheats.md §2.
    public static class NesRawCodec
    {
        // Exactly 6 hex digits once separators are stripped.
        public static bool CanDecode(string code) => TryNormalize(code, out _);

        public static (int Address, byte Value) Decode(string code)
        {
            if (!TryNormalize(code, out string hex))
            {
                throw new FormatException(
                    $"'{code}' isn't a valid NES address:value code - expected 6 hex digits (4-digit address + 2-digit value).");
            }

            return (Convert.ToInt32(hex.Substring(0, 4), 16), Convert.ToByte(hex.Substring(4, 2), 16));
        }

        private static bool TryNormalize(string code, out string hex)
        {
            hex = new string((code ?? "").Where(Uri.IsHexDigit).ToArray());
            return hex.Length == 6;
        }
    }
}

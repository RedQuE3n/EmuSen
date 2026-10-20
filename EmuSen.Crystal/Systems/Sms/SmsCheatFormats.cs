using System;
using System.Linq;

namespace EmuSen.Crystal.Systems.Sms
{
    // The Master System's and the SG-1000's two code formats, from SMS Power!'s "Pro Action Replay" and "Game Genie" pages - see EmuSen_Cheats.md §11.
    public static class SmsCheatFormats
    {
        // The work RAM's 8 KiB, which a Pro Action Replay address from $C000 reaches through its mirrors.
        private const int RamMask = 0x1FFF;

        // Nothing but hex digits, dashes and spaces: a database's ?? for a value the player picks is no digit to drop.
        private static string? Digits(string? code, params int[] lengths)
        {
            if ((code ?? "").Any(c => !(Uri.IsHexDigit(c) || c == '-' || c == ' '))) return null;
            string hex = new string((code ?? "").Where(Uri.IsHexDigit).ToArray()).ToUpperInvariant();
            return lengths.Contains(hex.Length) ? hex : null;
        }

        // 00AA-AADD: the middle four digits an address from $C000; the first two, which the page says have no effect, are not read.
        private static (int Address, byte Value)? Par(string code) =>
            Digits(code, 8) is { } d && Convert.ToInt32(d.Substring(2, 4), 16) is var address && address >= 0xC000 ? (address, Convert.ToByte(d.Substring(6, 2), 16)) : null;

        public static bool IsProActionReplay(string code) => Par(code) is not null;

        public static (int Address, byte Value) DecodeProActionReplay(string code) =>
            Par(code) is var (address, value) ? (address & RamMask, value)
            : throw new FormatException($"'{code}' isn't a valid Pro Action Replay code - expected 8 hex digits (00AA-AADD) with an address from $C000.");

        public static bool IsGameGenie(string code) => Digits(code, 6, 9) is not null;

        private static string Genie(string code) =>
            Digits(code, 6, 9) ?? throw new FormatException($"'{code}' isn't a valid Game Genie code - expected 6 or 9 hex digits (DDA-AAA-RRR).");

        public static (int Address, byte Value) DecodeGameGenie(string code) => Cheats.GenieCipher.Decode(Genie(code));

        public static byte? GameGenieCompare(string code) => Cheats.GenieCipher.Compare(Genie(code));
    }
}

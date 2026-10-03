using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.DianaOS.DianaOS.Sys.Systems.Snes
{
    // The SNES's Pro Action Replay and Game Genie codes, written from fullsnes's "SNES Cart Cheat Devices - Code Formats" - see EmuSen_CoreAPI.md §20.
    public static class SnesCheatFormats
    {
        // Spaces, dashes and colons between digits are how codes are printed, not part of them.
        private static string Digits(string code) => new(code.Where(c => !char.IsWhiteSpace(c) && c != '-' && c != ':').ToArray());

        private static bool IsHex(string s) => s.Length > 0 && s.All(Uri.IsHexDigit);

        // ---- Pro Action Replay: AAAAAADD, a 24-bit address and a byte ---------------------------------------------

        // 7E000000 is the device's "do nothing" padding code.
        public const uint DoNothing = 0x7E000000;

        // Eight hex digits printed as one word; the PAR1 pre-boot codes (FE, FF) and the multi-byte and device-control prefixes are not cheats this engine can apply, so they are refused.
        public static bool IsProActionReplay(string code)
        {
            string d = code.Trim();
            if (d.Length != 8 || !IsHex(d)) return false;
            uint word = Convert.ToUInt32(d, 16);
            return word >> 24 is not (0xFE or 0xFF) && word != 0xDEADC0DE && word >> 16 != 0xC0DE;
        }

        public static (int Address, byte Value) DecodeProActionReplay(string code)
        {
            string d = Digits(code);
            if (d.Length != 8 || !IsHex(d)) throw new FormatException($"'{code}' is not eight hex digits (a Pro Action Replay code is AAAAAADD).");
            uint word = Convert.ToUInt32(d, 16);
            return ((int)(word >> 8), (byte)word);
        }

        // WRAM is written at 7E0000-7FFFFF, which is the only form the device accepts for it; any other address is the CPU bus's, where the device patches cartridge memory.
        public static IReadOnlyList<CheatWrite>? ProActionReplayWrites(string code)
        {
            var (address, value) = DecodeProActionReplay(code);
            if (((uint)address << 8 | value) == DoNothing) return Array.Empty<CheatWrite>();
            bool wram = address is >= 0x7E0000 and <= 0x7FFFFF;
            return new[] { new CheatWrite { Space = wram ? SnesSystem.Wram : SnesSystem.CpuBus, Address = wram ? address - 0x7E0000 : address, Value = value, Width = 1 } };
        }

        // ---- Game Genie: DDAA-AAAA, enciphered digits, the address's bits shuffled -----------------------------------

        // Genie digits in the order of the hex digits they stand for: D is 0, F is 1, and so on to E for F.
        public const string GenieDigits = "DF4709156BC8A23E";

        // Bit 23 first: where each address bit sits in the code's six address digits, and where it belongs in the CPU address.
        public const string GenieOrder = "ijklqrstopabcduvwxefghmn";
        public const string AddressOrder = "abcdefghijklmnopqrstuvwx";

        // The printed form, four digits, a dash, four digits, so that eight bare digits stay the Action Replay's.
        public static bool IsGameGenie(string code)
        {
            string t = code.Trim();
            return t.Length == 9 && t[4] == '-' && IsHex(t[..4]) && IsHex(t[5..]);
        }

        public static (int Address, byte Value) DecodeGameGenie(string code)
        {
            string d = Digits(code).ToUpperInvariant();
            if (d.Length != 8 || !d.All(c => GenieDigits.Contains(c))) throw new FormatException($"'{code}' is not a Game Genie code (DDAA-AAAA in Genie digits).");
            uint word = 0;
            foreach (char c in d) word = (word << 4) | (uint)GenieDigits.IndexOf(c);
            uint genie = word & 0xFFFFFF;
            uint address = 0;
            for (int i = 0; i < 24; i++)
                if ((genie >> (23 - GenieOrder.IndexOf(AddressOrder[i])) & 1) != 0) address |= 1u << (23 - i);
            return ((int)address, (byte)(word >> 24));
        }

        // The inverse, for tests and for showing a patch as a code.
        public static string EncodeGameGenie(int address, byte value)
        {
            uint genie = 0;
            for (int i = 0; i < 24; i++)
                if (((uint)address >> (23 - i) & 1) != 0) genie |= 1u << (23 - GenieOrder.IndexOf(AddressOrder[i]));
            uint word = ((uint)value << 24) | genie;
            var digits = Enumerable.Range(0, 8).Select(i => GenieDigits[(int)(word >> (28 - 4 * i)) & 0xF]).ToArray();
            return new string(digits, 0, 4) + "-" + new string(digits, 4, 4);
        }
    }
}

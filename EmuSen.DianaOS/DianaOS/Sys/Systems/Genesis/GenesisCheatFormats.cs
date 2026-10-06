using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.DianaOS.DianaOS.Sys.Systems.Genesis
{
    // The Genesis's Game Genie and Action Replay codes; the formats, their sources and the argued choices are in EmuSen_Cheats.md §9.
    public static class GenesisCheatFormats
    {
        // ---- Game Genie: ABCD-EFGH, eight digits of five bits holding a 24-bit address and a 16-bit word ----------------

        // The digits in the order of the values they stand for, 0 to 31.
        public const string GenieDigits = "ABCDEFGHJKLMNPRSTVWXYZ0123456789";

        // Each of the forty bits in the code's order: A-X are the address's bits from 23 down, a-p the word's from 15 down.
        public const string GenieOrder = "ijklmnopIJKLMNOPABCDEFGHdefghabcQRSTUVWX";

        private static readonly Regex GenieCode = new("^[A-Z0-9]{4}-[A-Z0-9]{4}$", RegexOptions.CultureInvariant);

        public static bool IsGameGenie(string code)
        {
            string t = code.Trim().ToUpperInvariant();
            return GenieCode.IsMatch(t) && t.Where(c => c != '-').All(c => GenieDigits.Contains(c));
        }

        public static (int Address, ushort Value) DecodeGameGenie(string code)
        {
            string d = new string(code.Trim().ToUpperInvariant().Where(c => c != '-').ToArray());
            if (d.Length != 8 || !d.All(c => GenieDigits.Contains(c))) throw new FormatException($"'{code}' is not a Game Genie code (ABCD-EFGH in the Genie's digits, without I, O, Q or U).");
            ulong bits = 0;
            foreach (char c in d) bits = (bits << 5) | (uint)GenieDigits.IndexOf(c);
            int address = 0, value = 0;
            for (int i = 0; i < 40; i++)
            {
                if ((bits >> (39 - i) & 1) == 0) continue;
                char label = GenieOrder[i];
                if (char.IsUpper(label)) address |= 1 << (23 - (label - 'A'));
                else value |= 1 << (15 - (label - 'a'));
            }
            return (address, (ushort)value);
        }

        // The inverse, for tests and for showing a patch as a code.
        public static string EncodeGameGenie(int address, ushort value)
        {
            ulong bits = 0;
            for (int i = 0; i < 40; i++)
            {
                char label = GenieOrder[i];
                int bit = char.IsUpper(label) ? address >> (23 - (label - 'A')) & 1 : value >> (15 - (label - 'a')) & 1;
                bits |= (ulong)bit << (39 - i);
            }
            var digits = Enumerable.Range(0, 8).Select(i => GenieDigits[(int)(bits >> (35 - 5 * i)) & 31]).ToArray();
            return new string(digits, 0, 4) + "-" + new string(digits, 4, 4);
        }

        // ---- Action Replay: an address and a value, printed AAAAAA:VVVV, AAAAA AVVVV as the device does, or bare ----------

        private static readonly Regex Colon = new("^([0-9A-F]{6}):([0-9A-F]{1,4}|[0-9A-F]{8})$", RegexOptions.CultureInvariant);
        private static readonly Regex Device = new("^([0-9A-F]{5})[ -]([0-9A-F])([0-9A-F]{4})$", RegexOptions.CultureInvariant);
        private static readonly Regex Bare = new("^([0-9A-F]{6})([0-9A-F]{2}|[0-9A-F]{4})$", RegexOptions.CultureInvariant);

        // The address, the value and its width in bytes, which the number of digits gives: one or two a byte, three or four a word, eight a long word.
        public static bool TryParseActionReplay(string code, out int address, out uint value, out int width)
        {
            (address, value, width) = (0, 0, 0);
            string t = code.Trim().ToUpperInvariant();
            string a, v;
            if (Colon.Match(t) is { Success: true } c) (a, v) = (c.Groups[1].Value, c.Groups[2].Value);
            else if (Device.Match(t) is { Success: true } m) (a, v) = (m.Groups[1].Value + m.Groups[2].Value, m.Groups[3].Value);
            else if (Bare.Match(t) is { Success: true } b) (a, v) = (b.Groups[1].Value, b.Groups[2].Value);
            else return false;
            address = int.Parse(a, NumberStyles.HexNumber, CultureInfo.InvariantCulture);
            value = uint.Parse(v, NumberStyles.HexNumber, CultureInfo.InvariantCulture);
            width = v.Length <= 2 ? 1 : v.Length <= 4 ? 2 : 4;
            return true;
        }

        public static bool IsRam(int address) => address is >= 0xE0_0000 and <= 0xFF_FFFF;
        public static bool IsCartridge(int address) => address is >= 0 and <= 0x3F_FFFF;

        // Codes joined with '+' are one cheat, as the libretro database writes them.
        private static IEnumerable<string> Parts(string code) => code.Split('+', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);

        private static bool ForRam(string part) => TryParseActionReplay(part, out int address, out _, out _) && IsRam(address);

        // An Action Replay code for the cartridge's range, which the device makes as a ROM patch, as the Game Genie does.
        private static bool ForCartridge(string part) => TryParseActionReplay(part, out int address, out _, out _) && IsCartridge(address);

        // Action Replay codes for the main RAM, which the RAM's mirrors from $E00000 all reach.
        public static bool IsActionReplay(string code) => Parts(code).Any() && Parts(code).All(ForRam);

        public static IReadOnlyList<CheatWrite> ActionReplayWrites(string code) => Parts(code).Select(p =>
        {
            if (!TryParseActionReplay(p, out int address, out uint value, out int width) || !IsRam(address))
                throw new FormatException($"'{p}' is not an Action Replay code for the main RAM (AAAAAA:VVVV at $E00000-$FFFFFF).");
            return CheatWrite.Poke(GenesisSystems.Wram, address & 0xFFFF, value, width) with { BigEndian = true };
        }).ToList();

        public static (int Address, byte Value) DecodeActionReplay(string code)
        {
            CheatWrite w = ActionReplayWrites(code)[0];
            return (w.Address, w.ByteAt(w.Value, 0));
        }

        // Each Game Genie code is the word on the 16-bit bus at its address, which has no line for the address's lowest bit.
        public static IReadOnlyList<CheatWrite> PatchWrites(string code) => Parts(code).Select(p =>
        {
            if (IsGameGenie(p))
            {
                var (address, value) = DecodeGameGenie(p);
                return CheatWrite.Patch(address & 0x3F_FFFE, value, 2) with { BigEndian = true };
            }
            if (TryParseActionReplay(p, out int a, out uint v, out int width) && IsCartridge(a)) return CheatWrite.Patch(a, v, width) with { BigEndian = true };
            throw new FormatException($"'{p}' is not a Game Genie code or an Action Replay code for the cartridge.");
        }).ToList();

        public static bool IsPatch(string code) => Parts(code).Any() && Parts(code).All(p => IsGameGenie(p) || ForCartridge(p));

        public static (int Address, byte Value) DecodePatch(string code)
        {
            CheatWrite w = PatchWrites(code)[0];
            return (w.Address, w.ByteAt(w.Value, 0));
        }
    }
}

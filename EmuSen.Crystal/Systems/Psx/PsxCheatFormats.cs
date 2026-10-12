using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.Crystal.Cheats;

namespace EmuSen.Crystal.Systems.Psx
{
    // The PlayStation's GameShark (with Game Buster and Action Replay, the same Datel format) and Xploder codes, as RAM writes and tests; the formats, the choices and the refusals are EmuSen_Cheats.md §10.
    public static class PsxCheatFormats
    {
        // A code's line: the command word, whose top byte is the type, and the value.
        public readonly record struct Line(uint Command, ushort Value)
        {
            public byte Type => (byte)(Command >> 24);
            public int Address => (int)(Command & 0x1F_FFFF);
            public override string ToString() => $"{Command:X8} {Value:X4}";
        }

        // Every hex digit of the code, in twelves: the libretro database joins its lines and even its halves with '+'.
        public static bool TryReadLines(string? code, out List<Line> lines)
        {
            lines = new List<Line>();
            string digits = new((code ?? "").Where(Uri.IsHexDigit).ToArray());
            if (digits.Length == 0 || digits.Length % 12 != 0 || (code ?? "").Any(c => char.IsLetterOrDigit(c) && !Uri.IsHexDigit(c))) return false;
            for (int at = 0; at < digits.Length; at += 12)
                lines.Add(new Line(uint.Parse(digits.Substring(at, 8), NumberStyles.HexNumber, CultureInfo.InvariantCulture), ushort.Parse(digits.Substring(at + 8, 4), NumberStyles.HexNumber, CultureInfo.InvariantCulture)));
            return true;
        }

        // ---- GameShark ---------------------------------------------------------------------------------------------

        // The first digits psx-spx gives the Datel format, 3 and 8 shared with the Xploder, 0 for its dummy line and 5 for its slide.
        private static bool GameSharkLine(Line l) => (l.Type >> 4) is 0x1 or 0x2 or 0x3 or 0x5 or 0x8 or 0xC or 0xD or 0xE || (l.Command == 0 && l.Value == 0);

        public static bool IsGameShark(string code) => TryReadLines(code, out var lines) && lines.All(GameSharkLine);

        // Each line a write or a test, a slide and its line one write; a code the device's chapter describes and EmuSen does not apply is refused by name.
        public static IReadOnlyList<CheatWrite> GameSharkWrites(string code)
        {
            if (!TryReadLines(code, out var lines) || !lines.All(GameSharkLine))
                throw new FormatException($"'{code}' is not a PlayStation GameShark code: lines of eight and four hex digits (TTAAAAAA VVVV) whose types are the Datel device's.");
            var writes = new List<CheatWrite>();
            for (int i = 0; i < lines.Count; i++)
            {
                Line l = lines[i];
                switch (l.Type)
                {
                    case 0x30: writes.Add(Write(l, 1, CheatWriteType.Set)); break;
                    case 0x80: writes.Add(Write(l, 2, CheatWriteType.Set)); break;
                    case 0x10: writes.Add(Write(l, 2, CheatWriteType.Increase)); break;
                    case 0x11: writes.Add(Write(l, 2, CheatWriteType.Decrease)); break;
                    case 0x20: writes.Add(Write(l, 1, CheatWriteType.Increase)); break;
                    case 0x21: writes.Add(Write(l, 1, CheatWriteType.Decrease)); break;
                    case 0xD0: writes.Add(Write(l, 2, CheatWriteType.IfEqual)); break;
                    case 0xD1: writes.Add(Write(l, 2, CheatWriteType.IfNotEqual)); break;
                    case 0xD2: writes.Add(Write(l, 2, CheatWriteType.IfGreater)); break;
                    case 0xD3: writes.Add(Write(l, 2, CheatWriteType.IfLess)); break;
                    case 0xE0: writes.Add(Write(l, 1, CheatWriteType.IfEqual)); break;
                    case 0xE1: writes.Add(Write(l, 1, CheatWriteType.IfNotEqual)); break;
                    case 0xE2: writes.Add(Write(l, 1, CheatWriteType.IfGreater)); break;
                    case 0xE3: writes.Add(Write(l, 1, CheatWriteType.IfLess)); break;
                    case 0x00 when l.Command == 0 && l.Value == 0: break;
                    case 0x50:
                        if (i + 1 >= lines.Count) throw new FormatException($"'{l}' is a slide with no line after it to repeat.");
                        writes.Add(Slide(l, lines[++i]));
                        break;
                    default: throw new FormatException(GameSharkRefusal(l));
                }
            }
            return Guarded(writes, lines, code);
        }

        // 5000nnbb dddd then the write it repeats, aaaaaaaa ??ee: nn writes of dddd, bb apart in address and ee apart in value, as psx-spx's slide gives it.
        private static CheatWrite Slide(Line slide, Line next)
        {
            int count = (int)(slide.Command >> 8 & 0xFF), step = (int)(slide.Command & 0xFF);
            if ((slide.Command & 0x00FF_0000) != 0) throw new FormatException($"'{slide}' is not a slide the chapter gives, whose second byte is 00.");
            if (count == 0) throw new FormatException($"'{slide}' slides zero times, which writes nothing.");
            int width = next.Type switch
            {
                0x30 => 1,
                0x80 => 2,
                _ => throw new FormatException($"'{slide}' must be followed by the 30 or 80 line it repeats, not '{next}'."),
            };
            return Write(next, width, CheatWriteType.Set) with { Value = width == 1 ? (uint)(slide.Value & 0xFF) : slide.Value, RepeatCount = count, RepeatAddAddress = step, RepeatAddValue = next.Value };
        }

        private static string GameSharkRefusal(Line l) => l.Type switch
        {
            0xC0 => $"'{l}' turns every code on when its test passes; whether the device keeps them on after is not in its chapter, so EmuSen applies no such code yet.",
            0xC1 => $"'{l}' delays the codes from the device's own start, which EmuSen does not have; leave it out and the codes apply from the first frame.",
            0xC2 => $"'{l}' copies a block of memory, a kind of code EmuSen's cheats do not have.",
            0xC3 => $"'{l}' is an indirect write of one homebrew loader's, not the Datel device's.",
            0xD4 or 0xD5 or 0xD6 => $"'{l}' tests the pad's buttons as the device reads them, which a cheat in EmuSen cannot.",
            0x12 or 0x22 => $"'{l}' is a 32-bit increment of one homebrew loader's, not the Datel device's.",
            _ => $"'{l}' is not a GameShark code type EmuSen decodes: it takes 30, 80, 10, 11, 20, 21, D0 to D3, E0 to E3 and 50.",
        };

        // ---- Xploder -----------------------------------------------------------------------------------------------

        // The six bytes AABBCCDD EEFF decrypted by the key in the type's low three bits, psx-spx's decrypt_xplorer_cheat_code.
        public static Line Decrypt(Line l)
        {
            var x = new byte[] { (byte)(l.Command >> 24), (byte)(l.Command >> 16), (byte)(l.Command >> 8), (byte)l.Command, (byte)(l.Value >> 8), (byte)l.Value };
            int key = x[0] & 7;
            x[0] ^= (byte)key;
            unchecked
            {
                switch (key)
                {
                    case 0: break;
                    case 4:
                        x[1] ^= 0x25;
                        x[2] ^= (byte)(0xFA + (x[1] & 0x11));
                        x[3] ^= (byte)(0xC0 + (x[2] & 0x11) + (x[1] ^ 0x12));
                        x[4] ^= (byte)(0x7E + (x[3] & 0x11) + (x[2] ^ 0x12) + x[1]);
                        x[5] ^= (byte)(0x26 + (x[4] & 0x11) + (x[3] ^ 0x12) + x[2] + x[1]);
                        break;
                    case 5:
                        x[1] += 0x57; x[2] += 0x42; x[3] += 0x31; x[4] += 0x32; x[5] += 0x33;
                        break;
                    case 6:
                        for (int i = 1; i <= 5; i++) x[i] = (byte)((x[i] + 0xAB) ^ i);
                        break;
                    case 7:
                        x[5] += 0xCB;
                        x[4] = (byte)(x[4] + 0xCB + (x[5] & 0x73));
                        x[3] = (byte)(x[3] + 0x5A + (x[4] & 0x73) - (x[5] ^ 0x90));
                        x[2] = (byte)(x[2] + 0x16 + (x[3] & 0x73) - (x[4] ^ 0x90) + x[5]);
                        x[1] = (byte)(x[1] + 0xF5 + (x[2] & 0x73) - (x[3] ^ 0x90) + x[4] + x[5]);
                        break;
                    default: throw new FormatException($"'{l}' names encryption key {key}, which the Xploder's chapter gives no rule for.");
                }
            }
            return new Line((uint)(x[0] << 24 | x[1] << 16 | x[2] << 8 | x[3]), (ushort)(x[4] << 8 | x[5]));
        }

        private static bool XploderLine(Line l) => (l.Type >> 4) is 0x0 or 0x3 or 0x4 or 0x5 or 0x6 or 0x7 or 0x8 or 0x9 or 0xB or 0xC or 0xD or 0xF;

        public static bool IsXploder(string code) => TryReadLines(code, out var lines) && lines.All(XploderLine);

        // The type's high digit after decryption names the code; its low digit's bit 3 is the device's own default-off flag, which a cheat's switch stands for.
        public static IReadOnlyList<CheatWrite> XploderWrites(string code)
        {
            if (!TryReadLines(code, out var raw) || !raw.All(XploderLine))
                throw new FormatException($"'{code}' is not a PlayStation Xploder code: lines of eight and four hex digits (TtAAAAAA VVVV) whose types are the Xploder's.");
            var lines = raw.Select(Decrypt).ToList();
            var writes = new List<CheatWrite>();
            foreach (var l in lines)
            {
                switch (l.Type >> 4)
                {
                    case 0x3: writes.Add(Write(l, 1, CheatWriteType.Set)); break;
                    case 0x8: writes.Add(Write(l, 2, CheatWriteType.Set)); break;
                    case 0x0: writes.Add(Write(l, 4, CheatWriteType.Set)); break;
                    case 0x7: writes.Add(Write(l, 2, CheatWriteType.IfEqual)); break;
                    case 0x9: writes.Add(Write(l, 2, CheatWriteType.IfNotEqual)); break;
                    default: throw new FormatException(XploderRefusal(l));
                }
            }
            return Guarded(writes, lines, code);
        }

        private static string XploderRefusal(Line l) => (l.Type >> 4) switch
        {
            0x4 => $"'{l}' slows the game down, by an amount the Xploder's chapter does not give, so EmuSen applies no such code.",
            0x5 => $"'{l}' writes a run of bytes listed after it, which this decoder does not take yet.",
            0x6 => $"'{l}' sets a processor breakpoint for code of the device's own, which a cheat in EmuSen cannot run.",
            0xB => $"'{l}' is a slide whose end the Xploder's chapter leaves unclear, so EmuSen applies no such code.",
            0xF => $"'{l}' turns on other selected codes, a rule the Xploder's chapter does not give.",
            _ => $"'{l}' is a type the Xploder's chapter calls a mirror of another or does not give.",
        };

        // ---- Either -------------------------------------------------------------------------------------------------

        // A code whose types are all the Datel device's is a GameShark code, 3 and 8 meaning the same on both; any other the Xploder's.
        public static bool IsEither(string code) => IsGameShark(code) || IsXploder(code);

        public static IReadOnlyList<CheatWrite> EitherWrites(string code) => IsGameShark(code) ? GameSharkWrites(code) : XploderWrites(code);

        // ---- Shared ------------------------------------------------------------------------------------------------

        // A write's value is the line's low byte for an 8-bit code; the PlayStation is little-endian, every address is main RAM's, and an odd one is written byte by byte.
        private static CheatWrite Write(Line l, int width, CheatWriteType type)
        {
            return new CheatWrite
            {
                Space = PsxSystem.Ram,
                Address = l.Address,
                Value = width == 1 ? (uint)(l.Value & 0xFF) : l.Value,
                Width = width,
                Type = type,
                RepeatCount = 1,
            };
        }

        // A test with nothing after it guards nothing.
        private static IReadOnlyList<CheatWrite> Guarded(List<CheatWrite> writes, List<Line> lines, string code)
        {
            if (writes.Count == 0) throw new FormatException($"'{code}' holds only dummy lines and writes nothing.");
            if (writes[^1].IsTest) throw new FormatException($"'{lines[^1]}' tests memory but guards nothing - a test must be followed by the code it guards.");
            return writes;
        }

        // The single byte the older callers ask for, only where the code is one byte written once.
        public static (int Address, byte Value) OneByte(IReadOnlyList<CheatWrite> writes, string code) =>
            writes is [{ EffectiveWidth: 1, EffectiveRepeatCount: 1, Type: CheatWriteType.Set } only]
                ? (only.Address, (byte)only.Value)
                : throw new FormatException($"'{code}' is more than one byte written once - it has to be added whole, through DecodeWrites.");
    }
}

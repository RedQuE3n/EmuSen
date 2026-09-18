using System;
using System.Collections.Generic;
using System.Globalization;
using System.Text;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.Mars.Cheats
{
    // The N64 GameShark: lines of an eight-digit command and a four-digit value, decoded to RDRAM writes and tests - see Mars_Cheats.md.
    public static class N64GameSharkCodec
    {
        // The command's top byte is the type, so the rest is a physical address and KSEG0 and KSEG1 name one byte - see Mars_Cheats.md §3.
        public const uint AddressMask = 0x00FF_FFFF;

        public readonly record struct Line(uint Command, ushort Value)
        {
            public byte Type => (byte)(Command >> 24);
            public int Address => (int)(Command & AddressMask);
            public override string ToString() => $"{Command:X8} {Value:X4}";
        }

        public static bool CanDecode(string code) => TryReadLines(code, out _);

        // Every line becomes a write or a test, a repeater and its line become one write, and anything else is refused by name - see Mars_Cheats.md §2.
        public static IReadOnlyList<CheatWrite> DecodeWrites(string code)
        {
            if (!TryReadLines(code, out List<Line> lines))
            {
                throw new FormatException(
                    $"'{code}' isn't an N64 GameShark code - expected lines of 8 and 4 hex digits (TTAAAAAA VVVV), joined by '+' or spaces.");
            }

            if (code.Contains('?'))
            {
                throw new FormatException(
                    $"'{code}' holds '?', Project64's placeholder for a value picked from a list - put the value itself in its place.");
            }

            var writes = new List<CheatWrite>(lines.Count);
            for (int i = 0; i < lines.Count; i++)
            {
                Line line = lines[i];
                switch (line.Type)
                {
                    case 0x80:
                    case 0xA0:
                        writes.Add(Write(line, 1));
                        break;
                    case 0x81:
                    case 0xA1:
                        writes.Add(Write(line, 2));
                        break;
                    case 0xD0: writes.Add(Test(line, 1, CheatWriteType.IfEqual)); break;
                    case 0xD1: writes.Add(Test(line, 2, CheatWriteType.IfEqual)); break;
                    case 0xD2: writes.Add(Test(line, 1, CheatWriteType.IfNotEqual)); break;
                    case 0xD3: writes.Add(Test(line, 2, CheatWriteType.IfNotEqual)); break;
                    case 0x50:
                        if (i + 1 >= lines.Count) throw new FormatException($"'{line}' is a repeater with no line after it to repeat.");
                        writes.Add(Repeat(line, lines[++i]));
                        break;
                    default:
                        throw new FormatException(Refusal(line));
                }
            }

            if (writes[^1].IsTest)
            {
                throw new FormatException($"'{lines[^1]}' tests memory but guards nothing - a D0-D3 line must be followed by the code it guards.");
            }

            return writes;
        }

        // The single-byte answer the older callers ask for, which only a lone 8-bit write can give honestly.
        public static (int Address, byte Value) Decode(string code)
        {
            IReadOnlyList<CheatWrite> writes = DecodeWrites(code);
            CheatWrite only = writes[0];

            if (writes.Count != 1 || only.EffectiveWidth != 1 || only.EffectiveRepeatCount != 1)
            {
                throw new FormatException($"'{code}' is more than one byte written once - it has to be added whole, through DecodeWrites.");
            }

            return (only.Address, (byte)only.Value);
        }

        private static CheatWrite Write(Line line, int width)
        {
            RequireEven(line, line.Address, width);

            // Both references take an 8-bit code's value from its low byte - see Mars_Cheats.md §2.1.
            return new CheatWrite
            {
                Space = MarsCore.SpaceRdram,
                Address = line.Address,
                Value = width == 1 ? (uint)(line.Value & 0xFF) : line.Value,
                Width = width,
                BigEndian = true,
                Type = CheatWriteType.Set,
                RepeatCount = 1,
            };
        }

        // An 8-bit test compares the low byte, as mupen64plus does and Project64 does not - see Mars_Cheats.md §2.2.
        private static CheatWrite Test(Line line, int width, CheatWriteType type) => Write(line, width) with { Type = type };

        // 5000CCSS VVVV: count, address step and value step, applied to the one write after it - see Mars_Cheats.md §2.3.
        private static CheatWrite Repeat(Line repeater, Line next)
        {
            int count = (int)((repeater.Command >> 8) & 0xFF);
            int step = (int)(repeater.Command & 0xFF);

            if (count == 0) throw new FormatException($"'{repeater}' repeats zero times, which writes nothing - both reference emulators skip it too.");

            int width = next.Type switch
            {
                0x80 or 0xA0 => 1,
                0x81 or 0xA1 => 2,
                _ => throw new FormatException($"'{repeater}' must be followed by the 80, 81, A0 or A1 line it repeats, not '{next}'."),
            };

            CheatWrite first = Write(next, width);
            if (count > 1) RequireEven(next, next.Address + step, width);

            return first with { RepeatCount = count, RepeatAddAddress = step, RepeatAddValue = repeater.Value };
        }

        private static void RequireEven(Line line, int address, int width)
        {
            if (width == 2 && (address & 1) != 0)
            {
                throw new FormatException(
                    $"'{line}' puts 16 bits at an odd address, which no halfword store can do; both reference emulators tear it across two words - see Mars_Cheats.md §2.4.");
            }
        }

        // Named refusals, so a code Mars will not apply says why instead of doing nothing - see Mars_Cheats.md §4.
        private static string Refusal(Line line) => line.Type switch
        {
            0x88 or 0x89 or 0xA8 or 0xA9 or 0xD8 or 0xD9 or 0xDA or 0xDB =>
                $"'{line}' applies only while the GameShark's own button is held, and Mars has no such button - write it as 80 or 81 to hold the value every frame.",
            0xF0 or 0xF1 =>
                $"'{line}' is a boot-time write, and Mars applies cheats only at frame boundaries, to a list handed over after the game is loaded - there is no boot to write it at.",
            0xEE =>
                $"'{line}' hides the Expansion Pak from the game; Mars decides the Pak when the console is built, so leave this code out.",
            0xDE =>
                $"'{line}' has no effect on memory in either reference emulator, and nothing here documents what it asks of a GameShark - leave it out.",
            0xCC or 0xFF =>
                $"'{line}' is not a code type either reference emulator applies, so Mars refuses it rather than guess what it does.",
            _ =>
                $"'{line}' is not a code type Mars decodes - it takes 80, 81, A0, A1, D0, D1, D2, D3 and 50.",
        };

        // Hex runs split on anything else: an 8-digit run then a 1-4 digit one is a line, and a 12-digit run is one run-together line.
        private static bool TryReadLines(string? code, out List<Line> lines)
        {
            lines = new List<Line>();
            List<string> runs = Runs(code ?? "");
            if (runs.Count == 0) return false;

            for (int i = 0; i < runs.Count; i++)
            {
                string run = runs[i];
                if (run.Length == 8 && i + 1 < runs.Count && runs[i + 1].Length is >= 1 and <= 4)
                {
                    lines.Add(Parse(run, runs[++i]));
                    continue;
                }

                if (run.Length % 12 != 0) return false;
                for (int at = 0; at < run.Length; at += 12) lines.Add(Parse(run.Substring(at, 8), run.Substring(at + 8, 4)));
            }

            return true;
        }

        // '?' rides in a run so the placeholder reaches DecodeWrites' own message rather than a generic one.
        private static List<string> Runs(string code)
        {
            var runs = new List<string>();
            var current = new StringBuilder();

            foreach (char c in code)
            {
                if (Uri.IsHexDigit(c) || c == '?')
                {
                    current.Append(c);
                    continue;
                }

                if (current.Length > 0) runs.Add(current.ToString());
                current.Clear();
            }

            if (current.Length > 0) runs.Add(current.ToString());
            return runs;
        }

        private static Line Parse(string command, string value) => new(
            uint.Parse(command.Replace('?', '0'), NumberStyles.HexNumber, CultureInfo.InvariantCulture),
            ushort.Parse(value.Replace('?', '0'), NumberStyles.HexNumber, CultureInfo.InvariantCulture));
    }
}

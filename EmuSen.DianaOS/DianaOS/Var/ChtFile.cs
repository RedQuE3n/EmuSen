using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // One cheat read out of a .cht file, before it reaches the registry.
    public readonly struct ChtCheat
    {
        public string Description { get; init; }
        public bool Enabled { get; init; }
        public IReadOnlyList<CheatWrite> Writes { get; init; }
    }

    public readonly struct ChtParseResult
    {
        public IReadOnlyList<ChtCheat> Cheats { get; init; }

        // Entries that were present but could not be turned into writes -
        // an undecodable code, a cheat_type of 0, a malformed number.
        public int Skipped { get; init; }
    }

    // RetroArch's .cht format - the one the libretro cheat database ships
    // in. See `man cheat` for what maps onto what and what deliberately
    // does not.
    //
    // Core-agnostic: the only core-specific part of the format is the
    // `cheatN_code` string, whose layout differs per system, and that is
    // decoded through the same ICheatCodeCodec seam `cheat add` already
    // uses rather than being hardcoded here.
    public static class ChtFile
    {
        // RetroArch's memory_search_size enum: 0-2 are sub-byte, 3-5 are
        // 1, 2 and 4 bytes.
        private const int SizeByte = 3;
        private const int SizeWord = 4;
        private const int SizeDword = 5;

        // RetroArch's cheat_type enum. 0 means the entry exists but does
        // nothing, which is not a cheat we can add.
        private const int TypeDisabled = 0;
        private const int TypeSet = 1;
        private const int TypeIncrease = 2;
        private const int TypeDecrease = 3;

        public static ChtParseResult Parse(string text, ICheatCodeCodec? codeDecoder, string spaceName)
        {
            Dictionary<string, string> fields = ReadFields(text);
            var cheats = new List<ChtCheat>();
            int skipped = 0;

            if (!fields.TryGetValue("cheats", out string? countText) ||
                !int.TryParse(countText, NumberStyles.Integer, CultureInfo.InvariantCulture, out int count))
            {
                return new ChtParseResult { Cheats = cheats, Skipped = 0 };
            }

            for (int i = 0; i < count; i++)
            {
                if (TryReadCheat(fields, i, codeDecoder, spaceName, out ChtCheat cheat)) cheats.Add(cheat);
                else skipped++;
            }

            return new ChtParseResult { Cheats = cheats, Skipped = skipped };
        }

        private static bool TryReadCheat(Dictionary<string, string> fields, int index, ICheatCodeCodec? codeDecoder, string spaceName, out ChtCheat cheat)
        {
            cheat = default;

            string prefix = $"cheat{index}_";
            string description = Get(fields, prefix + "desc") ?? $"cheat{index}";
            bool enabled = string.Equals(Get(fields, prefix + "enable"), "true", StringComparison.OrdinalIgnoreCase);

            List<CheatWrite>? writes = HasField(fields, prefix + "address")
                ? ReadExplicit(fields, prefix, spaceName)
                : ReadCode(Get(fields, prefix + "code"), codeDecoder, spaceName);

            if (writes is null || writes.Count == 0) return false;

            cheat = new ChtCheat { Description = description, Enabled = enabled, Writes = writes };
            return true;
        }

        // The "retro" handler's explicit fields. Every number here is
        // DECIMAL in a .cht file, unlike the code string's hex.
        private static List<CheatWrite>? ReadExplicit(Dictionary<string, string> fields, string prefix, string spaceName)
        {
            int cheatType = Int(fields, prefix + "cheat_type", TypeSet);
            if (cheatType == TypeDisabled) return null;

            CheatWriteType type = cheatType switch
            {
                TypeIncrease => CheatWriteType.Increase,
                TypeDecrease => CheatWriteType.Decrease,
                _ => CheatWriteType.Set,
            };

            long address = Long(fields, prefix + "address", -1);
            if (address < 0) return null;

            int searchSize = Int(fields, prefix + "memory_search_size", SizeByte);
            int width = searchSize switch
            {
                SizeWord => 2,
                SizeDword => 4,
                _ => 1,
            };

            // Sub-byte sizes are bit writes; address_bit_position says which.
            int? bitPosition = searchSize < SizeByte ? Int(fields, prefix + "address_bit_position", 0) : null;

            return new List<CheatWrite>
            {
                new CheatWrite
                {
                    Space = spaceName,
                    Address = (int)address,
                    Value = (uint)Long(fields, prefix + "value", 0),
                    Width = width,
                    Type = type,
                    BitPosition = bitPosition,
                    BigEndian = string.Equals(Get(fields, prefix + "big_endian"), "true", StringComparison.OrdinalIgnoreCase),
                    RepeatCount = Int(fields, prefix + "repeat_count", 1),
                    RepeatAddAddress = Int(fields, prefix + "repeat_add_to_address", 0),
                    RepeatAddValue = (uint)Long(fields, prefix + "repeat_add_to_value", 0),
                },
            };
        }

        // The core-native form: one or more codes joined with '+'.
        private static List<CheatWrite>? ReadCode(string? code, ICheatCodeCodec? codeDecoder, string spaceName)
        {
            if (string.IsNullOrWhiteSpace(code) || codeDecoder is null) return null;

            var writes = new List<CheatWrite>();
            foreach (string token in code.Split('+', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
            {
                if (!codeDecoder.CanDecode(token)) return null;
                try
                {
                    (int address, byte value) = codeDecoder.Decode(token);
                    writes.Add(CheatWrite.Poke(spaceName, address, value));
                }
                catch (FormatException)
                {
                    return null;
                }
            }
            return writes;
        }

        // Writes the retro-handler form, which round-trips every field the
        // model has. RomPatch cheats have no equivalent in RetroArch's
        // model and are the caller's business to filter - see `man cheat`.
        public static string Write(IReadOnlyList<ChtCheat> cheats)
        {
            var sb = new StringBuilder();
            sb.Append("cheats = ").Append(cheats.Count).Append('\n');

            for (int i = 0; i < cheats.Count; i++)
            {
                ChtCheat cheat = cheats[i];

                // One .cht entry holds one write, so a multi-write cheat's
                // extra writes ride along in the code string the way
                // RetroArch's own multi-code entries do.
                CheatWrite first = cheat.Writes.Count > 0 ? cheat.Writes[0] : default;
                string prefix = $"cheat{i}_";

                sb.Append('\n');
                sb.Append(prefix).Append("desc = \"").Append(Escape(cheat.Description)).Append("\"\n");
                sb.Append(prefix).Append("handler = 1\n");
                sb.Append(prefix).Append("enable = ").Append(cheat.Enabled ? "true" : "false").Append('\n');
                sb.Append(prefix).Append("cheat_type = ").Append(first.Type switch
                {
                    CheatWriteType.Increase => TypeIncrease,
                    CheatWriteType.Decrease => TypeDecrease,
                    _ => TypeSet,
                }).Append('\n');
                sb.Append(prefix).Append("memory_search_size = ").Append(first.BitPosition.HasValue
                    ? 0
                    : first.EffectiveWidth switch { 2 => SizeWord, 4 => SizeDword, _ => SizeByte }).Append('\n');
                sb.Append(prefix).Append("address = ").Append(first.Address).Append('\n');
                sb.Append(prefix).Append("value = ").Append(first.Value).Append('\n');
                sb.Append(prefix).Append("address_bit_position = ").Append(first.BitPosition ?? 0).Append('\n');
                sb.Append(prefix).Append("big_endian = ").Append(first.BigEndian ? "true" : "false").Append('\n');
                sb.Append(prefix).Append("repeat_count = ").Append(first.EffectiveRepeatCount).Append('\n');
                sb.Append(prefix).Append("repeat_add_to_address = ").Append(first.RepeatAddAddress).Append('\n');
                sb.Append(prefix).Append("repeat_add_to_value = ").Append(first.RepeatAddValue).Append('\n');

                if (cheat.Writes.Count > 1)
                {
                    sb.Append(prefix).Append("code = \"")
                      .Append(string.Join("+", cheat.Writes.Select(w => $"{w.Address:X6}{w.Value:X2}")))
                      .Append("\"\n");
                }
            }

            return sb.ToString();
        }

        private static string Escape(string text) => text.Replace("\"", "'");

        // key = value, one per line, values optionally quoted. Unknown keys
        // are kept rather than rejected - a newer RetroArch field this build
        // does not read must not make the whole file unparseable.
        private static Dictionary<string, string> ReadFields(string text)
        {
            var fields = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);

            foreach (string rawLine in text.Split('\n'))
            {
                string line = rawLine.Trim();
                if (line.Length == 0 || line.StartsWith('#') || line.StartsWith("//", StringComparison.Ordinal)) continue;

                int equals = line.IndexOf('=');
                if (equals <= 0) continue;

                string key = line[..equals].Trim();
                string value = line[(equals + 1)..].Trim();
                if (value.Length >= 2 && value[0] == '"' && value[^1] == '"') value = value[1..^1];

                fields[key] = value;
            }

            return fields;
        }

        private static bool HasField(Dictionary<string, string> fields, string key) => fields.ContainsKey(key);

        private static string? Get(Dictionary<string, string> fields, string key) =>
            fields.TryGetValue(key, out string? value) ? value : null;

        private static int Int(Dictionary<string, string> fields, string key, int fallback) =>
            (int)Long(fields, key, fallback);

        private static long Long(Dictionary<string, string> fields, string key, long fallback) =>
            Get(fields, key) is string text && long.TryParse(text, NumberStyles.Integer, CultureInfo.InvariantCulture, out long value)
                ? value
                : fallback;
    }
}

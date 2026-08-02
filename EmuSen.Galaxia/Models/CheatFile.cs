using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;

namespace EmuSen.Galaxia.Models
{
    // One write inside a saved cheat. Addresses and values are hex TEXT, not
    // numbers, because these files exist to be read and edited by hand from
    // the shell and no one writes a SNES address in decimal - see
    // EmuSen_Config_Reference.md §3.4.
    //
    // A dumb carrier: EmuSen.Galaxia cannot reference EmuSen.DianaOS (the
    // dependency runs the other way), so the mapping onto CheatWrite lives
    // in CheatRegistry and this side only parses text into primitives.
    public class CheatFileWrite
    {
        // RamPoke only - which named memory space to write into.
        public string Space { get; set; } = "";

        public string Address { get; set; } = "0";
        public string Value { get; set; } = "00";

        // 1, 2 or 4 bytes.
        public int Width { get; set; } = 1;

        // "Set", "Increase" or "Decrease"; matched case-insensitively.
        public string Type { get; set; } = "Set";

        // Absent means the write covers Width bytes rather than one bit.
        public int? BitPosition { get; set; }

        public bool BigEndian { get; set; }

        public int RepeatCount { get; set; } = 1;
        public string RepeatAddAddress { get; set; } = "0";
        public string RepeatAddValue { get; set; } = "0";

        public bool TryParseNumbers(out uint address, out uint value, out uint repeatAddAddress, out uint repeatAddValue)
        {
            value = repeatAddAddress = repeatAddValue = 0;
            if (!TryHex(Address, out address)) return false;
            if (!TryHex(Value, out value)) return false;

            // Absent repeat fields are 0, not an error - hand-written files
            // routinely omit them entirely.
            if (!string.IsNullOrWhiteSpace(RepeatAddAddress) && !TryHex(RepeatAddAddress, out repeatAddAddress)) return false;
            if (!string.IsNullOrWhiteSpace(RepeatAddValue) && !TryHex(RepeatAddValue, out repeatAddValue)) return false;
            return true;
        }

        public static bool TryHex(string? text, out uint value)
        {
            value = 0;
            return !string.IsNullOrWhiteSpace(text)
                && uint.TryParse(Strip(text), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out value);
        }

        // Tolerates the 0x/$ prefixes people paste in alongside bare hex.
        internal static string Strip(string text)
        {
            string t = text.Trim();
            if (t.StartsWith("0x", StringComparison.OrdinalIgnoreCase)) return t[2..];
            if (t.StartsWith('$')) return t[1..];
            return t;
        }

        public static string Hex(uint value, int digits) => value.ToString("X" + digits, CultureInfo.InvariantCulture);
    }

    // One cheat as it appears on disk - see EmuSen_Config_Reference.md §3.4.
    public class CheatFileEntry
    {
        // "RamPoke" or "RomPatch"; matched case-insensitively on load.
        public string Kind { get; set; } = "RamPoke";

        public List<CheatFileWrite> Writes { get; set; } = new();

        // RomPatch only - null/absent means an unconditional patch.
        public string? Compare { get; set; }

        public string Description { get; set; } = "";
        public bool Enabled { get; set; } = true;

        // --- Pre-multi-write fields, still read so existing files load ---
        // Left null when saving, so a re-save quietly migrates the file.
        // See EmuSen_Config_Reference.md §3.4.
        public string? Space { get; set; }
        public string? Address { get; set; }
        public string? Value { get; set; }

        public bool IsRomPatch => string.Equals(Kind, "RomPatch", StringComparison.OrdinalIgnoreCase);

        // Writes as written, or the single write a pre-multi-write file's
        // flat Space/Address/Value fields describe.
        public IReadOnlyList<CheatFileWrite> EffectiveWrites
        {
            get
            {
                if (Writes.Count > 0) return Writes;
                if (Address is null) return Array.Empty<CheatFileWrite>();
                return new[]
                {
                    new CheatFileWrite
                    {
                        Space = Space ?? "",
                        Address = Address,
                        Value = Value ?? "00",
                        Width = 1,
                        Type = "Set",
                        RepeatCount = 1,
                    },
                };
            }
        }

        // True with a null result for "absent", false only for text that was
        // there but wasn't hex - an unwritable compare byte is a broken entry.
        public bool TryParseCompare(out byte? compare)
        {
            compare = null;
            if (string.IsNullOrWhiteSpace(Compare) || Compare.Trim() == "-") return true;
            if (!byte.TryParse(CheatFileWrite.Strip(Compare), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out byte parsed)) return false;
            compare = parsed;
            return true;
        }
    }

    // A named set of cheats, one file per game - see EmuSen_Config_Reference.md §3.4.
    public class CheatFile
    {
        public const string CategoryDirName = "cheats";

        public List<CheatFileEntry> Cheats { get; set; } = new();

        // The name is user-typed, so it is checked rather than sanitized:
        // silently rewriting it would save to a file they didn't name.
        public static bool IsValidName(string name) =>
            !string.IsNullOrWhiteSpace(name)
            && name.IndexOfAny(Path.GetInvalidFileNameChars()) < 0
            && name != "." && name != "..";

        public static ConfigFile<CheatFile> For(string name) =>
            new(CategoryDirName, name + ".json");

        public static string DirectoryPath => Path.Combine(ConfigStore.Directory, CategoryDirName);

        public static IReadOnlyList<string> ListNames()
        {
            try
            {
                return Directory.EnumerateFiles(DirectoryPath, "*.json")
                    .Select(Path.GetFileNameWithoutExtension)
                    .Where(n => !string.IsNullOrEmpty(n))
                    .Select(n => n!)
                    .OrderBy(n => n, StringComparer.OrdinalIgnoreCase)
                    .ToList();
            }
            catch
            {
                return Array.Empty<string>();
            }
        }
    }
}

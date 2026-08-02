using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;

namespace EmuSen.Galaxia.Models
{
    // One cheat as it appears on disk. Addresses and bytes are hex TEXT, not
    // numbers, because these files exist to be read and edited by hand from
    // the shell and no one writes a SNES address in decimal - see
    // EmuSen_Config_Reference.md §3.4.
    public class CheatFileEntry
    {
        // "RamPoke" or "RomPatch"; matched case-insensitively on load.
        public string Kind { get; set; } = "RamPoke";

        // RamPoke only - which named memory space to write into.
        public string Space { get; set; } = "";

        public string Address { get; set; } = "0";
        public string Value { get; set; } = "00";

        // RomPatch only - null/absent means an unconditional patch.
        public string? Compare { get; set; }

        public string Description { get; set; } = "";
        public bool Enabled { get; set; } = true;

        public bool IsRomPatch => string.Equals(Kind, "RomPatch", StringComparison.OrdinalIgnoreCase);

        public bool TryParseAddress(out int address) =>
            int.TryParse(Strip(Address), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out address);

        public bool TryParseValue(out byte value)
        {
            bool ok = byte.TryParse(Strip(Value), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out value);
            return ok;
        }

        // True with a null result for "absent", false only for text that was
        // there but wasn't hex - an unwritable compare byte is a broken entry.
        public bool TryParseCompare(out byte? compare)
        {
            compare = null;
            if (string.IsNullOrWhiteSpace(Compare) || Compare.Trim() == "-") return true;
            if (!byte.TryParse(Strip(Compare), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out byte parsed)) return false;
            compare = parsed;
            return true;
        }

        // Tolerates the 0x/$ prefixes people paste in alongside bare hex.
        private static string Strip(string text)
        {
            string t = text.Trim();
            if (t.StartsWith("0x", StringComparison.OrdinalIgnoreCase)) return t[2..];
            if (t.StartsWith('$')) return t[1..];
            return t;
        }

        public static CheatFileEntry RamPoke(string space, int address, byte value, string description, bool enabled) => new()
        {
            Kind = "RamPoke",
            Space = space,
            Address = address.ToString("X", CultureInfo.InvariantCulture),
            Value = value.ToString("X2", CultureInfo.InvariantCulture),
            Description = description,
            Enabled = enabled,
        };

        public static CheatFileEntry RomPatch(int address, byte value, byte? compare, string description, bool enabled) => new()
        {
            Kind = "RomPatch",
            Address = address.ToString("X6", CultureInfo.InvariantCulture),
            Value = value.ToString("X2", CultureInfo.InvariantCulture),
            Compare = compare?.ToString("X2", CultureInfo.InvariantCulture),
            Description = description,
            Enabled = enabled,
        };
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

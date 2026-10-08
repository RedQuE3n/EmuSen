using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Models
{
    // One write inside a saved cheat, its addresses and values hex text; a carrier only, mapped onto CheatWrite by CheatRegistry - see EmuSen_Config_Reference.md §3.4.
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

            // Absent repeat fields are 0, not an error: files written by hand routinely omit them.
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

        // The fields of a file from before writes were a list: still read, and left null when saving - see EmuSen_Config_Reference.md §3.4.
        public string? Space { get; set; }
        public string? Address { get; set; }
        public string? Value { get; set; }

        public bool IsRomPatch => string.Equals(Kind, "RomPatch", StringComparison.OrdinalIgnoreCase);

        // Writes as written, or the single write an older file's flat Space/Address/Value fields describe.
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

        // True with a null result for "absent"; false only for text that was there and wasn't hex, which is a broken entry.
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

        // The name is typed by the player, so it is checked rather than sanitized: rewriting it would save to a file they didn't name.
        public static bool IsValidName(string name) =>
            GalaxiaNative.Active && GalaxiaNative.Crosses(name) ? GalaxiaNative.CheatNameValid(name) : Managed.IsValidName(name);

        public static ConfigFile<CheatFile> For(string name) =>
            new(CategoryDirName, name + ".json");

        // Any path the player picked, outside the auto-loaded set - see EmuSen_Settings_Reference.md §4.15.
        public static bool SaveTo(string path, CheatFile file) => GalaxiaNative.Active ? SaveToNative(path, file) : Managed.SaveTo(path, file);

        // Null for missing, unreadable or corrupt, matching ConfigFile.Load's own contract.
        public static CheatFile? LoadFrom(string path) => GalaxiaNative.Active ? LoadFromNative(path) : Managed.LoadFrom(path);

        public static string DirectoryPath => Path.Combine(ConfigStore.Directory, CategoryDirName);

        public static IReadOnlyList<string> ListNames() => GalaxiaNative.Active ? ListNamesNative() : Managed.ListNames();

        // The list is serialized here and written there; one that will not serialize still has its folder made, as before.
        internal static bool SaveToNative(string path, CheatFile file)
        {
            try
            {
                string? document;
                try { document = System.Text.Json.JsonSerializer.Serialize(file, ConfigJson.Options); }
                catch { document = null; }
                return GalaxiaNative.ModelSaveTo(GalaxiaModel.CheatFile, path, document);
            }
            catch
            {
                return false;
            }
        }

        internal static CheatFile? LoadFromNative(string path)
        {
            try
            {
                return GalaxiaNative.ModelLoadFrom(GalaxiaModel.CheatFile, path) is { } document
                    ? System.Text.Json.JsonSerializer.Deserialize<CheatFile>(document, ConfigJson.Options)
                    : null;
            }
            catch
            {
                return null;
            }
        }

        internal static IReadOnlyList<string> ListNamesNative()
        {
            try
            {
                return GalaxiaNative.CheatNames();
            }
            catch
            {
                return Array.Empty<string>();
            }
        }

        // The C# rules: the default, and what the library's are held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static bool IsValidName(string name) =>
                !string.IsNullOrWhiteSpace(name)
                && name.IndexOfAny(Path.GetInvalidFileNameChars()) < 0
                && name != "." && name != "..";

            public static bool SaveTo(string path, CheatFile file)
            {
                try
                {
                    Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(path))!);
                    File.WriteAllText(path, System.Text.Json.JsonSerializer.Serialize(file, ConfigJson.Options));
                    return true;
                }
                catch
                {
                    return false;
                }
            }

            public static CheatFile? LoadFrom(string path)
            {
                try
                {
                    return System.Text.Json.JsonSerializer.Deserialize<CheatFile>(File.ReadAllText(path), ConfigJson.Options);
                }
                catch
                {
                    return null;
                }
            }

            public static IReadOnlyList<string> ListNames()
            {
                try
                {
                    return Directory.EnumerateFiles(Path.Combine(ConfigStore.Managed.Directory, CategoryDirName), "*.json")
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
}

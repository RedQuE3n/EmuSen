using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Galaxia.Native;

namespace EmuSen.WiseMan.Bindings.Systems
{
    // The systems table, emusen-systems' data read through the platform library, each pack's entry with the library's code; there is no C# table since 2026-11-04 - see EmuSen_Backend.md §12, §40.1.
    public static unsafe class SystemTable
    {
        // Each data property of SystemEntry and the key the table gives it, which the schema test holds to both.
        public static readonly IReadOnlyList<(string Property, string Key)> Keys = new[]
        {
            (nameof(SystemEntry.Id), "id"), (nameof(SystemEntry.Name), "name"), (nameof(SystemEntry.Console), "console"),
            (nameof(SystemEntry.Manufacturer), "manufacturer"), (nameof(SystemEntry.ReleaseYear), "release_year"),
            (nameof(SystemEntry.Extensions), "extensions"), (nameof(SystemEntry.CheatSystems), "cheat_systems"),
            (nameof(SystemEntry.OpenVgdbSystems), "openvgdb_systems"), (nameof(SystemEntry.CoverAspect), "cover_aspect"),
            (nameof(SystemEntry.EsdeSystem), "esde_system"), (nameof(SystemEntry.EsdeFullName), "esde_full_name"),
            (nameof(SystemEntry.SelfDelimitingImages), "self_delimiting_images"), (nameof(SystemEntry.AttachedTo), "attached_to"),
            (nameof(SystemEntry.ScreenScraperSystem), "screenscraper_system"),
        };

        // The keys a row holds beyond SystemEntry's: the name a list shows, the firmware's words, the space its RAM cheats write into, its save media, byte order, libretro names, firmware note, rumble and look.
        public static readonly IReadOnlyList<string> PackKeys = new[] { "display_name", "firmware_name", "cheat_space", "media", "byte_order", "libretro_databases", "libretro_systemids", "firmware_note", "rumble", "look" };

        // A save medium as a row names it, by the system whose row it is in.
        public sealed record RowMedium(string System, string Id, string Title, string Noun);

        private sealed record Table(IReadOnlyList<SystemEntry>? Rows, string Report, string? Json, IReadOnlyList<RowMedium>? Media = null, IReadOnlyDictionary<string, string?>? CheatSpaces = null, IReadOnlyDictionary<string, string?>? ByteOrders = null, IReadOnlyDictionary<string, string?>? FirmwareNotes = null);

        private static readonly Lazy<Table> Loaded = new(Load);

        // Whether the library gave a table; without one no pack can be made, and the reason is Report.
        public static bool Ready => Loaded.Value.Rows is not null;

        public static string Report => Loaded.Value.Report;

        private static Table Required => Loaded.Value.Rows is not null ? Loaded.Value : throw new InvalidOperationException($"The systems table is emusen_platform's, and it gave none: {Loaded.Value.Report}");

        // The table's rows in its order, each with no code.
        public static IReadOnlyList<SystemEntry> Rows => Required.Rows!;

        // Each row's cheat space by its system's id.
        public static IReadOnlyDictionary<string, string?> CheatSpaces => Required.CheatSpaces!;

        // Each row's byte order by its system's id, big, little or null.
        public static IReadOnlyDictionary<string, string?> ByteOrders => Required.ByteOrders!;

        // Each row's firmware note by its system's id, null for most - see EmuSen_Cores.md §4.10.
        public static IReadOnlyDictionary<string, string?> FirmwareNotes => Required.FirmwareNotes!;

        // Every row's media in the table's order.
        public static IReadOnlyList<RowMedium> Media => Required.Media!;

        // The text the library gave, for a test that reads it as a person would.
        public static string Json => Required.Json!;

        // A pack's entry: the table's row for the id with the library's OpenVGDB rule, and its recogniser where the library has one for the system.
        public static SystemEntry Entry(string id)
        {
            SystemEntry row = Rows.FirstOrDefault(r => r.Id == id) ?? throw new InvalidOperationException($"The systems table has no row {id}");
            bool recognises = SystemCode.Code.Any(c => c.Id == id && c.Recognises);
            return row with { OpenVgdbBytes = file => SystemCode.OpenVgdbBytes(id, file), Recognise = recognises ? probe => SystemCode.Recognise(id, probe) : null };
        }

        // A pack's medium: the table's words for the id, the extension and the format module being the pack's (F28).
        public static SaveMedia.SaveMediumEntry Medium(string id, string extension, SaveMedia.ISaveMediumFormat format) =>
            Media.FirstOrDefault(m => m.Id == id) is { } row ? new SaveMedia.SaveMediumEntry(id, row.Title, row.Noun, extension, format) : throw new InvalidOperationException($"The systems table names no medium {id}");

        private static Table Load()
        {
            RetiredSwitch.Note("EMUSEN_SYSTEMS_NATIVE", "the systems table");
            try
            {
                if (!PlatformLibrary.Available) return new(null, PlatformLibrary.Report, null);
                nint export = PlatformLibrary.Export("emusen_systems_table");
                if (export == 0) return new(null, $"{PlatformLibrary.FileName} has no emusen_systems_table", null);
                if (SystemCode.Exports.FirstOrDefault(e => PlatformLibrary.Export(e) == 0) is { } missing) return new(null, $"{PlatformLibrary.FileName} has no {missing}", null);
                var call = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)export;
                long length = call(null, 0);
                if (length < 0) return new(null, $"{PlatformLibrary.FileName} gave no table: {SystemCode.LastError()}", null);
                byte[] bytes = new byte[length];
                fixed (byte* b = bytes) length = call(b, (nuint)bytes.Length);
                string json = Encoding.UTF8.GetString(bytes);
                return new(Parse(json), $"{PlatformLibrary.FileName}, the systems table", json, ParseMedia(json), ParseColumn(json, "cheat_space"), ParseColumn(json, "byte_order"), ParseColumn(json, "firmware_note"));
            }
            catch (Exception e) when (e is JsonException or InvalidOperationException or KeyNotFoundException or FormatException)
            {
                return new(null, $"{PlatformLibrary.FileName} gave a table this build cannot read: {e.Message}", null);
            }
        }

        // The rows as entries; a code part stands in as the identity until Entry gives the library's.
        private static IReadOnlyList<SystemEntry> Parse(string json)
        {
            using var document = JsonDocument.Parse(json);
            return document.RootElement.EnumerateArray().Select(r => new SystemEntry(
                Text(r, "id")!, Text(r, "name")!, Text(r, "console")!, Text(r, "manufacturer")!, r.GetProperty("release_year").GetInt32(),
                Words(r, "extensions"), Words(r, "cheat_systems"), Words(r, "openvgdb_systems"), file => file, r.GetProperty("cover_aspect").GetDouble(),
                Text(r, "esde_system")!, Text(r, "esde_full_name")!,
                SelfDelimitingImages: r.GetProperty("self_delimiting_images").GetBoolean(),
                AttachedTo: Text(r, "attached_to"),
                ScreenScraperSystem: r.GetProperty("screenscraper_system").ValueKind == JsonValueKind.Null ? null : r.GetProperty("screenscraper_system").GetInt32())).ToArray();
        }

        private static IReadOnlyList<RowMedium> ParseMedia(string json)
        {
            using var document = JsonDocument.Parse(json);
            return document.RootElement.EnumerateArray().SelectMany(r => r.GetProperty("media").EnumerateArray().Select(m =>
                new RowMedium(Text(r, "id")!, Text(m, "id")!, Text(m, "title")!, Text(m, "noun")!))).ToArray();
        }

        // One optional text column of every row, by the row's id.
        private static IReadOnlyDictionary<string, string?> ParseColumn(string json, string key)
        {
            using var document = JsonDocument.Parse(json);
            return document.RootElement.EnumerateArray().ToDictionary(r => Text(r, "id")!, r => Text(r, key));
        }

        private static string? Text(JsonElement row, string key) => row.GetProperty(key).ValueKind == JsonValueKind.Null ? null : row.GetProperty(key).GetString();

        private static string[] Words(JsonElement row, string key) => row.GetProperty(key).EnumerateArray().Select(w => w.GetString()!).ToArray();
    }
}

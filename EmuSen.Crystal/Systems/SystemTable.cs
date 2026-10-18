using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Systems
{
    // The systems table's facade: each pack's entry from emusen-systems' data, its code kept from the C# literal, which EMUSEN_SYSTEMS_NATIVE=0 keeps whole - see EmuSen_Backend.md §12.
    public static unsafe class SystemTable
    {
        public const string Variable = "EMUSEN_SYSTEMS_NATIVE";

        // Each data property of SystemEntry and the key the table gives it, which the parity class holds to both sides.
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

        // The keys a row holds beyond SystemEntry's, read into the pack's other parts: the space its RAM cheats write into, and its save media.
        public static readonly IReadOnlyList<string> PackKeys = new[] { "cheat_space", "media", "byte_order" };

        // A save medium as a row names it, by the system whose row it is in.
        public sealed record RowMedium(string System, string Id, string Title, string Noun);

        private sealed record Table(IReadOnlyList<SystemEntry>? Rows, string Report, string? Json, IReadOnlyList<RowMedium>? Media = null, IReadOnlyDictionary<string, string?>? CheatSpaces = null, IReadOnlyDictionary<string, string?>? ByteOrders = null);

        private static readonly Lazy<Table> Loaded = new(Load);

        private static readonly PlatformSwitch Chosen = new(() => Environment.GetEnvironmentVariable(Variable), () => (Loaded.Value.Rows is not null, Loaded.Value.Report), PlatformSwitch.Announce, "the systems table");

        // State 2: the library's table unless the variable is 0; a library that is absent, refused or unreadable leaves the literals, said once.
        public static bool Active => Chosen.Active;

        // The library's table itself, whichever the switch chose: what the parity class reads.
        public static bool Ready => Loaded.Value.Rows is not null;

        public static string Report => Loaded.Value.Report;

        // The table's rows in its order, each with no code; empty where the library is not in use.
        public static IReadOnlyList<SystemEntry> Rows => Loaded.Value.Rows ?? Array.Empty<SystemEntry>();

        // Each row's cheat space by its system's id; empty where the library is not in use.
        public static IReadOnlyDictionary<string, string?> CheatSpaces => Loaded.Value.CheatSpaces ?? new Dictionary<string, string?>();

        // Each row's byte order by its system's id, big, little or null; empty where the library is not in use.
        public static IReadOnlyDictionary<string, string?> ByteOrders => Loaded.Value.ByteOrders ?? new Dictionary<string, string?>();

        // Every row's media in the table's order; empty where the library is not in use.
        public static IReadOnlyList<RowMedium> Media => Loaded.Value.Media ?? Array.Empty<RowMedium>();

        // A pack's medium: the table's words for the literal's id with the literal's extension and format, or the literal itself under the switch's 0 or with no such medium.
        public static SaveMedia.SaveMediumEntry Medium(SaveMedia.SaveMediumEntry literal) =>
            Active && Media.FirstOrDefault(m => m.Id == literal.Id) is { } row ? literal with { Title = row.Title, Noun = row.Noun } : literal;

        // The text the library gave, for a test that reads it as a person would.
        public static string? Json => Loaded.Value.Json;

        // A pack's entry: the table's row for the literal's id with the library's code where the literal has code of that kind, or the literal itself under the switch's 0 or with no such row.
        public static SystemEntry Entry(SystemEntry literal) => Active && FromTable(literal) is { } row
            ? row with { OpenVgdbBytes = file => SystemCode.OpenVgdbBytes(literal.Id, file), Recognise = literal.Recognise is null ? null : probe => SystemCode.Recognise(literal.Id, probe) }
            : literal;

        // The table's row for the literal's id with its recogniser and OpenVGDB bytes, whichever the switch chose; null where there is none.
        public static SystemEntry? FromTable(SystemEntry literal) =>
            Rows.FirstOrDefault(r => r.Id == literal.Id) is { } row ? row with { OpenVgdbBytes = literal.OpenVgdbBytes, Recognise = literal.Recognise } : null;

        private static Table Load()
        {
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
                return new(Parse(json), $"{PlatformLibrary.FileName}, the systems table", json, ParseMedia(json), ParseCheatSpaces(json), ParseByteOrders(json));
            }
            catch (Exception e) when (e is JsonException or InvalidOperationException or KeyNotFoundException or FormatException)
            {
                return new(null, $"{PlatformLibrary.FileName} gave a table this build cannot read: {e.Message}", null);
            }
        }

        // The rows as entries; a code part stands in as the identity until FromTable gives a literal's.
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

        private static IReadOnlyDictionary<string, string?> ParseCheatSpaces(string json)
        {
            using var document = JsonDocument.Parse(json);
            return document.RootElement.EnumerateArray().ToDictionary(r => Text(r, "id")!, r => Text(r, "cheat_space"));
        }

        private static IReadOnlyDictionary<string, string?> ParseByteOrders(string json)
        {
            using var document = JsonDocument.Parse(json);
            return document.RootElement.EnumerateArray().ToDictionary(r => Text(r, "id")!, r => Text(r, "byte_order"));
        }

        private static string? Text(JsonElement row, string key) => row.GetProperty(key).ValueKind == JsonValueKind.Null ? null : row.GetProperty(key).GetString();

        private static string[] Words(JsonElement row, string key) => row.GetProperty(key).EnumerateArray().Select(w => w.GetString()!).ToArray();
    }
}

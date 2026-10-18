using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress.Library
{
    // A console's core as the library shows it: what the runtime's shelves and systems say of it, joined.
    public sealed record ConsoleCore(string DisplayName, string Console, IReadOnlyList<string> Extensions, IReadOnlyList<string> Systems,
        IReadOnlyList<string> CheatSystems, IReadOnlyList<string> OpenVgdbSystems, double? CoverAspect)
    {
        public bool SupportsExtension(string extension) => Extensions.Contains(extension, StringComparer.OrdinalIgnoreCase);
    }

    // A shelf of the library: one system of a console's core, named for the core.
    public sealed record LibraryShelf(string Name, string Label, string System, string EsdeSystem, string EsdeFullName, ConsoleCore Core);

    // A console's pad, its most ports and each player's controller as the console's settings stand.
    public sealed record PadFacts(IReadOnlyList<PadButton> Buttons, IReadOnlyList<PadAxis> Axes, int Ports, IReadOnlyList<string?> Players)
    {
        public IReadOnlyList<PadControl> Controls => PadControls.For(Buttons, Axes);
    }

    // A system as the runtime's systems answer gives it.
    public sealed record SystemFacts(string Id, string Name, string Console, string Manufacturer, int ReleaseYear, string EsdeSystem, int? ScreenScraperSystem,
        string? AttachedTo, bool StartsEmpty, IReadOnlyList<string> Engines, IReadOnlyList<string> CheatSystems, IReadOnlyList<string> OpenVgdbSystems, double? CoverAspect, PadFacts Pad);

    // A row of a console's settings schema, its note as the stored values stand, and the row it is shown with.
    public sealed record SettingRow(string Key, string Label, string Hint, string Kind, string Default, int Min, int Max, IReadOnlyList<string> Choices,
        IReadOnlyList<string> ChoiceLabels, string? Group, string? Note, string? ShownWithKey, string? ShownWithValue, bool HasNote = false)
    {
        public const string Switch = "switch", Count = "count", Choice = "choice";
    }

    // What opens a file, as the runtime's route answers.
    public sealed record FileRoute(string? System, string? Console, string? Engine, string? Shelf, string? Reason);

    // The catalog's facts as the frontend API answers them - the shelves, the systems and a file's route - read once and kept until forgotten - see EmuSen_Backend.md §14.13.
    public static class RuntimeCatalog
    {
        private sealed record Facts(IReadOnlyList<LibraryShelf> Shelves, IReadOnlyList<ConsoleCore> Consoles, IReadOnlyList<SystemFacts> Systems);

        private static Facts? _facts;
        private static readonly object Gate = new();
        private static readonly ConcurrentDictionary<(string Path, long Bytes, long Ticks), FileRoute> Routes = new();

        // The settings key the engine a game opens on is stored under, as the runtime's schema names it.
        public const string EngineKey = "Engine";

        // The no-filter choice, spelled once in Galaxia because that is what persists it.
        public static string AllConsoles => AppSettings.AllConsoles;

        // Read again on the next ask: engines found again, or a library scanned again.
        public static void Forget()
        {
            lock (Gate) _facts = null;
            Routes.Clear();
            EngineChoicesByConsole.Clear();
        }

        private static Facts Current
        {
            get
            {
                lock (Gate) return _facts ??= Read();
            }
        }

        private static IReadOnlyList<string> Strings(JsonElement e, string name) =>
            e.TryGetProperty(name, out JsonElement a) && a.ValueKind == JsonValueKind.Array ? a.EnumerateArray().Select(s => s.GetString() ?? "").ToArray() : Array.Empty<string>();

        private static string? Text(JsonElement e, string name) =>
            e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        private static Facts Read()
        {
            using JsonDocument systemsDoc = JsonDocument.Parse(CoreSettings.Runtime.Systems());
            var systems = new List<SystemFacts>();
            foreach (JsonElement s in systemsDoc.RootElement.EnumerateArray())
            {
                JsonElement pad = s.GetProperty("controllers");
                var facts = new PadFacts(
                    pad.GetProperty("buttons").EnumerateArray().Select(b => (PadButton)b.GetInt32()).ToArray(),
                    pad.GetProperty("axes").EnumerateArray().Select(a => (PadAxis)a.GetInt32()).ToArray(),
                    pad.GetProperty("ports").GetInt32(),
                    pad.GetProperty("players").EnumerateArray().Select(p => p.ValueKind == JsonValueKind.String ? p.GetString() : null).ToArray());
                string id = s.GetProperty("id").GetString()!;
                string? coreConsole = s.TryGetProperty("core", out JsonElement core) && core.ValueKind == JsonValueKind.Object ? Text(core, "console") : null;
                systems.Add(new SystemFacts(id, Text(s, "name") ?? id, Text(s, "console") ?? coreConsole ?? id,
                    Text(s, "manufacturer") ?? "", s.TryGetProperty("release_year", out JsonElement year) ? year.GetInt32() : 0, Text(s, "esde_system") ?? "",
                    s.TryGetProperty("screenscraper_system", out JsonElement ss) && ss.ValueKind == JsonValueKind.Number ? ss.GetInt32() : null,
                    Text(s, "attached_to"), s.GetProperty("starts_empty").GetBoolean(), Strings(s, "engines"), Strings(s, "cheat_systems"), Strings(s, "openvgdb_systems"),
                    s.TryGetProperty("cover_aspect", out JsonElement aspect) ? aspect.GetDouble() : null, facts));
            }

            using JsonDocument shelvesDoc = JsonDocument.Parse(CoreSettings.Runtime.Shelves());
            var cores = new Dictionary<string, ConsoleCore>(StringComparer.OrdinalIgnoreCase);
            var order = new List<string>();
            var rows = shelvesDoc.RootElement.EnumerateArray().Select(s => (Shelf: s, Core: s.GetProperty("core"))).ToArray();
            foreach (var group in rows.GroupBy(r => r.Core.GetProperty("display_name").GetString()!, StringComparer.OrdinalIgnoreCase))
            {
                JsonElement core = group.First().Core;
                string[] ids = group.Select(r => r.Shelf.GetProperty("system").GetString()!).ToArray();
                SystemFacts[] own = ids.Select(i => systems.FirstOrDefault(s => s.Id == i)).OfType<SystemFacts>().ToArray();
                cores[group.Key] = new ConsoleCore(group.Key, core.GetProperty("console").GetString()!, Strings(core, "extensions"), ids,
                    own.SelectMany(s => s.CheatSystems).Distinct().ToArray(), own.SelectMany(s => s.OpenVgdbSystems).Distinct().ToArray(), own.FirstOrDefault()?.CoverAspect);
                order.Add(group.Key);
            }
            LibraryShelf[] shelves = rows.Select(r => new LibraryShelf(r.Shelf.GetProperty("name").GetString()!, r.Shelf.GetProperty("label").GetString()!, r.Shelf.GetProperty("system").GetString()!,
                r.Shelf.GetProperty("esde_system").GetString()!, r.Shelf.GetProperty("esde_full_name").GetString()!, cores[r.Core.GetProperty("display_name").GetString()!])).ToArray();
            return new Facts(shelves, order.Select(n => cores[n]).ToArray(), systems);
        }

        // The library's shelves in release order.
        public static IReadOnlyList<LibraryShelf> Shelves => Current.Shelves;

        // One entry a console's core, in release order: what a "which console?" list shows.
        public static IReadOnlyList<ConsoleCore> Consoles => Current.Consoles;

        public static IReadOnlyList<SystemFacts> Systems => Current.Systems;

        // Every console's name, in release order.
        public static IEnumerable<string> ConsoleNames => Consoles.Select(c => c.Console);

        // Null for AllConsoles or a name no shelf has.
        public static LibraryShelf? ShelfByName(string? name) =>
            Shelves.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase));

        // Null for AllConsoles, an unknown name, or a name from a build that had a core this one lacks.
        public static ConsoleCore? ByDisplayName(string? displayName) =>
            Consoles.FirstOrDefault(c => string.Equals(c.DisplayName, displayName, StringComparison.OrdinalIgnoreCase));

        // Either name a console goes by: "SNES (Venus)" or "SNES".
        public static ConsoleCore? ByAnyName(string? name) =>
            ByDisplayName(name) ?? Consoles.FirstOrDefault(c => string.Equals(c.Console, name, StringComparison.OrdinalIgnoreCase));

        // A system by the console that is its name, the first of a console's systems for a core of several.
        public static SystemFacts? SystemOf(string? console) =>
            console is null ? null : Systems.FirstOrDefault(s => string.Equals(s.Console, console, StringComparison.OrdinalIgnoreCase));

        // The console a system of the table is, by its id.
        public static string? ConsoleOfSystem(string id) => Systems.FirstOrDefault(s => s.Id == id)?.Console;

        // The console whose screen and pads a console uses: an attachment's host, else the console itself.
        public static string HostOf(string console) => SystemOf(console)?.AttachedTo ?? console;

        // The library's console filter: every console, then each shelf in release order.
        public static IReadOnlyList<string> FilterChoices => new[] { AllConsoles }.Concat(Shelves.Select(s => s.Name)).ToArray();

        // Every extension a console's core takes, for the file picker and the scan.
        public static IReadOnlyList<string> RomExtensions => Consoles.SelectMany(c => c.Extensions).Distinct(StringComparer.OrdinalIgnoreCase).ToArray();

        // The cheat database's folders of every system an engine of this build runs, the ones a prune keeps.
        public static IReadOnlyCollection<string> SupportedCheatSystems => Systems.Where(s => s.Engines.Count > 0).SelectMany(s => s.CheatSystems).ToHashSet(StringComparer.OrdinalIgnoreCase);

        // The consoles an engine of this build starts with nothing in it, in release order.
        public static IReadOnlyList<string> ConsolesThatStartEmpty =>
            Systems.Where(s => s.StartsEmpty).OrderBy(s => s.Manufacturer, StringComparer.OrdinalIgnoreCase).ThenBy(s => s.ReleaseYear).Select(s => s.Console).Distinct(StringComparer.OrdinalIgnoreCase).ToArray();

        public static bool StartsEmpty(string? console) => console is not null && ConsolesThatStartEmpty.Contains(console, StringComparer.OrdinalIgnoreCase);

        // A console's pad as its engine reads it; every button and no stick for a console this build does not know.
        public static PadFacts PadOf(string? console) =>
            SystemOf(console)?.Pad ?? new PadFacts(Enum.GetValues<PadButton>(), Array.Empty<PadAxis>(), 1, new string?[1]);

        // The same, read afresh, for each player's controller as the console's settings stand now.
        public static PadFacts PadNow(string? console)
        {
            lock (Gate) _facts = null;
            return PadOf(console);
        }

        public static IReadOnlyList<PadControl> ControlsFor(string console) => PadOf(console).Controls;

        public static IReadOnlyList<PadAxis> AxesFor(string console) => PadOf(console).Axes;

        // A console's settings as its schema stands now, the engine's row first where it has one; read afresh, since a row's note follows the stored values.
        public static IReadOnlyList<SettingRow> SettingsFor(string console)
        {
            using JsonDocument schema = JsonDocument.Parse(CoreSettings.Runtime.SettingsSchema(console));
            return schema.RootElement.EnumerateArray().Select(r =>
            {
                JsonElement with = r.TryGetProperty("shown_with", out JsonElement w) && w.ValueKind == JsonValueKind.Object ? w : default;
                return new SettingRow(r.GetProperty("key").GetString()!, r.GetProperty("label").GetString()!, r.GetProperty("hint").GetString()!, r.GetProperty("kind").GetString()!,
                    r.GetProperty("default").GetString()!, r.GetProperty("min").GetInt32(), r.GetProperty("max").GetInt32(), Strings(r, "choices"), Strings(r, "choice_labels"),
                    Text(r, "group"), Text(r, "note"), with.ValueKind == JsonValueKind.Object ? Text(with, "key") : null, with.ValueKind == JsonValueKind.Object ? Text(with, "value") : null,
                    r.TryGetProperty("note", out _));
            }).ToArray();
        }

        // The engines a console offers a choice of, where it offers more than one; empty where one engine runs it.
        public static IReadOnlyList<string> EngineChoices(string console) =>
            EngineChoicesByConsole.GetOrAdd(console, c => SettingsFor(c).FirstOrDefault(r => r.Key == EngineKey)?.Choices is { Count: > 1 } choices ? choices : Array.Empty<string>());

        private static readonly ConcurrentDictionary<string, IReadOnlyList<string>> EngineChoicesByConsole = new(StringComparer.OrdinalIgnoreCase);

        // What opens a file, asked once for each version of it until forgotten.
        public static FileRoute Route(string path)
        {
            var key = Stamp(path);
            return Routes.GetOrAdd(key, _ =>
            {
                using JsonDocument route = JsonDocument.Parse(CoreSettings.Runtime.Route(path));
                JsonElement r = route.RootElement;
                return new FileRoute(Text(r, "system"), Text(r, "console"), Text(r, "engine"), Text(r, "shelf"), Text(r, "reason"));
            });
        }

        private static (string, long, long) Stamp(string path)
        {
            try
            {
                var file = new FileInfo(path);
                return file.Exists ? (file.FullName, file.Length, file.LastWriteTimeUtc.Ticks) : (path, -1, 0);
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException or ArgumentException)
            {
                return (path, -1, 0);
            }
        }

        // The console a file runs as, answerable before anything runs; null where none does.
        public static string? ConsoleForRom(string path) => Route(path).Console;

        // The shelf a file sits on, or null for a file no core claims.
        public static string? ShelfFor(string path) => Route(path).Shelf;

        // The core that runs a file's console, or null.
        public static ConsoleCore? CoreForRom(string path) => ShelfByName(ShelfFor(path))?.Core;
    }
}

using System.Text.Json;

namespace EmuSen.Tools.Cores
{
    // One core as Cores lists it, joined from the core tool's listing and the runtime's - see EmuSen_Cores.md §4.1.
    public sealed record CoreRow
    {
        public const string Project = "EmuSen", Libretro = "libretro", Added = "added by you";

        // A libretro core's file stem, or a runtime core's engine name; it names the row across refreshes.
        public required string Key { get; init; }
        public required string Name { get; init; }
        public required string Source { get; init; }
        // The core tool's state word (installed, update, available, unsupported, added), or for a runtime core "built in" or its trust.
        public required string State { get; init; }
        public IReadOnlyList<string> Systems { get; init; } = [];
        // The consoles its systems belong to, as the runtime names them.
        public IReadOnlyList<string> Consoles { get; init; } = [];
        public string Version { get; init; } = "";
        public string? Reason { get; init; }
        public string Licence { get; init; } = "";
        public IReadOnlyList<string> Authors { get; init; } = [];
        public long? Size { get; init; }
        public string? Stem { get; init; }
        // The runtime's engine name, which Make Default and Options… need; none for a libretro core discovery does not list yet.
        public string? Engine { get; init; }
        public string? Library { get; init; }
        // The hash an approval names: the library's, or for an installed libretro core its core file's, as the runtime's listing gives it.
        public string? Sha256 { get; init; }
        // The runtime's trust of the library: approved, manifest, needs_approval, not_read; none where the runtime does not list it.
        public string? Trust { get; init; }
        // Whether it is the engine of one of its consoles.
        public bool IsDefault { get; init; }

        public bool IsLibretro => Source == Libretro;

        public bool Installed => IsLibretro ? State is "installed" or "update" : Library is not null;

        public bool AwaitsApproval => Trust == "needs_approval";

        // A row of the core tool's `core` event (EmuSen_Cores.md §6.1).
        public static CoreRow? OfTool(JsonElement e, IReadOnlyDictionary<string, string> consoleOf)
        {
            if (Text(e, "event") != "core" || Text(e, "stem") is not { } stem) return null;
            IReadOnlyList<string> systems = Strings(e, "systems");
            return new CoreRow
            {
                Key = "libretro:" + stem,
                Stem = stem,
                Name = Text(e, "name") ?? stem,
                Source = Text(e, "state") == "added" ? Added : Libretro,
                State = Text(e, "state") ?? "",
                Reason = Text(e, "reason"),
                Systems = systems,
                Consoles = systems.Select(s => consoleOf.TryGetValue(s, out string? c) ? c : s).Distinct().ToArray(),
                Version = string.Join(" ", new[] { Text(e, "version"), Text(e, "build") }.Where(v => !string.IsNullOrEmpty(v))),
                Licence = Text(e, "license") ?? "",
                Authors = Strings(e, "authors"),
                Size = e.TryGetProperty("size", out JsonElement s) && s.ValueKind == JsonValueKind.Number && s.TryGetInt64(out long n) ? n : null,
            };
        }

        // A row of the runtime's `cores` listing, a core library discovery found (EmuSen_Backend.md §29.40.5).
        public static CoreRow? OfRuntime(JsonElement c, IReadOnlyDictionary<string, string> consoleOf, Func<string, string, bool> isDefault)
        {
            if (Text(c, "library") is not { } library) return null;
            string engine = Text(c, "engine") ?? "";
            bool shipped = Text(c, "directory") == "installation";
            IReadOnlyList<string> systems = Strings(c, "systems");
            string[] consoles = systems.Select(s => consoleOf.TryGetValue(s, out string? name) ? name : s).Distinct().ToArray();
            string trust = Text(c, "trust") ?? "";
            return new CoreRow
            {
                Key = engine.Length > 0 ? "engine:" + engine : "library:" + library,
                Name = Text(c, "name") ?? Path.GetFileName(library),
                Source = shipped ? Project : Added,
                State = shipped ? "built in" : trust,
                Systems = systems,
                Consoles = consoles,
                Licence = Text(c, "license") ?? "",
                Authors = Strings(c, "authors"),
                Size = File.Exists(library) ? new FileInfo(library).Length : null,
                Engine = engine.Length > 0 ? engine : null,
                Library = library,
                Sha256 = c.TryGetProperty("libretro", out JsonElement l) && l.ValueKind == JsonValueKind.Object ? Text(l, "sha256") : Text(c, "sha256"),
                Trust = trust,
                IsDefault = engine.Length > 0 && consoles.Any(console => isDefault(console, engine)),
            };
        }

        internal static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        internal static IReadOnlyList<string> Strings(JsonElement e, string name) =>
            e.TryGetProperty(name, out JsonElement a) && a.ValueKind == JsonValueKind.Array ? a.EnumerateArray().Where(x => x.ValueKind == JsonValueKind.String).Select(x => x.GetString()!).ToArray() : [];
    }
}

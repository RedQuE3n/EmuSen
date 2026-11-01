using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;

namespace EmuSen.Tools.Scraper
{
    // One game as `scrape ls` chooses it: its path, ES-DE system, package, and each asked kind's source or none - see EmuSen_LibraryTools.md §2 and §9.
    public sealed record ScrapeRow
    {
        public required string Path { get; init; }

        public required string System { get; init; }

        public string? Package { get; init; }

        // Each kind the run asks of the game's system, in the module's order, with the scraper's source word or null where it has none.
        public IReadOnlyList<(string Kind, string? Source)> Media { get; init; } = [];

        public string Title => global::System.IO.Path.GetFileNameWithoutExtension(Path);

        public int Have => Media.Count(m => m.Source is not null);

        public IReadOnlyList<string> Missing => Media.Where(m => m.Source is null).Select(m => m.Kind).ToList();

        // A game of `ls`'s answer as a row; null for one without a path or system.
        public static ScrapeRow? Of(JsonElement g)
        {
            if (Text(g, "path") is not { Length: > 0 } path || Text(g, "system") is not { Length: > 0 } system) return null;
            var media = new List<(string, string?)>();
            if (g.TryGetProperty("media", out JsonElement m) && m.ValueKind == JsonValueKind.Object)
                foreach (JsonProperty k in m.EnumerateObject()) media.Add((k.Name, k.Value.ValueKind == JsonValueKind.String ? k.Value.GetString() : null));
            return new ScrapeRow { Path = path, System = system, Package = Text(g, "package"), Media = media };
        }

        // Every game of an `ls` answer, and its notes.
        public static (List<ScrapeRow> Rows, List<string> Notes) Listing(string answer)
        {
            using JsonDocument doc = JsonDocument.Parse(answer);
            var rows = doc.RootElement.TryGetProperty("games", out JsonElement games) && games.ValueKind == JsonValueKind.Array
                ? games.EnumerateArray().Select(Of).OfType<ScrapeRow>().ToList() : [];
            return (rows, Notes(doc.RootElement));
        }

        internal static List<string> Notes(JsonElement e) =>
            e.TryGetProperty("notes", out JsonElement n) && n.ValueKind == JsonValueKind.Array ? n.EnumerateArray().Select(x => x.GetString() ?? "").Where(x => x.Length > 0).ToList() : [];

        internal static string? Text(JsonElement e, string name) =>
            e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;
    }
}

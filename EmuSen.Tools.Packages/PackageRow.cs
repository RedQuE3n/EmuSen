using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using static EmuSen.Tools.Packages.ToolEvents;

namespace EmuSen.Tools.Packages
{
    // One game of the library as Packages lists it: packaged or not, from one event of `pkg ls --all` - see EmuSen_Packages.md §3 and §7.
    public sealed record PackageRow
    {
        // What names the row across refreshes: the package's folder under Packages, else the game's path in the library.
        public required string Key { get; init; }

        public required string Title { get; init; }

        public required string Console { get; init; }

        // The game's path in the library with / between its parts, null where its game is not there.
        public string? Source { get; init; }

        // The package's folder under Packages, null for a game not yet packaged.
        public string? Package { get; init; }

        // linked or self-contained; empty for a game not yet packaged.
        public string Mode { get; init; } = "";

        // Where the manifest's game lies: library, package, compressed or absolute.
        public string Where { get; init; } = "";

        // The tool's state: ok, changed, moved, missing, refused, or unpackaged.
        public required string State { get; init; }

        public string? Words { get; init; }

        // The game's bytes where it lies, and the package folder's own.
        public long Size { get; init; }

        public long PackageSize { get; init; }

        // Whether chd convert takes the game: a CD system's disc, not yet a CHD.
        public bool Convertible { get; init; }

        public IReadOnlyList<string> Patches { get; init; } = [];

        public string Identity { get; init; } = "original";

        // The game's file as it is named, for its format.
        public string File { get; init; } = "";

        public bool Packaged => Package is not null;

        public bool Problem => State is "changed" or "moved" or "missing" or "refused" or "failed";

        // The disc's or cartridge's form as the Format column says it (§3a).
        public string Format => Path.GetExtension(File).ToLowerInvariant() switch
        {
            ".cue" => "CUE/BIN",
            ".bin" => "BIN",
            ".iso" => "ISO",
            ".chd" => "CHD",
            ".gdi" => "GDI",
            ".m3u" => "M3U",
            "" => "",
            string e => e[1..].ToUpperInvariant(),
        };

        // A title from a file's name: the name less its extension, the package folder's for a package.
        private static string TitleOf(string path) => Path.GetFileNameWithoutExtension(path.TrimEnd('/'));

        // The row an event of the listing names, null for an event that names none.
        public static PackageRow? Of(JsonElement e)
        {
            string? kind = Text(e, "event");
            if (kind == "game" && Text(e, "source") is { } source)
                return new PackageRow
                {
                    Key = "library:" + source,
                    Title = TitleOf(source),
                    Console = Text(e, "system") ?? "",
                    Source = source,
                    State = Text(e, "state") ?? "unpackaged",
                    Size = Number(e, "size"),
                    Convertible = e.TryGetProperty("convertible", out JsonElement c) && c.ValueKind == JsonValueKind.True,
                    File = source,
                };
            if (kind != "package" || Text(e, "package") is not { } package) return null;
            string where = Text(e, "where") ?? "";
            string game = Text(e, "game") ?? "";
            return new PackageRow
            {
                Key = "package:" + package,
                Title = TitleOf(package),
                Console = Text(e, "console") ?? Text(e, "system") ?? "",
                Source = where == "library" && game.Length > 0 ? game : null,
                Package = package,
                Mode = Text(e, "mode") ?? "",
                Where = where,
                State = Text(e, "state") ?? "",
                Words = Text(e, "words") ?? (Text(e, "to") is { } to ? $"found again at {to}" : null),
                Size = Number(e, "game_size"),
                PackageSize = Number(e, "size"),
                Convertible = e.TryGetProperty("convertible", out JsonElement cv) && cv.ValueKind == JsonValueKind.True,
                Patches = e.TryGetProperty("patches", out JsonElement p) && p.ValueKind == JsonValueKind.Array ? p.EnumerateArray().Select(x => x.GetString() ?? "").ToList() : [],
                Identity = Text(e, "identity") ?? "original",
                File = game.Length > 0 ? game : package,
            };
        }
    }
}

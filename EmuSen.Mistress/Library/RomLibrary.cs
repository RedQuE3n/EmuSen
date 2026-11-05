using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.Mistress.Library
{
    // Why a ROM directory can produce no list - see EmuSen_Settings_Reference.md §4.11.
    public enum RomLibraryStatus
    {
        NoDirectoryConfigured,
        DirectoryNotFound,
        Empty,
        Ok
    }

    // Title is the filename without extension; FileName keeps it.
    public sealed record RomEntry(string FullPath)
    {
        public string FileName => Path.GetFileName(FullPath);
        public string Title => Path.GetFileNameWithoutExtension(FullPath);

        // The shelf the library's scan routed the file to, so it is not asked again; an entry is still its path alone - see EmuSen_Backend.md §28.5.
        internal string? KnownShelf { get; init; }

        // Which console this file belongs to, by extension and, where several take it, by content - see EmuSen_Multicore.md §10.
        public string CoreDisplayName => KnownShelf is { } known
            ? RuntimeCatalog.ShelfByName(known)?.Core.DisplayName ?? "Unknown"
            : RuntimeCatalog.CoreForRom(FullPath)?.DisplayName ?? "Unknown";

        // The library shelf, which splits the Game Boy's core into Game Boy and Game Boy Color - see EmuSen_Settings_Reference.md §4.46.
        public string Shelf => KnownShelf ?? RuntimeCatalog.ShelfFor(FullPath) ?? "Unknown";

        // The folder of the package this is the game of, null for a file no package claims - see EmuSen_GamePackages.md §3.6.
        public string? Package { get; init; }

        public bool Equals(RomEntry? other) => other is not null && FullPath == other.FullPath;

        public override int GetHashCode() => FullPath.GetHashCode();
    }

    // CoreDisplayName is the console the scan was narrowed to, null when it was not.
    public sealed record RomLibraryResult(RomLibraryStatus Status, string? Directory, IReadOnlyList<RomEntry> Entries,
        string? CoreDisplayName = null);

    // Enumerates playable ROMs in a directory, with no Avalonia dependency
    // so it stays testable headlessly - see EmuSen_Settings_Reference.md §4.11.
    public static class RomLibrary
    {
        // Whatever cores this build has, not a second copy of the list, and the archives a game may be packed in - see EmuSen_Multicore.md §3.
        public static string[] Extensions => RuntimeCatalog.RomExtensions.Concat(EmuSen.Galaxia.Media.ArchiveImages.Extensions).ToArray();

        private static readonly IReadOnlyList<RomEntry> None = Array.Empty<RomEntry>();

        // The folders a scan walks beside the library since game packages - see EmuSen_GamePackages.md §3.6.
        public sealed record Beside(string? Packages, string? Compressed)
        {
            public static Beside None { get; } = new(null, null);

            public static Beside Tree => new(EmuSen.Galaxia.Library.DataStore.Packages, EmuSen.Galaxia.Library.DataStore.Compressed);
        }

        // <coreDisplayName> narrows to one console; null or RuntimeCatalog.AllConsoles keeps everything; the tree's Packages and Compressed beside it.
        public static RomLibraryResult Scan(string? romDirectory, string? coreDisplayName = null) => Scan(romDirectory, Beside.Tree, coreDisplayName);

        // The library's scan, on a database of its own that lives for the call - see EmuSen_Backend.md §28.5.
        public static RomLibraryResult Scan(string? romDirectory, Beside beside, string? coreDisplayName = null)
        {
            using var lib = EmuSen.Crystal.Client.Library.GameLibrary.Open(":memory:");
            return ScanWith(lib, romDirectory, coreDisplayName, beside);
        }

        // A scan through a handle, kept there for the reclaim and identity that follow it.
        internal static RomLibraryResult ScanWith(EmuSen.Crystal.Client.Library.GameLibrary lib, string? romDirectory, string? coreDisplayName = null, Beside? beside = null)
        {
            var stream = new MemoryStream();
            using (var w = new System.Text.Json.Utf8JsonWriter(stream))
            {
                w.WriteStartObject();
                w.WriteStartArray("roots");
                w.WriteStringValue(romDirectory ?? "");
                w.WriteEndArray();
                w.WriteStartArray("systems");
                foreach (string system in RuntimeCatalog.Shelves.Select(s => s.System).Distinct()) w.WriteStringValue(system);
                w.WriteEndArray();
                if (beside?.Packages is { } packages) w.WriteString("packages", packages);
                if (beside?.Compressed is { } compressed) w.WriteString("compressed", compressed);
                w.WriteEndObject();
            }
            lib.Scan(System.Text.Encoding.UTF8.GetString(stream.ToArray()));
            using var games = System.Text.Json.JsonDocument.Parse(lib.Games(""));
            var shelves = RuntimeCatalog.Shelves.GroupBy(s => s.System).ToDictionary(g => g.Key, g => g.First().Name);
            var entries = games.RootElement.GetProperty("games").EnumerateArray()
                .Select(g => new RomEntry(g.GetProperty("path").GetString()!)
                {
                    KnownShelf = g.GetProperty("system").GetString() is { } system && shelves.TryGetValue(system, out string? shelf) ? shelf : "Unknown",
                    Package = g.TryGetProperty("package", out var package) ? package.GetString() : null,
                })
                .ToList();
            RomLibraryStatus status = games.RootElement.GetProperty("status").GetString() switch
            {
                "no_directory" => RomLibraryStatus.NoDirectoryConfigured,
                "not_found" => RomLibraryStatus.DirectoryNotFound,
                "empty" => RomLibraryStatus.Empty,
                _ => RomLibraryStatus.Ok,
            };
            var all = new RomLibraryResult(status, status == RomLibraryStatus.NoDirectoryConfigured ? null : romDirectory, status is RomLibraryStatus.Ok or RomLibraryStatus.Empty ? entries : None);
            return Narrow(all, coreDisplayName);
        }

        // One console's share of a scan of all of them, so a sidebar can count every console from one walk.
        public static RomLibraryResult Narrow(RomLibraryResult all, string? coreDisplayName)
        {
            string? console = Narrowed(coreDisplayName);
            if (console is null || all.Status is not (RomLibraryStatus.Ok or RomLibraryStatus.Empty)) return all with { CoreDisplayName = console };
            List<RomEntry> entries = all.Entries.Where(e => e.Shelf == console).ToList();
            return new RomLibraryResult(entries.Count == 0 ? RomLibraryStatus.Empty : RomLibraryStatus.Ok, all.Directory, entries, console);
        }

        // The shelf's name only when it names a real shelf in this build.
        private static string? Narrowed(string? coreDisplayName) =>
            RuntimeCatalog.ShelfByName(coreDisplayName)?.Name;

        // The one place the "why is my list empty" wording lives, so the
        // inline library and the modal browser can't drift apart.
        public static string DescribeEmpty(RomLibraryResult result) => result.Status switch
        {
            RomLibraryStatus.NoDirectoryConfigured => "No ROM directory set - choose one in Settings ▸ Library ▸ ROM folder.",
            RomLibraryStatus.DirectoryNotFound => $"ROM directory not found: {result.Directory}",
            RomLibraryStatus.Empty when result.CoreDisplayName is { } console =>
                $"No {console} games under {result.Directory}",
            RomLibraryStatus.Empty => $"No {string.Join(" or ", Extensions)} files under {result.Directory}",
            _ => string.Empty
        };
    }
}

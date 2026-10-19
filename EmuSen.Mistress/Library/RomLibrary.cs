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

        // Recurses, so one root over per-console subfolders works - see EmuSen_Multicore.md §10.
        private static readonly EnumerationOptions Recursive = new()
        {
            RecurseSubdirectories = true,
            IgnoreInaccessible = true,
            AttributesToSkip = FileAttributes.Hidden | FileAttributes.System,
        };

        // <coreDisplayName> narrows to one console; null or RuntimeCatalog.AllConsoles keeps everything; the library's scan when the switch says so.
        public static RomLibraryResult Scan(string? romDirectory, string? coreDisplayName = null) =>
            EmuSen.Crystal.Client.Library.GameLibrary.Active ? ScanNative(romDirectory, coreDisplayName) : ScanManaged(romDirectory, coreDisplayName);

        // The library's scan, on a database of its own that lives for the call.
        public static RomLibraryResult ScanNative(string? romDirectory, string? coreDisplayName = null)
        {
            using var lib = EmuSen.Crystal.Client.Library.GameLibrary.Open(":memory:");
            return ScanWith(lib, romDirectory, coreDisplayName);
        }

        // A scan through a handle, kept there for the reclaim and identity that follow it.
        internal static RomLibraryResult ScanWith(EmuSen.Crystal.Client.Library.GameLibrary lib, string? romDirectory, string? coreDisplayName = null)
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
                w.WriteEndObject();
            }
            lib.Scan(System.Text.Encoding.UTF8.GetString(stream.ToArray()));
            using var games = System.Text.Json.JsonDocument.Parse(lib.Games(""));
            var shelves = RuntimeCatalog.Shelves.GroupBy(s => s.System).ToDictionary(g => g.Key, g => g.First().Name);
            var entries = games.RootElement.GetProperty("games").EnumerateArray()
                .Select(g => new RomEntry(g.GetProperty("path").GetString()!) { KnownShelf = g.GetProperty("system").GetString() is { } system && shelves.TryGetValue(system, out string? shelf) ? shelf : "Unknown" })
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

        // The C# scan: the failover and the parity class's oracle.
        public static RomLibraryResult ScanManaged(string? romDirectory, string? coreDisplayName = null)
        {
            if (string.IsNullOrWhiteSpace(romDirectory))
            {
                return new RomLibraryResult(RomLibraryStatus.NoDirectoryConfigured, null, None, Narrowed(coreDisplayName));
            }

            if (!Directory.Exists(romDirectory))
            {
                return new RomLibraryResult(RomLibraryStatus.DirectoryNotFound, romDirectory, None, Narrowed(coreDisplayName));
            }

            List<RomEntry> entries;
            try
            {
                var shelf = RuntimeCatalog.ShelfByName(coreDisplayName);
                string[] extensions = Extensions;

                var files = Directory.EnumerateFiles(romDirectory, "*", Recursive)
                    .Where(f => extensions.Contains(Path.GetExtension(f), StringComparer.OrdinalIgnoreCase))
                    .ToList();
                // A sheet's tracks and a list's discs are the sheet's and the list's game, not games of their own.
                entries = EmuSen.Galaxia.Media.ImageSets.Shelved(files)
                    .Select(f => new RomEntry(f))
                    .Where(e => shelf is null || e.Shelf == shelf.Name)
                    .OrderBy(e => e.Title, StringComparer.OrdinalIgnoreCase)
                    .ToList();
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                // An unreadable directory is the same outcome as a missing one, from here.
                return new RomLibraryResult(RomLibraryStatus.DirectoryNotFound, romDirectory, None, Narrowed(coreDisplayName));
            }

            RomLibraryStatus status = entries.Count == 0 ? RomLibraryStatus.Empty : RomLibraryStatus.Ok;
            return new RomLibraryResult(status, romDirectory, entries, Narrowed(coreDisplayName));
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
            RomLibraryStatus.NoDirectoryConfigured => "No ROM directory set - choose one in Settings > Preferences...",
            RomLibraryStatus.DirectoryNotFound => $"ROM directory not found: {result.Directory}",
            RomLibraryStatus.Empty when result.CoreDisplayName is { } console =>
                $"No {console} games under {result.Directory}",
            RomLibraryStatus.Empty => $"No {string.Join(" or ", Extensions)} files under {result.Directory}",
            _ => string.Empty
        };
    }
}

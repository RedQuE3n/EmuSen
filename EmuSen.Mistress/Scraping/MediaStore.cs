using System;
using EmuSen.Galaxia.Library;

namespace EmuSen.Mistress.Scraping
{
    public enum ScrapeState { Found, Unknown, Error }

    public enum ScrapePriority { Shown = 0, Console = 1, Library = 2 }

    // What a game's scrape kept: ScreenScraper's ids, the text of §3.7's metadata variants, and which media it offered.
    public sealed record ScrapedRecord(string Md5, long Bytes, ScrapeState State)
    {
        public long? GameId { get; init; }
        public long? RomId { get; init; }
        public int? SystemId { get; init; }
        public string? Name { get; init; }
        public string? Description { get; init; }
        public string? Developer { get; init; }
        public string? Publisher { get; init; }
        public string? Genre { get; init; }
        public string? Players { get; init; }
        public float? Rating { get; init; }
        public DateTime? ReleaseDate { get; init; }
        public string? Region { get; init; }
        public string? Language { get; init; }
        public string? MatchedBy { get; init; }
        public string? Offered { get; init; }
        public string? Detail { get; init; }
        public DateTime FetchedAt { get; init; }

        // The kinds Offered speaks for; an answer kept before Pass 8 knew only ScrapeRules.FirstKinds (§38).
        public string? KindsKnown { get; init; }

        // A big file's fingerprint where the record is keyed by it, else empty (EmuSen_LibraryTools.md §9.12).
        public string Fingerprint { get; init; } = "";
    }

    public sealed record StoredMedia(string Type, string RelativePath, string? Region, string? Sha1);

    // Where the media store lives and how its files are named; media.db itself is emusen-scrape's, the C# store having been deleted on 2026-11-05 - see EmuSen_Backend.md §40.5.
    public static class MediaStore
    {
        public const string FileName = "media.db";

        public const string CleanupFolder = "CLEANUP";

        // The media folder: <root>/<es-de system>/<type folder>/<game's folder>/<rom stem>.<ext>, and media.db in it (§30).
        public static string DefaultRoot => DataStore.Media;

        private static string[] Parts(string relative) => relative.Split(['/', '\\'], StringSplitOptions.RemoveEmptyEntries);

        public static string KindFolderOf(string relative) => Parts(relative) switch { { Length: >= 3 } p => p[1], { Length: 2 } p => p[0], _ => "" };
    }
}

using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.Json;

namespace EmuSen.Mistress.Scraping
{
    // A text keyed by a region or a language, as ScreenScraper lists names, synopses, dates and genre names.
    public sealed record Localised(string Key, string Text);

    public sealed record ScrapedGenre(IReadOnlyList<Localised> Names, bool Main);

    // One media file the game has, with the address ScreenScraper serves it from; the address carries the credentials and is redacted before it is shown.
    public sealed record ScrapedMedia(string Type, string? Region, string Url, string? Format)
    {
        // The file's own checksum and size as the answer states them, so a kept file can be compared with no request (§38).
        public string? Sha1 { get; init; }
        public long? Size { get; init; }
    }

    // The member's limits and today's counts, as every response carries them; null where a field was absent.
    public sealed record ScrapeQuota(int? MaxThreads, int? MaxDownloadKBps, int? RequestsToday, int? RequestsKoToday,
        int? MaxRequestsPerMinute, int? MaxRequestsPerDay, int? MaxRequestsKoPerDay);

    public sealed record ScrapedGame(long Id, long? RomId, int? SystemId)
    {
        public IReadOnlyList<Localised> Names { get; init; } = [];
        public IReadOnlyList<Localised> Synopses { get; init; } = [];
        public IReadOnlyList<Localised> Dates { get; init; } = [];
        public IReadOnlyList<ScrapedGenre> Genres { get; init; } = [];
        public string? Developer { get; init; }
        public string? Publisher { get; init; }
        public string? Players { get; init; }
        public double? Note { get; init; }
        public IReadOnlyList<ScrapedMedia> Media { get; init; } = [];
    }
}

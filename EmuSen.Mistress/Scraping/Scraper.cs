using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Mistress.Scraping
{
    public enum ScrapeOutcome { Found, Unknown, Error, Retry, Stopped, Missing }

    // How a run ended: its queue emptied, the quota stopped it (what is left stays queued for Resume), or the player cancelled.
    public enum ScrapeRunEnd { Done, Stopped, Cancelled }

    // What one game's turn came to; HasCover says whether the store now holds its cover.
    public sealed record ScrapeResult(string Path, string System, ScrapeOutcome Outcome, bool HasCover, IReadOnlyList<string> Written, string Detail, string? MatchedBy = null, bool Skipped = false)
    {
        // Kept files a refresh found unchanged by the answer's checksum, which cost no request (§38).
        public int Unchanged { get; init; }
    }

    public enum ScrapeStep { LookingUp, Downloading, Arrived }

    // What a worker is doing now, for the status window; Kind is the picture's ES-DE type, Saved the file that has just arrived.
    public sealed record ScrapeActivity(string Path, string System, ScrapeStep Step, string? Kind = null, string? Saved = null);

    // ScreenScraper's system ids, and the run's words; the run itself is emusen-scrape's, the C# Scraper having been deleted on 2026-11-05 - see EmuSen_Backend.md §40.5.
    public static class Scraper
    {
        // ScreenScraper's systemeid for each ES-DE system name the library's shelves carry, as the systems table gives them - see EmuSen_BigPicture.md §5.3, EmuSen_Backend.md §12.
        public static IReadOnlyDictionary<string, int> SystemIds => Library.RuntimeCatalog.Systems
            .Where(e => e.ScreenScraperSystem is not null && e.EsdeSystem.Length > 0).GroupBy(e => e.EsdeSystem).ToDictionary(g => g.Key, g => g.First().ScreenScraperSystem!.Value, StringComparer.Ordinal);
    }
}

using System;
using System.Collections.Generic;

namespace EmuSen.Mistress.Scraping
{
    // What the player chose to scrape: one game; one shelf or the whole library; optionally only the games with no cover - see EmuSen_Settings_Reference.md §4.60.
    public sealed record ScrapeScope(string? Game = null, string? Shelf = null, bool MissingArtOnly = false)
    {
        public static ScrapeScope ThisGame(string path) => new(Game: path);

        public bool IsWholeLibrary => Game is null && Shelf is null;
    }

    // What a scope comes to before it is started: the count, what ScreenScraper has not been asked, and the cost - shown in the confirm step.
    public sealed record ScrapePlan(int Games, int NotYetAsked, int MaxRequests, int? RequestsLeftToday, TimeSpan Time, bool ScreenScraperUsable, string? WhyNot, bool Failover)
    {
        // Measured in the live run (plan §17.9): a found game with the default kinds took a median 13 s on one thread at 128 KB/s.
        public static readonly TimeSpan PerGame = TimeSpan.FromSeconds(13);

        public string Describe()
        {
            string games = Games == 1 ? "1 game" : $"{Games:N0} games";
            if (Games == 0) return "No game in this choice needs scraping.";
            string ask = ScreenScraperUsable
                ? NotYetAsked == 0
                    ? $"{games}, all asked of ScreenScraper before: no request is made for what is already kept."
                    : $"{games}, {NotYetAsked:N0} not yet asked of ScreenScraper: up to {MaxRequests:N0} requests"
                      + (RequestsLeftToday is int left ? $" of the {left:N0} left today" : "")
                      + $", about {Duration(Time)} on one thread."
                : $"{games}. ScreenScraper cannot be used: {WhyNot}.";
            string failover = Failover
                ? " Where ScreenScraper has no cover, OpenEmu's sources are asked (OpenVGDB, libretro thumbnails)."
                : "";
            return ask + failover;
        }

        private static string Duration(TimeSpan t) =>
            t.TotalMinutes < 1 ? $"{Math.Max(1, (int)t.TotalSeconds)} seconds" : t.TotalHours < 1 ? $"{(int)Math.Ceiling(t.TotalMinutes)} minutes" : $"{t.TotalHours:F1} hours";
    }

    public enum ScrapeRunState { Running, Done, Stopped, Cancelled }

    // A run as the player watches it: how far it is, what it found, and how it ended.
    public sealed class ScrapeProgress
    {
        public ScrapeProgress(int total) => Total = total;

        public int Total { get; }
        public int Done { get; set; }
        public int Found { get; set; }
        public int Unknown { get; set; }
        public int FailoverAsked { get; set; }
        public int FailoverFound { get; set; }
        public ScrapeRunState State { get; set; } = ScrapeRunState.Running;
        public string? Why { get; set; }
        public HashSet<string> FailoverPending { get; } = new(StringComparer.Ordinal);

        public string Describe() => State switch
        {
            ScrapeRunState.Running => $"Scraping: {Done:N0} of {Total:N0}" + (FailoverAsked > 0 ? $", {FailoverAsked} asked of OpenEmu's sources" : "") + ".",
            ScrapeRunState.Done => $"Scraped {Done:N0} of {Total:N0}: {Found:N0} found, {Unknown:N0} unknown" + (FailoverFound > 0 ? $", {FailoverFound} covers from OpenEmu's sources" : "") + ".",
            ScrapeRunState.Stopped => $"Stopped after {Done:N0} of {Total:N0}: {Why}. Resume when the quota allows.",
            _ => $"Cancelled after {Done:N0} of {Total:N0}. Resume goes on from there.",
        };
    }
}

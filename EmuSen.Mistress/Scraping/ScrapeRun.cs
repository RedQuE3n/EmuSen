using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;

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

    // One game of a run as the status window lists it: its outcome, the pictures that arrived, and why it failed.
    public sealed record ScrapeRecent(string Path, string System, ScrapeOutcome Outcome, IReadOnlyList<string> Kinds, string Detail, bool Skipped)
    {
        public bool FromFailover { get; init; }
    }

    // A run as the player watches it: how far it is, what it found, and how it ended - see EmuSen_Settings_Reference.md §4.57.
    public sealed class ScrapeProgress
    {
        // The recent-results list keeps this many, newest first.
        public const int RecentLimit = 200;

        public ScrapeProgress(int total, DateTimeOffset started = default)
        {
            Total = total;
            Started = started;
        }

        public int Total { get; }
        public int Done { get; set; }
        public int Found { get; set; }
        public int Unknown { get; set; }
        public int Failed { get; set; }
        public int Skipped { get; set; }
        public int Retrying { get; set; }
        public string? LastFailure { get; set; }
        public int FailoverAsked { get; set; }
        public int FailoverFound { get; set; }
        public ScrapeRunState State { get; set; } = ScrapeRunState.Running;
        public string? Why { get; set; }
        public HashSet<string> FailoverPending { get; } = new(StringComparer.Ordinal);

        public DateTimeOffset Started { get; }
        public DateTimeOffset? Ended { get; set; }
        public int? RequestsAtStart { get; set; }
        public string? DayAtStart { get; set; }

        private readonly List<ScrapeRecent> _recent = new();
        private readonly object _gate = new();
        private ScrapeActivity? _current;
        private string? _picture;
        private int _version;
        private DateTimeOffset? _pausedAt;
        private TimeSpan _pausedFor;

        // Bumped by every change, so a window redraws only when something moved.
        public int Version => Volatile.Read(ref _version);

        public void Touch() => Interlocked.Increment(ref _version);

        // From a worker's thread: what it is doing now, and the last picture that arrived.
        public void Note(ScrapeActivity activity)
        {
            lock (_gate)
            {
                _current = activity;
                if (activity is { Step: ScrapeStep.Arrived, Saved: string saved }) _picture = saved;
            }
            Touch();
        }

        public ScrapeActivity? Current { get { lock (_gate) return _current; } }

        public string? LastPicture { get { lock (_gate) return _picture; } }

        public IReadOnlyList<ScrapeRecent> Recent { get { lock (_gate) return _recent.ToArray(); } }

        // A game's turn as the window counts it: done, found, not found, failed with its reason, skipped, or to be retried.
        public void Count(ScrapeResult result)
        {
            if (result.Outcome is ScrapeOutcome.Found or ScrapeOutcome.Unknown or ScrapeOutcome.Error or ScrapeOutcome.Missing)
            {
                Done++;
                if (result.Skipped) Skipped++;
                else if (result.Outcome == ScrapeOutcome.Found) Found++;
                else if (result.Outcome == ScrapeOutcome.Unknown) Unknown++;
                else if (result.Outcome == ScrapeOutcome.Error)
                {
                    Failed++;
                    LastFailure = ScrapeRedactor.Redact(result.Detail);
                }
                lock (_gate)
                {
                    if (_current?.Path == result.Path) _current = null;
                    _recent.Insert(0, new ScrapeRecent(result.Path, result.System, result.Outcome, result.Written.Select(KindOf).ToList(), ScrapeRedactor.Redact(result.Detail), result.Skipped));
                    if (_recent.Count > RecentLimit) _recent.RemoveAt(_recent.Count - 1);
                }
            }
            else if (result.Outcome == ScrapeOutcome.Retry) Retrying++;
            Touch();
        }

        // The failover filled a game's cover: its row says so.
        public void FilledByFailover(string path)
        {
            lock (_gate)
            {
                int at = _recent.FindIndex(r => r.Path == path);
                if (at >= 0) _recent[at] = _recent[at] with { FromFailover = true };
            }
            Touch();
        }

        // The ES-DE folder a written file sits in names its kind: snes/covers/x.png is a cover.
        public static string KindOf(string relative)
        {
            string folder = System.IO.Path.GetFileName(System.IO.Path.GetDirectoryName(relative) ?? "");
            return ScrapeRules.AllKinds.FirstOrDefault(k => k.Folder == folder)?.EsdeType ?? folder;
        }

        public bool IsPaused { get { lock (_gate) return _pausedAt is not null; } }

        public void SetPaused(bool paused, DateTimeOffset now)
        {
            lock (_gate)
            {
                if (paused && _pausedAt is null) _pausedAt = now;
                else if (!paused && _pausedAt is { } since)
                {
                    _pausedFor += now - since;
                    _pausedAt = null;
                }
            }
            Touch();
        }

        // Wall time since the start, to the end once there is one.
        public TimeSpan Elapsed(DateTimeOffset now) => Max0((Ended ?? now) - Started);

        // Time spent working: the elapsed time less every pause.
        public TimeSpan Working(DateTimeOffset now)
        {
            DateTimeOffset end = Ended ?? now;
            lock (_gate) return Max0(end - Started - _pausedFor - (_pausedAt is { } since ? end - since : TimeSpan.Zero));
        }

        // The run's own pace so far, games done over time worked, applied to what is left; null until a game has cost time.
        public TimeSpan? EstimateLeft(DateTimeOffset now)
        {
            if (State != ScrapeRunState.Running) return null;
            int left = Total - Done;
            if (left <= 0) return TimeSpan.Zero;
            TimeSpan worked = Working(now);
            int asked = Done - Skipped;
            if (asked <= 0 || worked <= TimeSpan.Zero) return null;
            return TimeSpan.FromTicks(worked.Ticks / asked * left);
        }

        private static TimeSpan Max0(TimeSpan t) => t < TimeSpan.Zero ? TimeSpan.Zero : t;

        public string Describe() => State switch
        {
            ScrapeRunState.Running when IsPaused => $"Scraping paused at {Done:N0} of {Total:N0}.",
            ScrapeRunState.Running => $"Scraping: {Done:N0} of {Total:N0}" + (FailoverAsked > 0 ? $", {FailoverAsked} asked of OpenEmu's sources" : "") + ".",
            ScrapeRunState.Done => $"Scraped {Done:N0} of {Total:N0}: {Found:N0} found, {Unknown:N0} unknown" + (FailoverFound > 0 ? $", {FailoverFound} covers from OpenEmu's sources" : "") + ".",
            ScrapeRunState.Stopped => $"Stopped after {Done:N0} of {Total:N0}: {Why}. Resume when the quota allows.",
            _ => $"Cancelled after {Done:N0} of {Total:N0}. Resume goes on from there.",
        };
    }
}

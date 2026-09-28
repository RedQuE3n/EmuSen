using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;

namespace EmuSen.Mistress.Scraping
{
    // ES-DE's "Scrape these games", as far as Mistress can answer it; NoCover is Mistress's own and its default - see EmuSen_BigPicture.md §38.
    public enum ScrapeCriteria { All, Favourites, NoMetadata, NoGameImage, NoGameVideo, NoCover }

    // What the player chose to scrape: one game; one shelf or the whole library; which of its games; and whether what is kept is asked again - see EmuSen_Settings_Reference.md §4.60 and §4.76.
    public sealed record ScrapeScope(string? Game = null, string? Shelf = null, bool MissingArtOnly = false)
    {
        public static ScrapeScope ThisGame(string path) => new(Game: path);

        public bool IsWholeLibrary => Game is null && Shelf is null;

        public ScrapeCriteria Criteria { get; init; } = ScrapeCriteria.All;

        // ES-DE's "Overwrite files and data", for this run only.
        public bool Refresh { get; init; }

        public ScrapeCriteria Effective => MissingArtOnly ? ScrapeCriteria.NoCover : Criteria;

        public static readonly IReadOnlyList<(ScrapeCriteria Value, string Setting, string Text)> CriteriaChoices =
        [
            (ScrapeCriteria.NoCover, "nocover", "Games with no cover"), (ScrapeCriteria.All, "all", "All games"), (ScrapeCriteria.Favourites, "favorites", "Favourite games"),
            (ScrapeCriteria.NoMetadata, "nometadata", "No metadata"), (ScrapeCriteria.NoGameImage, "nogameimage", "No game image"), (ScrapeCriteria.NoGameVideo, "nogamevideo", "No game video"),
        ];

        public static ScrapeCriteria CriteriaOf(string? setting) => CriteriaChoices.FirstOrDefault(c => c.Setting == setting) is { Setting: not null } c ? c.Value : ScrapeCriteria.NoCover;
    }

    // What each kind is expected to cost a found game: the share of §17.9's 39 found answers offering it, and the mean size of the file the region rule would take - see EmuSen_BigPicture.md §38.
    public static class ScrapeCost
    {
        public static readonly IReadOnlyDictionary<string, (double Offered, double MeanKB)> Kinds = new Dictionary<string, (double, double)>(StringComparer.Ordinal)
        {
            ["cover"] = (33 / 39.0, 653), ["screenshot"] = (36 / 39.0, 5), ["marquee"] = (36 / 39.0, 90), ["titlescreen"] = (36 / 39.0, 6), ["miximage"] = (39 / 39.0, 565),
            ["backcover"] = (33 / 39.0, 468), ["3dbox"] = (33 / 39.0, 351), ["physicalmedia"] = (33 / 39.0, 449), ["fanart"] = (25 / 39.0, 397),
            ["manual"] = (27 / 39.0, 2254), ["video"] = (32 / 39.0, 1354),
        };

        // §17.9: a found game with the four default kinds took a median 13.3 s at 128 KB/s, of which about 9.4 s is their 1.2 MB; the rest is the requests' own time.
        public static readonly TimeSpan RequestsPerGame = TimeSpan.FromSeconds(3.9);

        public const int AssumedKBps = 128;

        public static double Requests(IEnumerable<ScrapeMediaKind> kinds) => 1 + kinds.Sum(k => Kinds.TryGetValue(k.EsdeType, out var c) ? c.Offered : 1);

        public static double KB(IEnumerable<ScrapeMediaKind> kinds) => kinds.Sum(k => Kinds.TryGetValue(k.EsdeType, out var c) ? c.Offered * c.MeanKB : 0);

        public static TimeSpan Time(int games, double kb, int? kbps) => RequestsPerGame * games + TimeSpan.FromSeconds(kb / Math.Max(1, kbps ?? AssumedKBps));
    }

    // What a scope comes to before it is started: the count, what ScreenScraper has not been asked, and the cost - shown in the confirm step.
    public sealed record ScrapePlan(int Games, int NotYetAsked, int MaxRequests, int? RequestsLeftToday, TimeSpan Time, bool ScreenScraperUsable, string? WhyNot, bool Failover)
    {
        // Measured in the live run (plan §17.9): a found game with the default kinds took a median 13 s on one thread at 128 KB/s.
        public static readonly TimeSpan PerGame = TimeSpan.FromSeconds(13);

        // Found games asked again: every one under a refresh, else those wanting a kind they may offer and lack.
        public int AskedAgain { get; init; }

        // The expected requests and megabytes by §38's offer rates, beside MaxRequests' bound.
        public double ExpectedRequests { get; init; }
        public double ExpectedMB { get; init; }
        public bool Refresh { get; init; }

        public string Describe()
        {
            string games = Games == 1 ? "1 game" : $"{Games:N0} games";
            if (Games == 0) return "No game in this choice needs scraping.";
            int asking = NotYetAsked + AskedAgain;
            string again = AskedAgain == 0 ? "" : Refresh ? $", {AskedAgain:N0} found before and asked again to refresh them" : $", {AskedAgain:N0} found before and asked again for kinds they lack";
            string ask = ScreenScraperUsable
                ? asking == 0
                    ? $"{games}, all asked of ScreenScraper before: no request is made for what is already kept."
                    : $"{games}, {NotYetAsked:N0} not yet asked of ScreenScraper{again}: up to {MaxRequests:N0} requests"
                      + (RequestsLeftToday is int left ? $" of the {left:N0} left today" : "")
                      + (ExpectedRequests > 0 ? $" (about {ExpectedRequests:N0} and {Megabytes(ExpectedMB)} expected)" : "")
                      + $", about {Duration(Time)} on one thread."
                : $"{games}. ScreenScraper cannot be used: {WhyNot}.";
            string failover = Failover
                ? " Where ScreenScraper has no cover, OpenEmu's sources are asked (OpenVGDB, libretro thumbnails)."
                : "";
            return ask + failover;
        }

        private static string Megabytes(double mb) => mb >= 1024 ? $"{mb / 1024:0.#} GB" : mb >= 10 ? $"{mb:N0} MB" : $"{mb:0.#} MB";

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
        public int Unchanged { get; set; }
        public string? LastFailure { get; set; }
        public int FailoverAsked { get; set; }
        public int FailoverFound { get; set; }
        public ScrapeRunState State { get; set; } = ScrapeRunState.Running;
        public string? Why { get; set; }
        public HashSet<string> FailoverPending { get; } = new(StringComparer.Ordinal);

        public DateTimeOffset Started { get; }
        public DateTimeOffset? Ended { get; set; }

        // Requests this run sent to ScreenScraper: each lookup and each picture, as plan §17.9 measured they are counted.
        public int Requests => Volatile.Read(ref _requests);

        private int _requests;

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
            if (activity.Step is ScrapeStep.LookingUp or ScrapeStep.Downloading) Interlocked.Increment(ref _requests);
            lock (_gate)
            {
                _current = activity;
                if (activity is { Step: ScrapeStep.Arrived, Saved: string saved } && ScrapeRules.KindOf(activity.Kind ?? "")?.Payload is null or MediaPayload.Image) _picture = saved;
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
                Unchanged += result.Unchanged;
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

        // The ES-DE type folder a written file sits in names its kind: snes/covers/x.png and nes/covers/USA/x.png are covers.
        public static string KindOf(string relative)
        {
            string folder = MediaStore.KindFolderOf(relative);
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

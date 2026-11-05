using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Http;
using System.Text.Json;
using EmuSen.Crystal.Client.Library;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The library's scraper on a store of the test's, sent to a fake ScreenScraper through the replay server on a test clock, through the binding Mistress uses: what the C# Scraper's tests drove, now driving emusen-scrape - see EmuSen_Backend.md §40.5.
    internal sealed class LibraryScraper : IDisposable
    {
        public static readonly DateTimeOffset Then = new(2026, 7, 14, 10, 0, 0, TimeSpan.Zero);

        public static readonly byte[] Blob = ScrapeCredentialTests.Scramble(JsonSerializer.Serialize(new { devid = FakeScreenScraper.DevId, devpassword = FakeScreenScraper.DevPassword, softname = FakeScreenScraper.SoftName }));

        // One answer of a run as the progress lays it out; Kinds are the ES-DE types written.
        public sealed record Result(string Path, string System, ScrapeOutcome Outcome, bool HasCover, IReadOnlyList<string> Kinds, string Detail, string? MatchedBy, bool Skipped);

        private readonly Dictionary<string, object?> _settings;
        private readonly GameLibrary? _library;
        private readonly Queue<Result> _pending = new();
        private long _seen;

        public ReplayServer Replay { get; }

        public NativeScraping Scraper { get; private set; }

        public string Media { get; }

        public LibraryScraper(HttpMessageHandler server, string media, IReadOnlyDictionary<string, object?>? settings = null, DateTimeOffset? clock = null, GameLibrary? library = null, long? fingerprintAbove = null)
        {
            Replay = new ReplayServer(server);
            Media = media;
            _library = library;
            _settings = new Dictionary<string, object?>
            {
                ["developer_files"] = false, ["openemu"] = false, ["user_agent"] = "EmuSen/test",
                ["cores"] = RuntimeCatalog.Consoles.Select(c => c.Systems).ToArray(),
            };
            foreach (var (key, value) in settings ?? new Dictionary<string, object?>()) _settings[key] = value;
            var test = new Dictionary<string, object?> { ["connect"] = Replay.Address, ["roots"] = ReplayServer.RootPem, ["clock"] = NativeScraping.Ticks(clock ?? Then) };
            if (fingerprintAbove is long above) test["fingerprint_above"] = above;
            _settings["test"] = test;
            Scraper = NativeScraping.Open(media, Json, library, Blob);
        }

        public string Json => JsonSerializer.Serialize(_settings);

        // Closed and opened again, as a restart of the program would.
        public void Reopen()
        {
            Scraper.Dispose();
            Scraper = NativeScraping.Open(Media, Json, _library, Blob);
            _seen = 0;
            _pending.Clear();
        }

        // A run of these games, as the player's Scrape starts one; how many it queued.
        public long Run(IEnumerable<(string Path, string System)> games, ScrapeChoices? choices = null, ScrapePriority priority = ScrapePriority.Shown, bool one = false, bool resume = false, bool refresh = false, ScrapedGame? pick = null, bool enabled = true)
        {
            _seen = 0;
            _pending.Clear();
            return Scraper.Start(games, priority, one, resume, refresh, pick, enabled, choices ?? new ScrapeChoices());
        }

        public JsonDocument Progress() => JsonDocument.Parse(Scraper.Progress(0));

        public string State
        {
            get
            {
                using JsonDocument p = Progress();
                return p.RootElement.GetProperty("state").GetString()!;
            }
        }

        private void Drain()
        {
            using JsonDocument p = Progress();
            foreach (JsonElement r in p.RootElement.GetProperty("results").EnumerateArray().Reverse().Where(r => r.GetProperty("serial").GetInt64() > _seen))
            {
                _seen = r.GetProperty("serial").GetInt64();
                string? Text(string name) => r.GetProperty(name).ValueKind == JsonValueKind.String ? r.GetProperty(name).GetString() : null;
                ScrapeOutcome outcome = Text("outcome") switch
                {
                    "found" => ScrapeOutcome.Found, "unknown" => ScrapeOutcome.Unknown, "error" => ScrapeOutcome.Error, "retry" => ScrapeOutcome.Retry, "stopped" => ScrapeOutcome.Stopped, _ => ScrapeOutcome.Missing,
                };
                _pending.Enqueue(new Result(Text("path")!, Text("system")!, outcome, r.GetProperty("has_cover").GetBoolean(),
                    r.GetProperty("kinds").EnumerateArray().Select(k => k.GetString()!).ToList(), Text("detail") ?? "", Text("matched_by"), r.GetProperty("skipped").GetBoolean()));
            }
        }

        // The next game's answer, waiting for it.
        public Result Next(int seconds = 20)
        {
            var deadline = DateTime.UtcNow.AddSeconds(seconds);
            while (true)
            {
                Drain();
                if (_pending.Count > 0) return _pending.Dequeue();
                Assert.True(DateTime.UtcNow < deadline, "no result");
                Scraper.Wait(-1, 50);
            }
        }

        // The run's end, waited for: done, stopped or cancelled.
        public string Ends(int seconds = 20)
        {
            var deadline = DateTime.UtcNow.AddSeconds(seconds);
            while (State == "running")
            {
                Assert.True(DateTime.UtcNow < deadline, "the run did not end");
                Scraper.Wait(-1, 50);
            }
            return State;
        }

        // No answer beyond those already taken.
        public void NothingMore()
        {
            Ends();
            Drain();
            Assert.Empty(_pending);
        }

        public T Get<T>(object request) => JsonSerializer.Deserialize<T>(Scraper.Service.Get(JsonSerializer.Serialize(request)))!;

        public void Dispose()
        {
            Scraper.Dispose();
            Replay.Dispose();
        }
    }
}

using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text.Json;
using System.Threading;
using EmuSen.Crystal.Client.Library;
using EmuSen.Crystal.Client.Scrape;

namespace EmuSen.Mistress.Scraping
{
    // The scraper in Rust behind the binding (EMUSEN_SCRAPE_NATIVE, state 2): the window's reads, runs, sign-ins, searches and pictures asked of emusen_scrape.h, answered in the C# types the windows already read - see EmuSen_Backend.md §30.14.
    public sealed class NativeScraping : IDisposable
    {
        private readonly MediaService _service;
        private readonly ConditionalWeakTable<ScrapedGame, object> _picks = new();

        private NativeScraping(MediaService service) => _service = service;

        public static NativeScraping Open(string root, string settings, GameLibrary? library, byte[]? embedded) =>
            new(MediaService.Open(root, settings, library, embedded));

        public MediaService Service => _service;

        private static readonly DateTimeOffset Epoch = DateTimeOffset.UnixEpoch;

        public static long Ticks(DateTimeOffset t) => t.UtcTicks - Epoch.UtcTicks;

        public static DateTimeOffset At(long ticks) => new(Epoch.UtcTicks + ticks, TimeSpan.Zero);

        private static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        private static long? Long(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.Number ? v.GetInt64() : null;

        private static DateTimeOffset? Time(JsonElement e, string name) =>
            Text(e, name) is string t ? DateTimeOffset.Parse(t, CultureInfo.InvariantCulture, DateTimeStyles.AssumeUniversal) : null;

        public void Settings(string json) => _service.Change(json.Insert(1, "\"op\":\"settings\","));

        // A scraped record as media.db holds it, read from the scraper's JSON.
        public static ScrapedRecord Record(JsonElement e) => new(Text(e, "md5")!, Long(e, "bytes") ?? 0, Enum.Parse<ScrapeState>(Text(e, "state")!))
        {
            GameId = Long(e, "game_id"), RomId = Long(e, "rom_id"), SystemId = (int?)Long(e, "system_id"),
            Name = Text(e, "name"), Description = Text(e, "description"), Developer = Text(e, "developer"), Publisher = Text(e, "publisher"),
            Genre = Text(e, "genre"), Players = Text(e, "players"),
            Rating = e.GetProperty("rating").ValueKind == JsonValueKind.Number ? e.GetProperty("rating").GetSingle() : null,
            ReleaseDate = Text(e, "release_date") is string d ? DateTime.ParseExact(d, "yyyy-MM-dd", CultureInfo.InvariantCulture) : null,
            Region = Text(e, "region"), Language = Text(e, "language"), MatchedBy = Text(e, "matched_by"), Offered = Text(e, "offered"), Detail = Text(e, "detail"),
            FetchedAt = DateTime.Parse(Text(e, "fetched_at")!, CultureInfo.InvariantCulture, DateTimeStyles.RoundtripKind), KindsKnown = Text(e, "kinds_known"),
        };

        public IReadOnlyDictionary<string, ScrapedRecord> FoundByPath()
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Get("{\"what\":\"found\"}"));
            return doc.RootElement.EnumerateObject().ToDictionary(p => p.Name, p => Record(p.Value), StringComparer.Ordinal);
        }

        public IReadOnlyDictionary<string, (ScrapeState State, bool HasCover)> OutcomesByPath()
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Get("{\"what\":\"outcomes\"}"));
            return doc.RootElement.EnumerateObject().ToDictionary(p => p.Name, p => (Enum.Parse<ScrapeState>(p.Value.GetProperty("state").GetString()!), p.Value.GetProperty("has_cover").GetBoolean()), StringComparer.Ordinal);
        }

        public ScrapedRecord? FoundFor(string path)
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Get(JsonSerializer.Serialize(new { what = "record", path })));
            return doc.RootElement.ValueKind == JsonValueKind.Null ? null : Record(doc.RootElement);
        }

        public IReadOnlyList<StoredMedia> Media(string md5, long bytes)
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Get(JsonSerializer.Serialize(new { what = "stored", md5, bytes })));
            return doc.RootElement.EnumerateArray().Select(m => new StoredMedia(Text(m, "type")!, Text(m, "path")!, Text(m, "region"), Text(m, "sha1"))).ToList();
        }

        public int QueueLength
        {
            get
            {
                using JsonDocument doc = JsonDocument.Parse(_service.Get("{\"what\":\"queue\"}"));
                return doc.RootElement.GetArrayLength();
            }
        }

        public IReadOnlyList<(string Path, string System)> Queue()
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Get("{\"what\":\"queue\"}"));
            return doc.RootElement.EnumerateArray().Select(q => (Text(q, "path")!, Text(q, "system")!)).ToList();
        }

        public QuotaSnapshot Quota()
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Get("{\"what\":\"quota\"}"));
            return Quota(doc.RootElement);
        }

        public static QuotaSnapshot Quota(JsonElement q) => new(Text(q, "day")!, (int)Long(q, "requests")!.Value, (int?)Long(q, "max_per_day"), (int)Long(q, "ko")!.Value, (int?)Long(q, "max_ko_per_day"),
            (int)Long(q, "threads")!.Value, (int?)Long(q, "per_minute"), TimeSpan.FromTicks(Long(q, "interval_ticks") ?? 0), Text(q, "gate") switch { "wait" => QuotaGate.Wait, "stopped" => QuotaGate.Stopped, _ => QuotaGate.Go },
            Time(q, "until"), Text(q, "reason"), q.GetProperty("from_server").GetBoolean());

        public static ScrapeQuota? Limits(JsonElement q) => q.GetProperty("from_server").GetBoolean()
            ? new ScrapeQuota((int?)Long(q, "threads"), (int?)Long(q, "max_kbps"), (int?)Long(q, "requests"), (int?)Long(q, "ko"), (int?)Long(q, "per_minute"), (int?)Long(q, "max_per_day"), (int?)Long(q, "max_ko_per_day"))
            : null;

        public IReadOnlyList<string> Forget(string path, string? system, string folder) =>
            JsonSerializer.Deserialize<string[]>(_service.Change(JsonSerializer.Serialize(new { op = "forget", path, system, folder })))!;

        public void ForgetUnfound(string path) => _service.Change(JsonSerializer.Serialize(new { op = "forget_unfound", path }));

        public IReadOnlyList<string> Orphans(IEnumerable<(string Path, string System)> games) =>
            JsonSerializer.Deserialize<string[]>(_service.Get(JsonSerializer.Serialize(new { what = "orphans", games = games.Select(g => new { path = g.Path, system = g.System }) })))!;

        public (int Moved, long Bytes, string Folder) CleanUp(IReadOnlyList<string> relatives, DateTimeOffset at)
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Change(JsonSerializer.Serialize(new { op = "clean_up", relatives, at = at.ToString("o", CultureInfo.InvariantCulture) })));
            return (doc.RootElement.GetProperty("moved").GetInt32(), doc.RootElement.GetProperty("bytes").GetInt64(), doc.RootElement.GetProperty("folder").GetString()!);
        }

        public ScrapePlan Plan(IEnumerable<string> games, bool one, bool refresh, IEnumerable<ScrapeMediaKind> kinds, bool enabled, bool failover)
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Get(JsonSerializer.Serialize(new { what = "plan", games, one, refresh, kinds = kinds.Select(k => k.EsdeType), enabled })));
            JsonElement p = doc.RootElement;
            return new ScrapePlan(p.GetProperty("games").GetInt32(), p.GetProperty("not_yet_asked").GetInt32(), p.GetProperty("max_requests").GetInt32(), (int?)Long(p, "requests_left_today"),
                TimeSpan.FromTicks(p.GetProperty("time_ticks").GetInt64()), p.GetProperty("usable").GetBoolean(), Text(p, "why_not"), failover)
            {
                AskedAgain = p.GetProperty("asked_again").GetInt32(), Refresh = refresh, ExpectedRequests = p.GetProperty("expected_requests").GetDouble(), ExpectedMB = p.GetProperty("expected_mb").GetDouble(),
            };
        }

        public (MediaSource Source, string? Path) Locate(string esdeSystem, string romPath, string mediaType, string? folderPath = null, bool folder = false)
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Get(JsonSerializer.Serialize(new { what = "media", path = romPath, system = esdeSystem, type = mediaType, folder, folder_path = folderPath })));
            if (doc.RootElement.ValueKind == JsonValueKind.Null) return (MediaSource.None, null);
            MediaSource source = Text(doc.RootElement, "source") switch
            {
                "other_game" => MediaSource.OtherGame, "hand" => MediaSource.HandPlaced, "screenscraper" => MediaSource.ScreenScraper, "esde" => MediaSource.EsdeFolder, "openemu" => MediaSource.OpenEmu, "package" => MediaSource.Package, _ => MediaSource.None,
            };
            return (source, Text(doc.RootElement, "path"));
        }

        public IReadOnlySet<string> Present(string esdeSystem, IEnumerable<(string File, string FolderPath, bool Folder)> games) =>
            JsonSerializer.Deserialize<string[]>(_service.Get(JsonSerializer.Serialize(new { what = "present", system = esdeSystem, games = games.Select(g => new { path = g.File, folder_path = g.FolderPath, folder = g.Folder }) })))!.ToHashSet(StringComparer.Ordinal);

        public string Stamp(string esdeSystem, string choice) => JsonSerializer.Deserialize<string>(_service.Get(JsonSerializer.Serialize(new { what = "stamp", system = esdeSystem, choice })))!;

        // The scraper's clock: the system's, or a test's started by its settings.
        public DateTimeOffset Now => DateTimeOffset.Parse(JsonSerializer.Deserialize<string>(_service.Get("{\"what\":\"now\"}"))!, CultureInfo.InvariantCulture, DateTimeStyles.AssumeUniversal);

        public void ArtAdded(string? console, string stem, string path) => _service.Change(JsonSerializer.Serialize(new { op = "art_added", console, stem, path }));

        // A run started; how many games are queued.
        public long Start(IEnumerable<(string Path, string System)> games, ScrapePriority priority, bool one, bool resume, bool refresh, ScrapedGame? pick, bool enabled, ScrapeChoices choices)
        {
            long? pickId = pick is not null && _picks.TryGetValue(pick, out object? id) ? (long)id : null;
            string request = JsonSerializer.Serialize(new
            {
                games = games.Select(g => new { path = g.Path, system = g.System }), priority = priority.ToString().ToLowerInvariant(), one, resume, refresh, pick = pickId, enabled,
                choices = new { kinds = choices.Kinds().Select(k => k.EsdeType), region = choices.Region, language = choices.Language, region_fallback = choices.RegionFallback, threads = choices.Threads },
            });
            return _service.Start(request);
        }

        public string Progress(long since) => _service.Progress(since);

        public long Wait(long serial, int timeoutMs) => _service.Wait(serial, timeoutMs);

        public bool Pause(bool paused) => _service.Pause(paused);

        public bool Cancel() => _service.Cancel();

        // Log In or Check through the scraper, its answer in the window's words.
        public SignInAnswer SignIn(string? user, string? password)
        {
            string request = user is null ? "{\"op\":\"check\"}" : JsonSerializer.Serialize(new { op = "sign_in", user, password });
            using JsonDocument doc = JsonDocument.Parse(_service.Change(request));
            JsonElement r = doc.RootElement;
            string said = Text(r, "said") ?? "";
            int? Q(int i) => r.GetProperty("quota")[i].ValueKind == JsonValueKind.Number ? r.GetProperty("quota")[i].GetInt32() : null;
            ScrapeQuota? quota = r.TryGetProperty("quota", out JsonElement qs) && qs.ValueKind == JsonValueKind.Array ? new ScrapeQuota(Q(0), Q(1), Q(2), Q(3), Q(4), Q(5), Q(6)) : null;
            ScrapeMember? member = Text(r, "name") is string name ? new ScrapeMember(name, Text(r, "level"), quota) : null;
            return Text(r, "result") switch
            {
                "signed_in" => new(SignInResult.SignedIn, member, $"Signed in as {member?.Name}."),
                "wrong_account" => new(SignInResult.WrongAccount, null, "ScreenScraper did not accept that name and password."),
                "developer_refused" => new(SignInResult.DeveloperRefused, null, "ScreenScraper refused EmuSen's developer credentials (403), so no account can be checked with this build."),
                "no_developer" => SignInAnswer.NoDeveloper(),
                "incomplete" => new(SignInResult.Incomplete, null, "Type your ScreenScraper name and password first."),
                "busy" => new(SignInResult.Busy, null, "ScreenScraper is too busy to check an account now (401). Try again in a few minutes."),
                "closed" => new(SignInResult.Closed, null, "ScreenScraper's API is closed (423). Try again later."),
                "blocked" => new(SignInResult.Blocked, null, "ScreenScraper has blocked this version of the software (426); a newer build is needed."),
                "too_many" => new(SignInResult.TooMany, null, "ScreenScraper is refusing more requests for now. Try again later."),
                "unreachable" => new(SignInResult.Unreachable, null, ScrapeRedactor.Redact($"ScreenScraper did not answer normally: {said}")),
                _ => new(SignInResult.Unreadable, null, "ScreenScraper's answer could not be read."),
            };
        }

        public bool SignOut()
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Change("{\"op\":\"sign_out\"}"));
            return doc.RootElement.GetProperty("had").GetBoolean();
        }

        // One name search; each game found is kept with its pick, which a Choose names.
        public SearchAnswer Search(string path, string system, string text)
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Change(JsonSerializer.Serialize(new { op = "search", path, system, text })));
            JsonElement r = doc.RootElement;
            ScrapeStatus status = Text(r, "status") switch
            {
                "bad_request" => ScrapeStatus.BadRequest, "failed" => ScrapeStatus.Failed, "daily_quota" => ScrapeStatus.DailyQuota,
                string s when Enum.TryParse(s, out ScrapeStatus named) => named, _ => ScrapeStatus.Failed,
            };
            var games = new List<ScrapedGame>();
            foreach (JsonElement g in r.GetProperty("games").EnumerateArray())
            {
                IReadOnlyList<Localised> Pairs(string name) => g.GetProperty(name).EnumerateArray().Select(p => new Localised(p[0].GetString()!, p[1].GetString()!)).ToList();
                var game = new ScrapedGame(g.GetProperty("id").GetInt64(), null, (int?)Long(g, "system_id")) { Names = Pairs("names"), Dates = Pairs("dates"), Publisher = Text(g, "publisher"), Developer = Text(g, "developer") };
                _picks.AddOrUpdate(game, g.GetProperty("pick").GetInt64());
                games.Add(game);
            }
            return new SearchAnswer(status, games, null, Text(r, "detail") ?? "", TimeSpan.Zero);
        }

        public long? OpenVgdbBytes
        {
            get
            {
                using JsonDocument doc = JsonDocument.Parse(_service.Get("{\"what\":\"openvgdb\"}"));
                return Long(doc.RootElement, "bytes");
            }
        }

        // OpenVGDB fetched now: the tag, or null with the reason why not.
        public (string? Tag, string? Failed) DownloadOpenVgdb()
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Change("{\"op\":\"openvgdb_download\"}"));
            return (Text(doc.RootElement, "tag"), Text(doc.RootElement, "failed"));
        }

        public long? RemoveOpenVgdb()
        {
            using JsonDocument doc = JsonDocument.Parse(_service.Change("{\"op\":\"openvgdb_remove\"}"));
            return Long(doc.RootElement, "bytes");
        }

        public void Dispose() => _service.Dispose();
    }
}

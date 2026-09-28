using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Cores;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Runs only with EMUSEN_SCRAPE_LIVE=1: the real server, the developer's own file, and a library that is only read.
    public sealed class ScrapeLiveFactAttribute : FactAttribute
    {
        public ScrapeLiveFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_SCRAPE_LIVE") != "1")
                Skip = "Scrapes 40 random ROMs from ScreenScraper for real; set EMUSEN_SCRAPE_LIVE=1 - see EmuSen_BigPicture.md §17";
        }
    }

    // Stage (d)'s one live run (P9, P10, P60-P67): 40 random files from the library, one thread, media into ~/.cache - see EmuSen_BigPicture.md §17.
    public class ScrapeLiveTool(ITestOutputHelper output)
    {
        // Paces as the real clock does and records every wait the quota asked for.
        private sealed class RecordingClock : IScrapeClock
        {
            public readonly ConcurrentQueue<TimeSpan> Waits = new();
            public DateTimeOffset Now => DateTimeOffset.UtcNow;
            public Task Delay(TimeSpan wait, CancellationToken stop)
            {
                if (wait > TimeSpan.Zero) Waits.Enqueue(wait);
                return SystemScrapeClock.Instance.Delay(wait, stop);
            }
        }

        [ScrapeLiveFact]
        public async Task Forty_random_files()
        {
            string library = Environment.GetEnvironmentVariable("EMUSEN_SCRAPE_ROMS") ?? "/home/red/Documents/Roms";
            int seed = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_SCRAPE_SEED"), out int s) ? s : 20260926;
            string developerFile = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "EmuSen", DeveloperCredentials.FileName);
            DeveloperCredentials developer = DeveloperCredentials.Read(developerFile) ?? throw new InvalidOperationException("no developer file");
            string outRoot = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "scrape-live", DateTime.Now.ToString("yyyyMMdd-HHmmss"));
            string answers = Path.Combine(outRoot, "answers");
            Directory.CreateDirectory(answers);
            var log = new StringBuilder();
            void Say(string line) { string safe = ScrapeRedactor.Redact(line); log.AppendLine(safe); output.WriteLine(safe); }

            // The library is read, never written: the scan lists, the hash reads, nothing else touches it.
            RomLibraryResult scan = RomLibrary.Scan(library);
            var candidates = scan.Entries.Where(e => CoreCatalog.ShelfByName(e.Shelf)?.EsdeSystem is { Length: > 0 } && new FileInfo(e.FullPath).Length <= CoverFetcher.HashLimitBytes).ToList();
            var random = new Random(seed);
            var chosen = candidates.OrderBy(e => e.FullPath, StringComparer.Ordinal).OrderBy(_ => random.Next()).Take(40).ToList();
            Say($"library {library}: {scan.Entries.Count} files, {candidates.Count} on a shelf; seed {seed}; chosen {chosen.Count}");
            foreach (var g in chosen.GroupBy(e => e.Shelf)) Say($"  {g.Key}: {g.Count()}");

            using var http = new HttpClient { Timeout = TimeSpan.FromSeconds(60) };
            http.DefaultRequestHeaders.UserAgent.ParseAdd("EmuSen/stage-d-live");
            var client = new ScreenScraperClient(http, developer, null);

            var systems = await client.SystemsAsync(CancellationToken.None);
            Say($"systemesListe: {systems.Status}; " + string.Join(", ", Scraper.SystemIds.Select(kv => $"{kv.Key}={kv.Value}:{(systems.Systems.TryGetValue(kv.Value, out string? n) ? n : "absent")}")));
            var before = await client.UserInfosAsync(CancellationToken.None);
            Say($"ssuserInfos before: {before.Status} {Describe(before.Quota)} {before.Detail}");

            using MediaStore store = MediaStore.Open(Path.Combine(outRoot, "Media"));
            var clock = new RecordingClock();
            var quota = new ScrapeQuotaManager(store, clock);
            var results = new BlockingCollection<ScrapeResult>();
            var jeuInfos = new ConcurrentQueue<(string Path, ScrapeStatus Status, double Seconds, ScrapeQuota? Quota)>();
            int answerIndex = 0;
            var scraper = new Scraper(client, store, quota, () => new ScrapeChoices(), (_, _) => false,
                p => CoreCatalog.ByExtension(Path.GetExtension(p))?.OpenVgdbBytes, results.Add, clock)
            {
                Answered = (path, a) =>
                {
                    jeuInfos.Enqueue((path, a.Status, a.Took.TotalSeconds, a.Quota));
                    int n = Interlocked.Increment(ref answerIndex);
                    File.WriteAllText(Path.Combine(answers, $"{n:D3}-{a.Status}.json"), ScrapeRedactor.Redact($"{Path.GetFileName(path)}\n{a.Body}"));
                },
            };

            var total = Stopwatch.StartNew();
            var perGame = new Dictionary<string, double>();
            foreach (RomEntry e in chosen) scraper.Enqueue(e.FullPath, CoreCatalog.ShelfByName(e.Shelf)!.EsdeSystem, ScrapePriority.Library);
            var clockPerGame = Stopwatch.StartNew();
            scraper.Start();
            var finished = new List<ScrapeResult>();
            while (finished.Count(r => r.Outcome is not (ScrapeOutcome.Retry)) < chosen.Count)
            {
                if (!results.TryTake(out ScrapeResult? r, TimeSpan.FromMinutes(3))) { Say("no result for three minutes; stopping"); break; }
                perGame[r.Path] = clockPerGame.Elapsed.TotalSeconds;
                clockPerGame.Restart();
                finished.Add(r);
                Say($"{finished.Count,2} {r.Outcome,-8} {r.MatchedBy ?? "",-11} cover={(r.HasCover ? "y" : "n")} media={r.Written.Count} {perGame[r.Path],6:F1}s {Path.GetFileName(r.Path)} {r.Detail}");
                if (r.Outcome == ScrapeOutcome.Stopped) { Say("stopped: " + r.Detail); break; }
            }
            scraper.Dispose();
            total.Stop();

            var after = await client.UserInfosAsync(CancellationToken.None);
            Say($"ssuserInfos after: {after.Status} {Describe(after.Quota)} {after.Detail}");

            // --- the measurements ---
            var final = finished.Where(r => r.Outcome is ScrapeOutcome.Found or ScrapeOutcome.Unknown or ScrapeOutcome.Error).ToList();
            int found = final.Count(r => r.Outcome == ScrapeOutcome.Found);
            int byFile = final.Count(r => r.Outcome == ScrapeOutcome.Found && r.MatchedBy == "file");
            int byTransform = final.Count(r => r.Outcome == ScrapeOutcome.Found && r.MatchedBy == "transformed");
            Say($"P9: {found}/{final.Count} found; step 1 {byFile}, step 2 {byTransform}; unknown {final.Count(r => r.Outcome == ScrapeOutcome.Unknown)}, error {final.Count(r => r.Outcome == ScrapeOutcome.Error)}");
            foreach (var shelf in final.GroupBy(r => CoreCatalog.ShelfFor(r.Path)))
                Say($"  {shelf.Key}: {shelf.Count(r => r.Outcome == ScrapeOutcome.Found)}/{shelf.Count()} (step 2: {shelf.Count(r => r.MatchedBy == "transformed")})");
            foreach (ScrapeResult r in final.Where(r => r.Path.EndsWith(".z64") || r.Path.EndsWith(".n64") || r.Path.EndsWith(".v64")))
                Say($"P60: {Path.GetFileName(r.Path)}: {r.Outcome} by {r.MatchedBy ?? "-"}");

            var answered = jeuInfos.ToList();
            int requests = answered.Count, media = finished.Sum(r => r.Written.Count);
            ScrapeQuota? first = answered.Select(a => a.Quota).FirstOrDefault(q => q is not null), last = answered.Select(a => a.Quota).LastOrDefault(q => q is not null);
            Say($"P61: jeuInfos {requests}, media files {media}; requeststoday {first?.RequestsToday}->{last?.RequestsToday} across the answers; ssuserInfos {before.Quota?.RequestsToday}->{after.Quota?.RequestsToday}");
            Say($"P62: quota fields present in {answered.Count(a => a.Quota is not null)}/{requests} answers; last: {Describe(last)}");
            var foundTimes = answered.Where(a => a.Status == ScrapeStatus.Found).Select(a => a.Seconds).ToList();
            var koTimes = answered.Where(a => a.Status == ScrapeStatus.NotFound).Select(a => a.Seconds).ToList();
            Say($"P63: jeuInfos found {Stats(foundTimes)}; unknown {Stats(koTimes)}; per found game {Stats(final.Where(r => r.Outcome == ScrapeOutcome.Found).Select(r => perGame[r.Path]).ToList())}; per unknown game {Stats(final.Where(r => r.Outcome == ScrapeOutcome.Unknown).Select(r => perGame[r.Path]).ToList())}; whole run {total.Elapsed.TotalSeconds:F0}s");
            var waits = clock.Waits.ToList();
            Say($"P65: pacer waits {waits.Count}, total {waits.Sum(w => w.TotalSeconds):F1}s, interval at the end {quota.Interval.TotalSeconds:F2}s");

            int regionTagged = 0, regionMatched = 0;
            foreach (ScrapeResult r in final.Where(r => r.Outcome == ScrapeOutcome.Found))
            {
                string? tag = ScrapeRules.RegionOfFileName(Path.GetFileName(r.Path));
                if (tag is not ("us" or "eu" or "jp")) continue;
                string md5 = RomHashes.Of(r.Path).Md5;
                StoredMedia? cover = store.Media(md5, new FileInfo(r.Path).Length).FirstOrDefault(m => m.Type == "cover");
                if (cover is null) continue;
                regionTagged++;
                if (cover.Region == tag) regionMatched++;
                else Say($"  P66 miss: {Path.GetFileName(r.Path)} tagged {tag}, cover {cover.Region}");
            }
            Say($"P66: {regionMatched}/{regionTagged} tagged games with a cover got their own region's");

            File.WriteAllText(Path.Combine(outRoot, "log.txt"), log.ToString());
            int leaks = 0;
            foreach (string file in Directory.EnumerateFiles(outRoot, "*", SearchOption.AllDirectories))
            {
                byte[] bytes = File.ReadAllBytes(file);
                string text = Encoding.Latin1.GetString(bytes);
                if (text.Contains(developer.DevPassword, StringComparison.Ordinal) || text.Contains(developer.DevId, StringComparison.Ordinal)) leaks++;
            }
            Say($"P67: files holding a developer credential: {leaks}");
            File.WriteAllText(Path.Combine(outRoot, "log.txt"), log.ToString());
            Assert.Equal(0, leaks);
        }

        private static string Describe(ScrapeQuota? q) => q is null ? "(no quota fields)" :
            $"threads {q.MaxThreads}, {q.MaxDownloadKBps} KB/s, today {q.RequestsToday}/{q.MaxRequestsPerDay}, ko {q.RequestsKoToday}/{q.MaxRequestsKoPerDay}, per minute {q.MaxRequestsPerMinute}";

        private static string Stats(List<double> xs) => xs.Count == 0 ? "n=0" :
            $"n={xs.Count} median {xs.OrderBy(x => x).ElementAt(xs.Count / 2):F2}s range {xs.Min():F2}-{xs.Max():F2}s";
    }
}

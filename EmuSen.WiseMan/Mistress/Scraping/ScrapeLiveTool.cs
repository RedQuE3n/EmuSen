using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using Xunit.Abstractions;
using EmuSen.Galaxia.Library;

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

    // Stage (d)'s live run (P9, P10, P60-P67) on the library's scraper: 40 random files from the library, media and the scrape log into ~/.cache; the developer's file is read by the scraper, never by the test - see EmuSen_BigPicture.md §17 and EmuSen_Backend.md §40.5.
    public class ScrapeLiveTool(ITestOutputHelper output)
    {
        // A store under ~/.cache on the real server, the developer file read from the config directory by the scraper.
        private static NativeScraping Open(string outRoot, string? roms) => NativeScraping.Open(Path.Combine(outRoot, "Media"), JsonSerializer.Serialize(new Dictionary<string, object?>
        {
            ["roms"] = roms, ["developer_files"] = true, ["openemu"] = false, ["user_agent"] = "EmuSen/scrape-live",
            ["cores"] = RuntimeCatalog.Consoles.Select(c => c.Systems).ToArray(),
        }), null, null);

        // Every result of the run as it arrives, until it ends or three minutes pass with no change.
        private static List<JsonElement> Drain(NativeScraping scraper, Action<JsonElement> each)
        {
            var seen = new List<JsonElement>();
            long last = 0, serial = 0;
            var quiet = Stopwatch.StartNew();
            while (true)
            {
                using JsonDocument p = JsonDocument.Parse(scraper.Progress(0));
                foreach (JsonElement r in p.RootElement.GetProperty("results").EnumerateArray().Reverse().Where(r => r.GetProperty("serial").GetInt64() > last))
                {
                    last = r.GetProperty("serial").GetInt64();
                    seen.Add(r.Clone());
                    each(r);
                    quiet.Restart();
                }
                if (p.RootElement.GetProperty("state").GetString() != "running" || quiet.Elapsed > TimeSpan.FromMinutes(3)) return seen;
                serial = scraper.Wait(serial, 1000);
            }
        }

        private static string Text(JsonElement r, string name) => r.GetProperty(name).ValueKind == JsonValueKind.String ? r.GetProperty(name).GetString()! : "";

        [ScrapeLiveFact]
        public void Forty_random_files()
        {
            string library = Environment.GetEnvironmentVariable("EMUSEN_SCRAPE_ROMS") ?? "/home/red/Documents/Roms";
            int seed = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_SCRAPE_SEED"), out int s) ? s : 20260926;
            string outRoot = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "scrape-live", DateTime.Now.ToString("yyyyMMdd-HHmmss"));
            Directory.CreateDirectory(outRoot);
            var log = new StringBuilder();
            void Say(string line) { string safe = ScrapeRedactor.Redact(line); log.AppendLine(safe); output.WriteLine(safe); }

            // The library is read, never written: the scan lists, the hash reads, nothing else touches it.
            RomLibraryResult scan = RomLibrary.Scan(library);
            var candidates = scan.Entries.Where(e => RuntimeCatalog.ShelfByName(e.Shelf)?.EsdeSystem is { Length: > 0 }).ToList();
            var random = new Random(seed);
            var chosen = candidates.OrderBy(e => e.FullPath, StringComparer.Ordinal).OrderBy(_ => random.Next()).Take(40).ToList();
            Say($"library {library}: {scan.Entries.Count} files, {candidates.Count} on a shelf; seed {seed}; chosen {chosen.Count}");

            using NativeScraping scraper = Open(outRoot, library);
            var total = Stopwatch.StartNew();
            scraper.Start(chosen.Select(e => (e.FullPath, RuntimeCatalog.ShelfByName(e.Shelf)!.EsdeSystem)), ScrapePriority.Library, false, false, false, null, true, new ScrapeChoices());
            int n = 0;
            var finished = Drain(scraper, r => Say($"{++n,2} {Text(r, "outcome"),-8} {Text(r, "matched_by"),-11} cover={(r.GetProperty("has_cover").GetBoolean() ? "y" : "n")} media={r.GetProperty("kinds").GetArrayLength()} {Path.GetFileName(Text(r, "path"))} {Text(r, "detail")}"));
            total.Stop();

            int found = finished.Count(r => Text(r, "outcome") == "found");
            Say($"P9: {found}/{finished.Count} found; step 1 {finished.Count(r => Text(r, "matched_by") == "file")}, step 2 {finished.Count(r => Text(r, "matched_by") == "transformed")}; unknown {finished.Count(r => Text(r, "outcome") == "unknown")}, error {finished.Count(r => Text(r, "outcome") == "error")}; whole run {total.Elapsed.TotalSeconds:F0}s");
            QuotaSnapshot q = scraper.Quota();
            Say($"P61: requests today {q.RequestsToday}, ko {q.KoToday}; interval {q.Interval.TotalSeconds:F2}s");

            File.WriteAllText(Path.Combine(outRoot, "log.txt"), log.ToString());
            string[] secrets = DeveloperCredentials.Load() is { } d ? [d.DevId, d.DevPassword] : [];
            int leaks = Directory.EnumerateFiles(outRoot, "*", SearchOption.AllDirectories).Count(f => secrets.Any(x => Encoding.Latin1.GetString(File.ReadAllBytes(f)).Contains(x, StringComparison.Ordinal)));
            Say($"P67: files holding a developer credential: {leaks}");
            File.WriteAllText(Path.Combine(outRoot, "log.txt"), log.ToString());
            Assert.Equal(0, leaks);
        }

        // §46's live check: named files, "system=path" pairs separated by ';' in EMUSEN_SCRAPE_FILES, covers only.
        [ScrapeLiveFact]
        public void Named_files()
        {
            string pairs = Environment.GetEnvironmentVariable("EMUSEN_SCRAPE_FILES") ?? throw new InvalidOperationException("set EMUSEN_SCRAPE_FILES");
            var files = pairs.Split(';', StringSplitOptions.RemoveEmptyEntries).Select(p => p.Split('=', 2)).Select(p => (Path: p[1], System: p[0])).ToList();
            string outRoot = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "scrape-live", DateTime.Now.ToString("yyyyMMdd-HHmmss") + "-named");
            Directory.CreateDirectory(outRoot);
            var log = new StringBuilder();
            void Say(string line) { string safe = ScrapeRedactor.Redact(line); log.AppendLine(safe); output.WriteLine(safe); }

            using NativeScraping scraper = Open(outRoot, null);
            scraper.Start(files, ScrapePriority.Shown, false, false, false, null, true, new ScrapeChoices { Screenshots = false, Marquees = false, Miximages = false });
            var finished = Drain(scraper, r =>
            {
                ScrapedRecord? game = scraper.FoundFor(Text(r, "path"));
                Say($"{Text(r, "outcome"),-8} {Path.GetFileName(Text(r, "path"))}: game {game?.GameId} \"{game?.Name}\" by {Text(r, "matched_by")}; cover {(r.GetProperty("has_cover").GetBoolean() ? "kept" : "none")} {Text(r, "detail")}");
            });
            File.WriteAllText(Path.Combine(outRoot, "log.txt"), log.ToString());
            Assert.Equal(files.Count, finished.Count);
        }
    }
}

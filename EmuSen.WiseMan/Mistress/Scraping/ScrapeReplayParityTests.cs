using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading;
using EmuSen.Crystal.Client.Scrape;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Runs replayed, held to the C#'s answers recorded before its deletion: one fake ScreenScraper served at its real addresses, the C# Scraper and the Rust run each sent to it, and their requests compared byte for byte after the redactor, their stores row for row and file for file, their results and their clocks' waits - see EmuSen_Backend.md §30.7.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class ScrapeReplayParityTests : IDisposable
    {
        private const string Agent = "EmuSen/replay";
        private static readonly DateTimeOffset Start = new(2026, 7, 14, 10, 0, 0, TimeSpan.Zero);
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeReplay", Guid.NewGuid().ToString("N"));
        private readonly FakeScreenScraper _server = new();
        private readonly ReplayServer _replay;

        // Answers the test takes over before the fake server sees the request; each answers once.
        private readonly ConcurrentDictionary<string, int> _once = new(StringComparer.OrdinalIgnoreCase);

        private sealed class Once(ScrapeReplayParityTests test) : System.Net.Http.DelegatingHandler(test._server)
        {
            protected override System.Threading.Tasks.Task<System.Net.Http.HttpResponseMessage> SendAsync(System.Net.Http.HttpRequestMessage request, CancellationToken cancellationToken)
            {
                string md5 = FakeScreenScraper.Param(request.RequestUri!.AbsoluteUri, "md5");
                if (md5.Length > 0 && test._once.TryRemove(md5, out int code))
                    return System.Threading.Tasks.Task.FromResult(new System.Net.Http.HttpResponseMessage((System.Net.HttpStatusCode)code) { Content = new System.Net.Http.StringContent($"Erreur : once {code}") });
                return base.SendAsync(request, cancellationToken);
            }
        }

        public ScrapeReplayParityTests()
        {
            Assert.True(MediaService.Ready, MediaService.Report);
            Directory.CreateDirectory(Roms);
            _server.User!["maxdownloadspeed"] = "100000000";
            _replay = new ReplayServer(new Once(this));
        }

        public void Dispose()
        {
            _replay.Dispose();
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Roms => Path.Combine(_root, "Roms");

        private string Rom(string relative, byte[] bytes)
        {
            string path = Path.Combine(Roms, relative.Replace('/', Path.DirectorySeparatorChar));
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, bytes);
            // One modification time, so the path index's rows are the same on every run.
            File.SetLastWriteTimeUtc(path, Start.UtcDateTime);
            return path;
        }

        private static byte[] Filled(int length, int seed)
        {
            var bytes = new byte[length];
            new Random(seed).NextBytes(bytes);
            return bytes;
        }

        private static string Ticks(DateTimeOffset t) => (t.UtcTicks - DateTimeOffset.UnixEpoch.UtcTicks).ToString(CultureInfo.InvariantCulture);

        private static readonly byte[] Blob = ScrapeCredentialTests.Scramble(JsonSerializer.Serialize(new { devid = FakeScreenScraper.DevId, devpassword = FakeScreenScraper.DevPassword, softname = FakeScreenScraper.SoftName }));

        private string Settings() =>
            $"{{\"roms\":{JsonSerializer.Serialize(Roms)},\"developer_files\":false,\"openemu\":false,\"user_agent\":\"{Agent}\",\"cores\":[[\"nes\"],[\"snes\"],[\"gb\",\"gbc\"],[\"n64\"]]," +
            $"\"test\":{{\"connect\":\"{_replay.Address}\",\"roots\":{JsonSerializer.Serialize(ReplayServer.RootPem)},\"clock\":{Ticks(Start)}}}}}";

        private static string Pack(string path) => Path.GetExtension(path).ToLowerInvariant() switch { ".nes" => "nes", ".sfc" or ".smc" => "snes", ".gb" => "gb", ".gbc" => "gbc", _ => "n64" };

        // The requests as they came, header names in lower case, each line through the C#'s redactor; and that each carried the test's own credentials.
        private string Requests()
        {
            var lines = new List<string>();
            foreach (string head in _replay.Heads)
            {
                string first = head.Split("\r\n")[0];
                if (first.Contains("screenscraper.fr") || head.Contains("screenscraper.fr"))
                    Assert.True(first.Contains($"devid={FakeScreenScraper.DevId}&devpassword={FakeScreenScraper.DevPassword}&softname={FakeScreenScraper.SoftName}") || !first.Contains("/api2/jeu"), first);
                lines.Add(ScrapeRedactor.Redact(string.Join("\n", head.Split("\r\n").Select((l, i) => i == 0 || !l.Contains(':') ? l : l[..l.IndexOf(':')].ToLowerInvariant() + l[l.IndexOf(':')..]))));
            }
            return string.Join("\n---\n", lines);
        }

        private static string Dump(string db)
        {
            var sb = new StringBuilder();
            using var c = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = db, Mode = SqliteOpenMode.ReadOnly, Pooling = false }.ToString());
            c.Open();
            foreach (string t in new[] { "scrape_game", "scrape_media", "scrape_file", "scrape_queue", "quota_day", "store_move", "store_step" })
            {
                using var q = c.CreateCommand();
                q.CommandText = $"SELECT * FROM {t}";
                using var r = q.ExecuteReader();
                var rows = new List<string>();
                while (r.Read())
                    rows.Add(t + string.Concat(Enumerable.Range(0, r.FieldCount).Select(i => r.IsDBNull(i) ? " | null" : $" | {r.GetValue(i).GetType().Name}:{Convert.ToString(r.GetValue(i), CultureInfo.InvariantCulture)}" + (r.GetValue(i) is double d ? ":" + BitConverter.DoubleToInt64Bits(d) : ""))));
                foreach (string row in rows.Order(StringComparer.Ordinal)) sb.AppendLine(row);
            }
            return sb.ToString();
        }

        private static string Tree(string root) => string.Join("\n", Directory.EnumerateFiles(root, "*", SearchOption.AllDirectories)
            .Where(f => !Path.GetFileName(f).StartsWith(MediaStore.FileName, StringComparison.Ordinal))
            .Select(f => Path.GetRelativePath(root, f).Replace(Path.DirectorySeparatorChar, '/') + " " + Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(f)))[..16])
            .Order(StringComparer.Ordinal));

        private sealed record Side(string Requests, string Db, string Tree, string Results, string Delays);

        private static string Kinds(IEnumerable<string> written) => string.Join(",", written.Select(ScrapeProgress.KindOf));

        // The C# Scraper over a store and a clock of its own, its client sent to the replay server.
        private Side CSharp(string media, IReadOnlyList<(string Path, string System)> games, ScrapeChoices choices, ScrapePriority priority)
        {
            _replay.Clear();
            var clock = new FakeScrapeClock(Start);
            using MediaStore store = MediaStore.Open(media);
            store.PutInFolders(p => GameFolders.Of(Roms, p), Start);
            var quota = new ScrapeQuotaManager(store, clock);
            var results = new ConcurrentQueue<ScrapeResult>();
            using var finished = new ManualResetEventSlim();
            using var http = _replay.Client(Agent);
            var client = new ScreenScraperClient(http, FakeScreenScraper.Developer, null);
            using (var scraper = new Scraper(client, store, quota, () => choices, (_, _) => false, path => bytes => OpenVgdbBytes.Of(Pack(path) is "gbc" ? "gb" : Pack(path), bytes), results.Enqueue, clock)
            {
                IdlePoll = TimeSpan.FromMilliseconds(20), FolderOf = p => GameFolders.Of(Roms, p),
            })
            {
                scraper.Finished += _ => finished.Set();
                store.ClearQueue();
                foreach (var (path, system) in games) store.Enqueue(path, system, priority);
                scraper.Start();
                Assert.True(finished.Wait(TimeSpan.FromSeconds(60)), "the C# run did not end");
            }
            string said = string.Join("\n", results.Where(r => r.Outcome is not (ScrapeOutcome.Retry or ScrapeOutcome.Stopped))
                .Select(r => $"{r.Path} {r.Outcome.ToString().ToLowerInvariant()} [{Kinds(r.Written)}] {r.Detail} skipped={r.Skipped} cover={r.HasCover} by={r.MatchedBy}"));
            return new Side(Requests(), Dump(Path.Combine(media, MediaStore.FileName)), Tree(media), said, string.Join(",", clock.Delays.Select(d => d.Ticks)));
        }

        private static string ChoicesJson(ScrapeChoices c) => JsonSerializer.Serialize(new
        {
            kinds = c.Kinds().Select(k => k.EsdeType), region = c.Region, language = c.Language, region_fallback = c.RegionFallback, threads = c.Threads,
        });

        private Side Rust(string media, IReadOnlyList<(string Path, string System)> games, ScrapeChoices choices, ScrapePriority priority)
        {
            _replay.Clear();
            using MediaService rust = MediaService.Open(media, Settings(), embedded: Blob);
            string request = $"{{\"games\":{JsonSerializer.Serialize(games.Select(g => new { path = g.Path, system = g.System }))},\"priority\":\"{priority.ToString().ToLowerInvariant()}\",\"refresh\":{(choices.Refresh ? "true" : "false")},\"choices\":{ChoicesJson(choices)}}}";
            rust.Start(request);
            var deadline = DateTime.UtcNow.AddSeconds(60);
            long serial = 0;
            JsonDocument doc;
            while (true)
            {
                doc = JsonDocument.Parse(rust.Progress());
                if (doc.RootElement.GetProperty("state").GetString() != "running") break;
                Assert.True(DateTime.UtcNow < deadline, "the Rust run did not end");
                serial = rust.Wait(doc.RootElement.GetProperty("serial").GetInt64(), 500);
                doc.Dispose();
            }
            string said = string.Join("\n", doc.RootElement.GetProperty("results").EnumerateArray().Reverse().Select(r =>
                $"{r.GetProperty("path").GetString()} {r.GetProperty("outcome").GetString()} [{string.Join(",", r.GetProperty("kinds").EnumerateArray().Select(k => k.GetString()))}] {r.GetProperty("detail").GetString()} skipped={(r.GetProperty("skipped").GetBoolean() ? "True" : "False")} cover={(r.GetProperty("has_cover").GetBoolean() ? "True" : "False")} by={(r.GetProperty("matched_by").ValueKind == JsonValueKind.Null ? "" : r.GetProperty("matched_by").GetString())}"));
            doc.Dispose();
            string delays = string.Join(",", JsonSerializer.Deserialize<long[]>(rust.Get("{\"what\":\"delays\"}"))!);
            return new Side(Requests(), Dump(Path.Combine(media, MediaStore.FileName)), Tree(media), said, delays);
        }

        // The half the class runs on: the C# to record the goldens, the library otherwise.
        private Side Run(string media, IReadOnlyList<(string Path, string System)> games, ScrapeChoices choices, ScrapePriority priority) =>
            ScrapeGoldens.CSharp ? CSharp(media, games, choices, priority) : Rust(media, games, choices, priority);

        private void Hold(EmuSen.WiseMan.Crystal.ParityGolden golden, string name, Side side)
        {
            golden.Add($"{name} results", ScrapeGoldens.Plain(side.Results, _root));
            golden.Add($"{name} requests", ScrapeGoldens.Plain(side.Requests, _root));
            golden.Add($"{name} rows", ScrapeGoldens.Plain(side.Db, _root));
            golden.Add($"{name} tree", side.Tree);
            golden.Add($"{name} delays", side.Delays);
        }

        // A run from a copy of the seed store, held to the recording.
        private void Both(EmuSen.WiseMan.Crystal.ParityGolden golden, string name, IReadOnlyList<(string Path, string System)> games, ScrapeChoices choices, ScrapePriority priority = ScrapePriority.Library, string? seed = null)
        {
            string a = Path.Combine(_root, name, "store");
            if (seed is not null)
                foreach (string f in Directory.EnumerateFiles(seed, "*", SearchOption.AllDirectories))
                {
                    string target = Path.Combine(a, Path.GetRelativePath(seed, f));
                    Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                    File.Copy(f, target);
                }
            Hold(golden, name, Run(a, games, choices, priority));
        }

        [Fact]
        public void A_run_of_found_unknown_transformed_and_unreadable_games_replays_alike()
        {
            var golden = new EmuSen.WiseMan.Crystal.ParityGolden("ScrapeReplayParity.Mixed");
            var games = new List<(string, string)>();
            string Found(string relative, int seed, long id)
            {
                string rom = Rom(relative, Filled(2048 + seed, seed));
                _server.Games.Add(new FakeGame(id, Path.GetFileNameWithoutExtension(rom), RomHash.Of(rom).Md5));
                return rom;
            }
            games.Add((Found("snes/F-Zero (USA).sfc", 1, 77), "snes"));
            games.Add((Found("snes/RPG/Square/Mana (Japan).sfc", 2, 78), "snes"));
            games.Add((Found("snes/Pilotwings (Europe) (En,Fr,De).sfc", 3, 79), "snes"));
            games.Add((Rom("snes/Nobody Knows (USA).sfc", Filled(1500, 4)), "snes"));
            byte[] nes = SyntheticNesRom.Build(patches: (0x10, [1, 2, 3]));
            string headered = Rom("nes/Headered (USA).nes", nes);
            _server.Games.Add(new FakeGame(90, "Headered", RomHash.Of(nes[16..]).Md5));
            games.Add((headered, "nes"));
            string broken = Rom("snes/Broken Answer.sfc", Filled(900, 5));
            _server.StatusByMd5[RomHash.Of(broken).Md5] = (200, "<html>not json</html>");
            games.Add((broken, "snes"));
            string refused = Rom("snes/Refused.sfc", Filled(901, 6));
            _server.StatusByMd5[RomHash.Of(refused).Md5] = (400, "Erreur : bad request");
            games.Add((refused, "snes"));
            Both(golden, "mixed", games, new ScrapeChoices());
            golden.Check(5);
        }

        [Fact]
        public void Every_kind_in_another_region_and_language_with_the_fallback_off_replays_alike()
        {
            var golden = new EmuSen.WiseMan.Crystal.ParityGolden("ScrapeReplayParity.Kinds");
            var games = new List<(string, string)>();
            for (int i = 0; i < 3; i++)
            {
                string rom = Rom($"snes/Kinds {i} ({(i == 0 ? "Japan" : i == 1 ? "USA" : "Germany")}).sfc", Filled(1024 + i, 20 + i));
                _server.Games.Add(new FakeGame(200 + i, $"Kinds {i}", RomHash.Of(rom).Md5) { Media = FakeGame.AllMedia });
                games.Add((rom, "snes"));
            }
            var choices = new ScrapeChoices
            {
                TitleScreens = true, BackCovers = true, Boxes3D = true, PhysicalMedia = true, FanArt = true, Manuals = true, Videos = true,
                Region = "jp", Language = "fr", RegionFallback = false,
            };
            Both(golden, "kinds", games, choices, ScrapePriority.Console);
            Both(golden, "kinds-auto", games, new ScrapeChoices { Videos = true, Manuals = true, Language = "de" }, ScrapePriority.Shown);
            golden.Check(2 * 5);
        }

        [Fact]
        public void A_second_run_refreshes_skips_and_follows_a_rename_alike()
        {
            var golden = new EmuSen.WiseMan.Crystal.ParityGolden("ScrapeReplayParity.Again");
            string a = Rom("snes/Again (USA).sfc", Filled(3000, 40)), b = Rom("snes/Moved (USA).sfc", Filled(3001, 41));
            _server.Games.Add(new FakeGame(300, "Again", RomHash.Of(a).Md5));
            _server.Games.Add(new FakeGame(301, "Moved", RomHash.Of(b).Md5));
            string seed = Path.Combine(_root, "seed");
            var games = new List<(string, string)> { (a, "snes"), (b, "snes") };
            Hold(golden, "seed", Run(seed, games, new ScrapeChoices(), ScrapePriority.Library));
            string renamed = Path.Combine(Roms, "snes", "Sub", "Renamed (USA).sfc");
            Directory.CreateDirectory(Path.GetDirectoryName(renamed)!);
            File.Move(b, renamed);
            _server.MediaRevision["ss"] = 1;
            Both(golden, "again", [(a, "snes"), (renamed, "snes")], new ScrapeChoices { Refresh = true, TitleScreens = true }, seed: seed);
            Both(golden, "again-plain", [(a, "snes"), (renamed, "snes")], new ScrapeChoices(), seed: seed);
            golden.Check(3 * 5);
        }

        [Theory]
        [InlineData(430)]
        [InlineData(429)]
        [InlineData(401)]
        [InlineData(403)]
        public void A_status_that_stops_or_slows_the_run_replays_alike(int code)
        {
            var golden = new EmuSen.WiseMan.Crystal.ParityGolden($"ScrapeReplayParity.Status{code}");
            var games = new List<(string, string)>();
            for (int i = 0; i < 3; i++)
            {
                string rom = Rom($"snes/Status {code} {i}.sfc", Filled(700 + i, 60 + i));
                _server.Games.Add(new FakeGame(400 + i, $"Status {i}", RomHash.Of(rom).Md5));
                games.Add((rom, "snes"));
            }
            string md5 = RomHash.Of(games[1].Item1).Md5;
            if (code is 429 or 401)
            {
                _once[md5] = code;
                Hold(golden, $"status{code}", Run(Path.Combine(_root, $"status{code}", "store"), games, new ScrapeChoices(), ScrapePriority.Library));
                golden.Check(5);
                return;
            }
            _server.StatusByMd5[md5] = (code, $"Erreur : forced {code}");
            Both(golden, $"status{code}", games, new ScrapeChoices());
            golden.Check(5);
        }

        [Fact]
        public void The_failover_alone_asks_openvgdb_and_the_thumbnail_server_alike()
        {
            string home = Path.Combine(_root, "home");
            DataStore.OverrideDirectory = home;
            try
            {
                byte[] nes = SyntheticNesRom.Build(patches: (0x10, [9, 9]));
                string headered = Rom("nes/Odd Name.nes", nes);
                string byName = Rom("snes/Named Game (USA).sfc", Filled(4096, 70));
                string unknown = Rom("snes/Nobody (USA).sfc", Filled(4096, 71));
                string noArt = Rom("snes/Bare (USA).sfc", Filled(4096, 72));
                OnlineCoverTests.BuildOpenVgdb(OpenVgdb.DefaultPath,
                    ("SNES", "Named Game (USA)", "00", null), ("SNES", "Bare (USA)", "01", null),
                    ("NES", "Real Name (USA) (Rev 1)", Convert.ToHexString(MD5.HashData(nes[16..])), null));
                _server.Other = url => url.Contains("thumbnails.libretro.com") && !url.Contains("Bare") && (url.Contains("Named%20Game") || url.Contains("Real%20Name%20%28USA%29.png"))
                    ? OnlineCoverTests.FakeServer.Png() : new System.Net.Http.HttpResponseMessage(System.Net.HttpStatusCode.NotFound);
                var games = new List<(string Path, string System)> { (byName, "snes"), (headered, "nes"), (unknown, "snes"), (noArt, "snes") };

                var golden = new EmuSen.WiseMan.Crystal.ParityGolden("ScrapeReplayParity.Failover");
                _replay.Clear();
                string a = Path.Combine(_root, "failover", "store");
                string counts;
                if (ScrapeGoldens.CSharp)
                {
                    var results = new ConcurrentQueue<CoverResult>();
                    using (var http = _replay.Client(Agent))
                    using (var fetcher = new CoverFetcher(http, OpenVgdb.DefaultPath, Path.Combine(a, "openemu"), results.Enqueue))
                    {
                        foreach (var (path, _) in games) fetcher.Enqueue(new RomEntry(path), RuntimeCatalog.CoreForRom(path)!);
                        SpinWait.SpinUntil(() => results.Count == games.Count, TimeSpan.FromSeconds(30));
                    }
                    Assert.True(results.Count(r => r.Saved is not null) == 2, string.Join("; ", results.Select(r => $"{Path.GetFileName(r.RomPath)} {r.Outcome} {r.RomName} {r.Detail}")));
                    counts = $"asked {results.Count} found {results.Count(r => r.Saved is not null)} state stopped";
                }
                else
                {
                    using MediaService rust = MediaService.Open(a, Settings().Replace("\"openemu\":false", "\"openemu\":true"), embedded: Blob);
                    rust.Start($"{{\"games\":{JsonSerializer.Serialize(games.Select(g => new { path = g.Path, system = g.System }))},\"enabled\":false}}");
                    var deadline = DateTime.UtcNow.AddSeconds(30);
                    while (JsonDocument.Parse(rust.Progress()).RootElement.GetProperty("state").GetString() == "running" && DateTime.UtcNow < deadline) Thread.Sleep(50);
                    using JsonDocument done = JsonDocument.Parse(rust.Progress());
                    counts = $"asked {done.RootElement.GetProperty("failover").GetProperty("asked").GetInt32()} found {done.RootElement.GetProperty("failover").GetProperty("found").GetInt32()} state {done.RootElement.GetProperty("state").GetString()}";
                }
                golden.Add("counts", counts);
                golden.Add("requests", ScrapeGoldens.Plain(Requests(), _root));
                string tree = Directory.Exists(Path.Combine(a, "openemu")) ? TreeUnder(a) : "";
                golden.Add("tree", tree);
                Assert.NotEqual("", tree);
                golden.Check(3);
            }
            finally
            {
                DataStore.OverrideDirectory = null;
            }
        }

        // A packaged game's art goes into its package in both halves, the store keeping none of it; each half's package compared, emptied between them - see EmuSen_GamePackages.md §18.3.
        [Fact]
        public void A_packaged_game_s_art_is_written_into_its_package_alike()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            try
            {
                new EmuSen.Galaxia.Models.AppSettings { RomDirectory = Roms }.Save();
                string rom = Rom("snes/Packed (USA).sfc", Filled(2500, 50)), plain = Rom("snes/Plain (USA).sfc", Filled(2501, 51));
                _server.Games.Add(new FakeGame(400, "Packed", RomHash.Of(rom).Md5));
                _server.Games.Add(new FakeGame(401, "Plain", RomHash.Of(plain).Md5));
                string package = Path.Combine(DataStore.Packages, "SNES", "Packed (USA).sfc");
                EmuSen.WiseMan.Crystal.CrystalParityTests.PackageFor(package, Roms, "snes/Packed (USA).sfc", "snes");
                string media = Path.Combine(package, EmuSen.Galaxia.Media.GamePackage.MediaFolder);
                var games = new List<(string, string)> { (rom, "snes"), (plain, "snes") };
                var golden = new EmuSen.WiseMan.Crystal.ParityGolden("ScrapeReplayParity.Packaged");
                Side side = Run(Path.Combine(_root, "packed", "store"), games, new ScrapeChoices(), ScrapePriority.Library);
                string packaged = Tree(media);
                golden.Add("results", ScrapeGoldens.Plain(side.Results, _root));
                golden.Add("rows", ScrapeGoldens.Plain(side.Db, _root));
                golden.Add("tree", side.Tree);
                golden.Add("package", packaged);
                Assert.Contains("covers/Packed (USA).png", packaged);
                Assert.DoesNotContain("Packed", side.Tree);
                Assert.Contains("Plain (USA).png", side.Tree);
                golden.Check(4);
            }
            finally
            {
                DataStore.OverrideDirectory = null;
                ConfigStore.OverrideDirectory = null;
            }
        }

        // The store's failover covers alone, by their path below openemu/.
        private static string TreeUnder(string store) => Tree(Path.Combine(store, "openemu"));
    }
}

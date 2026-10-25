using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading;
using EmuSen.Cores;
using EmuSen.Mistress.Scraping;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The queue's workers against the fake server: §5.2's identity steps, the media kept, the stops, resuming, renames and threads - see EmuSen_BigPicture.md §17.
    public class ScraperTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScraperTests", Guid.NewGuid().ToString("N"));
        private readonly FakeScreenScraper _server = new();
        private readonly FakeScrapeClock _clock = new(new DateTimeOffset(2026, 7, 14, 10, 0, 0, TimeSpan.Zero));
        private readonly BlockingCollection<ScrapeResult> _results = new();
        private readonly List<IDisposable> _open = new();
        private MediaStore _store;
        private ScrapeQuotaManager _quota;

        private string Roms => Path.Combine(_root, "Roms");
        private string Media => Path.Combine(_root, "Media");

        public ScraperTests()
        {
            Directory.CreateDirectory(Roms);
            _store = MediaStore.Open(Media);
            _quota = new ScrapeQuotaManager(_store, _clock);
        }

        public void Dispose()
        {
            foreach (IDisposable d in Enumerable.Reverse(_open)) d.Dispose();
            _store.Dispose();
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        // The fake server's answers with their Content-Type's charset replaced, as a server misconfigured would send them.
        private sealed class Charset(HttpMessageHandler inner, string charset) : DelegatingHandler(inner)
        {
            protected override async System.Threading.Tasks.Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
            {
                HttpResponseMessage response = await base.SendAsync(request, cancellationToken);
                if (response.Content.Headers.ContentType is { } type) type.CharSet = charset;
                return response;
            }
        }

        private HttpMessageHandler? _handler;

        private Scraper Start(ScrapeChoices? choices = null, Func<string, string, bool>? handCover = null, bool start = true, Func<string, string>? folderOf = null, Func<string, string?>? packageOf = null)
        {
            var client = new ScreenScraperClient(new HttpClient(_handler ?? _server), FakeScreenScraper.Developer, null);
            ScrapeChoices chosen = choices ?? new ScrapeChoices();
            var scraper = new Scraper(client, _store, _quota, () => chosen, handCover ?? ((_, _) => false),
                path => CoreCatalog.ByExtension(Path.GetExtension(path))?.OpenVgdbBytes, _results.Add, _clock)
            {
                IdlePoll = TimeSpan.FromMilliseconds(20), FolderOf = folderOf ?? (_ => ""), PackageOf = packageOf ?? (_ => null),
            };
            _open.Add(scraper);
            if (start) scraper.Start();
            return scraper;
        }

        // A game queued and the run started, as the player's Scrape does; Start is a no-op while a run is live.
        private static void Queue(Scraper scraper, string path, string system, ScrapePriority priority)
        {
            scraper.Enqueue(path, system, priority);
            scraper.Start();
        }

        private string Rom(string name, byte[] bytes)
        {
            string path = Path.Combine(Roms, name);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        private static byte[] Filled(int length, int seed)
        {
            var bytes = new byte[length];
            new Random(seed).NextBytes(bytes);
            return bytes;
        }

        private ScrapeResult Next()
        {
            Assert.True(_results.TryTake(out ScrapeResult? r, TimeSpan.FromSeconds(10)), "no result");
            return r!;
        }

        private void NothingMore()
        {
            Thread.Sleep(80);
            Assert.Empty(_results);
        }

        private string Stored(params string[] parts) => Path.Combine([Media, .. parts]);

        [Fact]
        public void An_answer_in_a_charset_dotnet_has_not_is_an_answer_that_could_not_be_read_and_the_run_goes_on()
        {
            string rom = Rom("Charset (USA).sfc", Filled(512, 9));
            _server.Games.Add(new FakeGame(78, "Charset", RomHash.Of(rom).Md5));
            _handler = new Charset(_server, "klingon");
            Scraper scraper = Start();

            Queue(scraper, rom, "snes", ScrapePriority.Shown);
            ScrapeResult result = Next();

            Assert.Equal(ScrapeOutcome.Retry, result.Outcome);
            Assert.Contains("character set", result.Detail);
            Assert.True(scraper.IsRunning);
        }

        // A patched package's manifest beside the test's games: its patched image's hashes recorded, and the identity the scraper sends.
        private string PatchedPackage(string name, byte[] patchedImage, string identity)
        {
            string folder = Path.Combine(_root, "Packages", name);
            Directory.CreateDirectory(folder);
            RomHashes t = RomHash.Of(patchedImage);
            var p = EmuSen.Galaxia.Media.GamePackage.Create(name, "snes", "linked", new EmuSen.Galaxia.Media.PackageFile("image", "library", "snes/" + name));
            p.Document["patches"] = new System.Text.Json.Nodes.JsonArray(new System.Text.Json.Nodes.JsonObject { ["file"] = "a.bps", ["format"] = "bps" });
            p.Document["patched"] = new System.Text.Json.Nodes.JsonObject { ["size"] = t.Size, ["crc32"] = t.Crc32, ["md5"] = t.Md5, ["sha1"] = t.Sha1 };
            p.Document["identity"] = identity;
            Assert.True(p.Write(Path.Combine(folder, EmuSen.Galaxia.Media.GamePackage.FileName)));
            return folder;
        }

        [Fact]
        public void A_patched_package_is_scraped_by_its_dump_unless_its_identity_is_patched_and_then_by_the_patched_image_first()
        {
            byte[] dump = Filled(2048, 61), hack = Filled(2304, 62), base2 = Filled(2048, 63);
            string original = Rom("Original (USA).sfc", dump), hacked = Rom("Hacked (USA).sfc", base2);
            RomHashes d = RomHash.Of(dump), h = RomHash.Of(hack), d2 = RomHash.Of(base2);
            var packages = new Dictionary<string, string>
            {
                [original] = PatchedPackage("Original (USA).sfc", hack, "original"),
                [hacked] = PatchedPackage("Hacked (USA).sfc", hack, "patched"),
            };
            _server.Games.Add(new FakeGame(81, "Original", d.Md5));
            _server.Games.Add(new FakeGame(82, "Hacked base", d2.Md5));
            Scraper scraper = Start(packageOf: p => packages.GetValueOrDefault(p));

            Queue(scraper, original, "snes", ScrapePriority.Shown);
            Assert.Equal(ScrapeOutcome.Found, Next().Outcome);
            string asked = _server.JeuInfos.Single();
            Assert.Equal((d.Md5, d.Crc32, d.Sha1, "2048"), (FakeScreenScraper.Param(asked, "md5"), FakeScreenScraper.Param(asked, "crc"), FakeScreenScraper.Param(asked, "sha1"), FakeScreenScraper.Param(asked, "romtaille")));

            Queue(scraper, hacked, "snes", ScrapePriority.Shown);
            ScrapeResult second = Next();
            Assert.Equal(ScrapeOutcome.Found, second.Outcome);
            string[] both = _server.JeuInfos.Skip(1).ToArray();
            Assert.Equal(2, both.Length);
            Assert.Equal((h.Md5, h.Sha1, "2304"), (FakeScreenScraper.Param(both[0], "md5"), FakeScreenScraper.Param(both[0], "sha1"), FakeScreenScraper.Param(both[0], "romtaille")));
            Assert.Equal(d2.Md5, FakeScreenScraper.Param(both[1], "md5"));
            Assert.Equal("file", second.MatchedBy);
        }

        [Fact]
        public void A_game_found_by_its_file_s_hashes_gets_its_text_and_each_default_kind_in_es_de_s_layout()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(4096, 1));
            RomHashes h = RomHash.Of(rom);
            _server.Games.Add(new FakeGame(77, "F-Zero", h.Md5));
            Scraper scraper = Start();

            Queue(scraper, rom, "snes", ScrapePriority.Shown);
            ScrapeResult result = Next();

            Assert.Equal(ScrapeOutcome.Found, result.Outcome);
            Assert.True(result.HasCover);
            string asked = _server.JeuInfos.Single();
            Assert.Equal((h.Md5, h.Crc32, h.Sha1, "4096", "F-Zero (USA).sfc", "4"),
                (FakeScreenScraper.Param(asked, "md5"), FakeScreenScraper.Param(asked, "crc"), FakeScreenScraper.Param(asked, "sha1"),
                 FakeScreenScraper.Param(asked, "romtaille"), FakeScreenScraper.Param(asked, "romnom"), FakeScreenScraper.Param(asked, "systemeid")));
            foreach (string folder in (string[])["covers", "screenshots", "marquees", "miximages"])
                Assert.True(File.Exists(Stored("snes", folder, "F-Zero (USA).png")), folder);
            Assert.False(Directory.Exists(Stored("snes", "titlescreens")));
            Assert.Contains("media=box-2D(us)", string.Join(" ", _server.MediaAsked));
            Assert.Contains("media=wheel-hd(wor)", string.Join(" ", _server.MediaAsked));
            Assert.Contains("media=mixrbv2(wor)", string.Join(" ", _server.MediaAsked));
            Assert.DoesNotContain("media=wheel(wor)", string.Join(" ", _server.MediaAsked));

            ScrapedRecord record = _store.FoundByPath()[rom];
            Assert.Equal(("A synthetic game written for the tests.", "Synthetic Developer", "Synthetic Publisher", "Racing", "1-2", 0.8f, new DateTime(1991, 8, 23)),
                (record.Description, record.Developer, record.Publisher, record.Genre, record.Players, record.Rating, record.ReleaseDate));
            Assert.Equal("F-Zero (US title)", record.Name);
            Assert.Equal(4, _store.Media(h.Md5, h.Size).Count);
            Assert.Equal(0, _store.QueueLength);
        }

        [Fact]
        public void A_queue_left_by_an_earlier_run_asks_nothing_until_a_run_is_started_and_the_run_ends_when_it_is_empty()
        {
            string a = Rom("A (USA).sfc", Filled(1024, 80));
            string b = Rom("B (USA).sfc", Filled(1024, 81));
            _store.Enqueue(a, "snes", ScrapePriority.Library);
            _store.Enqueue(b, "snes", ScrapePriority.Library);
            Scraper scraper = Start(start: false);
            var ends = new BlockingCollection<ScrapeRunEnd>();
            scraper.Finished += ends.Add;
            Thread.Sleep(300);
            Assert.Empty(_server.Asked);
            Assert.False(scraper.IsRunning);

            scraper.Start();
            Next();
            Next();
            Assert.True(ends.TryTake(out ScrapeRunEnd end, TimeSpan.FromSeconds(10)));
            Assert.Equal(ScrapeRunEnd.Done, end);
            Assert.False(scraper.IsRunning);
            int asked = _server.Asked.Count;

            // A game queued after the run ended waits for the next run.
            scraper.Enqueue(Rom("C (USA).sfc", Filled(1024, 82)), "snes", ScrapePriority.Shown);
            Thread.Sleep(300);
            Assert.Equal(asked, _server.Asked.Count);
        }

        [Fact]
        public void A_run_the_quota_stops_ends_as_stopped_and_cancelling_ends_it_as_cancelled()
        {
            _server.ForcedStatus = 430;
            Scraper scraper = Start(start: false);
            var ends = new BlockingCollection<ScrapeRunEnd>();
            scraper.Finished += ends.Add;
            Queue(scraper, Rom("A (USA).sfc", Filled(1024, 83)), "snes", ScrapePriority.Shown);
            Assert.True(ends.TryTake(out ScrapeRunEnd end, TimeSpan.FromSeconds(10)));
            Assert.Equal(ScrapeRunEnd.Stopped, end);

            _server.ForcedStatus = 0;
            _store.Dispose();
            _store = MediaStore.Open(Path.Combine(_root, "Other"));
            _quota = new ScrapeQuotaManager(_store, _clock);
            _server.Gate = new SemaphoreSlim(0);
            Scraper second = Start(start: false);
            second.Finished += ends.Add;
            Queue(second, Rom("B (USA).sfc", Filled(1024, 84)), "snes", ScrapePriority.Shown);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!_server.JeuInfos.Any() && clock.ElapsedMilliseconds < 10_000) Thread.Sleep(5);
            second.Dispose();
            Assert.True(ends.TryTake(out end, TimeSpan.FromSeconds(10)));
            Assert.Equal(ScrapeRunEnd.Cancelled, end);
            Assert.Equal(1, _store.QueueLength);
        }

        [Fact]
        public void The_headerless_hash_is_asked_only_after_a_404_and_only_when_the_core_s_transform_changes_the_bytes()
        {
            byte[] body = Filled(0x4000, 2);
            byte[] ines = new byte[] { (byte)'N', (byte)'E', (byte)'S', 0x1A }.Concat(new byte[12]).Concat(body).ToArray();
            string nes = Rom("Metroid (USA).nes", ines);
            string gb = Rom("Unknown Game (USA).gb", Filled(0x8000, 3));
            _server.Games.Add(new FakeGame(5, "Metroid", RomHash.Of(body).Md5) { SystemId = 3 });
            Scraper scraper = Start();

            Queue(scraper, nes, "nes", ScrapePriority.Shown);
            ScrapeResult found = Next();
            Assert.Equal((ScrapeOutcome.Found, "transformed"), (found.Outcome, found.MatchedBy));
            Assert.Equal([RomHash.Of(ines).Md5, RomHash.Of(body).Md5], _server.JeuInfos.Select(u => FakeScreenScraper.Param(u, "md5")));
            Assert.Equal(body.Length.ToString(), FakeScreenScraper.Param(_server.JeuInfos.Last(), "romtaille"));

            int before = _server.JeuInfos.Count();
            Queue(scraper, gb, "gb", ScrapePriority.Shown);
            Assert.Equal(ScrapeOutcome.Unknown, Next().Outcome);
            Assert.Equal(before + 1, _server.JeuInfos.Count());
            Assert.DoesNotContain(_server.Asked, u => u.Contains("jeuRecherche"));
        }

        [Fact]
        public void A_cover_the_player_already_has_is_not_fetched_and_a_file_already_there_is_not_replaced()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(2048, 4));
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            Directory.CreateDirectory(Stored("snes", "screenshots"));
            File.WriteAllBytes(Stored("snes", "screenshots", "F-Zero (USA).png"), [7, 7, 7]);
            Scraper scraper = Start(handCover: (system, path) => path.EndsWith("F-Zero (USA).sfc"));

            Queue(scraper, rom, "snes", ScrapePriority.Shown);
            ScrapeResult result = Next();

            Assert.DoesNotContain(_server.MediaAsked, u => u.Contains("box-2D"));
            Assert.DoesNotContain(_server.MediaAsked, u => u.Contains("media=ss("));
            Assert.False(File.Exists(Stored("snes", "covers", "F-Zero (USA).png")));
            Assert.Equal(new byte[] { 7, 7, 7 }, File.ReadAllBytes(Stored("snes", "screenshots", "F-Zero (USA).png")));
            Assert.False(result.HasCover);
        }

        [Fact]
        public void A_game_answered_before_costs_no_request_and_an_unknown_one_is_not_asked_again()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(2048, 5));
            string hack = Rom("My Hack.sfc", Filled(2048, 6));
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            Scraper scraper = Start();
            Queue(scraper, rom, "snes", ScrapePriority.Shown);
            Queue(scraper, hack, "snes", ScrapePriority.Shown);
            Next();
            Next();
            int asked = _server.Asked.Count;

            Queue(scraper, rom, "snes", ScrapePriority.Shown);
            Queue(scraper, hack, "snes", ScrapePriority.Shown);
            Assert.Equal([ScrapeOutcome.Found, ScrapeOutcome.Unknown], new[] { Next(), Next() }.OrderBy(r => r.Outcome).Select(r => r.Outcome));
            Assert.Equal(asked, _server.Asked.Count);
        }

        [Fact]
        public void A_kind_turned_on_later_is_asked_for_only_when_the_game_offered_it()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(2048, 7));
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            Scraper scraper = Start();
            Queue(scraper, rom, "snes", ScrapePriority.Shown);
            Next();
            scraper.Dispose();

            Scraper titles = Start(new ScrapeChoices { TitleScreens = true });
            Queue(titles, rom, "snes", ScrapePriority.Shown);
            Next();
            Assert.Equal(2, _server.JeuInfos.Count());
            Assert.True(File.Exists(Stored("snes", "titlescreens", "F-Zero (USA).png")));
        }

        [Fact]
        public void A_kind_turned_on_later_that_the_game_never_offered_costs_no_request()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(2048, 70));
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5) { Media = [("box-2D", "us", "png"), ("ss", "wor", "png")] });
            Scraper scraper = Start();
            Queue(scraper, rom, "snes", ScrapePriority.Shown);
            Next();
            scraper.Dispose();

            Scraper titles = Start(new ScrapeChoices { TitleScreens = true });
            Queue(titles, rom, "snes", ScrapePriority.Shown);
            Assert.Equal(ScrapeOutcome.Found, Next().Outcome);
            Assert.Single(_server.JeuInfos);
        }

        [Fact]
        public void A_renamed_file_takes_its_media_with_it_and_a_second_copy_gets_its_own_with_no_request()
        {
            byte[] bytes = Filled(2048, 8);
            string rom = Rom("F-Zero (USA).sfc", bytes);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            Scraper scraper = Start();
            Queue(scraper, rom, "snes", ScrapePriority.Shown);
            Next();
            int asked = _server.Asked.Count;

            string renamed = Path.Combine(Roms, "F-Zero.sfc");
            File.Move(rom, renamed);
            Queue(scraper, renamed, "snes", ScrapePriority.Shown);
            Assert.Equal(ScrapeOutcome.Found, Next().Outcome);
            Assert.True(File.Exists(Stored("snes", "covers", "F-Zero.png")));
            Assert.False(File.Exists(Stored("snes", "covers", "F-Zero (USA).png")));

            string copy = Rom("F-Zero (copy).sfc", bytes);
            Queue(scraper, copy, "snes", ScrapePriority.Shown);
            Assert.Equal(ScrapeOutcome.Found, Next().Outcome);
            Assert.True(File.Exists(Stored("snes", "covers", "F-Zero (copy).png")));
            Assert.True(File.Exists(Stored("snes", "covers", "F-Zero.png")));
            Assert.Equal(asked, _server.Asked.Count);
        }

        // ES-DE keeps a foldered game's media under its folder (USERGUIDE, "Manually copying game media files"); a file moved to another folder takes them there (§30).
        [Fact]
        public void A_game_in_a_folder_keeps_its_media_under_that_folder_and_moved_to_another_folder_takes_them_with_it()
        {
            Directory.CreateDirectory(Path.Combine(Roms, "SNES", "USA"));
            Directory.CreateDirectory(Path.Combine(Roms, "SNES", "Racing"));
            string rom = Rom(Path.Combine("SNES", "USA", "F-Zero (USA).sfc"), Filled(2048, 21));
            RomHashes h = RomHash.Of(rom);
            _server.Games.Add(new FakeGame(77, "F-Zero", h.Md5));
            Scraper scraper = Start(folderOf: p => EmuSen.Mistress.Library.GameFolders.Of(Roms, p));
            Queue(scraper, rom, "snes", ScrapePriority.Shown);
            ScrapeResult first = Next();

            string[] kinds = ["covers", "screenshots", "marquees", "miximages"];
            foreach (string folder in kinds) Assert.True(File.Exists(Stored("snes", folder, "USA", "F-Zero (USA).png")), folder);
            Assert.False(File.Exists(Stored("snes", "covers", "F-Zero (USA).png")));
            Assert.All(_store.Media(h.Md5, h.Size), m => Assert.Equal("USA", MediaStore.FolderOf(m.RelativePath)));
            Assert.Equal(["cover", "marquee", "miximage", "screenshot"], first.Written.Select(ScrapeProgress.KindOf).Order());
            int asked = _server.Asked.Count;

            string moved = Path.Combine(Roms, "SNES", "Racing", "F-Zero (USA).sfc");
            File.Move(rom, moved);
            Queue(scraper, moved, "snes", ScrapePriority.Shown);
            Assert.Equal(ScrapeOutcome.Found, Next().Outcome);
            foreach (string folder in kinds)
            {
                Assert.True(File.Exists(Stored("snes", folder, "Racing", "F-Zero (USA).png")), folder);
                Assert.False(File.Exists(Stored("snes", folder, "USA", "F-Zero (USA).png")), folder);
            }
            Assert.Equal(asked, _server.Asked.Count);
        }

        [Fact]
        public void A_european_file_gets_the_european_box_and_a_region_the_game_lacks_falls_back_in_order()
        {
            string eu = Rom("Game (Europe).sfc", Filled(1024, 9));
            string kr = Rom("Game (Korea).sfc", Filled(1024, 10));
            _server.Games.Add(new FakeGame(1, "Game", RomHash.Of(eu).Md5, RomHash.Of(kr).Md5)
            {
                Media = [("box-2D", "us", "png"), ("box-2D", "eu", "jpg"), ("box-2D", "jp", "png")],
            });
            Scraper scraper = Start(new ScrapeChoices { Screenshots = false, Marquees = false, Miximages = false });
            Queue(scraper, eu, "snes", ScrapePriority.Shown);
            Next();
            Queue(scraper, kr, "snes", ScrapePriority.Shown);
            Next();

            Assert.Equal(["box-2D(eu)", "box-2D(us)"], _server.MediaAsked.Select(u => FakeScreenScraper.Param(u, "media")));
            Assert.True(File.Exists(Stored("snes", "covers", "Game (Europe).jpg")));
            Assert.True(File.Exists(Stored("snes", "covers", "Game (Korea).png")));
        }

        [Theory]
        [InlineData(430)]
        [InlineData(431)]
        [InlineData(423)]
        [InlineData(426)]
        [InlineData(403)]
        public void A_stopping_answer_leaves_the_game_queued_and_asks_nothing_more(int code)
        {
            string a = Rom("A (USA).sfc", Filled(1024, 11));
            string b = Rom("B (USA).sfc", Filled(1024, 12));
            _server.ForcedStatus = code;
            Scraper scraper = Start();
            Queue(scraper, a, "snes", ScrapePriority.Shown);
            Queue(scraper, b, "snes", ScrapePriority.Shown);

            ScrapeResult result = Next();
            Assert.Equal(ScrapeOutcome.Stopped, result.Outcome);
            Assert.Contains(code.ToString(), result.Detail);
            NothingMore();
            Assert.Single(_server.JeuInfos);
            Assert.Equal(2, _store.QueueLength);
            Assert.False(_quota.Usable);
        }

        [Fact]
        public void After_the_day_s_stop_the_next_day_resumes_where_it_stopped_even_after_a_restart()
        {
            string a = Rom("A (USA).sfc", Filled(1024, 13));
            string b = Rom("B (USA).sfc", Filled(1024, 14));
            _server.Games.Add(new FakeGame(1, "A", RomHash.Of(a).Md5));
            _server.Games.Add(new FakeGame(2, "B", RomHash.Of(b).Md5));
            _server.ForcedStatus = 430;
            Scraper scraper = Start();
            Queue(scraper, a, "snes", ScrapePriority.Shown);
            Queue(scraper, b, "snes", ScrapePriority.Library);
            Assert.Equal(ScrapeOutcome.Stopped, Next().Outcome);
            scraper.Dispose();
            _store.Dispose();

            _server.ForcedStatus = 0;
            _store = MediaStore.Open(Media);
            _quota = new ScrapeQuotaManager(_store, _clock);
            Assert.False(_quota.Usable);
            _clock.Now = new DateTimeOffset(2026, 7, 14, 22, 0, 1, TimeSpan.Zero);
            Start();

            Assert.Equal([a, b], new[] { Next(), Next() }.Select(r => r.Path));
            Assert.Equal(0, _store.QueueLength);
        }

        [Fact]
        public void An_interrupted_queue_resumes_without_asking_again_for_what_was_done()
        {
            var roms = Enumerable.Range(0, 4).Select(i => Rom($"G{i} (USA).sfc", Filled(1024, 20 + i))).ToList();
            foreach ((string rom, int i) in roms.Select((r, i) => (r, i))) _server.Games.Add(new FakeGame(100 + i, $"G{i}", RomHash.Of(rom).Md5));
            _server.Gate = new SemaphoreSlim(0);
            Scraper scraper = Start(new ScrapeChoices { Screenshots = false, Marquees = false, Miximages = false });
            foreach (string rom in roms) scraper.Enqueue(rom, "snes", ScrapePriority.Library);
            scraper.Start();
            _server.Gate.Release(2);
            Assert.Equal(roms[0], Next().Path);
            scraper.Dispose();
            _server.Gate = null;
            int asked = _server.JeuInfos.Count();

            Start(new ScrapeChoices { Screenshots = false, Marquees = false, Miximages = false });
            Assert.Equal(roms.Skip(1), new[] { Next(), Next(), Next() }.Select(r => r.Path));
            Assert.Equal(asked + 3, _server.JeuInfos.Count());
            Assert.Equal(roms.Count, _server.JeuInfos.Select(u => FakeScreenScraper.Param(u, "md5")).Distinct().Count());
        }

        [Theory]
        [InlineData(1, 4, 1)]
        [InlineData(2, 4, 2)]
        [InlineData(4, 2, 2)]
        public void Never_more_requests_at_once_than_maxthreads(int maxThreads, int wanted, int most)
        {
            _server.User = FakeScreenScraper.Quota(maxThreads, perMinute: 6000, perDay: 20000, koPerDay: 2000, today: 0, koToday: 0);
            _quota.Observe(new ScrapeQuota(maxThreads, 128, 0, 0, 6000, 20000, 2000));
            var roms = Enumerable.Range(0, 8).Select(i => Rom($"T{i}.sfc", Filled(1024, 40 + i))).ToList();
            _server.Gate = new SemaphoreSlim(0);
            Scraper scraper = Start(new ScrapeChoices { Threads = wanted });
            foreach (string rom in roms) scraper.Enqueue(rom, "snes", ScrapePriority.Library);
            scraper.Start();
            Thread.Sleep(300);
            _server.Gate.Release(100);
            for (int i = 0; i < roms.Count; i++) Next();
            Assert.Equal(most, _server.MostAtOnce);
        }

        [Fact]
        public void A_picture_that_came_faster_than_maxdownloadspeed_is_followed_by_a_wait_for_the_difference()
        {
            _server.User!["maxdownloadspeed"] = "1";
            string a = Rom("A (USA).sfc", Filled(1024, 62));
            _server.Games.Add(new FakeGame(1, "A", RomHash.Of(a).Md5) { Media = [("box-2D", "us", "png")] });
            Scraper scraper = Start();
            Queue(scraper, a, "snes", ScrapePriority.Shown);
            Next();
            Assert.Contains(_clock.Delays, d => Math.Abs(d.TotalSeconds - 400 / 1024.0) < 0.05);
        }

        [Fact]
        public void Every_request_waits_its_turn_at_the_pace_and_an_unknown_game_counts_against_the_ko_allowance()
        {
            _server.User = null;
            string a = Rom("A (USA).sfc", Filled(1024, 60));
            string b = Rom("B (USA).sfc", Filled(1024, 61));
            _server.Games.Add(new FakeGame(1, "A", RomHash.Of(a).Md5) { Media = [("box-2D", "us", "png")] });
            Scraper scraper = Start();
            Queue(scraper, a, "snes", ScrapePriority.Shown);
            Queue(scraper, b, "snes", ScrapePriority.Shown);
            Next();
            Next();

            QuotaSnapshot q = _quota.Snapshot();
            Assert.Equal((2, 1), (q.RequestsToday, q.KoToday));
            Assert.Equal(2, _clock.Delays.Count(d => d > TimeSpan.Zero));
            Assert.All(_clock.Delays.Where(d => d > TimeSpan.Zero), d => Assert.Equal(_quota.Interval, d));
        }
    }
}

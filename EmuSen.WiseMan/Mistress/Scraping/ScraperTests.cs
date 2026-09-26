using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading;
using EmuSen.Cores;
using EmuSen.Mistress.Scraping;

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

        private Scraper Start(ScrapeChoices? choices = null, Func<string, string, bool>? handCover = null, bool start = true)
        {
            var client = new ScreenScraperClient(new HttpClient(_server), FakeScreenScraper.Developer, null);
            ScrapeChoices chosen = choices ?? new ScrapeChoices();
            var scraper = new Scraper(client, _store, _quota, () => chosen, handCover ?? ((_, _) => false),
                path => CoreCatalog.ByExtension(Path.GetExtension(path))?.OpenVgdbBytes, _results.Add, _clock) { IdlePoll = TimeSpan.FromMilliseconds(20) };
            _open.Add(scraper);
            if (start) scraper.Start();
            return scraper;
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
        public void A_game_found_by_its_file_s_hashes_gets_its_text_and_each_default_kind_in_es_de_s_layout()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(4096, 1));
            RomHashes h = RomHashes.Of(rom);
            _server.Games.Add(new FakeGame(77, "F-Zero", h.Md5));
            Scraper scraper = Start();

            scraper.Enqueue(rom, "snes", ScrapePriority.Shown);
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
        public void The_headerless_hash_is_asked_only_after_a_404_and_only_when_the_core_s_transform_changes_the_bytes()
        {
            byte[] body = Filled(0x4000, 2);
            byte[] ines = new byte[] { (byte)'N', (byte)'E', (byte)'S', 0x1A }.Concat(new byte[12]).Concat(body).ToArray();
            string nes = Rom("Metroid (USA).nes", ines);
            string gb = Rom("Unknown Game (USA).gb", Filled(0x8000, 3));
            _server.Games.Add(new FakeGame(5, "Metroid", RomHashes.Of(body).Md5) { SystemId = 3 });
            Scraper scraper = Start();

            scraper.Enqueue(nes, "nes", ScrapePriority.Shown);
            ScrapeResult found = Next();
            Assert.Equal((ScrapeOutcome.Found, "transformed"), (found.Outcome, found.MatchedBy));
            Assert.Equal([RomHashes.Of(ines).Md5, RomHashes.Of(body).Md5], _server.JeuInfos.Select(u => FakeScreenScraper.Param(u, "md5")));
            Assert.Equal(body.Length.ToString(), FakeScreenScraper.Param(_server.JeuInfos.Last(), "romtaille"));

            int before = _server.JeuInfos.Count();
            scraper.Enqueue(gb, "gb", ScrapePriority.Shown);
            Assert.Equal(ScrapeOutcome.Unknown, Next().Outcome);
            Assert.Equal(before + 1, _server.JeuInfos.Count());
            Assert.DoesNotContain(_server.Asked, u => u.Contains("jeuRecherche"));
        }

        [Fact]
        public void A_cover_the_player_already_has_is_not_fetched_and_a_file_already_there_is_not_replaced()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(2048, 4));
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5));
            Directory.CreateDirectory(Stored("snes", "screenshots"));
            File.WriteAllBytes(Stored("snes", "screenshots", "F-Zero (USA).png"), [7, 7, 7]);
            Scraper scraper = Start(handCover: (system, path) => path.EndsWith("F-Zero (USA).sfc"));

            scraper.Enqueue(rom, "snes", ScrapePriority.Shown);
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
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5));
            Scraper scraper = Start();
            scraper.Enqueue(rom, "snes", ScrapePriority.Shown);
            scraper.Enqueue(hack, "snes", ScrapePriority.Shown);
            Next();
            Next();
            int asked = _server.Asked.Count;

            scraper.Enqueue(rom, "snes", ScrapePriority.Shown);
            scraper.Enqueue(hack, "snes", ScrapePriority.Shown);
            Assert.Equal([ScrapeOutcome.Found, ScrapeOutcome.Unknown], new[] { Next(), Next() }.OrderBy(r => r.Outcome).Select(r => r.Outcome));
            Assert.Equal(asked, _server.Asked.Count);
        }

        [Fact]
        public void A_kind_turned_on_later_is_asked_for_only_when_the_game_offered_it()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(2048, 7));
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5));
            Scraper scraper = Start();
            scraper.Enqueue(rom, "snes", ScrapePriority.Shown);
            Next();
            scraper.Dispose();

            Scraper titles = Start(new ScrapeChoices { TitleScreens = true });
            titles.Enqueue(rom, "snes", ScrapePriority.Shown);
            Next();
            Assert.Equal(2, _server.JeuInfos.Count());
            Assert.True(File.Exists(Stored("snes", "titlescreens", "F-Zero (USA).png")));
        }

        [Fact]
        public void A_renamed_file_takes_its_media_with_it_and_a_second_copy_gets_its_own_with_no_request()
        {
            byte[] bytes = Filled(2048, 8);
            string rom = Rom("F-Zero (USA).sfc", bytes);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5));
            Scraper scraper = Start();
            scraper.Enqueue(rom, "snes", ScrapePriority.Shown);
            Next();
            int asked = _server.Asked.Count;

            string renamed = Path.Combine(Roms, "F-Zero.sfc");
            File.Move(rom, renamed);
            scraper.Enqueue(renamed, "snes", ScrapePriority.Shown);
            Assert.Equal(ScrapeOutcome.Found, Next().Outcome);
            Assert.True(File.Exists(Stored("snes", "covers", "F-Zero.png")));
            Assert.False(File.Exists(Stored("snes", "covers", "F-Zero (USA).png")));

            string copy = Rom("F-Zero (copy).sfc", bytes);
            scraper.Enqueue(copy, "snes", ScrapePriority.Shown);
            Assert.Equal(ScrapeOutcome.Found, Next().Outcome);
            Assert.True(File.Exists(Stored("snes", "covers", "F-Zero (copy).png")));
            Assert.True(File.Exists(Stored("snes", "covers", "F-Zero.png")));
            Assert.Equal(asked, _server.Asked.Count);
        }

        [Fact]
        public void A_european_file_gets_the_european_box_and_a_region_the_game_lacks_falls_back_in_order()
        {
            string eu = Rom("Game (Europe).sfc", Filled(1024, 9));
            string kr = Rom("Game (Korea).sfc", Filled(1024, 10));
            _server.Games.Add(new FakeGame(1, "Game", RomHashes.Of(eu).Md5, RomHashes.Of(kr).Md5)
            {
                Media = [("box-2D", "us", "png"), ("box-2D", "eu", "jpg"), ("box-2D", "jp", "png")],
            });
            Scraper scraper = Start(new ScrapeChoices { Screenshots = false, Marquees = false, Miximages = false });
            scraper.Enqueue(eu, "snes", ScrapePriority.Shown);
            Next();
            scraper.Enqueue(kr, "snes", ScrapePriority.Shown);
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
            scraper.Enqueue(a, "snes", ScrapePriority.Shown);
            scraper.Enqueue(b, "snes", ScrapePriority.Shown);

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
            _server.Games.Add(new FakeGame(1, "A", RomHashes.Of(a).Md5));
            _server.Games.Add(new FakeGame(2, "B", RomHashes.Of(b).Md5));
            _server.ForcedStatus = 430;
            Scraper scraper = Start();
            scraper.Enqueue(a, "snes", ScrapePriority.Shown);
            scraper.Enqueue(b, "snes", ScrapePriority.Library);
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
            foreach ((string rom, int i) in roms.Select((r, i) => (r, i))) _server.Games.Add(new FakeGame(100 + i, $"G{i}", RomHashes.Of(rom).Md5));
            _server.Gate = new SemaphoreSlim(0);
            Scraper scraper = Start(new ScrapeChoices { Screenshots = false, Marquees = false, Miximages = false });
            foreach (string rom in roms) scraper.Enqueue(rom, "snes", ScrapePriority.Library);
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
            Thread.Sleep(300);
            _server.Gate.Release(100);
            for (int i = 0; i < roms.Count; i++) Next();
            Assert.Equal(most, _server.MostAtOnce);
        }

        [Fact]
        public void Every_request_waits_its_turn_at_the_pace_and_an_unknown_game_counts_against_the_ko_allowance()
        {
            _server.User = null;
            string a = Rom("A (USA).sfc", Filled(1024, 60));
            string b = Rom("B (USA).sfc", Filled(1024, 61));
            _server.Games.Add(new FakeGame(1, "A", RomHashes.Of(a).Md5) { Media = [("box-2D", "us", "png")] });
            Scraper scraper = Start();
            scraper.Enqueue(a, "snes", ScrapePriority.Shown);
            scraper.Enqueue(b, "snes", ScrapePriority.Shown);
            Next();
            Next();

            QuotaSnapshot q = _quota.Snapshot();
            Assert.Equal((2, 1), (q.RequestsToday, q.KoToday));
            Assert.Equal(2, _clock.Delays.Count(d => d > TimeSpan.Zero));
            Assert.All(_clock.Delays.Where(d => d > TimeSpan.Zero), d => Assert.Equal(_quota.Interval, d));
        }
    }
}

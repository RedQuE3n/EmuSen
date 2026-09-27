using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Cores;
using EmuSen.Mistress.Scraping;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Pass 8 at the worker and the client: the new kinds, what an older answer says of them, Refresh by checksum, a game chosen by name, and every request counted - see EmuSen_BigPicture.md §38.
    public class ScrapeExtrasTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeExtrasTests", Guid.NewGuid().ToString("N"));
        private readonly FakeScreenScraper _server = new();
        private readonly FakeScrapeClock _clock = new(new DateTimeOffset(2026, 9, 27, 10, 0, 0, TimeSpan.Zero));
        private readonly BlockingCollection<ScrapeResult> _results = new();
        private readonly List<IDisposable> _open = new();
        private readonly MediaStore _store;
        private readonly ScrapeQuotaManager _quota;

        private string Roms => Path.Combine(_root, "Roms");
        private string Media => Path.Combine(_root, "Media");

        private static readonly ScrapeChoices AllOn = new()
        {
            TitleScreens = true, BackCovers = true, Boxes3D = true, PhysicalMedia = true, FanArt = true, Manuals = true, Videos = true,
        };

        public ScrapeExtrasTests()
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

        private Scraper Worker(ScrapeChoices? choices = null)
        {
            var client = new ScreenScraperClient(new HttpClient(_server), FakeScreenScraper.Developer, null);
            ScrapeChoices chosen = choices ?? new ScrapeChoices();
            var scraper = new Scraper(client, _store, _quota, () => chosen, (_, _) => false,
                path => CoreCatalog.ByExtension(Path.GetExtension(path))?.OpenVgdbBytes, _results.Add, _clock) { IdlePoll = TimeSpan.FromMilliseconds(20) };
            _open.Add(scraper);
            return scraper;
        }

        private ScrapeResult Run(string rom, ScrapeChoices? choices = null, ScrapedGame? chosen = null)
        {
            // The last run's worker is joined first; two on one store would race for the next game, as they never do in the window, which holds one.
            foreach (IDisposable d in _open) d.Dispose();
            _open.Clear();
            Scraper scraper = Worker(choices);
            if (chosen is not null) scraper.Choose(rom, "snes", chosen);
            else scraper.Enqueue(rom, "snes", ScrapePriority.Shown);
            scraper.Start();
            Assert.True(_results.TryTake(out ScrapeResult? r, TimeSpan.FromSeconds(10)), "no result");
            return r!;
        }

        private string Rom(string name, int seed)
        {
            string path = Path.Combine(Roms, name);
            var bytes = new byte[4096];
            new Random(seed).NextBytes(bytes);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        private string Stored(params string[] parts) => Path.Combine([Media, .. parts]);

        private static string Sha1(string file) => Convert.ToHexStringLower(System.Security.Cryptography.SHA1.HashData(File.ReadAllBytes(file)));

        private int Requests => _server.Asked.Count;

        // --- the kinds ---

        [Fact]
        public void Each_new_kind_is_fetched_when_turned_on_into_es_de_s_folder_with_its_own_file_type_and_each_is_one_request()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5) { Media = FakeGame.AllMedia });

            ScrapeResult result = Run(rom, AllOn);

            Assert.Equal(ScrapeOutcome.Found, result.Outcome);
            foreach ((string folder, string file) in new[] { ("backcovers", "F-Zero (USA).png"), ("3dboxes", "F-Zero (USA).png"), ("physicalmedia", "F-Zero (USA).png"),
                         ("fanart", "F-Zero (USA).jpg"), ("manuals", "F-Zero (USA).pdf"), ("videos", "F-Zero (USA).mp4") })
                Assert.True(File.Exists(Stored("snes", folder, file)), folder);
            Assert.StartsWith("%PDF-", Encoding.ASCII.GetString(File.ReadAllBytes(Stored("snes", "manuals", "F-Zero (USA).pdf")), 0, 5));
            Assert.Equal("ftyp", Encoding.ASCII.GetString(File.ReadAllBytes(Stored("snes", "videos", "F-Zero (USA).mp4")), 4, 4));
            // One lookup and one request for each of the eleven kinds the game offers.
            Assert.Single(_server.JeuInfos);
            Assert.Equal(11, _server.MediaAsked.Count());
            Assert.Contains(_server.MediaAsked, u => u.Contains("media=manuel(us)"));
            Assert.Contains(_server.MediaAsked, u => u.Contains("media=video-normalized()"));
            Assert.Equal(new[] { "3dbox", "backcover", "cover", "fanart", "manual", "marquee", "miximage", "physicalmedia", "screenshot", "titlescreen", "video" },
                _store.Media(RomHashes.Of(rom).Md5, 4096).Select(m => m.Type).Order(StringComparer.Ordinal));
        }

        [Fact]
        public void With_the_new_kinds_off_as_they_are_by_default_a_game_costs_what_it_did()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5) { Media = FakeGame.AllMedia });

            Run(rom);

            Assert.Equal(5, Requests);
            Assert.Equal(4, _server.MediaAsked.Count());
            foreach (string folder in (string[])["backcovers", "3dboxes", "physicalmedia", "fanart", "manuals", "videos", "titlescreens"])
                Assert.False(Directory.Exists(Stored("snes", folder)), folder);
            ScrapedRecord kept = _store.Game(RomHashes.Of(rom).Md5, 4096)!;
            // The answer's offer is recorded for every kind, so turning one on later knows whether to ask.
            Assert.Contains("manual", kept.Offered!.Split(','));
            Assert.Equal(string.Join(",", ScrapeRules.AllKinds.Select(k => k.EsdeType)), kept.KindsKnown);
        }

        [Fact]
        public void A_kind_turned_on_later_asks_again_only_a_game_that_may_have_it()
        {
            string offers = Rom("Offers (USA).sfc", 1), lacks = Rom("Lacks (USA).sfc", 2), older = Rom("Older (USA).sfc", 3);
            _server.Games.Add(new FakeGame(1, "Offers", RomHashes.Of(offers).Md5) { Media = FakeGame.AllMedia });
            _server.Games.Add(new FakeGame(2, "Lacks", RomHashes.Of(lacks).Md5) { Media = [("box-2D", "us", "png")] });
            _server.Games.Add(new FakeGame(3, "Older", RomHashes.Of(older).Md5) { Media = FakeGame.AllMedia });
            Run(offers);
            Run(lacks);
            // An answer kept before Pass 8 listed only the first five kinds and has no record of the rest.
            Run(older);
            ScrapedRecord old = _store.Game(RomHashes.Of(older).Md5, 4096)!;
            _store.Record(old with { Offered = "cover,screenshot,marquee,miximage,titlescreen", KindsKnown = null });
            int before = Requests;

            var fanArt = new ScrapeChoices { FanArt = true };
            Assert.Equal(ScrapeOutcome.Found, Run(offers, fanArt).Outcome);
            Assert.Equal(before + 2, Requests);
            ScrapeResult skipped = Run(lacks, fanArt);
            Assert.True(skipped.Skipped);
            Assert.Equal(before + 2, Requests);
            Assert.False(Run(older, fanArt).Skipped);
            Assert.Equal(before + 4, Requests);
            Assert.True(File.Exists(Stored("snes", "fanart", "Older (USA).jpg")));
        }

        [Fact]
        public void A_media_db_from_before_pass_8_opens_with_its_rows_and_knows_nothing_of_the_new_kinds()
        {
            string dir = Path.Combine(_root, "Old");
            using (MediaStore fresh = MediaStore.Open(dir))
                fresh.Record(new ScrapedRecord("aa", 64, ScrapeState.Found) { Offered = "cover,screenshot", Name = "Old", FetchedAt = DateTime.UtcNow });
            using (var db = new Microsoft.Data.Sqlite.SqliteConnection($"Data Source={Path.Combine(dir, MediaStore.FileName)};Pooling=False"))
            {
                db.Open();
                using var c = db.CreateCommand();
                c.CommandText = "ALTER TABLE scrape_game DROP COLUMN kinds_known; PRAGMA user_version = 2;";
                c.ExecuteNonQuery();
            }

            using MediaStore reopened = MediaStore.Open(dir);
            ScrapedRecord old = reopened.Game("aa", 64)!;
            Assert.Equal(("Old", null), (old.Name, old.KindsKnown));
            Assert.True(old.MayOffer("cover"));
            Assert.False(old.MayOffer("titlescreen"));
            Assert.True(old.MayOffer("fanart"));
            Assert.True(old.MayOffer("manual"));
            Assert.False((old with { KindsKnown = "cover,screenshot,fanart" }).MayOffer("fanart"));
        }

        [Fact]
        public void A_manual_that_is_not_a_pdf_and_a_clip_that_is_not_an_mp4_are_not_kept()
        {
            string target = Path.Combine(_root, "x.bin");
            async Task<MediaOutcome> Get(MediaPayload payload, byte[] bytes, string type)
            {
                var content = new ByteArrayContent(bytes);
                content.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue(type);
                var client = new ScreenScraperClient(new HttpClient(new ScreenScraperClientTests.Answering(_ => new HttpResponseMessage(HttpStatusCode.OK) { Content = content })), FakeScreenScraper.Developer, null);
                MediaAnswer answer = await client.DownloadAsync("https://x/mediaJeu.php", target, CancellationToken.None, payload);
                if (File.Exists(target)) File.Delete(target);
                return answer.Outcome;
            }
            byte[] pdf = Encoding.ASCII.GetBytes("%PDF-1.7\n" + new string('x', 200));
            byte[] mp4 = [0, 0, 0, 0x18, .. "ftypisom"u8.ToArray(), .. new byte[200]];
            byte[] html = Encoding.ASCII.GetBytes("<html>" + new string('x', 200));

            Assert.Equal(MediaOutcome.Saved, Get(MediaPayload.Pdf, pdf, "application/pdf").GetAwaiter().GetResult());
            Assert.Equal(MediaOutcome.Saved, Get(MediaPayload.Pdf, pdf, "application/octet-stream").GetAwaiter().GetResult());
            Assert.Equal(MediaOutcome.NotAnImage, Get(MediaPayload.Pdf, html, "application/pdf").GetAwaiter().GetResult());
            Assert.Equal(MediaOutcome.NotAnImage, Get(MediaPayload.Pdf, pdf.Take(4).Concat(new byte[200]).ToArray(), "image/png").GetAwaiter().GetResult());
            Assert.Equal(MediaOutcome.Saved, Get(MediaPayload.Video, mp4, "application/octet-stream").GetAwaiter().GetResult());
            Assert.Equal(MediaOutcome.NotAnImage, Get(MediaPayload.Video, html, "text/html").GetAwaiter().GetResult());
            Assert.Equal(MediaOutcome.NotAnImage, Get(MediaPayload.Image, pdf, "application/pdf").GetAwaiter().GetResult());
            Assert.False(File.Exists(target + ".part"));
        }

        // --- Refresh ---

        [Fact]
        public void Refreshing_unchanged_files_costs_one_lookup_and_no_byte()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5));
            Run(rom);
            int before = Requests;
            long bytes = _server.BytesSent;
            DateTime written = File.GetLastWriteTimeUtc(Stored("snes", "covers", "F-Zero (USA).png"));

            ScrapeResult refreshed = Run(rom, new ScrapeChoices { Refresh = true });

            Assert.Equal(ScrapeOutcome.Found, refreshed.Outcome);
            Assert.Equal(4, refreshed.Unchanged);
            Assert.Empty(refreshed.Written);
            Assert.Equal(before + 1, Requests);
            Assert.Single(_server.JeuInfos.Skip(1));
            Assert.Equal(bytes, _server.BytesSent);
            Assert.Equal(written, File.GetLastWriteTimeUtc(Stored("snes", "covers", "F-Zero (USA).png")));
        }

        [Fact]
        public void Without_refresh_a_found_game_costs_nothing()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5));
            Run(rom);
            int before = Requests;
            _server.MediaRevision["box-2D"] = 1;
            Assert.True(Run(rom).Skipped);
            Assert.Equal(before, Requests);
        }

        [Fact]
        public void Refresh_fetches_a_changed_file_over_the_kept_one_and_only_that_one()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5));
            Run(rom);
            string cover = Stored("snes", "covers", "F-Zero (USA).png"), shot = Stored("snes", "screenshots", "F-Zero (USA).png");
            string oldCover = Sha1(cover), oldShot = Sha1(shot);
            int before = Requests;
            _server.MediaRevision["box-2D"] = 1;

            ScrapeResult refreshed = Run(rom, new ScrapeChoices { Refresh = true });

            Assert.Equal(before + 2, Requests);
            Assert.Contains("media=box-2D(us)", _server.MediaAsked.Last());
            Assert.Equal(3, refreshed.Unchanged);
            Assert.NotEqual(oldCover, Sha1(cover));
            Assert.Equal(oldShot, Sha1(shot));
            Assert.Equal(Sha1(cover), _store.Media(RomHashes.Of(rom).Md5, 4096).Single(m => m.Type == "cover").Sha1);
            Assert.Empty(Directory.EnumerateFiles(Media, "*.part", SearchOption.AllDirectories));
        }

        [Fact]
        public void Where_an_answer_states_no_checksum_the_kept_file_s_is_sent_and_SHA1OK_costs_a_request_and_no_byte()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5));
            Run(rom);
            _server.StateChecksums = false;
            int before = Requests;
            long bytes = _server.BytesSent;
            string cover = Stored("snes", "covers", "F-Zero (USA).png");

            ScrapeResult refreshed = Run(rom, new ScrapeChoices { Refresh = true });

            Assert.Equal(before + 5, Requests);
            Assert.All(_server.MediaAsked.TakeLast(4), u => Assert.Matches("[&?]sha1=[0-9a-f]{40}", u));
            Assert.Contains(_server.MediaAsked, u => FakeScreenScraper.Param(u, "sha1") == Sha1(cover));
            Assert.Equal(4, refreshed.Unchanged);
            Assert.Equal(bytes, _server.BytesSent);
        }

        [Fact]
        public void A_game_no_longer_listed_keeps_what_was_found_when_refreshed()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5));
            Run(rom);
            _server.Games.Clear();

            ScrapeResult refreshed = Run(rom, new ScrapeChoices { Refresh = true });

            Assert.Equal(ScrapeOutcome.Found, refreshed.Outcome);
            Assert.Equal(ScrapeState.Found, _store.Game(RomHashes.Of(rom).Md5, 4096)!.State);
            Assert.Equal("F-Zero (US title)", _store.Game(RomHashes.Of(rom).Md5, 4096)!.Name);
            Assert.True(File.Exists(Stored("snes", "covers", "F-Zero (USA).png")));
        }

        // --- a game chosen by name ---

        [Fact]
        public void A_game_chosen_by_name_is_kept_for_the_file_with_no_lookup_and_its_pictures_fetched()
        {
            string rom = Rom("SMW Hack (USA).sfc", 5);
            _server.Games.Add(new FakeGame(900, "Super Mario World"));
            ScrapedGame picked = Search("Super Mario").Games.Single();
            int before = Requests;

            ScrapeResult result = Run(rom, chosen: picked);

            Assert.Equal(ScrapeOutcome.Found, result.Outcome);
            Assert.Equal("name", result.MatchedBy);
            Assert.Empty(_server.JeuInfos);
            Assert.Equal(before + 4, Requests);
            ScrapedRecord kept = _store.Game(RomHashes.Of(rom).Md5, 4096)!;
            Assert.Equal((ScrapeState.Found, 900L, "name"), (kept.State, kept.GameId!.Value, kept.MatchedBy));
            Assert.True(File.Exists(Stored("snes", "covers", "SMW Hack (USA).png")));
        }

        [Fact]
        public void A_game_chosen_by_name_is_asked_by_its_id_later_and_a_404_never_makes_it_unknown()
        {
            string rom = Rom("SMW Hack (USA).sfc", 5);
            _server.Games.Add(new FakeGame(900, "Super Mario World") { Media = FakeGame.AllMedia });
            Run(rom, chosen: Search("Super Mario").Games.Single());

            Run(rom, new ScrapeChoices { BackCovers = true });
            string asked = _server.JeuInfos.Single();
            Assert.Equal("900", FakeScreenScraper.Param(asked, "gameid"));
            Assert.True(File.Exists(Stored("snes", "backcovers", "SMW Hack (USA).png")));

            _server.Games.Clear();
            ScrapeResult later = Run(rom, new ScrapeChoices { Refresh = true });
            Assert.Equal(ScrapeOutcome.Found, later.Outcome);
            Assert.Equal("name", _store.Game(RomHashes.Of(rom).Md5, 4096)!.MatchedBy);
        }

        private SearchAnswer Search(string text) =>
            new ScreenScraperClient(new HttpClient(_server), FakeScreenScraper.Developer, null).JeuRechercheAsync(4, text, CancellationToken.None).GetAwaiter().GetResult();

        [Fact]
        public void A_search_sends_the_credentials_the_system_and_the_text_and_reads_up_to_thirty_games_or_none()
        {
            var client = new ScreenScraperClient(new HttpClient(_server), FakeScreenScraper.Developer, null);
            string url = client.JeuRechercheUrl(3, "Mega Man & Bass");
            Assert.StartsWith(ScreenScraperClient.Api + "jeuRecherche.php?devid=", url);
            Assert.Equal(("3", "Mega Man & Bass", "json"), (FakeScreenScraper.Param(url, "systemeid"), FakeScreenScraper.Param(url, "recherche"), FakeScreenScraper.Param(url, "output")));

            for (int i = 0; i < 35; i++) _server.Games.Add(new FakeGame(100 + i, $"Racer {i}"));
            _server.Games.Add(new FakeGame(500, "Racer on the NES") { SystemId = 3 });
            SearchAnswer many = Search("racer");
            Assert.Equal(ScrapeStatus.Found, many.Status);
            Assert.Equal(30, many.Games.Count);
            Assert.NotNull(many.Quota);
            Assert.Contains(many.Games[0].Media, m => m.Type == "box-2D" && m.Sha1 is { Length: 40 });

            SearchAnswer none = Search("Nothing Like It");
            Assert.Equal(ScrapeStatus.NotFound, none.Status);
            Assert.Empty(none.Games);
            Assert.Equal(2, _server.Searches.Count());
        }

        [Theory]
        [InlineData("Super Mario World (USA).sfc", "Super Mario World")]
        [InlineData("Mygame (U) [v2].zip", "Mygame")]
        [InlineData("Zelda_no_Densetsu (Japan) (Rev 1) [T-Eng].sfc", "Zelda no Densetsu")]
        [InlineData("F-Zero.sfc", "F-Zero")]
        public void The_search_text_is_the_file_s_name_without_its_tags_as_es_de_strips_it(string file, string expected) =>
            Assert.Equal(expected, ScrapeRules.SearchName(file));

        // --- pacing ---

        [Fact]
        public void Every_request_of_the_new_kinds_waits_its_turn_at_the_quota_s_pace()
        {
            _server.User = FakeScreenScraper.Quota(maxThreads: 1, perMinute: 6, perDay: 20000, koPerDay: 2000, today: 10, koToday: 1);
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHashes.Of(rom).Md5) { Media = FakeGame.AllMedia });
            DateTimeOffset start = _clock.Now;

            Run(rom, AllOn);

            // Twelve requests at 6 a minute less 10%: eleven gaps of 60 / 5.4 s after the first answer set the pace.
            Assert.Equal(12, Requests);
            Assert.True(_clock.Now - start >= TimeSpan.FromSeconds(60 / 5.4 * 10), $"{(_clock.Now - start).TotalSeconds} s");
        }
    }
}

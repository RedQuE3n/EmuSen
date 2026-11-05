using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Text.Json;
using EmuSen.Crystal.Client.Scrape;
using EmuSen.Mistress.Scraping;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Pass 8 at the library's run: the new kinds, what an older answer says of them, Refresh by checksum, a game chosen by name, and every request counted - see EmuSen_BigPicture.md §38 and EmuSen_Backend.md §40.5.
    public class ScrapeExtrasTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeExtrasTests", Guid.NewGuid().ToString("N"));
        private readonly FakeScreenScraper _server = new();
        private readonly List<IDisposable> _open = new();
        private LibraryScraper? _scraper;

        private string Roms => Path.Combine(_root, "Roms");
        private string Media => Path.Combine(_root, "Media");

        // These cases are about Pass 8's kinds turned on over the four the default asked before 2026-11-02, so they keep those four; the default itself is the case below and ScrapeRulesTests'.
        private static ScrapeChoices Four => new() { TitleScreens = false, BackCovers = false, Boxes3D = false, PhysicalMedia = false, FanArt = false };

        private static readonly ScrapeChoices AllOn = new()
        {
            TitleScreens = true, BackCovers = true, Boxes3D = true, PhysicalMedia = true, FanArt = true, Manuals = true, Videos = true,
        };

        private static readonly DateTimeOffset Start = new(2026, 9, 27, 10, 0, 0, TimeSpan.Zero);

        public ScrapeExtrasTests() => Directory.CreateDirectory(Roms);

        public void Dispose()
        {
            foreach (IDisposable d in Enumerable.Reverse(_open)) d.Dispose();
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private LibraryScraper Scraper(System.Net.Http.HttpMessageHandler? handler = null, string? media = null)
        {
            if (handler is null && media is null && _scraper is not null) return _scraper;
            var s = new LibraryScraper(handler ?? _server, media ?? Media, new Dictionary<string, object?> { ["roms"] = Roms }, Start);
            _open.Add(s);
            if (handler is null && media is null) _scraper = s;
            return s;
        }

        // One game's run on the store; the run's own count of files a refresh found unchanged beside its answer.
        private (LibraryScraper.Result Result, int Unchanged) RunCounted(string rom, ScrapeChoices? choices = null, ScrapedGame? chosen = null, LibraryScraper? on = null)
        {
            LibraryScraper s = on ?? Scraper();
            ScrapeChoices c = choices ?? Four;
            s.Run([(rom, "snes")], c, ScrapePriority.Shown, one: true, refresh: c.Refresh, pick: chosen);
            LibraryScraper.Result r = s.Next();
            s.Ends();
            using JsonDocument p = s.Progress();
            return (r, p.RootElement.GetProperty("unchanged").GetInt32());
        }

        private LibraryScraper.Result Run(string rom, ScrapeChoices? choices = null, ScrapedGame? chosen = null) => RunCounted(rom, choices, chosen).Result;

        private ScrapedRecord? Game(string md5, long bytes)
        {
            using JsonDocument d = JsonDocument.Parse(Scraper().Scraper.Service.Get(JsonSerializer.Serialize(new { what = "game", md5, bytes })));
            return d.RootElement.ValueKind == JsonValueKind.Null ? null : NativeScraping.Record(d.RootElement);
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
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5) { Media = FakeGame.AllMedia });

            LibraryScraper.Result result = Run(rom, AllOn);

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
                Scraper().Scraper.Media(RomHash.Of(rom).Md5, 4096).Select(m => m.Type).Order(StringComparer.Ordinal));
        }

        [Fact]
        // The default since 2026-11-02 is every picture a theme can draw, so that changing themes never needs a second scrape (the tester's decision, EmuSen_LibraryTools.md §9.7); manuals and videos stay off.
        public void By_default_a_game_asks_every_picture_a_theme_can_draw_and_no_manual_or_video()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5) { Media = FakeGame.AllMedia });

            Run(rom, new ScrapeChoices());

            Assert.Equal(10, Requests);
            Assert.Equal(9, _server.MediaAsked.Count());
            foreach (string folder in (string[])["backcovers", "3dboxes", "physicalmedia", "fanart", "titlescreens"])
                Assert.True(Directory.Exists(Stored("snes", folder)), folder);
            foreach (string folder in (string[])["manuals", "videos"])
                Assert.False(Directory.Exists(Stored("snes", folder)), folder);
            ScrapedRecord kept = Game(RomHash.Of(rom).Md5, 4096)!;
            // The answer's offer is recorded for every kind, so turning one on later knows whether to ask.
            Assert.Contains("manual", kept.Offered!.Split(','));
            Assert.Equal(string.Join(",", ScrapeRules.AllKinds.Select(k => k.EsdeType)), kept.KindsKnown);
        }

        [Fact]
        public void A_kind_turned_on_later_asks_again_only_a_game_that_may_have_it()
        {
            string offers = Rom("Offers (USA).sfc", 1), lacks = Rom("Lacks (USA).sfc", 2), older = Rom("Older (USA).sfc", 3);
            _server.Games.Add(new FakeGame(1, "Offers", RomHash.Of(offers).Md5) { Media = FakeGame.AllMedia });
            _server.Games.Add(new FakeGame(2, "Lacks", RomHash.Of(lacks).Md5) { Media = [("box-2D", "us", "png")] });
            _server.Games.Add(new FakeGame(3, "Older", RomHash.Of(older).Md5) { Media = FakeGame.AllMedia });
            Run(offers);
            Run(lacks);
            // An answer kept before Pass 8 listed only the first five kinds and has no record of the rest.
            Run(older);
            using (var rows = new MediaDbRows(Media)) rows.Record(Game(RomHash.Of(older).Md5, 4096)! with { Offered = "cover,screenshot,marquee,miximage,titlescreen", KindsKnown = null });
            int before = Requests;

            var fanArt = Four with { FanArt = true };
            Assert.Equal(ScrapeOutcome.Found, Run(offers, fanArt).Outcome);
            Assert.Equal(before + 2, Requests);
            LibraryScraper.Result skipped = Run(lacks, fanArt);
            Assert.True(skipped.Skipped);
            Assert.Equal(before + 2, Requests);
            Assert.False(Run(older, fanArt).Skipped);
            Assert.Equal(before + 4, Requests);
            Assert.True(File.Exists(Stored("snes", "fanart", "Older (USA).jpg")));
        }

        // Which kinds an older answer may offer is the run's rule, held by A_kind_turned_on_later_asks_again_only_a_game_that_may_have_it.
        [Fact]
        public void A_media_db_from_before_pass_8_opens_with_its_rows_and_knows_nothing_of_the_new_kinds()
        {
            string dir = Path.Combine(_root, "Old");
            using (var rows = new MediaDbRows(dir))
                rows.Record(new ScrapedRecord("aa", 64, ScrapeState.Found) { Offered = "cover,screenshot", Name = "Old", FetchedAt = DateTime.UtcNow });
            using (var db = new Microsoft.Data.Sqlite.SqliteConnection($"Data Source={Path.Combine(dir, MediaStore.FileName)};Pooling=False"))
            {
                db.Open();
                using var c = db.CreateCommand();
                c.CommandText = "ALTER TABLE scrape_game DROP COLUMN kinds_known; ALTER TABLE scrape_file DROP COLUMN crc32; ALTER TABLE scrape_file DROP COLUMN sha1; ALTER TABLE scrape_file DROP COLUMN fingerprint; PRAGMA user_version = 2;";
                c.ExecuteNonQuery();
            }

            using var reopened = MediaService.Open(dir, "{\"developer_files\":false,\"test\":{\"offline\":true}}");
            using JsonDocument d = JsonDocument.Parse(reopened.Get("{\"what\":\"game\",\"md5\":\"aa\",\"bytes\":64}"));
            ScrapedRecord old = NativeScraping.Record(d.RootElement);
            Assert.Equal(("Old", null, "cover,screenshot"), (old.Name, old.KindsKnown, old.Offered));
        }

        // The fake server, its pictures answered by the test.
        private sealed class Pictures(FakeScreenScraper server, Func<HttpResponseMessage> picture) : DelegatingHandler(server)
        {
            protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken) =>
                request.RequestUri!.AbsoluteUri.Contains("/mediaJeu.php") ? Task.FromResult(picture()) : base.SendAsync(request, cancellationToken);
        }

        [Fact]
        public void A_manual_that_is_not_a_pdf_and_a_clip_that_is_not_an_mp4_are_not_kept()
        {
            byte[] pdf = Encoding.ASCII.GetBytes("%PDF-1.7\n" + new string('x', 200));
            byte[] mp4 = [0, 0, 0, 0x18, .. "ftypisom"u8.ToArray(), .. new byte[200]];
            byte[] html = Encoding.ASCII.GetBytes("<html>" + new string('x', 200));
            var none = Four with { Covers = false, Screenshots = false, Marquees = false, Miximages = false };
            var cases = new (string Ss, ScrapeChoices Choices, string Folder, string Ext, byte[] Bytes, string Type, bool Kept)[]
            {
                ("manuel", none with { Manuals = true }, "manuals", ".pdf", pdf, "application/pdf", true),
                ("manuel", none with { Manuals = true }, "manuals", ".pdf", pdf, "application/octet-stream", true),
                ("manuel", none with { Manuals = true }, "manuals", ".pdf", html, "application/pdf", false),
                ("manuel", none with { Manuals = true }, "manuals", ".pdf", pdf.Take(4).Concat(new byte[200]).ToArray(), "image/png", false),
                ("video-normalized", none with { Videos = true }, "videos", ".mp4", mp4, "application/octet-stream", true),
                ("video-normalized", none with { Videos = true }, "videos", ".mp4", html, "text/html", false),
                ("box-2D", none with { Covers = true }, "covers", ".png", pdf, "application/pdf", false),
            };
            for (int i = 0; i < cases.Length; i++)
            {
                var c = cases[i];
                string rom = Rom($"Payload {i} (USA).sfc", 40 + i);
                var server = new FakeScreenScraper();
                server.Games.Add(new FakeGame(77, "Payload", RomHash.Of(rom).Md5) { Media = FakeGame.AllMedia.Where(m => m.Type == c.Ss).ToList() });
                string media = Path.Combine(_root, $"Payload{i}");
                LibraryScraper s = Scraper(new Pictures(server, () =>
                {
                    var content = new ByteArrayContent(c.Bytes);
                    content.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue(c.Type);
                    return new HttpResponseMessage(HttpStatusCode.OK) { Content = content };
                }), media);
                RunCounted(rom, c.Choices, on: s);
                string kept = Path.Combine(media, "snes", c.Folder, $"Payload {i} (USA){c.Ext}");
                Assert.True(c.Kept == File.Exists(kept), $"case {i}: {c.Ss} {c.Type}");
                Assert.Empty(Directory.EnumerateFiles(media, "*.part", SearchOption.AllDirectories));
            }
        }

        // --- Refresh ---

        [Fact]
        public void Refreshing_unchanged_files_costs_one_lookup_and_no_byte()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            Run(rom);
            int before = Requests;
            long bytes = _server.BytesSent;
            DateTime written = File.GetLastWriteTimeUtc(Stored("snes", "covers", "F-Zero (USA).png"));

            var (refreshed, unchanged) = RunCounted(rom, Four with { Refresh = true });

            Assert.Equal(ScrapeOutcome.Found, refreshed.Outcome);
            Assert.Equal(4, unchanged);
            Assert.Empty(refreshed.Kinds);
            Assert.Equal(before + 1, Requests);
            Assert.Single(_server.JeuInfos.Skip(1));
            Assert.Equal(bytes, _server.BytesSent);
            Assert.Equal(written, File.GetLastWriteTimeUtc(Stored("snes", "covers", "F-Zero (USA).png")));
        }

        [Fact]
        public void Without_refresh_a_found_game_costs_nothing()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
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
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            Run(rom);
            string cover = Stored("snes", "covers", "F-Zero (USA).png"), shot = Stored("snes", "screenshots", "F-Zero (USA).png");
            string oldCover = Sha1(cover), oldShot = Sha1(shot);
            int before = Requests;
            _server.MediaRevision["box-2D"] = 1;

            var (_, unchanged) = RunCounted(rom, Four with { Refresh = true });

            Assert.Equal(before + 2, Requests);
            Assert.Contains("media=box-2D(us)", _server.MediaAsked.Last());
            Assert.Equal(3, unchanged);
            Assert.NotEqual(oldCover, Sha1(cover));
            Assert.Equal(oldShot, Sha1(shot));
            Assert.Equal(Sha1(cover), Scraper().Scraper.Media(RomHash.Of(rom).Md5, 4096).Single(m => m.Type == "cover").Sha1);
            Assert.Empty(Directory.EnumerateFiles(Media, "*.part", SearchOption.AllDirectories));
        }

        [Fact]
        public void Where_an_answer_states_no_checksum_the_kept_file_s_is_sent_and_SHA1OK_costs_a_request_and_no_byte()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            Run(rom);
            _server.StateChecksums = false;
            int before = Requests;
            long bytes = _server.BytesSent;
            string cover = Stored("snes", "covers", "F-Zero (USA).png");

            var (_, unchanged) = RunCounted(rom, Four with { Refresh = true });

            Assert.Equal(before + 5, Requests);
            Assert.All(_server.MediaAsked.TakeLast(4), u => Assert.Matches("[&?]sha1=[0-9a-f]{40}", u));
            Assert.Contains(_server.MediaAsked, u => FakeScreenScraper.Param(u, "sha1") == Sha1(cover));
            Assert.Equal(4, unchanged);
            Assert.Equal(bytes, _server.BytesSent);
        }

        [Fact]
        public void A_game_no_longer_listed_keeps_what_was_found_when_refreshed()
        {
            string rom = Rom("F-Zero (USA).sfc", 1);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            Run(rom);
            _server.Games.Clear();

            LibraryScraper.Result refreshed = Run(rom, Four with { Refresh = true });

            Assert.Equal(ScrapeOutcome.Found, refreshed.Outcome);
            Assert.Equal(ScrapeState.Found, Game(RomHash.Of(rom).Md5, 4096)!.State);
            Assert.Equal("F-Zero (US title)", Game(RomHash.Of(rom).Md5, 4096)!.Name);
            Assert.True(File.Exists(Stored("snes", "covers", "F-Zero (USA).png")));
        }

        // --- a game chosen by name ---

        [Fact]
        public void A_game_chosen_by_name_is_kept_for_the_file_with_no_lookup_and_its_pictures_fetched()
        {
            string rom = Rom("SMW Hack (USA).sfc", 5);
            _server.Games.Add(new FakeGame(900, "Super Mario World"));
            ScrapedGame picked = Search(rom, "Super Mario").Games.Single();
            int before = Requests;

            LibraryScraper.Result result = Run(rom, chosen: picked);

            Assert.Equal(ScrapeOutcome.Found, result.Outcome);
            Assert.Equal("name", result.MatchedBy);
            Assert.Empty(_server.JeuInfos);
            Assert.Equal(before + 4, Requests);
            ScrapedRecord kept = Game(RomHash.Of(rom).Md5, 4096)!;
            Assert.Equal((ScrapeState.Found, 900L, "name"), (kept.State, kept.GameId!.Value, kept.MatchedBy));
            Assert.True(File.Exists(Stored("snes", "covers", "SMW Hack (USA).png")));
        }

        [Fact]
        public void A_game_chosen_by_name_is_asked_by_its_id_later_and_a_404_never_makes_it_unknown()
        {
            string rom = Rom("SMW Hack (USA).sfc", 5);
            _server.Games.Add(new FakeGame(900, "Super Mario World") { Media = FakeGame.AllMedia });
            Run(rom, chosen: Search(rom, "Super Mario").Games.Single());

            Run(rom, Four with { BackCovers = true });
            string asked = _server.JeuInfos.Single();
            Assert.Equal("900", FakeScreenScraper.Param(asked, "gameid"));
            Assert.True(File.Exists(Stored("snes", "backcovers", "SMW Hack (USA).png")));

            _server.Games.Clear();
            LibraryScraper.Result later = Run(rom, Four with { Refresh = true });
            Assert.Equal(ScrapeOutcome.Found, later.Outcome);
            Assert.Equal("name", Game(RomHash.Of(rom).Md5, 4096)!.MatchedBy);
        }

        // A name search through the scraper, paced and counted as every request is; each game found is kept with its pick.
        private SearchAnswer Search(string rom, string text, string system = "snes") => Scraper().Scraper.Search(rom, system, text);

        [Fact]
        public void A_search_sends_the_credentials_the_system_and_the_text_and_reads_up_to_thirty_games_or_none()
        {
            string nes = Path.Combine(Roms, "Mega.nes");
            File.WriteAllBytes(nes, new byte[64]);
            Search(nes, "Mega Man & Bass", "nes");
            string url = _server.Searches.Single();
            Assert.StartsWith("https://api.screenscraper.fr/api2/jeuRecherche.php?devid=", url);
            Assert.Equal(("3", "Mega Man & Bass", "json"), (FakeScreenScraper.Param(url, "systemeid"), FakeScreenScraper.Param(url, "recherche"), FakeScreenScraper.Param(url, "output")));

            string rom = Rom("Racer (USA).sfc", 7);
            for (int i = 0; i < 35; i++) _server.Games.Add(new FakeGame(100 + i, $"Racer {i}"));
            _server.Games.Add(new FakeGame(500, "Racer on the NES") { SystemId = 3 });
            SearchAnswer many = Search(rom, "racer");
            Assert.Equal(ScrapeStatus.Found, many.Status);
            Assert.Equal(30, many.Games.Count);

            SearchAnswer none = Search(rom, "Nothing Like It");
            Assert.Equal(ScrapeStatus.NotFound, none.Status);
            Assert.Empty(none.Games);
            Assert.Equal(3, _server.Searches.Count());
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
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5) { Media = FakeGame.AllMedia });
            Run(rom, AllOn);

            // Twelve requests at 6 a minute less 10%: eleven gaps of 60 / 5.4 s after the first answer set the pace, waited on the run's clock.
            Assert.Equal(12, Requests);
            double waited = Scraper().Get<long[]>(new { what = "delays" }).Sum() / 1e7;
            Assert.True(waited >= 60 / 5.4 * 10, $"{waited} s");
        }
    }
}

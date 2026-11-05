using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The library's run against the fake server: §5.2's identity steps, the media kept, the stops, resuming, renames and threads, through the binding Mistress uses - see EmuSen_BigPicture.md §17 and EmuSen_Backend.md §40.5.
    [Collection(EmuSen.WiseMan.Fixtures.TestCollections.ProcessGlobals)]
    public class ScraperTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScraperTests", Guid.NewGuid().ToString("N"));
        private readonly FakeScreenScraper _server = new();
        private readonly List<IDisposable> _open = new();

        private string Roms => Path.Combine(_root, "Roms");
        private string Media => Path.Combine(_root, "Media");

        public ScraperTests() => Directory.CreateDirectory(Roms);

        // A tree of the test's own whose library is Roms, so a package there claims a game the way the runtime resolves it.
        private void PackagesTree()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            EmuSen.Galaxia.ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            new EmuSen.Galaxia.Models.AppSettings { RomDirectory = Roms }.Save();
        }

        public void Dispose()
        {
            foreach (IDisposable d in Enumerable.Reverse(_open)) d.Dispose();
            DataStore.OverrideDirectory = null;
            EmuSen.Galaxia.ConfigStore.OverrideDirectory = null;
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

        // These cases are written on the four kinds the default asked before 2026-11-02; the default itself is ScrapeRulesTests'.
        private static ScrapeChoices Four => new() { TitleScreens = false, BackCovers = false, Boxes3D = false, PhysicalMedia = false, FanArt = false };

        // The library's scraper over the test's store; art names a folder of the player's covers, packages each packaged game's folder.
        private LibraryScraper Scraper(HttpMessageHandler? handler = null, string? art = null, IReadOnlyDictionary<string, string>? packages = null, DateTimeOffset? clock = null, string? media = null)
        {
            var settings = new Dictionary<string, object?> { ["roms"] = Roms };
            if (art is not null) settings["art"] = art;
            if (packages is not null) settings["packages"] = packages;
            var s = new LibraryScraper(handler ?? _server, media ?? Media, settings, clock);
            _open.Add(s);
            return s;
        }

        private string Rom(string name, byte[] bytes)
        {
            string path = Path.Combine(Roms, name);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        private static byte[] Filled(int length, int seed)
        {
            var bytes = new byte[length];
            new Random(seed).NextBytes(bytes);
            return bytes;
        }

        private string Stored(params string[] parts) => Path.Combine([Media, .. parts]);

        private long[] Delays(LibraryScraper s) => s.Get<long[]>(new { what = "delays" });

        // An answer in a character set the reader does not know is an answer that could not be read, and the run goes on to its end.
        [Fact]
        public void An_answer_in_a_charset_the_reader_has_not_is_an_answer_that_could_not_be_read_and_the_run_goes_on()
        {
            string rom = Rom("Charset (USA).sfc", Filled(512, 9));
            _server.Games.Add(new FakeGame(78, "Charset", RomHash.Of(rom).Md5));
            LibraryScraper s = Scraper(new Charset(_server, "klingon"));
            s.Run([(rom, "snes")], Four);
            Assert.Equal(ScrapeOutcome.Error, s.Next().Outcome);
            Assert.Equal("done", s.Ends());
        }

        // A patched package's manifest beside the test's games: its patched image's hashes recorded, and the identity the scraper sends.
        private string PatchedPackage(string name, byte[] patchedImage, string identity)
        {
            string folder = Path.Combine(DataStore.Packages, "SNES", name);
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
            PackagesTree();
            byte[] dump = Filled(2048, 61), hack = Filled(2304, 62), base2 = Filled(2048, 63);
            string original = Rom(Path.Combine("snes", "Original (USA).sfc"), dump), hacked = Rom(Path.Combine("snes", "Hacked (USA).sfc"), base2);
            RomHashes d = RomHash.Of(dump), h = RomHash.Of(hack), d2 = RomHash.Of(base2);
            PatchedPackage("Original (USA).sfc", hack, "original");
            PatchedPackage("Hacked (USA).sfc", hack, "patched");
            _server.Games.Add(new FakeGame(81, "Original", d.Md5));
            _server.Games.Add(new FakeGame(82, "Hacked base", d2.Md5));
            LibraryScraper s = Scraper();

            s.Run([(original, "snes")], Four);
            Assert.Equal(ScrapeOutcome.Found, s.Next().Outcome);
            s.Ends();
            string asked = _server.JeuInfos.Single();
            Assert.Equal((d.Md5, d.Crc32, d.Sha1, "2048"), (FakeScreenScraper.Param(asked, "md5"), FakeScreenScraper.Param(asked, "crc"), FakeScreenScraper.Param(asked, "sha1"), FakeScreenScraper.Param(asked, "romtaille")));

            s.Run([(hacked, "snes")], Four);
            LibraryScraper.Result second = s.Next();
            Assert.Equal(ScrapeOutcome.Found, second.Outcome);
            string[] both = _server.JeuInfos.Skip(1).ToArray();
            Assert.Equal(2, both.Length);
            Assert.Equal((h.Md5, h.Sha1, "2304"), (FakeScreenScraper.Param(both[0], "md5"), FakeScreenScraper.Param(both[0], "sha1"), FakeScreenScraper.Param(both[0], "romtaille")));
            Assert.Equal(d2.Md5, FakeScreenScraper.Param(both[1], "md5"));
            Assert.Equal("file", second.MatchedBy);
        }

        // A packaged game's art goes into its package's media folder by the package's name, and media.db names it there - see EmuSen_GamePackages.md §18.3.
        [Fact]
        public void A_packaged_game_s_art_is_written_into_its_package_by_the_package_s_name_and_not_into_the_store()
        {
            PackagesTree();
            string rom = Rom(Path.Combine("snes", "Packed (USA).sfc"), Filled(4096, 71));
            RomHashes h = RomHash.Of(rom);
            string package = Path.Combine(DataStore.Packages, "SNES", "Packed Package.sfc");
            EmuSen.WiseMan.Crystal.CrystalParityTests.PackageFor(package, Roms, "snes/Packed (USA).sfc", "snes");
            _server.Games.Add(new FakeGame(83, "Packed", h.Md5));
            LibraryScraper s = Scraper();

            s.Run([(rom, "snes")], Four);
            LibraryScraper.Result result = s.Next();

            Assert.Equal(ScrapeOutcome.Found, result.Outcome);
            Assert.True(result.HasCover);
            foreach (string folder in (string[])["covers", "screenshots", "marquees", "miximages"])
                Assert.True(File.Exists(Path.Combine(package, "media", folder, "Packed Package.png")), folder);
            Assert.False(Directory.Exists(Stored("snes")));
            Assert.All(s.Scraper.Media(h.Md5, h.Size), m => Assert.StartsWith(package, m.RelativePath));
        }

        [Fact]
        public void A_game_found_by_its_file_s_hashes_gets_its_text_and_each_default_kind_in_es_de_s_layout()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(4096, 1));
            RomHashes h = RomHash.Of(rom);
            _server.Games.Add(new FakeGame(77, "F-Zero", h.Md5));
            LibraryScraper s = Scraper();

            s.Run([(rom, "snes")], Four);
            LibraryScraper.Result result = s.Next();
            s.Ends();

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

            ScrapedRecord record = s.Scraper.FoundByPath()[rom];
            Assert.Equal(("A synthetic game written for the tests.", "Synthetic Developer", "Synthetic Publisher", "Racing", "1-2", 0.8f, new DateTime(1991, 8, 23)),
                (record.Description, record.Developer, record.Publisher, record.Genre, record.Players, record.Rating, record.ReleaseDate));
            Assert.Equal("F-Zero (US title)", record.Name);
            Assert.Equal(4, s.Scraper.Media(h.Md5, h.Size).Count);
            Assert.Equal(0, s.Scraper.QueueLength);
        }

        [Fact]
        public void A_queue_left_by_an_earlier_run_asks_nothing_until_a_run_is_started_and_the_run_ends_when_it_is_empty()
        {
            string a = Rom("A (USA).sfc", Filled(1024, 80));
            string b = Rom("B (USA).sfc", Filled(1024, 81));
            using (var rows = new MediaDbRows(Media))
            {
                rows.Enqueue(a, "snes", ScrapePriority.Library);
                rows.Enqueue(b, "snes", ScrapePriority.Library);
            }
            LibraryScraper s = Scraper();
            Thread.Sleep(300);
            Assert.Empty(_server.Asked);
            Assert.Equal("idle", s.State);

            s.Run([], Four, ScrapePriority.Library, resume: true);
            s.Next();
            s.Next();
            Assert.Equal("done", s.Ends());
            int asked = _server.Asked.Count;

            // Nothing is asked between runs.
            Rom("C (USA).sfc", Filled(1024, 82));
            Thread.Sleep(300);
            Assert.Equal(asked, _server.Asked.Count);
        }

        [Fact]
        public void A_run_the_quota_stops_ends_as_stopped_and_cancelling_ends_it_as_cancelled()
        {
            _server.ForcedStatus = 430;
            LibraryScraper s = Scraper();
            s.Run([(Rom("A (USA).sfc", Filled(1024, 83)), "snes")], Four);
            Assert.Equal("stopped", s.Ends());

            _server.ForcedStatus = 0;
            _server.Gate = new SemaphoreSlim(0);
            LibraryScraper second = Scraper(media: Path.Combine(_root, "Other"));
            second.Run([(Rom("B (USA).sfc", Filled(1024, 84)), "snes")], Four);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!_server.JeuInfos.Any() && clock.ElapsedMilliseconds < 10_000) Thread.Sleep(5);
            second.Scraper.Cancel();
            Assert.Equal("cancelled", second.Ends());
            _server.Gate.Release(100);
            Assert.Equal(1, second.Scraper.QueueLength);
        }

        [Fact]
        public void The_headerless_hash_is_asked_only_after_a_404_and_only_when_the_core_s_transform_changes_the_bytes()
        {
            byte[] body = Filled(0x4000, 2);
            byte[] ines = new byte[] { (byte)'N', (byte)'E', (byte)'S', 0x1A }.Concat(new byte[12]).Concat(body).ToArray();
            string nes = Rom("Metroid (USA).nes", ines);
            string gb = Rom("Unknown Game (USA).gb", Filled(0x8000, 3));
            _server.Games.Add(new FakeGame(5, "Metroid", RomHash.Of(body).Md5) { SystemId = 3 });
            LibraryScraper s = Scraper();

            s.Run([(nes, "nes")], Four);
            LibraryScraper.Result found = s.Next();
            s.Ends();
            Assert.Equal((ScrapeOutcome.Found, "transformed"), (found.Outcome, found.MatchedBy));
            Assert.Equal([RomHash.Of(ines).Md5, RomHash.Of(body).Md5], _server.JeuInfos.Select(u => FakeScreenScraper.Param(u, "md5")));
            Assert.Equal(body.Length.ToString(), FakeScreenScraper.Param(_server.JeuInfos.Last(), "romtaille"));

            int before = _server.JeuInfos.Count();
            s.Run([(gb, "gb")], Four);
            Assert.Equal(ScrapeOutcome.Unknown, s.Next().Outcome);
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
            string art = Path.Combine(_root, "Artwork");
            Directory.CreateDirectory(Path.Combine(art, "SNES"));
            File.WriteAllBytes(Path.Combine(art, "SNES", "F-Zero (USA).png"), new byte[100]);
            LibraryScraper s = Scraper(art: art);

            s.Run([(rom, "snes")], Four);
            LibraryScraper.Result result = s.Next();
            s.Ends();

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
            LibraryScraper s = Scraper();
            s.Run([(rom, "snes"), (hack, "snes")], Four, ScrapePriority.Library);
            s.Next();
            s.Next();
            s.Ends();
            int asked = _server.Asked.Count;

            s.Run([(rom, "snes"), (hack, "snes")], Four, ScrapePriority.Library);
            Assert.Equal([ScrapeOutcome.Found, ScrapeOutcome.Unknown], new[] { s.Next(), s.Next() }.OrderBy(r => r.Outcome).Select(r => r.Outcome));
            Assert.Equal(asked, _server.Asked.Count);
        }

        [Fact]
        public void A_kind_turned_on_later_is_asked_for_only_when_the_game_offered_it()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(2048, 7));
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            LibraryScraper s = Scraper();
            s.Run([(rom, "snes")], Four);
            s.Next();
            s.Ends();

            s.Run([(rom, "snes")], Four with { TitleScreens = true });
            s.Next();
            s.Ends();
            Assert.Equal(2, _server.JeuInfos.Count());
            Assert.True(File.Exists(Stored("snes", "titlescreens", "F-Zero (USA).png")));
        }

        [Fact]
        public void A_kind_turned_on_later_that_the_game_never_offered_costs_no_request()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(2048, 70));
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5) { Media = [("box-2D", "us", "png"), ("ss", "wor", "png")] });
            LibraryScraper s = Scraper();
            s.Run([(rom, "snes")], Four);
            s.Next();
            s.Ends();

            s.Run([(rom, "snes")], Four with { TitleScreens = true });
            Assert.Equal(ScrapeOutcome.Found, s.Next().Outcome);
            Assert.Single(_server.JeuInfos);
        }

        [Fact]
        public void A_renamed_file_takes_its_media_with_it_and_a_second_copy_gets_its_own_with_no_request()
        {
            byte[] bytes = Filled(2048, 8);
            string rom = Rom("F-Zero (USA).sfc", bytes);
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            LibraryScraper s = Scraper();
            s.Run([(rom, "snes")], Four);
            s.Next();
            s.Ends();
            int asked = _server.Asked.Count;

            string renamed = Path.Combine(Roms, "F-Zero.sfc");
            File.Move(rom, renamed);
            s.Run([(renamed, "snes")], Four);
            Assert.Equal(ScrapeOutcome.Found, s.Next().Outcome);
            s.Ends();
            Assert.True(File.Exists(Stored("snes", "covers", "F-Zero.png")));
            Assert.False(File.Exists(Stored("snes", "covers", "F-Zero (USA).png")));

            string copy = Rom("F-Zero (copy).sfc", bytes);
            s.Run([(copy, "snes")], Four);
            Assert.Equal(ScrapeOutcome.Found, s.Next().Outcome);
            s.Ends();
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
            LibraryScraper s = Scraper();
            s.Run([(rom, "snes")], Four);
            LibraryScraper.Result first = s.Next();
            s.Ends();

            string[] kinds = ["covers", "screenshots", "marquees", "miximages"];
            foreach (string folder in kinds) Assert.True(File.Exists(Stored("snes", folder, "USA", "F-Zero (USA).png")), folder);
            Assert.False(File.Exists(Stored("snes", "covers", "F-Zero (USA).png")));
            Assert.All(s.Scraper.Media(h.Md5, h.Size), m => Assert.Equal(Path.Combine("snes", MediaStore.KindFolderOf(m.RelativePath), "USA", Path.GetFileName(m.RelativePath)), m.RelativePath));
            Assert.Equal(["cover", "marquee", "miximage", "screenshot"], first.Kinds.Order());
            int asked = _server.Asked.Count;

            string moved = Path.Combine(Roms, "SNES", "Racing", "F-Zero (USA).sfc");
            File.Move(rom, moved);
            s.Run([(moved, "snes")], Four);
            Assert.Equal(ScrapeOutcome.Found, s.Next().Outcome);
            s.Ends();
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
            LibraryScraper s = Scraper();
            var covers = Four with { Screenshots = false, Marquees = false, Miximages = false };
            s.Run([(eu, "snes")], covers);
            s.Next();
            s.Ends();
            s.Run([(kr, "snes")], covers);
            s.Next();
            s.Ends();

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
            LibraryScraper s = Scraper();
            s.Run([(a, "snes"), (b, "snes")], Four);

            Assert.Equal("stopped", s.Ends());
            using (var p = s.Progress()) Assert.Contains(code.ToString(), p.RootElement.GetProperty("why").GetString());
            s.NothingMore();
            Assert.Single(_server.JeuInfos);
            Assert.Equal(2, s.Scraper.QueueLength);
            Assert.Equal(QuotaGate.Stopped, s.Scraper.Quota().Gate);
        }

        [Fact]
        public void After_the_day_s_stop_the_next_day_resumes_where_it_stopped_even_after_a_restart()
        {
            string a = Rom("A (USA).sfc", Filled(1024, 13));
            string b = Rom("B (USA).sfc", Filled(1024, 14));
            _server.Games.Add(new FakeGame(1, "A", RomHash.Of(a).Md5));
            _server.Games.Add(new FakeGame(2, "B", RomHash.Of(b).Md5));
            _server.ForcedStatus = 430;
            LibraryScraper s = Scraper();
            s.Run([(a, "snes")], Four, ScrapePriority.Shown);
            Assert.Equal("stopped", s.Ends());
            s.Dispose();
            _open.Remove(s);
            using (var rows = new MediaDbRows(Media)) rows.Enqueue(b, "snes", ScrapePriority.Library);

            _server.ForcedStatus = 0;
            LibraryScraper restarted = Scraper();
            Assert.Equal(QuotaGate.Stopped, restarted.Scraper.Quota().Gate);
            restarted.Dispose();
            _open.Remove(restarted);
            LibraryScraper tomorrow = Scraper(clock: new DateTimeOffset(2026, 7, 14, 22, 0, 1, TimeSpan.Zero));
            tomorrow.Run([], Four, ScrapePriority.Library, resume: true);

            Assert.Equal([a, b], new[] { tomorrow.Next(), tomorrow.Next() }.Select(r => r.Path));
            tomorrow.Ends();
            Assert.Equal(0, tomorrow.Scraper.QueueLength);
        }

        [Fact]
        public void An_interrupted_queue_resumes_without_asking_again_for_what_was_done()
        {
            var roms = Enumerable.Range(0, 4).Select(i => Rom($"G{i} (USA).sfc", Filled(1024, 20 + i))).ToList();
            foreach ((string rom, int i) in roms.Select((r, i) => (r, i))) _server.Games.Add(new FakeGame(100 + i, $"G{i}", RomHash.Of(rom).Md5));
            _server.Gate = new SemaphoreSlim(0);
            var covers = Four with { Screenshots = false, Marquees = false, Miximages = false };
            LibraryScraper s = Scraper();
            s.Run(roms.Select(r => (r, "snes")), covers, ScrapePriority.Library);
            _server.Gate.Release(2);
            Assert.Equal(roms[0], s.Next().Path);
            s.Scraper.Cancel();
            s.Ends();
            _server.Gate.Release(100);
            _server.Gate = null;
            int asked = _server.JeuInfos.Count();

            s.Run([], covers, ScrapePriority.Library, resume: true);
            Assert.Equal(roms.Skip(1), new[] { s.Next(), s.Next(), s.Next() }.Select(r => r.Path));
            Assert.Equal(asked + 3, _server.JeuInfos.Count());
            Assert.Equal(roms.Count, _server.JeuInfos.Select(u => FakeScreenScraper.Param(u, "md5")).Distinct().Count());
        }

        [Fact]
        public void The_default_asks_for_every_thread_the_account_allows()
        {
            Assert.Equal(0, new EmuSen.Galaxia.Models.AppSettings().ScrapeThreads);
            Assert.Equal(int.MaxValue, ScrapeRules.ThreadsWanted(0));
            Assert.Equal(3, ScrapeRules.ThreadsWanted(3));
        }

        [Fact]
        public void A_picture_that_came_faster_than_maxdownloadspeed_is_followed_by_a_wait_for_the_difference()
        {
            _server.User!["maxdownloadspeed"] = "1";
            string a = Rom("A (USA).sfc", Filled(1024, 62));
            _server.Games.Add(new FakeGame(1, "A", RomHash.Of(a).Md5) { Media = [("box-2D", "us", "png")] });
            LibraryScraper s = Scraper();
            s.Run([(a, "snes")], Four);
            s.Next();
            s.Ends();
            Assert.Contains(Delays(s), d => Math.Abs(d / 1e7 - 400 / 1024.0) < 0.05);
        }

        [Fact]
        public void Every_request_waits_its_turn_at_the_pace_and_an_unknown_game_counts_against_the_ko_allowance()
        {
            _server.User = null;
            string a = Rom("A (USA).sfc", Filled(1024, 60));
            string b = Rom("B (USA).sfc", Filled(1024, 61));
            _server.Games.Add(new FakeGame(1, "A", RomHash.Of(a).Md5) { Media = [("box-2D", "us", "png")] });
            LibraryScraper s = Scraper();
            s.Run([(a, "snes"), (b, "snes")], Four);
            s.Next();
            s.Next();
            s.Ends();

            QuotaSnapshot q = s.Scraper.Quota();
            Assert.Equal((2, 1), (q.RequestsToday, q.KoToday));
            long[] waits = Delays(s).Where(d => d > 0).ToArray();
            Assert.Equal(2, waits.Length);
            Assert.All(waits, d => Assert.Equal(q.Interval.Ticks, d));
        }
    }
}

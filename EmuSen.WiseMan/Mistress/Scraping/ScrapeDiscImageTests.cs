using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Text;
using EmuSen.Cores;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Disc and large games against the fake server, through the library's run: the file the library lists is the one asked about, at any size, and a failure is recorded and shown - see EmuSen_BigPicture.md §46 and EmuSen_Backend.md §40.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeDiscImageTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeDiscTests", Guid.NewGuid().ToString("N"));
        private readonly FakeScreenScraper _server = new();
        private readonly List<IDisposable> _open = new();

        private string Roms => Path.Combine(_root, "Roms");
        private string Media => Path.Combine(_root, "Media");

        public ScrapeDiscImageTests() => Directory.CreateDirectory(Roms);

        public void Dispose()
        {
            foreach (IDisposable d in Enumerable.Reverse(_open)) d.Dispose();
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private LibraryScraper Start()
        {
            var s = new LibraryScraper(_server, Media, new Dictionary<string, object?> { ["roms"] = Roms }, new DateTimeOffset(2026, 10, 16, 10, 0, 0, TimeSpan.Zero));
            _open.Add(s);
            return s;
        }

        private static LibraryScraper.Result Queue(LibraryScraper s, string path, string system)
        {
            s.Run([(path, system)], new ScrapeChoices());
            LibraryScraper.Result r = s.Next(60);
            s.Ends(60);
            return r;
        }

        private string File_(string relative, byte[] bytes)
        {
            string path = Path.Combine(Roms, relative);
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

        // A file past the 384 MiB the scraper once refused, sparse so it costs no disk, marked so its hashes are its own.
        private string Large(string name, int seed)
        {
            string path = Path.Combine(Roms, name);
            using (var stream = new FileStream(path, FileMode.Create, FileAccess.Write))
            {
                stream.Write(Filled(4096, seed));
                stream.SetLength((384L << 20) + 2352 * 16);
            }
            return path;
        }

        private (string Md5, string Romnom, string Romtaille, string System) Asked()
        {
            string url = _server.JeuInfos.Last();
            return (FakeScreenScraper.Param(url, "md5"), FakeScreenScraper.Param(url, "romnom"), FakeScreenScraper.Param(url, "romtaille"), FakeScreenScraper.Param(url, "systemeid"));
        }

        [Fact]
        public void A_cue_sheet_game_is_asked_by_the_sheet_s_own_hashes_size_and_name_and_its_media_follow_the_sheet()
        {
            File_("psx/Synthetic Disc (USA)/Synthetic Disc (USA) (Track 01).bin", Filled(2352 * 64, 1));
            File_("psx/Synthetic Disc (USA)/Synthetic Disc (USA) (Track 02).bin", Filled(2352 * 32, 2));
            string cue = File_("psx/Synthetic Disc (USA)/Synthetic Disc (USA).cue", Encoding.ASCII.GetBytes(
                "FILE \"Synthetic Disc (USA) (Track 01).bin\" BINARY\r\n  TRACK 01 MODE2/2352\r\n    INDEX 01 00:00:00\r\n" +
                "FILE \"Synthetic Disc (USA) (Track 02).bin\" BINARY\r\n  TRACK 02 AUDIO\r\n    INDEX 00 00:00:00\r\n    INDEX 01 00:02:00\r\n"));
            RomHashes sheet = RomHash.Of(cue);
            _server.Games.Add(new FakeGame(19275, "Synthetic Disc", sheet.Md5) { SystemId = 57 });
            LibraryScraper.Result result = Queue(Start(), cue, "psx");

            Assert.Equal(ScrapeOutcome.Found, result.Outcome);
            Assert.Equal((sheet.Md5, "Synthetic Disc (USA).cue", sheet.Size.ToString(), "57"), Asked());
            Assert.True(File.Exists(Path.Combine(Media, "psx", "covers", "Synthetic Disc (USA)", "Synthetic Disc (USA).png")));
        }

        [Fact]
        public void A_lone_bin_past_the_old_384_mib_cap_is_hashed_whole_found_and_not_read_again_for_its_next_turn()
        {
            string bin = Large("Synthetic Racing (USA).bin", 3);
            var info = new FileInfo(bin);
            RomHashes whole = RomHash.Of(bin);
            _server.Games.Add(new FakeGame(19700, "Synthetic Racing", whole.Md5) { SystemId = 57 });
            LibraryScraper scraper = Start();
            LibraryScraper.Result result = Queue(scraper, bin, "psx");

            Assert.Equal(ScrapeOutcome.Found, result.Outcome);
            Assert.Equal((whole.Md5, "Synthetic Racing (USA).bin", info.Length.ToString(), "57"), Asked());

            // Changed in place with its size and time kept, the file is known by the cached hash: it is not read, and nothing is asked.
            DateTime written = info.LastWriteTimeUtc;
            using (var stream = new FileStream(bin, FileMode.Open, FileAccess.Write)) stream.Write(Filled(4096, 4));
            File.SetLastWriteTimeUtc(bin, written);
            int asked = _server.JeuInfos.Count();
            LibraryScraper.Result again = Queue(scraper, bin, "psx");
            Assert.Equal((ScrapeOutcome.Found, true, "already scraped"), (again.Outcome, again.Skipped, again.Detail));
            Assert.Equal(asked, _server.JeuInfos.Count());
        }

        [Fact]
        public void A_chd_past_the_old_cap_is_hashed_whole_and_found()
        {
            string chd = Large("Synthetic Hedgehog CD (USA).chd", 5);
            RomHashes whole = RomHash.Of(chd);
            _server.Games.Add(new FakeGame(12702, "Synthetic Hedgehog CD", whole.Md5) { SystemId = 20 });
            LibraryScraper.Result result = Queue(Start(), chd, "segacd");

            Assert.Equal(ScrapeOutcome.Found, result.Outcome);
            Assert.Equal((whole.Md5, "Synthetic Hedgehog CD (USA).chd", new FileInfo(chd).Length.ToString(), "20"), Asked());
        }

        [Fact]
        public void An_m3u_is_asked_by_the_list_s_own_hashes_size_and_name()
        {
            File_("Synthetic Saga (USA) (Disc 1).cue", Encoding.ASCII.GetBytes("FILE \"Synthetic Saga (USA) (Disc 1).bin\" BINARY\r\n  TRACK 01 MODE2/2352\r\n    INDEX 01 00:00:00\r\n"));
            File_("Synthetic Saga (USA) (Disc 1).bin", Filled(2352 * 16, 6));
            string m3u = File_("Synthetic Saga (USA).m3u", Encoding.ASCII.GetBytes("Synthetic Saga (USA) (Disc 1).cue\n"));
            RomHashes list = RomHash.Of(m3u);
            _server.Games.Add(new FakeGame(214640, "Synthetic Saga", list.Md5) { SystemId = 57 });
            LibraryScraper.Result result = Queue(Start(), m3u, "psx");

            Assert.Equal(ScrapeOutcome.Found, result.Outcome);
            Assert.Equal((list.Md5, "Synthetic Saga (USA).m3u", list.Size.ToString(), "57"), Asked());
        }

        [Fact]
        public void A_refused_request_is_recorded_with_its_reason_and_named_in_the_run_s_line()
        {
            string bin = File_("Synthetic Refusal (USA).bin", Filled(2352 * 8, 7));
            RomHashes h = RomHash.Of(bin);
            _server.ForcedStatus = 400;
            LibraryScraper scraper = Start();
            var run = new ScrapeProgress(1);

            LibraryScraper.Result result = Queue(scraper, bin, "psx");
            run.Count(new ScrapeResult(result.Path, result.System, result.Outcome, result.HasCover, [], result.Detail));
            run.State = ScrapeRunState.Done;

            Assert.Equal(ScrapeOutcome.Error, result.Outcome);
            Assert.StartsWith("ScreenScraper refused the request", result.Detail);
            Assert.Equal(ScrapeState.Error, scraper.Scraper.OutcomesByPath()[bin].State);
            using (var d = System.Text.Json.JsonDocument.Parse(scraper.Scraper.Service.Get(System.Text.Json.JsonSerializer.Serialize(new { what = "game", md5 = h.Md5, bytes = h.Size }))))
                Assert.StartsWith("ScreenScraper refused the request", NativeScraping.Record(d.RootElement).Detail);
            Assert.Equal("Scraped 1 of 1: 0 found, 0 unknown, 1 failed. Last failure: " + result.Detail, run.Describe());
        }
    }
}

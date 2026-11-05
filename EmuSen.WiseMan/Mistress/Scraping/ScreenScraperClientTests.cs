using System;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The library's client against a fake server: what it sends, how it reads an answer, and which files it keeps. Each documented code's meaning is emusen-scrape's quota tests', a found game read whole is its answers test's, and a file already there is ScraperTests' - see EmuSen_BigPicture.md §17 and EmuSen_Backend.md §40.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScreenScraperClientTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScreenScraperClientTests", Guid.NewGuid().ToString("N"));

        public ScreenScraperClientTests()
        {
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Rom(string name, int length)
        {
            string path = Path.Combine(_root, "Roms", "snes", name);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            var bytes = new byte[length];
            new Random(length).NextBytes(bytes);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        // The first jeuInfos a one-game run sends.
        private string Asked(FakeScreenScraper server, string rom, string media)
        {
            using var s = new LibraryScraper(server, Path.Combine(_root, media));
            s.Run([(rom, "snes")]);
            s.Ends();
            return server.JeuInfos.First();
        }

        [Fact]
        public void Jeu_infos_carries_the_credentials_json_the_three_hashes_the_size_the_bare_file_name_and_the_system()
        {
            string rom = Rom("F-Zero (USA).sfc", 4096);
            RomHashes h = RomHash.Of(rom);
            string url = Asked(new FakeScreenScraper(), rom, "Media");

            Assert.StartsWith("https://api.screenscraper.fr/api2/jeuInfos.php?", url);
            Assert.Equal(FakeScreenScraper.DevId, FakeScreenScraper.Param(url, "devid"));
            Assert.Equal(FakeScreenScraper.DevPassword, FakeScreenScraper.Param(url, "devpassword"));
            Assert.Equal(FakeScreenScraper.SoftName, FakeScreenScraper.Param(url, "softname"));
            Assert.Equal("json", FakeScreenScraper.Param(url, "output"));
            Assert.Equal("rom", FakeScreenScraper.Param(url, "romtype"));
            Assert.Equal("4", FakeScreenScraper.Param(url, "systemeid"));
            Assert.Equal(h.Crc32, FakeScreenScraper.Param(url, "crc"));
            Assert.Equal(h.Md5, FakeScreenScraper.Param(url, "md5"));
            Assert.Equal(h.Sha1, FakeScreenScraper.Param(url, "sha1"));
            Assert.Equal("4096", FakeScreenScraper.Param(url, "romtaille"));
            Assert.Equal("F-Zero (USA).sfc", FakeScreenScraper.Param(url, "romnom"));
            Assert.DoesNotContain("ssid=", url);
            Assert.DoesNotContain("sspassword=", url);
        }

        [Fact]
        public void The_member_account_is_sent_only_when_both_halves_are_set()
        {
            string rom = Rom("A (USA).sfc", 1024);
            new MemberAccount("FAKEMEMBER", "FAKEMEMBERPW") { Verified = DateTime.UtcNow }.Save();
            string with = Asked(new FakeScreenScraper(), rom, "With");
            new MemberAccount("FAKEMEMBER", "") { Verified = DateTime.UtcNow }.Save();
            string half = Asked(new FakeScreenScraper(), rom, "Half");
            Assert.Equal("FAKEMEMBER", FakeScreenScraper.Param(with, "ssid"));
            Assert.Equal("FAKEMEMBERPW", FakeScreenScraper.Param(with, "sspassword"));
            Assert.DoesNotContain("ssid=", half);
        }

        [Fact]
        public void A_game_not_found_is_not_found_and_an_answer_of_another_shape_is_malformed()
        {
            string rom = Rom("X (USA).sfc", 2048);
            using (var s = new LibraryScraper(new FakeScreenScraper(), Path.Combine(_root, "NotFound")))
            {
                s.Run([(rom, "snes")]);
                Assert.Equal(ScrapeOutcome.Unknown, s.Next().Outcome);
            }
            var odd = new Answering(_ => new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("<html>maintenance</html>") });
            using (var s = new LibraryScraper(odd, Path.Combine(_root, "Odd")))
            {
                s.Run([(rom, "snes")]);
                LibraryScraper.Result r = s.Next();
                Assert.Equal(ScrapeOutcome.Error, r.Outcome);
                Assert.StartsWith("ScreenScraper's answer could not be read", r.Detail);
            }
        }

        // The fake server, its pictures answered by the test.
        private sealed class Pictures(FakeScreenScraper server, Func<HttpResponseMessage> picture) : DelegatingHandler(server)
        {
            protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken) =>
                request.RequestUri!.AbsoluteUri.Contains("/mediaJeu.php") ? Task.FromResult(picture()) : base.SendAsync(request, cancellationToken);
        }

        [Fact]
        public void A_picture_is_kept_under_its_final_name_and_nothing_else_is()
        {
            string rom = Rom("Game (USA).sfc", 3000);
            Func<HttpResponseMessage>[] refused =
            [
                () => new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("NOMEDIA") },
                () => new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent(new string('x', 500), Encoding.UTF8, "text/html") },
                () => Png(79),
                () => new HttpResponseMessage(HttpStatusCode.NotFound),
            ];
            for (int i = 0; i < refused.Length; i++)
            {
                var server = new FakeScreenScraper();
                server.Games.Add(new FakeGame(77, "Game", RomHash.Of(rom).Md5));
                string media = Path.Combine(_root, $"Refused{i}");
                using var s = new LibraryScraper(new Pictures(server, refused[i]), media);
                s.Run([(rom, "snes")], new ScrapeChoices { Screenshots = false, Marquees = false, Miximages = false, TitleScreens = false, BackCovers = false, Boxes3D = false, PhysicalMedia = false, FanArt = false });
                Assert.False(s.Next().HasCover);
                s.Ends();
                Assert.Empty(Directory.EnumerateFiles(media, "*", SearchOption.AllDirectories).Where(f => !Path.GetFileName(f).StartsWith(MediaStore.FileName, StringComparison.Ordinal)));
            }

            var kept = new FakeScreenScraper();
            kept.Games.Add(new FakeGame(77, "Game", RomHash.Of(rom).Md5));
            string store = Path.Combine(_root, "Kept");
            using (var s = new LibraryScraper(new Pictures(kept, () => Png(80)), store))
            {
                s.Run([(rom, "snes")], new ScrapeChoices { Screenshots = false, Marquees = false, Miximages = false, TitleScreens = false, BackCovers = false, Boxes3D = false, PhysicalMedia = false, FanArt = false });
                Assert.True(s.Next().HasCover);
                s.Ends();
            }
            string cover = Path.Combine(store, "snes", "covers", "Game (USA).png");
            Assert.Equal(80, new FileInfo(cover).Length);
            Assert.Empty(Directory.EnumerateFiles(store, "*.part", SearchOption.AllDirectories));
        }

        [Fact]
        public void The_three_hashes_come_from_one_read_and_the_md5_is_rom_hash_s()
        {
            string file = Path.Combine(_root, "check.bin");
            File.WriteAllBytes(file, Encoding.ASCII.GetBytes("123456789"));
            RomHashes h = RomHash.Of(file);
            Assert.Equal("cbf43926", h.Crc32);
            Assert.Equal("25f9e794323b453885f5181f1b624d0b", h.Md5);
            Assert.Equal("f7c3bc1d808e04732adf679965ccc34ca7ae3441", h.Sha1);
            Assert.Equal(9, h.Size);
            Assert.Equal(RomHash.Md5(file), h.Md5);
            Assert.Equal(h, RomHash.Of(File.ReadAllBytes(file)));
        }

        private static HttpResponseMessage Png(int bytes)
        {
            var content = new ByteArrayContent(new byte[bytes]);
            content.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue("image/png");
            return new HttpResponseMessage(HttpStatusCode.OK) { Content = content };
        }

        public sealed class Answering(Func<HttpRequestMessage, HttpResponseMessage> answer) : HttpMessageHandler
        {
            public int Count;

            protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
            {
                Interlocked.Increment(ref Count);
                return Task.FromResult(answer(request));
            }
        }
    }
}

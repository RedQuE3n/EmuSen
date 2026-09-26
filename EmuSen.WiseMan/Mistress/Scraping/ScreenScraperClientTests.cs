using System;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The client against a fake server: what it sends, how it reads an answer, and which files it keeps - see EmuSen_BigPicture.md §17.
    public class ScreenScraperClientTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScreenScraperClientTests", Guid.NewGuid().ToString("N"));

        public ScreenScraperClientTests() => Directory.CreateDirectory(_root);

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private static readonly RomHashes Hashes = new("0123456789abcdef0123456789abcdef", "cbf43926", "0123456789abcdef0123456789abcdef01234567", 524288);

        [Fact]
        public void Jeu_infos_carries_the_credentials_json_the_three_hashes_the_size_the_bare_file_name_and_the_system()
        {
            var client = new ScreenScraperClient(new HttpClient(), FakeScreenScraper.Developer, null);
            string url = client.JeuInfosUrl(4, Hashes, "/some/where/F-Zero (USA).sfc");

            Assert.StartsWith("https://api.screenscraper.fr/api2/jeuInfos.php?", url);
            Assert.Equal(FakeScreenScraper.DevId, FakeScreenScraper.Param(url, "devid"));
            Assert.Equal(FakeScreenScraper.DevPassword, FakeScreenScraper.Param(url, "devpassword"));
            Assert.Equal(FakeScreenScraper.SoftName, FakeScreenScraper.Param(url, "softname"));
            Assert.Equal("json", FakeScreenScraper.Param(url, "output"));
            Assert.Equal("rom", FakeScreenScraper.Param(url, "romtype"));
            Assert.Equal("4", FakeScreenScraper.Param(url, "systemeid"));
            Assert.Equal(Hashes.Crc32, FakeScreenScraper.Param(url, "crc"));
            Assert.Equal(Hashes.Md5, FakeScreenScraper.Param(url, "md5"));
            Assert.Equal(Hashes.Sha1, FakeScreenScraper.Param(url, "sha1"));
            Assert.Equal("524288", FakeScreenScraper.Param(url, "romtaille"));
            Assert.Equal("F-Zero (USA).sfc", FakeScreenScraper.Param(url, "romnom"));
            Assert.DoesNotContain("ssid=", url);
            Assert.DoesNotContain("sspassword=", url);
        }

        [Fact]
        public void The_member_account_is_sent_only_when_both_halves_are_set()
        {
            string with = new ScreenScraperClient(new HttpClient(), FakeScreenScraper.Developer, new MemberAccount("FAKEMEMBER", "FAKEMEMBERPW")).JeuInfosUrl(4, Hashes, "a.sfc");
            string half = new ScreenScraperClient(new HttpClient(), FakeScreenScraper.Developer, new MemberAccount("FAKEMEMBER", "")).JeuInfosUrl(4, Hashes, "a.sfc");
            Assert.Equal("FAKEMEMBER", FakeScreenScraper.Param(with, "ssid"));
            Assert.Equal("FAKEMEMBERPW", FakeScreenScraper.Param(with, "sspassword"));
            Assert.DoesNotContain("ssid=", half);
        }

        [Theory]
        [InlineData(200, ScrapeStatus.Found)]
        [InlineData(400, ScrapeStatus.BadRequest)]
        [InlineData(401, ScrapeStatus.ServerBusy)]
        [InlineData(403, ScrapeStatus.BadCredentials)]
        [InlineData(404, ScrapeStatus.NotFound)]
        [InlineData(423, ScrapeStatus.ApiClosed)]
        [InlineData(426, ScrapeStatus.Blacklisted)]
        [InlineData(429, ScrapeStatus.TooManyRequests)]
        [InlineData(430, ScrapeStatus.DailyQuota)]
        [InlineData(431, ScrapeStatus.DailyKoQuota)]
        [InlineData(500, ScrapeStatus.Failed)]
        public void Each_documented_code_has_its_own_meaning(int code, ScrapeStatus status)
        {
            Assert.Equal(status, ScreenScraperClient.StatusOf((HttpStatusCode)code));
        }

        [Fact]
        public async Task A_found_game_is_read_whole_with_its_quota()
        {
            var server = new FakeScreenScraper();
            server.Games.Add(new FakeGame(77, "F-Zero", Hashes.Md5));
            var client = new ScreenScraperClient(new HttpClient(server), FakeScreenScraper.Developer, null);

            JeuInfosAnswer answer = await client.JeuInfosAsync(4, Hashes, "F-Zero (USA).sfc", CancellationToken.None);

            Assert.Equal(ScrapeStatus.Found, answer.Status);
            ScrapedGame game = answer.Game!;
            Assert.Equal(77, game.Id);
            Assert.Equal(770, game.RomId);
            Assert.Equal(4, game.SystemId);
            Assert.Equal("F-Zero", game.Names.Single(n => n.Key == "ss").Text);
            Assert.Equal("Synthetic Developer", game.Developer);
            Assert.Equal("Synthetic Publisher", game.Publisher);
            Assert.Equal("1-2", game.Players);
            Assert.Equal(15.0, game.Note);
            Assert.Equal(["fr", "en"], game.Synopses.Select(s => s.Key));
            Assert.Contains(game.Media, m => m.Type == "mixrbv2" && m.Region == "wor");
            Assert.Equal(new ScrapeQuota(1, 128, 10, 1, 60, 20000, 2000), answer.Quota);
        }

        [Fact]
        public async Task A_game_not_found_is_not_found_and_an_answer_of_another_shape_is_malformed()
        {
            var server = new FakeScreenScraper();
            var client = new ScreenScraperClient(new HttpClient(server), FakeScreenScraper.Developer, null);
            Assert.Equal(ScrapeStatus.NotFound, (await client.JeuInfosAsync(4, Hashes, "x.sfc", CancellationToken.None)).Status);

            var odd = new Answering(_ => new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("<html>maintenance</html>") });
            JeuInfosAnswer answer = await new ScreenScraperClient(new HttpClient(odd), FakeScreenScraper.Developer, null).JeuInfosAsync(4, Hashes, "x.sfc", CancellationToken.None);
            Assert.Equal(ScrapeStatus.Malformed, answer.Status);
        }

        [Fact]
        public async Task A_picture_is_kept_under_its_final_name_and_nothing_else_is()
        {
            string target = Path.Combine(_root, "snes", "covers", "Game.png");
            async Task<MediaAnswer> Get(HttpResponseMessage response) =>
                await new ScreenScraperClient(new HttpClient(new Answering(_ => response)), FakeScreenScraper.Developer, null).DownloadAsync("https://x/mediaJeu.php", target, CancellationToken.None);

            Assert.Equal(MediaOutcome.NoMedia, (await Get(new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("NOMEDIA") })).Outcome);
            Assert.Equal(MediaOutcome.NotAnImage, (await Get(new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent(new string('x', 500), Encoding.UTF8, "text/html") })).Outcome);
            Assert.Equal(MediaOutcome.TooSmall, (await Get(Png(79))).Outcome);
            Assert.Equal(MediaOutcome.Failed, (await Get(new HttpResponseMessage(HttpStatusCode.NotFound))).Outcome);
            Assert.False(File.Exists(target));
            Assert.Empty(Directory.EnumerateFiles(_root, "*", SearchOption.AllDirectories));

            MediaAnswer saved = await Get(Png(80));
            Assert.Equal(MediaOutcome.Saved, saved.Outcome);
            Assert.Equal(80, new FileInfo(target).Length);
            Assert.Equal(40, saved.Sha1!.Length);
            Assert.False(File.Exists(target + ".part"));
        }

        [Fact]
        public async Task A_file_already_there_is_never_replaced_or_even_asked_for()
        {
            string target = Path.Combine(_root, "Game.png");
            File.WriteAllBytes(target, [1, 2, 3]);
            var asked = new Answering(_ => Png(400));
            MediaAnswer answer = await new ScreenScraperClient(new HttpClient(asked), FakeScreenScraper.Developer, null).DownloadAsync("https://x/m", target, CancellationToken.None);
            Assert.Equal(MediaOutcome.AlreadyThere, answer.Outcome);
            Assert.Equal(0, asked.Count);
            Assert.Equal(new byte[] { 1, 2, 3 }, File.ReadAllBytes(target));
        }

        [Fact]
        public void The_three_hashes_come_from_one_read_and_the_md5_is_rom_hash_s()
        {
            string file = Path.Combine(_root, "check.bin");
            File.WriteAllBytes(file, Encoding.ASCII.GetBytes("123456789"));
            RomHashes h = RomHashes.Of(file);
            Assert.Equal("cbf43926", h.Crc32);
            Assert.Equal("25f9e794323b453885f5181f1b624d0b", h.Md5);
            Assert.Equal("f7c3bc1d808e04732adf679965ccc34ca7ae3441", h.Sha1);
            Assert.Equal(9, h.Size);
            Assert.Equal(RomHash.Md5(file), h.Md5);
            Assert.Equal(h, RomHashes.Of(File.ReadAllBytes(file)));
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

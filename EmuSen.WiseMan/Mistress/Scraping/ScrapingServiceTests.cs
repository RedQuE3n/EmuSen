using System;
using System.IO;
using System.Text.Json;
using EmuSen.Crystal.Client.Library;
using EmuSen.Crystal.Client.Scrape;
using EmuSen.DianaSh.DianaSh.Bin.Native;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The process's one scraper, opened by the program and answered in its shells, refused there once it closes; a later open closing the one before; the theme's kinds a system draws - see EmuSen_LibraryTools.md §8.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class ScrapingServiceTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapingService", Guid.NewGuid().ToString("N"));

        private const string Offline = "{\"developer_files\":false,\"openemu\":false,\"test\":{\"offline\":true}}";

        private static readonly byte[] Blob = ScrapeCredentialTests.Scramble(JsonSerializer.Serialize(new { devid = FakeScreenScraper.DevId, devpassword = FakeScreenScraper.DevPassword, softname = FakeScreenScraper.SoftName }));

        public void Dispose()
        {
            MediaService.CloseAny();
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        [Fact]
        public void A_shell_answers_from_the_program_s_scraper_and_refuses_once_it_closes()
        {
            Assert.True(MediaService.Ready, MediaService.Report);
            if (!DianaShNative.Ready) return;
            Directory.CreateDirectory(_root);
            using var library = GameLibrary.Open(Path.Combine(_root, "games.db"));
            var service = MediaService.Open(Path.Combine(_root, "Media"), Offline, library, Blob, ["snes"], system => system == "snes" ? ["marquee"] : null);
            var shell = new NativeShell(keepsSessions: false);
            var (_, status, _) = shell.Submit(0, "scrape status");
            Assert.Contains("developer\tembedded", status);
            Assert.DoesNotContain(FakeScreenScraper.DevId, status);
            service.Dispose();
            var (_, refused, _) = shell.Submit(0, "scrape status");
            Assert.Contains("not open now", refused);
        }

        [Fact]
        public void An_open_closes_the_scraper_before_it_whose_calls_are_then_refused()
        {
            Assert.True(MediaService.Ready, MediaService.Report);
            Directory.CreateDirectory(_root);
            var first = MediaService.Open(Path.Combine(_root, "First"), Offline);
            using var second = MediaService.Open(Path.Combine(_root, "Second"), Offline);
            Assert.Throws<ObjectDisposedException>(() => first.Get("{\"what\":\"queue\"}"));
            first.Dispose();
            Assert.Equal("[]", second.Get("{\"what\":\"queue\"}"));
            using var again = MediaService.Open(Path.Combine(_root, "First"), Offline);
            Assert.Equal("[]", again.Get("{\"what\":\"queue\"}"));
        }

        [Fact]
        public void The_theme_s_kinds_are_its_gamelist_image_types_videos_and_variant_triggers()
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("<variant name=\"full\"><label>Full</label><override><trigger>noMedia</trigger><mediaType>3dbox, cover</mediaType><useVariant>full</useVariant></override></variant>")
                .Theme("<variant name=\"full\"><view name=\"gamelist\"><image name=\"art\"><imageType>miximage, screenshot</imageType></image><video name=\"clip\"><imageType>marquee</imageType></video>" +
                       "<image name=\"logo\"><path>./logo.png</path></image></view><view name=\"system\"><carousel name=\"systemcarousel\"/></view></variant>");
            ResolvedTheme loaded = theme.Load(new ThemeChoices { Variant = "full" }, SyntheticTheme.Snes, new MediaPresence(new System.Collections.Generic.HashSet<string>(ThemeCapabilities.MediaTypes)));
            Assert.Equal(["miximage", "screenshot", "marquee", "video", "3dbox", "cover"], ThemeMediaKinds.Of(loaded));
            Assert.Equal(ThemeMediaKinds.Of(loaded), ThemeMediaKinds.For(theme.Root, new ThemeChoices { Variant = "full" }, "snes", "Super Nintendo"));
        }
    }
}

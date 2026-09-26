using System;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The themed view fed from the media store: ScreenScraper's text in its metadata and its pictures in its image elements - see EmuSen_BigPicture.md §17.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedScrapeTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedScrapeTests).GetTypeInfo().Assembly);

        private readonly FakeScreenScraper _server = new();

        public ThemedScrapeTests()
        {
            NoNetwork.HttpFactory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(_server)));
            NoNetwork.ScrapeClock.SetValue(null, new FakeScrapeClock(new DateTimeOffset(2026, 7, 14, 10, 0, 0, TimeSpan.Zero)));
            NoNetwork.DeveloperSource.SetValue(null, (Func<DeveloperCredentials?>)(() => FakeScreenScraper.Developer));
        }

        public void Dispose()
        {
            NoNetwork.Refuse();
            NoNetwork.ScrapeClock.SetValue(null, SystemScrapeClock.Instance);
        }

        [Fact]
        public Task The_selected_game_is_scraped_and_the_gamelist_shows_its_text_and_pictures() => Session.Dispatch(() =>
        {
            _server.Games.Add(new FakeGame(9, "Aurora Drift", RomHashes.Of(SyntheticRom.BuildBlank()).Md5) { Synopsis = "Scraped description." });
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");

            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (s.Themed.SelectedGame?.Description != "Scraped description.")
            {
                s.Settle();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("the scraped text never reached the gamelist");
                Thread.Sleep(5);
            }

            SceneGame game = s.Themed.SelectedGame!;
            Assert.Equal(ThemedSession.SnesGames[0], game.Name);
            Assert.Equal(("Synthetic Developer", "Synthetic Publisher", "Racing", "1-2", 0.8f, new DateTime(1991, 8, 23)),
                (game.Developer, game.Publisher, game.Genre, game.Players, game.Rating, game.ReleaseDate));
            Assert.Contains(_server.JeuInfos, u => FakeScreenScraper.Param(u, "romnom") == ThemedSession.SnesGames[0] + ".sfc");

            ISceneMedia media = (ISceneMedia)typeof(MainWindow).GetMethod("ThemedMedia", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null)!;
            Assert.Equal(Path.Combine(DataStore.Media, "snes", "screenshots", ThemedSession.SnesGames[0] + ".png"), media.Find(s.Themed.SelectedSystem!.System, game, "screenshot"));
            Assert.Equal(Path.Combine(DataStore.Media, "snes", "miximages", ThemedSession.SnesGames[0] + ".png"), media.Find(s.Themed.SelectedSystem!.System, game, "miximage"));
        }, default);
    }
}

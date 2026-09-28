using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The themed view asks nothing as the pad moves through it; the pad menu's Scrape This Game fills its text and pictures - see EmuSen_BigPicture.md §17.14.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedScrapeTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedScrapeTests).GetTypeInfo().Assembly);

        private static readonly FieldInfo ConfirmField = typeof(MainWindow).GetField("ConfirmScrape", BindingFlags.Static | BindingFlags.NonPublic)!;

        private readonly FakeScreenScraper _server = new();
        private readonly object _realConfirm = ConfirmField.GetValue(null)!;

        public ThemedScrapeTests()
        {
            NoNetwork.HttpFactory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(_server)));
            NoNetwork.ScrapeClock.SetValue(null, new FakeScrapeClock(new DateTimeOffset(2026, 7, 14, 10, 0, 0, TimeSpan.Zero)));
            NoNetwork.DeveloperSource.SetValue(null, (Func<DeveloperCredentials?>)(() => FakeScreenScraper.Developer));
            ConfirmField.SetValue(null, (Func<MainWindow, string, string, Task<bool>>)((_, _, _) => Task.FromResult(true)));
        }

        public void Dispose()
        {
            ConfirmField.SetValue(null, _realConfirm);
            NoNetwork.Refuse();
            NoNetwork.ScrapeClock.SetValue(null, SystemScrapeClock.Instance);
        }

        [Fact]
        public Task Moving_through_the_gamelist_asks_nothing_and_scrape_this_game_fills_its_text_and_pictures() => Session.Dispatch(() =>
        {
            _server.Games.Add(new FakeGame(9, "Aurora Drift", RomHashes.Of(SyntheticRom.BuildBlank()).Md5) { Synopsis = "Scraped description." });
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down(2);
            s.Pad.Up(2);
            s.Run(1000);
            Assert.Empty(_server.Asked);
            Assert.Null(s.Themed.SelectedGame!.Description);

            var entries = (List<PadMenuEntry>)typeof(MainWindow).GetField("_padMenuEntries", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(s.Window)!;
            s.Pad.Start();
            entries.Single(e => e.Text() == "Scrape This Game...").Accept();

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
            Assert.Equal([ThemedSession.SnesGames[0] + ".sfc"], _server.JeuInfos.Select(u => FakeScreenScraper.Param(u, "romnom")));

            ISceneMedia media = (ISceneMedia)typeof(MainWindow).GetMethod("ThemedMedia", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null)!;
            Assert.Equal(Path.Combine(DataStore.Media, "snes", "screenshots", ThemedSession.SnesGames[0] + ".png"), media.Find(s.Themed.SelectedSystem!.System, game, "screenshot"));
            Assert.Equal(Path.Combine(DataStore.Media, "snes", "miximages", ThemedSession.SnesGames[0] + ".png"), media.Find(s.Themed.SelectedSystem!.System, game, "miximage"));
        }, default);
    }
}

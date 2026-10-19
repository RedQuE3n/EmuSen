using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A theme's gameselectors over a themed session's own records - see EmuSen_Backend.md §28.11.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedGameSelectorTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedGameSelectorTests).GetTypeInfo().Assembly);

        private const string LastPlayed = "<gameselector name=\"last\"><selection>lastplayed</selection><gameCount>3</gameCount></gameselector>";

        // Two games started at one instant, the scan's second a favourite the shelf lists first, and two older; three asked of four played.
        [Fact]
        public Task A_lastplayed_selector_picks_the_console_s_recently_played_games() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(extraSystem: LastPlayed);
            GameRecords records = ThemedCollectionsTests.Records(s);
            string Rom(int i) => ThemedCollectionsTests.Rom(s, ThemedSession.SnesGames[i]);
            var at = new DateTime(2026, 9, 1, 12, 0, 0, DateTimeKind.Local);
            records.Started(Rom(0), at);
            records.Started(Rom(1), at);
            records.Started(Rom(3), at.AddHours(-1));
            records.Started(Rom(4), at.AddHours(-2));
            records.ToggleFavourite(Rom(1));

            s.Settings.SelectedCore = RuntimeCatalog.Shelves.First(x => x.EsdeSystem == "snes").Name;
            s.Settings.LibraryCollection = MainWindow.RecentKey;
            s.Refresh();
            string[] library = ((IReadOnlyList<RomEntry>)typeof(MainWindow).GetField("_shownEntries", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(s.Window)!)
                .Select(e => e.FileName).ToArray();
            Assert.Equal(4, library.Length);

            for (int guard = 0; guard < 12 && s.System != "snes"; guard++) s.Pad.Right();
            Assert.Equal("snes", s.System);
            Assert.Equal(library.Take(3), s.Themed.Stage!.Current.Scene.Selectors.Games("last").Select(g => Path.GetFileName(g.File)));
        }, default);
    }
}

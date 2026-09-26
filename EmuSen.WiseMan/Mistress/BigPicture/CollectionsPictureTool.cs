using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class CollectionsPngFactAttribute : FactAttribute
    {
        public CollectionsPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the collections' pictures to ~/.cache/emusen/bigpicture/png/collections/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §22";
            else if (!File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")))
                Skip = "Art Book Next is needed for these pictures - see EmuSen_BigPicture.md §12.5";
        }
    }

    // The carousel with the collections, the grouped system, a filtered list, the options and filter sheets, a jump, and an edited collection's ticks, at 1280 by 800; outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class CollectionsPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CollectionsPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "collections");

        private static readonly string MediaRoot = Path.Combine(Path.GetTempPath(), "EmuSenCollectionsMedia");

        private readonly ITestOutputHelper _out;

        public CollectionsPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(ThemedSession s, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        private static void Library(ThemedSession s)
        {
            GameRecords records = ThemedCollectionsTests.Records(s);
            long platform = records.CreateCollection("Platform", DateTime.Now)!.Value, beat = records.CreateCollection("Beat", DateTime.Now)!.Value;
            foreach (string g in new[] { ThemedSession.SnesGames[0], ThemedSession.SnesGames[2], ThemedSession.SnesGames[4] }) records.AddToCollection(platform, ThemedCollectionsTests.Rom(s, g));
            records.AddToCollection(beat, ThemedCollectionsTests.Rom(s, ThemedSession.NesGames[1]));
            records.AddToCollection(beat, ThemedCollectionsTests.Rom(s, ThemedSession.SnesGames[1]));
            records.ToggleFavourite(ThemedCollectionsTests.Rom(s, ThemedSession.SnesGames[3]));
            records.Started(ThemedCollectionsTests.Rom(s, ThemedSession.GbGames[0]), new DateTime(2026, 9, 3));
            var text = new Dictionary<string, (string Genre, float Rating)>
            {
                [ThemedSession.SnesGames[0]] = ("Racing", 0.8f), [ThemedSession.SnesGames[1]] = ("Puzzle", 0.4f),
                [ThemedSession.SnesGames[2]] = ("Racing", 0.6f), [ThemedSession.SnesGames[4]] = ("Shooter", 1f),
            };
            var scraped = text.ToDictionary(p => ThemedCollectionsTests.Rom(s, p.Key), p => new ScrapedRecord("0", 0, ScrapeState.Found) { Genre = p.Value.Genre, Rating = p.Value.Rating, Developer = "Synthetic Developer" });
            typeof(MainWindow).GetField("_scrapedText", BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(s.Window, (IReadOnlyDictionary<string, ScrapedRecord>)scraped);
            ThemedCollectionsTests.Refresh(s);
        }

        [CollectionsPngFact]
        public Task Collections_and_gamelist_options_on_Art_Book_Next() => Session.Dispatch(() =>
        {
            SyntheticLibrary.WriteMedia(MediaRoot);
            using var s = new ThemedSession(1280, 800, a => { a.EsdeMediaDirectory = MediaRoot; ThemedCollectionsTests.AllAuto(a); }, themeDirectory: ArtBookNextFactAttribute.Folder);
            Library(s);
            s.Themed.Random = new Random(4);

            for (int guard = 0; guard < 12 && s.System != "collections"; guard++) s.Pad.Right();
            s.Run(800);
            Save(s, "carousel-collections-1280x800");
            s.Pad.Right();
            s.Run(800);
            Save(s, "carousel-all-games-1280x800");

            s.Pad.Left();
            s.Run(800);
            s.Pad.A();
            s.Pad.Down();
            s.Run(800);
            Save(s, "grouped-collections-folders-1280x800");
            s.Pad.B();
            s.Pad.B();

            ThemedCollectionsTests.Enter(s, "snes");
            s.Run(800);
            ThemedCollectionsTests.Choose(s, "Gamelist Options");
            ThemedCollectionsTests.Reach(s, e => e is Dropdown { Name: "GamelistSortBy" });
            s.Pad.Right(3);
            s.Settle();
            Save(s, "gamelist-options-sheet-1280x800");
            ThemedCollectionsTests.Reach(s, e => e is Button { Name: "GamelistFilterButton" });
            s.Pad.A();
            s.Settle();
            ThemedCollectionsTests.Reach(s, e => e is LunaSwitch { Label: "Racing" });
            s.Pad.A();
            s.Settle();
            Save(s, "filter-sheet-1280x800");
            s.Pad.B();
            s.Pad.B();
            s.Run(800);
            Save(s, "filtered-list-racing-by-rating-1280x800");

            ThemedCollectionsTests.Choose(s, "Gamelist Options");
            ThemedCollectionsTests.Reach(s, e => e is Button { Name: "GamelistFilterButton" });
            s.Pad.A();
            s.Settle();
            ThemedCollectionsTests.Reach(s, e => e is Button { Name: "FilterReset" });
            s.Pad.A();
            s.Pad.B();
            s.Settle();
            ThemedCollectionsTests.Reach(s, e => e is Dropdown { Name: "GamelistSortBy" });
            s.Pad.Left(3);
            s.Pad.B();
            s.Run(800);

            ThemedCollectionsTests.Choose(s, "Gamelist Options");
            ThemedCollectionsTests.Reach(s, e => e is Dropdown { Name: "GamelistJumpTo" });
            s.Pad.A();
            s.Settle();
            s.Pad.Down(3);
            s.Settle();
            Save(s, "jump-to-letter-open-1280x800");
            s.Pad.A();
            s.Pad.B();
            s.Run(800);
            Save(s, "jump-to-letter-after-1280x800");
        }, default);

        [CollectionsPngFact]
        public Task A_collection_being_edited_on_the_synthetic_theme() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(1280, 800, ThemedCollectionsTests.AllAuto);
            Library(s);
            ThemedCollectionsTests.Enter(s, "collections");
            s.Pad.Down();
            s.Pad.A();
            ThemedCollectionsTests.Choose(s, "Gamelist Options");
            ThemedCollectionsTests.Reach(s, e => e is Button { Name: "GamelistEditCollection" });
            s.Pad.A();
            s.Settle();
            s.Pad.B();
            s.Pad.B();
            s.Pad.B();
            ThemedCollectionsTests.Enter(s, "snes");
            s.Pad.Down(3);
            s.Pad.Y();
            s.Run(300);
            Save(s, "editing-platform-ticks-synthetic-1280x800");
        }, default);
    }
}

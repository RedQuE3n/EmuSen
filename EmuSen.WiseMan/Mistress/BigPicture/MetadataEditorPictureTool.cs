using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class MetadataEditorPngFactAttribute : FactAttribute
    {
        public MetadataEditorPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the big-screen metadata editor's pictures to ~/.cache/emusen/bigpicture/png/esde-menus-2/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §34";
        }
    }

    // The metadata editor in ES-DE's layout at 1280 by 800 and 1920 by 1200, on both themes, and the gamelist with a game's metadata fields hidden; written outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class MetadataEditorPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MetadataEditorPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "esde-menus-2");

        private readonly ITestOutputHelper _out;

        public MetadataEditorPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(ThemedSession s, string name)
        {
            s.Settle();
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        private void Walk(ThemedSession s, string prefix)
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Run(500);
            Save(s, $"{prefix}-gamelist");
            ThemedGameOptionsTests.OpenEditor(s);
            Save(s, $"{prefix}-editor");

            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Name);
            s.Pad.A();
            PadCheatsTests.TypeByPad(s.Pad, EmuSen.LunaP.Controls.OnScreenKeyboard.OpenOver(s.Window)!, "2");
            Save(s, $"{prefix}-editor-keyboard");
            s.Pad.Start();

            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Rating);
            s.Pad.Right(7);
            Save(s, $"{prefix}-editor-rating");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.ReleaseDate);
            s.Pad.Right();
            s.Pad.A();
            s.Pad.Right(2);
            Save(s, $"{prefix}-editor-date");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Completed);
            s.Pad.A();
            Save(s, $"{prefix}-editor-switch");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Controller);
            s.Pad.Right();
            Save(s, $"{prefix}-editor-choice");
            ThemedGameOptionsTests.Reach(s, "MetaReset_" + GameMetadata.Completed);
            Save(s, $"{prefix}-editor-reset");
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            Save(s, $"{prefix}-editor-buttons");
            ThemedGameOptionsTests.Reach(s, "MetadataHide");
            s.Pad.A();
            Save(s, $"{prefix}-editor-hide-confirm");
            ThemedGameOptionsTests.Answer(s, "Cancel");

            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.HideMetadata);
            s.Pad.A();
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            s.Run(500);
            Save(s, $"{prefix}-gamelist-metadata-hidden");
        }

        // A custom collection, where the editor adds the custom collections sortname.
        private void Collection(ThemedSession s, string prefix)
        {
            GameRecords records = ThemedCollectionsTests.Records(s);
            long id = records.CreateCollection("Platform", DateTime.Now)!.Value;
            foreach (string game in ThemedSession.SnesGames) records.AddToCollection(id, ThemedCollectionsTests.Rom(s, game));
            ThemedCollectionsTests.Refresh(s);
            ThemedCollectionsTests.Enter(s, "collections");
            s.Pad.A();
            s.Settle();
            ThemedGameOptionsTests.OpenEditor(s);
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.CustomSortName);
            Save(s, $"{prefix}-editor-custom-collection");
        }

        private static ThemedSession ArtBookNext(double width, double height)
        {
            Assert.True(File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")), "Art Book Next is needed for these pictures");
            string media = Path.Combine(Path.GetTempPath(), "EmuSenMetadataEditorMedia");
            SyntheticLibrary.WriteMedia(media);
            return new ThemedSession(width, height, a => a.EsdeMediaDirectory = media, themeDirectory: ArtBookNextFactAttribute.Folder);
        }

        [MetadataEditorPngFact]
        public Task Synthetic_theme() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                using (var s = new ThemedSession(w, h)) Walk(s, $"synthetic-{w}x{h}");
                using (var s = new ThemedSession(w, h)) Collection(s, $"synthetic-{w}x{h}");
            }
        }, default);

        [MetadataEditorPngFact]
        public Task Art_book_next() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                using (ThemedSession s = ArtBookNext(w, h)) Walk(s, $"artbooknext-{w}x{h}");
                using (ThemedSession s = ArtBookNext(w, h)) Collection(s, $"artbooknext-{w}x{h}");
            }
        }, default);
    }
}

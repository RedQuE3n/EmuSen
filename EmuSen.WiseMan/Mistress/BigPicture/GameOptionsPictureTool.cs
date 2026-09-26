using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class GameOptionsPngFactAttribute : FactAttribute
    {
        public GameOptionsPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the game options' pictures to ~/.cache/emusen/bigpicture/png/game-options/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §23";
        }
    }

    // The pictures of §23 at 1280 by 800: the options menu, the editor, the keyboard on a field, and an edited game in the view; written outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class GameOptionsPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GameOptionsPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "game-options");

        private readonly ITestOutputHelper _out;

        public GameOptionsPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(RenderedFrame frame, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        private void Walk(ThemedSession s, string prefix)
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down(2);
            s.Run(500);
            Save(s.Capture(), $"{prefix}-gamelist-before");
            ThemedGameOptionsTests.OpenOptions(s);
            Save(s.Capture(), $"{prefix}-options-menu");
            s.Pad.Select();

            ThemedGameOptionsTests.OpenEditor(s);
            Save(s.Capture(), $"{prefix}-editor");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Name);
            s.Pad.A();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(s.Window)!;
            PadCheatsTests.TypeByPad(s.Pad, keyboard, " 2");
            Save(s.Capture(), $"{prefix}-editor-keyboard");
            s.Pad.Start();
            ThemedGameOptionsTests.Type(s, "Meta_" + GameMetadata.Description, "a harbour at dusk, edited on the pad");
            ThemedGameOptionsTests.Type(s, "Meta_" + GameMetadata.Developer, "the player");
            ThemedGameOptionsTests.Type(s, "Meta_" + GameMetadata.Genre, "racing");
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Rating);
            s.Pad.Right(9);
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.ReleaseDate);
            s.Pad.Right();
            s.Pad.Right(3);
            Save(s.Capture(), $"{prefix}-editor-edited");
            ThemedGameOptionsTests.Reach(s, "MetadataHide");
            s.Pad.A();
            s.Settle();
            Save(s.Capture(), $"{prefix}-hide-question");
            ThemedGameOptionsTests.Answer(s, "Cancel");
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            s.Run(500);
            Save(s.Capture(), $"{prefix}-gamelist-edited");
        }

        [GameOptionsPngFact]
        public Task Synthetic_theme() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(1280, 800);
            Walk(s, "synthetic");
        }, default);

        [GameOptionsPngFact]
        public Task Art_book_next() => Session.Dispatch(() =>
        {
            Assert.True(File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")), "Art Book Next is needed for these pictures");
            string media = Path.Combine(Path.GetTempPath(), "EmuSenGameOptionsMedia");
            SyntheticLibrary.WriteMedia(media);
            using var s = new ThemedSession(1280, 800, a => a.EsdeMediaDirectory = media, themeDirectory: ArtBookNextFactAttribute.Folder);
            Walk(s, "artbooknext");
        }, default);
    }
}

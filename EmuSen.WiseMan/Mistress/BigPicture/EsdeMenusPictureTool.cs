using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.LunaP.Controls;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class EsdeMenusPngFactAttribute : FactAttribute
    {
        public EsdeMenusPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the big-screen menus' pictures to ~/.cache/emusen/bigpicture/png/esde-menus/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §32";
        }
    }

    // The pad menu, the game options and a folder's options in ES-DE's layout, at 1280 by 800 and 1920 by 1200, on both themes; written outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class EsdeMenusPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(EsdeMenusPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "esde-menus");

        private readonly ITestOutputHelper _out;

        public EsdeMenusPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(ThemedSession s, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        private void Walk(ThemedSession s, string prefix)
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down(2);
            s.Run(500);
            s.Pad.Start();
            s.Settle();
            Save(s, $"{prefix}-start-menu");
            s.Pad.Down(3);
            Save(s, $"{prefix}-start-menu-fourth-row");
            s.Pad.B();

            ThemedCollectionsTests.OpenMenu(s);
            s.Settle();
            Save(s, $"{prefix}-game-options");
            ThemedCollectionsTests.Reach(s, e => e is Dropdown { Name: "GamelistSortBy" });
            s.Pad.Right();
            Save(s, $"{prefix}-game-options-sort-stepped");
            s.Pad.Select();
            s.Settle();

            s.Pad.A();
            s.Settle();
            s.Pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            s.Settle();
            Save(s, $"{prefix}-in-game-menu");
        }

        private void Folder(ThemedSession s, string prefix)
        {
            ThemedLibraryPadTests.Enter(s, "nes");
            s.Run(800);
            ThemedCollectionsTests.OpenMenu(s);
            s.Settle();
            Save(s, $"{prefix}-folder-options");
        }

        private static ThemedSession ArtBookNext(double width, double height, Action<string>? roms = null)
        {
            Assert.True(File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")), "Art Book Next is needed for these pictures");
            string media = Path.Combine(Path.GetTempPath(), "EmuSenEsdeMenusMedia");
            SyntheticLibrary.WriteMedia(media);
            return new ThemedSession(width, height, a => a.EsdeMediaDirectory = media, themeDirectory: ArtBookNextFactAttribute.Folder, roms: roms);
        }

        [EsdeMenusPngFact]
        public Task Synthetic_theme() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                using (var s = new ThemedSession(w, h)) Walk(s, $"synthetic-{w}x{h}");
                using (var s = new ThemedSession(w, h, roms: ThemedFoldersTests.Library)) Folder(s, $"synthetic-{w}x{h}");
            }
        }, default);

        [EsdeMenusPngFact]
        public Task Art_book_next() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                using (ThemedSession s = ArtBookNext(w, h)) Walk(s, $"artbooknext-{w}x{h}");
                using (ThemedSession s = ArtBookNext(w, h, ThemedFoldersTests.Library)) Folder(s, $"artbooknext-{w}x{h}");
            }
        }, default);
    }
}

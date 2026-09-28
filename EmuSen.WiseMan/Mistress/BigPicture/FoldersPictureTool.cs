using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class FoldersPngFactAttribute : FactAttribute
    {
        public FoldersPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the folders' pictures to ~/.cache/emusen/bigpicture/png/pass6/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §30";
            else if (!File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")))
                Skip = "Art Book Next is needed for these pictures - see EmuSen_BigPicture.md §12.5";
        }
    }

    // NES opening on its folders, inside one, GB's letter folders, NES flattened, and the folder's options and editor, at 1280 by 800; outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class FoldersPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(FoldersPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "pass6");

        private static readonly string[] Regions = ["USA", "Europe", "World", "Japan", "Hacks", "Translated", "Unlicensed", "PD"];

        private static readonly string[] Letters = ["0-9", "A", "B", "C", "D", "E", "F", "[BIOS]"];

        private readonly ITestOutputHelper _out;

        public FoldersPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(ThemedSession s, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        // The shapes decided in §21.1, in small: NES by region and category, GB by letter, each folder with two games and their covers under the folder.
        private static void Library(string roms, string media)
        {
            void Game(string console, string system, string folder, string name, string ext, int n)
            {
                string rom = Path.Combine(roms, console, folder, name + ext);
                Directory.CreateDirectory(Path.GetDirectoryName(rom)!);
                File.WriteAllBytes(rom, new byte[64 + n]);
                string cover = Path.Combine(media, system, "covers", folder, name + ".png");
                Directory.CreateDirectory(Path.GetDirectoryName(cover)!);
                SyntheticLibrary.Label(cover, 600, 800, SyntheticLibrary.Colour(n, "covers"), $"{system} covers\n{folder}/\n{name}");
            }
            int i = 0;
            foreach (string region in Regions)
            {
                Game("NES", "nes", region, $"{region} Quest (Synthetic)", ".nes", i++);
                Game("NES", "nes", region, $"{region} Racer (Synthetic)", ".nes", i++);
            }
            string[] words = ["1943", "Amber", "Birch", "Cedar", "Dune", "Ember", "Fern", "BIOS"];
            for (int l = 0; l < Letters.Length; l++)
            {
                Game("GB", "gb", Letters[l], $"{words[l]} Beacon (Synthetic)", ".gb", i++);
                Game("GB", "gb", Letters[l], $"{words[l]} Grove (Synthetic)", ".gb", i++);
            }
        }

        [FoldersPngFact]
        public Task Folders_on_Art_Book_Next() => Session.Dispatch(() =>
        {
            string media = Path.Combine(Path.GetTempPath(), "EmuSenFoldersMedia", Guid.NewGuid().ToString("N"));
            try
            {
                SyntheticLibrary.WriteMedia(media);
                using var s = new ThemedSession(1280, 800, a => a.EsdeMediaDirectory = media, themeDirectory: ArtBookNextFactAttribute.Folder, roms: r => Library(r, media));
                ThemedLibraryPadTests.Enter(s, "nes");
                s.Run(800);
                Save(s, "nes-opening-on-its-folders-1280x800");
                s.Pad.A();
                s.Run(800);
                Save(s, $"nes-inside-{Regions.Order(StringComparer.OrdinalIgnoreCase).First()}-1280x800");
                s.Pad.B();
                s.Pad.B();
                s.Pad.Right();
                s.Pad.A();
                s.Pad.Down(2);
                s.Run(800);
                Save(s, "gb-letter-folders-1280x800");

                ThemedCollectionsTests.OpenMenu(s);
                s.Settle();
                Save(s, "folder-options-menu-1280x800");
                ThemedCollectionsTests.Reach(s, e => e is Button { Name: "GameOption_EditThisFoldersMetadata" });
                s.Pad.A();
                s.Settle();
                ((FolderEditorWindow)ThemedCollectionsTests.Sheets(s).Current!).Link.SelectedIndex = 1;
                s.Settle();
                Save(s, "folder-editor-with-a-link-1280x800");
                ((FolderEditorWindow)ThemedCollectionsTests.Sheets(s).Current!).Close();
                s.Settle();

                s.Pad.B();
                s.Pad.Left();
                Assert.Equal(("system", "nes"), (s.View, s.System));
                ThemedCollectionsTests.Choose(s, "Game Collection Settings");
                ThemedCollectionsTests.Reach(s, e => e is LunaSwitch { Name: "Flatten_nes" });
                s.Pad.A();
                s.Settle();
                Assert.True(ThemedCollectionsTests.Named<LunaSwitch>(s, "Flatten_nes").IsChecked);
                Save(s, "settings-sheet-nes-flattened-1280x800");
                ThemedCollectionsTests.Sheets(s).Current!.Close();
                s.Settle();
                s.Pad.A();
                s.Run(800);
                Save(s, "nes-flattened-1280x800");
            }
            finally
            {
                try { Directory.Delete(media, recursive: true); } catch { }
            }
        }, default);

        [FoldersPngFact]
        public Task Folders_on_the_synthetic_theme() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(1280, 800, roms: ThemedFoldersTests.Library);
            ThemedLibraryPadTests.Enter(s, "nes");
            s.Run(800);
            Save(s, "synthetic-nes-folders-marked-1280x800");
        }, default);
    }
}

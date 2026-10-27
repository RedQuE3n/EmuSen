using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using Avalonia.Headless;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Game packages in the themed gamelist: one entry each, never the folder or its files, with the package's art - see EmuSen_GamePackages.md §3.6 and EmuSen_BigPicture.md §30.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedPackagesTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedPackagesTests).GetTypeInfo().Assembly);

        private const string Linked = "Linked Lantern (Synthetic)", Inside = "Inside Harbor (Synthetic)";

        private static void WritePicture(string path)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            using var bitmap = new SkiaSharp.SKBitmap(8, 8);
            using var data = bitmap.Encode(SkiaSharp.SKEncodedImageFormat.Png, 100);
            File.WriteAllBytes(path, data.ToArray());
        }

        // A linked package for a library file and a self-contained one for another, each with a cover of its own in its media folder.
        private static void Library(string roms)
        {
            foreach (string name in new[] { Linked, Inside }) File.WriteAllBytes(Path.Combine(roms, name + ".sfc"), SyntheticRom.BuildBlank());
            string linked = Path.Combine(DataStore.Packages, "SNES", Linked + ".sfc"), inside = Path.Combine(DataStore.Packages, "SNES", Inside + ".sfc");
            CrystalParityTests.PackageFor(linked, roms, Linked + ".sfc", "snes");
            CrystalParityTests.SelfContainedFor(inside, roms, Inside + ".sfc", "snes");
            WritePicture(Path.Combine(linked, "media", "covers", Linked + ".png"));
            WritePicture(Path.Combine(inside, "media", "covers", Inside + ".png"));
        }

        [Fact]
        public Task A_package_is_one_entry_with_its_own_art_and_launches_as_its_game() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(roms: Library);
            ThemedLibraryPadTests.Enter(s, "snes");
            var games = s.Themed.Stage!.Current.Data.System.Games;
            Assert.Equal(ThemedSession.SnesGames.Length + 2, games.Count);
            Assert.All(games, g => Assert.False(g.Folder));
            Assert.All(games, g => Assert.Equal("", g.FolderPath));
            Assert.Single(games, g => g.Name == Linked);
            Assert.Single(games, g => g.Name == Inside);
            string packages = Path.GetFullPath(DataStore.Packages);
            Assert.Equal(Path.Combine(s.RomDirectory, Linked + ".sfc"), games.Single(g => g.Name == Linked).File);
            Assert.Equal(Path.Combine(packages, "SNES", Inside + ".sfc", Inside + ".sfc"), games.Single(g => g.Name == Inside).File);
            var media = (EmuSen.Mistress.Scraping.MediaSources)typeof(EmuSen.Mistress.Views.MainWindow).GetMethod("MediaSourcesNow", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null)!;
            foreach (string name in new[] { Linked, Inside })
            {
                var game = games.Single(g => g.Name == name);
                Assert.Equal(Path.Combine(packages, "SNES", name + ".sfc", "media", "covers", name + ".png"), media.Find(s.Themed.Stage!.Current.Data.System.System, game, "cover"));
            }
        }, default);

        private const string Disc = "Mickey Sample (US)";

        // What pkg make writes for a disc in a game folder of the library: the package at Packages/<console>/<game folder>/<name>.cue, linked to the library's sheet.
        private static void DiscLibrary(string roms)
        {
            string folder = Path.Combine(roms, "segacd", Disc);
            Directory.CreateDirectory(folder);
            File.WriteAllText(Path.Combine(folder, Disc + ".cue"), $"FILE \"{Disc} (Track 1).bin\" BINARY\n  TRACK 01 MODE1/2352\n    INDEX 01 00:00:00\nFILE \"{Disc} (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 01 00:00:00\n");
            File.WriteAllBytes(Path.Combine(folder, Disc + " (Track 1).bin"), new byte[2352]);
            File.WriteAllBytes(Path.Combine(folder, Disc + " (Track 2).bin"), new byte[2352]);
            string package = Path.Combine(DataStore.Packages, "Sega CD", Disc, Disc + ".cue");
            CrystalParityTests.PackageFor(package, roms, $"segacd/{Disc}/{Disc}.cue", "segacd");
            WritePicture(Path.Combine(package, "media", "covers", Disc + ".png"));
        }

        [Fact]
        public Task A_disc_package_made_from_a_game_folder_is_one_entry_not_a_folder_and_shows_its_art() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(roms: DiscLibrary);
            ThemedLibraryPadTests.Enter(s, "segacd");
            var games = s.Themed.Stage!.Current.Data.System.Games;
            var game = Assert.Single(games);
            Assert.Equal((Disc, false, ""), (game.Name, game.Folder, game.FolderPath));
            string folder = Path.Combine(Path.GetFullPath(DataStore.Packages), "Sega CD", Disc, Disc + ".cue");
            var media = (EmuSen.Mistress.Scraping.MediaSources)typeof(EmuSen.Mistress.Views.MainWindow).GetMethod("MediaSourcesNow", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null)!;
            Assert.Equal(Path.Combine(folder, "media", "covers", Disc + ".png"), media.Find(s.Themed.Stage!.Current.Data.System.System, game, "cover"));
            // Launching hands over the sheet the package claims, which opens as its package as the desktop library's entry does.
            Assert.Equal(Path.Combine(s.RomDirectory, "segacd", Disc, Disc + ".cue"), game.File);
            Assert.Equal(folder, EmuSen.Galaxia.Media.GamePackages.RootFor(game.File));
        }, default);
    }
}

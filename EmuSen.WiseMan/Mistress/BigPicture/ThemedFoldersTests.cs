using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Folders in the themed gamelist and the media of games inside them - see EmuSen_BigPicture.md §30.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedFoldersTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedFoldersTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly ITestOutputHelper _out;

        public ThemedFoldersTests(ITestOutputHelper output) => _out = output;

        internal const string UsaGame = "Tidal Keep (Synthetic)", EuropeGame = "Quartz Mill (Synthetic)";

        // The player's NES shape (§21.1): a console folder, then region folders, and no ROM at the console folder's top.
        internal static void RegionFolders(string roms)
        {
            Directory.CreateDirectory(Path.Combine(roms, "NES", "USA"));
            Directory.CreateDirectory(Path.Combine(roms, "NES", "Europe"));
            File.WriteAllBytes(Path.Combine(roms, "NES", "USA", UsaGame + ".nes"), new byte[64]);
            File.WriteAllBytes(Path.Combine(roms, "NES", "Europe", EuropeGame + ".nes"), new byte[80]);
        }

        private static MediaSources Sources(ThemedSession s) => (MediaSources)typeof(MainWindow).GetMethod("MediaSourcesNow", Hidden)!.Invoke(s.Window, null)!;

        private static string Touch(string path)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, [0x89, 0x50, 0x4E, 0x47]);
            return path;
        }

        // P108: USERGUIDE ("Manually copying game media files") keeps a foldered game's media at <system>/<type>/<folder>/<stem>; the reader looked only at <system>/<type>/<stem>.
        // Shown first on the unchanged reader (5bf3f8d4): 0 of 6 found, every Locate answering none.
        [Fact]
        public Task P108_media_of_a_game_in_a_folder_are_found_under_that_folder_in_an_ES_DE_tree_and_in_Mistress_s_store() => Session.Dispatch(() =>
        {
            string esde = Path.Combine(Path.GetTempPath(), "EmuSenP108", Guid.NewGuid().ToString("N"));
            try
            {
                using var s = new ThemedSession(settings: a => a.EsdeMediaDirectory = esde, roms: RegionFolders);
                string usa = Path.Combine(s.RomDirectory, "NES", "USA", UsaGame + ".nes"), europe = Path.Combine(s.RomDirectory, "NES", "Europe", EuropeGame + ".nes");
                var expected = new Dictionary<(string Rom, string Type), string>
                {
                    [(usa, "cover")] = Touch(Path.Combine(esde, "nes", "covers", "USA", UsaGame + ".png")),
                    [(usa, "screenshot")] = Touch(Path.Combine(esde, "nes", "screenshots", "USA", UsaGame + ".jpg")),
                    [(usa, "marquee")] = Touch(Path.Combine(esde, "nes", "marquees", "USA", UsaGame + ".webp")),
                    [(usa, "video")] = Touch(Path.Combine(esde, "nes", "videos", "USA", UsaGame + ".mp4")),
                    [(europe, "cover")] = Touch(Path.Combine(MediaStore.DefaultRoot, "nes", "covers", "Europe", EuropeGame + ".png")),
                    [(europe, "screenshot")] = Touch(Path.Combine(MediaStore.DefaultRoot, "nes", "screenshots", "Europe", EuropeGame + ".png")),
                };

                MediaSources sources = Sources(s);
                var found = expected.ToDictionary(p => p.Key, p => sources.Locate("nes", p.Key.Rom, p.Key.Type).Path);
                foreach (var p in found) _out.WriteLine($"{Path.GetFileName(p.Key.Rom)} {p.Key.Type}: {p.Value ?? "(none)"}");
                Assert.Equal(expected.Count, found.Count(p => p.Value == expected[p.Key]));
                Assert.Equal(MediaSource.EsdeFolder, sources.Locate("nes", usa, "cover").Source);
                Assert.Equal(MediaSource.ScreenScraper, sources.Locate("nes", europe, "cover").Source);
            }
            finally
            {
                try { Directory.Delete(esde, recursive: true); } catch { }
            }
        }, default);

        // A store an earlier build wrote flat is put in folders when Mistress opens it, and the game shows its picture from there.
        [Fact]
        public Task Mistress_puts_its_own_store_in_folders_when_it_opens_it_and_the_picture_is_still_found() => Session.Dispatch(() =>
        {
            string flat = "";
            using var s = new ThemedSession(roms: roms =>
            {
                RegionFolders(roms);
                using MediaStore store = MediaStore.Open(MediaStore.DefaultRoot);
                string rom = Path.Combine(roms, "NES", "USA", UsaGame + ".nes");
                store.RememberFile(rom, 64, 1, "u");
                store.Record(new ScrapedRecord("u", 64, ScrapeState.Found) { FetchedAt = DateTime.UtcNow });
                store.RecordMedia("u", 64, new StoredMedia("cover", Path.Combine("nes", "covers", UsaGame + ".png"), "us", null), DateTimeOffset.UtcNow);
                flat = Touch(Path.Combine(MediaStore.DefaultRoot, "nes", "covers", UsaGame + ".png"));
            });
            string foldered = Path.Combine(MediaStore.DefaultRoot, "nes", "covers", "USA", UsaGame + ".png");
            Assert.True(File.Exists(foldered));
            Assert.False(File.Exists(flat));
            Assert.Equal(foldered, Sources(s).Cover("nes", Path.Combine(s.RomDirectory, "NES", "USA", UsaGame + ".nes")));
        }, default);
    }
}

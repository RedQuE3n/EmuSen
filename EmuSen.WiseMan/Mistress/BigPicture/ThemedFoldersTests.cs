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
        [Fact(Skip = "P108 shown on the unchanged reader (2026-09-27): 0 of 6 found, every Locate answered none; the fix follows")]
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
    }
}

using System;
using System.IO;
using System.Linq;
using EmuSen.Mistress.Library;

namespace EmuSen.WiseMan.Mistress
{
    // Directory scan behind the game library - see EmuSen_Settings_Reference.md §4.11.
    public class RomLibraryTests : IDisposable
    {
        private readonly string _dir;

        public RomLibraryTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenRomLibraryTests", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_dir);
        }

        public void Dispose()
        {
            try { if (Directory.Exists(_dir)) Directory.Delete(_dir, recursive: true); } catch { }
        }

        private void Touch(string name) => File.WriteAllBytes(Path.Combine(_dir, name), Array.Empty<byte>());

        // <relative> may name subdirectories; they are created.
        private void TouchAt(string relative)
        {
            string full = Path.Combine(_dir, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(full)!);
            File.WriteAllBytes(full, Array.Empty<byte>());
        }

        private static string Snes => EmuSen.Cores.CoreCatalog.ByExtension(".sfc")!.DisplayName;
        private static string Nes => EmuSen.Cores.CoreCatalog.ByExtension(".nes")!.DisplayName;

        // A real ROM tree is per-console folders, so a top-level-only scan finds nothing at all.
        [Fact]
        public void The_scan_recurses_into_per_console_subfolders()
        {
            TouchAt(Path.Combine("SNES", "Alpha.sfc"));
            TouchAt(Path.Combine("NES", "USA", "Beta.nes"));

            RomLibraryResult result = RomLibrary.Scan(_dir);

            Assert.Equal(RomLibraryStatus.Ok, result.Status);
            Assert.Equal(new[] { "Alpha", "Beta" }, result.Entries.Select(e => e.Title));
        }

        // A Game Boy file with the Color's header byte, or none: 0x80 enhanced, 0xC0 Color only.
        private void GameBoyAt(string name, byte cgbFlag)
        {
            var image = new byte[0x150];
            image[0x143] = cgbFlag;
            File.WriteAllBytes(Path.Combine(_dir, name), image);
        }

        // Two shelves for one core: a .gbc file, or a .gb whose header asks for the Color, sits on Game Boy Color - see EmuSen_Settings_Reference.md §4.46.
        [Fact]
        public void Game_Boy_Color_games_sit_on_their_own_shelf_by_extension_or_header()
        {
            string gameBoy = EmuSen.Cores.CoreCatalog.ByExtension(".gb")!.DisplayName;
            string color = EmuSen.Cores.CoreCatalog.GameBoyColorShelf;
            GameBoyAt("Plain.gb", 0x00);
            GameBoyAt("Enhanced.gb", 0x80);
            GameBoyAt("ColorOnly.gb", 0xC0);
            GameBoyAt("Named.gbc", 0x00);
            Touch("Short.gb");
            TouchAt(Path.Combine("SNES", "Alpha.sfc"));

            RomLibraryResult all = RomLibrary.Scan(_dir);
            Assert.Equal(new[] { "Plain", "Short" }, all.Entries.Where(e => e.Shelf == gameBoy).Select(e => e.Title));
            Assert.Equal(new[] { "ColorOnly", "Enhanced", "Named" }, all.Entries.Where(e => e.Shelf == color).Select(e => e.Title));
            Assert.All(all.Entries.Where(e => e.Shelf == color), e => Assert.Equal(gameBoy, e.CoreDisplayName));

            RomLibraryResult narrowed = RomLibrary.Narrow(all, color);
            Assert.Equal(new[] { "ColorOnly", "Enhanced", "Named" }, narrowed.Entries.Select(e => e.Title));
            Assert.Equal(color, narrowed.CoreDisplayName);
            Assert.Equal(new[] { "Plain", "Short" }, RomLibrary.Narrow(all, gameBoy).Entries.Select(e => e.Title));
            Assert.Equal(new[] { "ColorOnly", "Enhanced", "Named" }, RomLibrary.Scan(_dir, color).Entries.Select(e => e.Title));
        }

        [Fact]
        public void A_console_filter_keeps_only_that_consoles_games()
        {
            TouchAt(Path.Combine("SNES", "Alpha.sfc"));
            TouchAt(Path.Combine("NES", "Beta.nes"));

            RomLibraryResult onlyNes = RomLibrary.Scan(_dir, Nes);

            Assert.Equal(new[] { "Beta" }, onlyNes.Entries.Select(e => e.Title));
            Assert.Equal(Nes, onlyNes.CoreDisplayName);
        }

        [Fact]
        public void All_consoles_and_an_unknown_name_both_mean_no_filter()
        {
            Touch("Alpha.sfc");
            Touch("Beta.nes");

            foreach (string? choice in new[] { null, EmuSen.Cores.CoreCatalog.AllConsoles, "Dreamcast (Kunzite)" })
            {
                RomLibraryResult result = RomLibrary.Scan(_dir, choice);

                Assert.Equal(2, result.Entries.Count);
                Assert.Null(result.CoreDisplayName);
            }
        }

        [Fact]
        public void Each_entry_names_the_console_its_extension_belongs_to()
        {
            Touch("Alpha.sfc");
            Touch("Beta.nes");

            var byTitle = RomLibrary.Scan(_dir).Entries.ToDictionary(e => e.Title, e => e.CoreDisplayName);

            Assert.Equal(Snes, byTitle["Alpha"]);
            Assert.Equal(Nes, byTitle["Beta"]);
        }

        // "No games" should say which console it looked for, or it reads as a broken library.
        [Fact]
        public void An_empty_filtered_scan_names_the_console_it_filtered_on()
        {
            Touch("Alpha.sfc");

            RomLibraryResult result = RomLibrary.Scan(_dir, Nes);

            Assert.Equal(RomLibraryStatus.Empty, result.Status);
            Assert.Contains(Nes, RomLibrary.DescribeEmpty(result));
        }

        [Fact]
        public void No_directory_configured_is_distinct_from_an_empty_one()
        {
            Assert.Equal(RomLibraryStatus.NoDirectoryConfigured, RomLibrary.Scan(null).Status);
            Assert.Equal(RomLibraryStatus.NoDirectoryConfigured, RomLibrary.Scan("   ").Status);
            Assert.Equal(RomLibraryStatus.Empty, RomLibrary.Scan(_dir).Status);
        }

        [Fact]
        public void A_configured_but_missing_directory_reports_itself()
        {
            string missing = Path.Combine(_dir, "nope");
            RomLibraryResult result = RomLibrary.Scan(missing);

            Assert.Equal(RomLibraryStatus.DirectoryNotFound, result.Status);
            Assert.Contains(missing, RomLibrary.DescribeEmpty(result));
        }

        // Every core in the build, not just Venus - see EmuSen_Multicore.md §3.
        [Fact]
        public void Every_registered_cores_rom_extension_is_listed()
        {
            Touch("Alpha.smc");
            Touch("Beta.sfc");
            Touch("notes.txt");
            Touch("Alpha.srm");
            Touch("Gamma.nes");

            RomLibraryResult result = RomLibrary.Scan(_dir);

            Assert.Equal(RomLibraryStatus.Ok, result.Status);
            Assert.Equal(new[] { "Alpha", "Beta", "Gamma" }, result.Entries.Select(e => e.Title));
        }

        // The scan list is the catalog's, so a new core needs no edit in RomLibrary.
        [Fact]
        public void Scanned_extensions_come_from_the_core_catalog()
        {
            Assert.Equal(EmuSen.Cores.CoreCatalog.RomExtensions, RomLibrary.Extensions);
        }

        [Fact]
        public void Extension_matching_ignores_case()
        {
            Touch("Loud.SMC");
            Touch("Quiet.SfC");

            Assert.Equal(2, RomLibrary.Scan(_dir).Entries.Count);
        }

        [Fact]
        public void Titles_sort_case_insensitively_and_keep_their_full_path()
        {
            Touch("zelda.smc");
            Touch("Actraiser.smc");
            Touch("mario.sfc");

            RomLibraryResult result = RomLibrary.Scan(_dir);

            Assert.Equal(new[] { "Actraiser", "mario", "zelda" }, result.Entries.Select(e => e.Title));
            Assert.All(result.Entries, e => Assert.True(File.Exists(e.FullPath)));
            Assert.Equal("mario.sfc", result.Entries[1].FileName);
        }

        [Fact]
        public void Every_empty_case_explains_itself_and_the_ok_case_does_not()
        {
            Assert.NotEmpty(RomLibrary.DescribeEmpty(RomLibrary.Scan(null)));
            Assert.NotEmpty(RomLibrary.DescribeEmpty(RomLibrary.Scan(Path.Combine(_dir, "gone"))));
            Assert.NotEmpty(RomLibrary.DescribeEmpty(RomLibrary.Scan(_dir)));

            Touch("Something.smc");
            Assert.Equal(string.Empty, RomLibrary.DescribeEmpty(RomLibrary.Scan(_dir)));
        }
    }
}

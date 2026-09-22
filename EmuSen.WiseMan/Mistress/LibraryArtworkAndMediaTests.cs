using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Mistress.Library;

namespace EmuSen.WiseMan.Mistress
{
    // Cover art matched by name, and save states and screenshots read back from their names - see EmuSen_Settings_Reference.md §4.33 and §4.35.
    public class LibraryArtworkAndMediaTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenArtworkTests", Guid.NewGuid().ToString("N"));

        public LibraryArtworkAndMediaTests() => Directory.CreateDirectory(_root);

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Touch(params string[] parts)
        {
            string path = Path.Combine(new[] { _root }.Concat(parts).ToArray());
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, new byte[] { 1 });
            return path;
        }

        [Fact]
        public void A_libretro_thumbnail_tree_is_matched_by_the_rom_s_own_name_and_scoped_to_its_console()
        {
            string nes = Touch("Nintendo - Nintendo Entertainment System", "Named_Boxarts", "Tetris (USA).png");
            string gb = Touch("Nintendo - Game Boy", "Named_Boxarts", "Tetris (World) (Rev 1).png");
            ArtworkIndex index = ArtworkIndex.Scan(_root, CoreCatalog.Cores);

            Assert.Equal(nes, index.Find("NES", "Tetris (USA)"));
            Assert.Equal(gb, index.Find("GB", "Tetris (World) (Rev 1)"));
            Assert.Null(index.Find("SNES", "Tetris (USA)"));
        }

        [Fact]
        public void A_name_with_other_tags_falls_back_to_the_untagged_title_and_the_unsafe_characters_are_libretro_s()
        {
            string mario = Touch("N64", "Super Mario 64 (USA).jpg");
            string zelda = Touch("Legend of Zelda, The - Link_s Awakening (USA).png");
            ArtworkIndex index = ArtworkIndex.Scan(_root, CoreCatalog.Cores);

            Assert.Equal(mario, index.Find("N64", "Super Mario 64 (Europe) (En,Fr,De)"));
            Assert.Equal(zelda, index.Find("GB", "Legend of Zelda, The - Link's Awakening (USA)".Replace("'", "&")));
            Assert.Equal("super mario 64", ArtworkIndex.Key(ArtworkIndex.Untagged("Super Mario 64 (Europe) [!]")));
        }

        [Fact]
        public void An_exact_name_wins_over_a_looser_one_anywhere()
        {
            Touch("N64", "Wave Race 64 (Japan).png");
            string exact = Touch("Wave Race 64 (USA) (Rev A).png");
            ArtworkIndex index = ArtworkIndex.Scan(_root, CoreCatalog.Cores);

            Assert.Equal(exact, index.Find("N64", "Wave Race 64 (USA) (Rev A)"));
        }

        [Fact]
        public void A_missing_directory_is_an_empty_index_rather_than_a_failure()
        {
            Assert.Equal(0, ArtworkIndex.Scan(Path.Combine(_root, "absent"), CoreCatalog.Cores).Count);
            Assert.Equal(0, ArtworkIndex.Scan(null, CoreCatalog.Cores).Count);
        }

        [Theory]
        [InlineData("Wave Race 64 (USA)", "Wave Race 64 (USA)", "Slot 1")]
        [InlineData("Wave Race 64 (USA).slot7", "Wave Race 64 (USA)", "Slot 7")]
        [InlineData("Wave Race 64 (USA).resume", "Wave Race 64 (USA)", "Where you left off")]
        [InlineData("Dr. Mario (USA).slotx", "Dr. Mario (USA).slotx", "Slot 1")]
        public void A_state_s_name_says_its_game_and_its_slot(string name, string stem, string label)
        {
            Assert.Equal((stem, label), MediaLibrary.Parse(name));
        }

        [Fact]
        public void States_and_screenshots_find_their_game_by_its_file_name()
        {
            string rom = Touch("Roms", "Kirby 64 (USA).z64");
            Touch("States", "Kirby 64 (USA).slot2.state");
            Touch("States", "Kirby 64 (USA).slot2.png");
            Touch("States", "Lost Game.state");
            Touch("Shots", "Kirby 64 (USA) 2026-09-21 20.15.00.123.png");
            var games = new[] { new RomEntry(rom) };

            var states = MediaLibrary.SaveStates(Path.Combine(_root, "States"), games);
            Assert.Equal(2, states.Count);
            MediaItem kirby = Assert.Single(states, s => s.Game is not null);
            Assert.Equal("Slot 2", kirby.Label);
            Assert.NotNull(kirby.PicturePath);
            Assert.Null(Assert.Single(states, s => s.Game is null).PicturePath);

            MediaItem shot = Assert.Single(MediaLibrary.Screenshots(Path.Combine(_root, "Shots"), games));
            Assert.Equal(rom, shot.Game!.FullPath);
        }
    }
}

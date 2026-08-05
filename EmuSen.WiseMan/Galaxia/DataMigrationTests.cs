using System;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Galaxia
{
    // Copy-never-move out of the old Usr/Home tree - see EmuSen_Galaxia.md §3.2.
    public class DataMigrationTests : IDisposable
    {
        private readonly string _root = Path.Combine(
            Path.GetTempPath(), "EmuSenMigrate_" + Guid.NewGuid().ToString("N"));

        private readonly string _legacy;
        private readonly string _destination;

        public DataMigrationTests()
        {
            _legacy = Path.Combine(_root, "EmuSen.DianaOS", "DianaOS", "Usr", "Home");
            _destination = Path.Combine(_root, "home");
        }

        public void Dispose()
        {
            if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true);
        }

        private string PutLegacy(string relative, string contents)
        {
            string path = Path.Combine(_legacy, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, contents);
            return path;
        }

        private int Run() => DataMigration.Run(_legacy, _destination);

        [Fact]
        public void Emulator_written_data_is_copied_across()
        {
            PutLegacy(Path.Combine("Saves", "ALTTP.srm"), "save");
            PutLegacy(Path.Combine("Saves", "Save States", "ALTTP.state"), "state");
            PutLegacy(Path.Combine("Firmware", "dsp1.rom"), "fw");
            PutLegacy(Path.Combine("Cheats", "SNES", "x.cht"), "cheat");
            PutLegacy(Path.Combine("Logs", "Venus", "run.txt"), "log");

            Assert.Equal(5, Run());

            Assert.Equal("save", File.ReadAllText(Path.Combine(_destination, "Saves", "ALTTP.srm")));
            Assert.Equal("state", File.ReadAllText(Path.Combine(_destination, "Saves", "Save States", "ALTTP.state")));
            Assert.Equal("fw", File.ReadAllText(Path.Combine(_destination, "Firmware", "dsp1.rom")));
            Assert.Equal("cheat", File.ReadAllText(Path.Combine(_destination, "Cheats", "SNES", "x.cht")));
            Assert.Equal("log", File.ReadAllText(Path.Combine(_destination, "Logs", "Venus", "run.txt")));
        }

        // Copy, not move: rolling back to an older build must still find its data.
        [Fact]
        public void The_source_files_are_left_exactly_where_they_were()
        {
            string source = PutLegacy(Path.Combine("Saves", "ALTTP.srm"), "save");

            Run();

            Assert.True(File.Exists(source));
            Assert.Equal("save", File.ReadAllText(source));
        }

        [Fact]
        public void An_existing_destination_file_is_never_overwritten()
        {
            PutLegacy(Path.Combine("Saves", "ALTTP.srm"), "old");
            string target = Path.Combine(_destination, "Saves", "ALTTP.srm");
            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            File.WriteAllText(target, "newer");

            Assert.Equal(0, Run());

            Assert.Equal("newer", File.ReadAllText(target));
        }

        [Fact]
        public void Running_twice_copies_nothing_the_second_time()
        {
            PutLegacy(Path.Combine("Saves", "ALTTP.srm"), "save");

            Assert.Equal(1, Run());
            Assert.Equal(0, Run());
        }

        // The user's ROM library is theirs; 95MB is not copied silently - see EmuSen_Galaxia.md §3.3.
        [Fact]
        public void The_rom_library_is_never_copied()
        {
            PutLegacy(Path.Combine("Games", "SNES", "ALTTP.smc"), "rom");
            PutLegacy(Path.Combine("Roms", "SMW.smc"), "rom");

            Run();

            Assert.False(Directory.Exists(Path.Combine(_destination, "Games")));
            Assert.False(Directory.Exists(Path.Combine(_destination, "Roms")));
        }

        [Fact]
        public void The_rom_library_is_left_untouched_on_disk()
        {
            string game = PutLegacy(Path.Combine("Games", "SNES", "ALTTP.smc"), "rom");
            DateTime before = File.GetLastWriteTimeUtc(game);

            Run();

            Assert.True(File.Exists(game));
            Assert.Equal("rom", File.ReadAllText(game));
            Assert.Equal(before, File.GetLastWriteTimeUtc(game));
        }

        [Fact]
        public void Remaining_library_directories_are_reported_so_a_caller_can_say_where_they_are()
        {
            PutLegacy(Path.Combine("Games", "SNES", "ALTTP.smc"), "rom");
            PutLegacy(Path.Combine("Roms", "SMW.smc"), "rom");

            var remaining = DataMigration.RemainingLibraryDirectories(_legacy).ToArray();

            Assert.Equal(2, remaining.Length);
            Assert.Contains(Path.Combine(_legacy, "Games"), remaining);
            Assert.Contains(Path.Combine(_legacy, "Roms"), remaining);
        }

        [Fact]
        public void An_empty_library_directory_is_not_reported()
        {
            Directory.CreateDirectory(Path.Combine(_legacy, "Games"));

            Assert.Empty(DataMigration.RemainingLibraryDirectories(_legacy));
        }

        [Fact]
        public void A_missing_legacy_tree_is_not_an_error()
        {
            Assert.Equal(0, Run());
        }

        // A published tree that never had the old layout must not migrate onto itself.
        [Fact]
        public void A_legacy_root_equal_to_the_destination_is_a_no_op()
        {
            Directory.CreateDirectory(_destination);
            File.WriteAllText(Path.Combine(_destination, "x.txt"), "x");

            Assert.Equal(0, DataMigration.Run(_destination, _destination));
        }
    }
}

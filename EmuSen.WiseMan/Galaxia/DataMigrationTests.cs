using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
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
            _legacy = Path.Combine(_root, "EmuSen.DianaSh", "DianaSh", "Usr", "Home");
            _destination = Path.Combine(_root, "home");
        }

        public void Dispose()
        {
            Links.DeleteTree(_root + "-links");
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

        // The player's ROM library is theirs; 95MB is not copied silently - see EmuSen_Galaxia.md §3.3.
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

        [DllImport("kernel32", CharSet = CharSet.Unicode, EntryPoint = "GetShortPathNameW")]
        private static extern uint ShortPathName(string path, StringBuilder buffer, uint length);

        [DllImport("kernel32", CharSet = CharSet.Unicode, EntryPoint = "GetLongPathNameW")]
        private static extern uint LongPathName(string path, StringBuilder buffer, uint length);

        private static string? Spelled(Func<string, StringBuilder, uint, uint> ask, string path)
        {
            var buffer = new StringBuilder(1024);
            uint length = ask(path, buffer, (uint)buffer.Capacity);
            return length > 0 && length < buffer.Capacity ? buffer.ToString() : null;
        }

        // Every entry under a directory with its bytes and when it was last written.
        private static List<string> Snapshot(string root) =>
            Directory.EnumerateFileSystemEntries(root, "*", SearchOption.AllDirectories)
                .Select(e => $"{e} | {(File.Exists(e) ? Convert.ToHexString(File.ReadAllBytes(e)) : "folder")} | {File.GetLastWriteTimeUtc(e).Ticks}")
                .OrderBy(line => line, StringComparer.Ordinal)
                .ToList();

        // A tree migrated onto itself, however the system lets it be spelled, copies nothing and changes nothing: the roots compare equal, or every file finds itself there - see EmuSen_Galaxia.md §3.2.
        [Fact]
        public void A_tree_spelled_any_way_the_system_reads_as_itself_is_left_alone()
        {
            string home = Path.Combine(_root, "Caf\u00e9"), links = Directory.CreateDirectory(_root + "-links").FullName;
            foreach (string file in new[] { "Saves/a.srm", "Saves/Save States/a.state", "Logs/x/run.txt", "Firmware/dsp1.rom", "Cheats/SNES/a.cht" })
            {
                string path = Path.Combine(home, file.Replace('/', Path.DirectorySeparatorChar));
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.WriteAllText(path, file);
            }
            var spellings = new List<string> { home, home + Path.DirectorySeparatorChar, Path.Combine(_root, "x", "..", "Caf\u00e9"), Path.Combine(_root, "CAF\u00c9"), Path.Combine(_root, "caf\u00e9") };
            if (Links.Allowed)
            {
                Directory.CreateSymbolicLink(Path.Combine(links, "linked"), home);
                spellings.Add(Path.Combine(links, "linked"));
            }
            if (OperatingSystem.IsMacOS()) spellings.Add(Path.Combine(_root, "Cafe\u0301"));
            if (OperatingSystem.IsWindows())
            {
                spellings.AddRange(new[] { home.Replace('\\', '/'), @"\\?\" + home, char.ToLowerInvariant(home[0]) + home[1..], home + ".", home + " ", home + ". ." });
                spellings.AddRange(new[] { Spelled(ShortPathName, home), Spelled(LongPathName, home) }.OfType<string>().Where(s => s != home));
                Links.Junction(Path.Combine(links, "junction"), home);
                spellings.Add(Path.Combine(links, "junction"));
            }
            if (OperatingSystem.IsWindows() || OperatingSystem.IsMacOS()) Assert.True(Directory.Exists(Path.Combine(_root, "CAF\u00c9")), "this file system ignores case");
            List<string> before = Snapshot(_root);

            foreach (string spelling in spellings)
            {
                Assert.True(DataMigration.Run(home, spelling) == 0, $"onto {spelling}");
                Assert.True(DataMigration.Run(spelling, home) == 0, $"from {spelling}");
            }

            List<string> after = Snapshot(_root);
            Assert.True(before.SequenceEqual(after), $"before:\n{string.Join("\n", before.Except(after))}\nafter:\n{string.Join("\n", after.Except(before))}");
        }

        // A Mac bundle's Contents/Resources/home reaches the player's home, and never over a file already there - see EmuSen_Galaxia.md §3.4.
        [Fact]
        public void A_bundle_seed_fills_what_is_missing_and_keeps_what_the_player_has()
        {
            string seed = Path.Combine(_root, "EmuSen.app", "Contents", "Resources", "home");
            Directory.CreateDirectory(Path.Combine(seed, "Documents", "Man pages"));
            File.WriteAllText(Path.Combine(seed, "Documents", "Man pages", "README.md"), "shipped");
            File.WriteAllText(Path.Combine(seed, "Documents", "Manual.md"), "shipped");
            Directory.CreateDirectory(Path.Combine(_destination, "Documents"));
            File.WriteAllText(Path.Combine(_destination, "Documents", "Manual.md"), "the player's notes");

            Assert.Equal(1, DataMigration.SeedFromBundle(seed, _destination));

            Assert.Equal("shipped", File.ReadAllText(Path.Combine(_destination, "Documents", "Man pages", "README.md")));
            Assert.Equal("the player's notes", File.ReadAllText(Path.Combine(_destination, "Documents", "Manual.md")));
            Assert.Equal("shipped", File.ReadAllText(Path.Combine(seed, "Documents", "Manual.md")));
            Assert.Equal(0, DataMigration.SeedFromBundle(seed, _destination));
        }

        [Fact]
        public void Outside_a_bundle_there_is_nothing_to_seed()
        {
            Assert.Equal(0, DataMigration.SeedFromBundle(null, _destination));
            Assert.False(Directory.Exists(_destination));
        }

        // The running test host is no bundle, so the production entry point is a no-op here.
        [Fact]
        public void The_test_host_has_no_seed_directory()
        {
            Assert.Null(EmuSen.Galaxia.ConfigRoot.SeedDirectory);
        }
    }
}

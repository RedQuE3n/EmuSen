using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // The platform library's Galaxia, held to .NET's own rules and to the answers recorded from the C# before its deletion - see EmuSen_RustPlatform.md §6.3, §10.5 and EmuSen_Backend.md §40.2.
    [Collection(TestCollections.ProcessGlobals)]
    public partial class GalaxiaParityTests : IDisposable
    {
        private readonly string _temp = Path.Combine(Path.GetTempPath(), "EmuSenParity_" + Guid.NewGuid().ToString("N"));
        private readonly List<string> _reported = new();
        private readonly string? _logOverride = ErrorLog.DirectoryOverride;
        private readonly Func<string, string>? _redactor = ErrorLog.Redactor;

        public GalaxiaParityTests()
        {
            Assert.True(GalaxiaNative.Ready, PlatformLibrary.Report);
            Directory.CreateDirectory(_temp);
            ConfigDiagnostics.Reset();
            ConfigDiagnostics.Sink = _reported.Add;
        }

        public void Dispose()
        {
            try { CheckGoldens(); }
            finally { Clean(); }
        }

        private void Clean()
        {
            ConfigDiagnostics.Sink = null;
            ConfigDiagnostics.Reset();
            ConfigStore.OverrideDirectory = null;
            ConfigStore.OverrideLegacyDirectory = null;
            DataStore.OverrideDirectory = null;
            ErrorLog.DirectoryOverride = _logOverride;
            ErrorLog.Redactor = _redactor;
            Links.DeleteTree(_temp);
        }

        // What .NET refuses as no path at all: the empty string, and on Windows one of spaces only.
        private static bool EffectivelyEmpty(string path) => path.Length == 0 || (OperatingSystem.IsWindows() && path.All(c => c == ' '));

        private string At(params string[] parts) => Path.Combine(parts.Prepend(_temp).ToArray());

        private string Put(string path, string contents = "x")
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, contents);
            return path;
        }

        // What a call reported, taken so the next call starts with nothing.
        private List<string> Reported()
        {
            var taken = new List<string>(_reported);
            _reported.Clear();
            return taken;
        }

        // Galaxia has no C# half: the library is in use, whatever a leftover variable says - see EmuSen_Backend.md §40.2.
        [Fact]
        public void The_library_is_in_use_and_names_its_interface()
        {
            Assert.True(GalaxiaNative.Ready, PlatformLibrary.Report);
            Assert.Contains($"interface {PlatformLibrary.InterfaceVersion}", PlatformLibrary.Report);
        }

        // A folder without the library is said in words, and a retired switch's line reaches the host's sink and the error log - see EmuSen_RustPlatform.md §13 and EmuSen_Backend.md §40.9.
        [Fact]
        public void A_missing_library_is_said_in_words_and_a_switch_s_line_once_in_the_log()
        {
            string logs = At("logs");
            ErrorLog.DirectoryOverride = logs;
            string empty = Directory.CreateDirectory(At("no library")).FullName;
            Assert.Equal(((nint)0, $"{PlatformLibrary.FileName} not found beside the assemblies"), PlatformLibrary.Open(empty));
            string line = $"EMUSEN_SERENITY_NATIVE=0 is ignored: Serenity's slang half has had no C# implementation since 2026-11-05 and runs on {PlatformLibrary.FileName}.";
            RetiredSwitch.Announce(line);
            Assert.Equal(new[] { line }, Reported());
            string written = string.Concat(Directory.GetFiles(logs).Select(File.ReadAllText));
            Assert.Equal(1, written.Split(line).Length - 1);
            Assert.Contains("platform", written);
        }

        // Seeded, so a failure names a case that fails again.
        private static IEnumerable<string> Paths(int count, int seed)
        {
            char separator = Path.DirectorySeparatorChar;
            var parts = new List<string>
            {
                "a", "b", "Game (U)", ".", "..", "...", " ", "x.y", ".hidden", "name.", "a.b.c", "é", "日本", "EmuSen.app", "Contents", "MacOS",
                separator.ToString(), separator.ToString(), separator.ToString(), new string(separator, 2), "/",
            };
            if (OperatingSystem.IsWindows()) parts.AddRange(new[] { "C:", "c:", @"\\server\share", "/" });
            var random = new Random(seed);
            for (int i = 0; i < count; i++)
                yield return string.Concat(Enumerable.Range(0, random.Next(0, 8)).Select(_ => parts[random.Next(parts.Count)]));
        }

        [Fact]
        public void The_root_is_found_the_same_way_from_every_kind_of_place()
        {
            string home = At("Users", "player");
            string marked = At("marked");
            Put(Path.Combine(marked, ConfigRoot.RootMarkerFileName), "");
            string checkout = At("checkout");
            Put(Path.Combine(checkout, "EmuSen.sln"), "");
            Directory.CreateDirectory(Path.Combine(checkout, "deep", ConfigRoot.RootMarkerFileName));
            string bundle = At("checkout", "out", "EmuSen.app", "Contents", "MacOS");
            string old = At("old");
            Put(Path.Combine(old, ConfigRoot.LegacyRootMarkerFileName), "");
            string oldBare = At("oldbare", "bin");
            Directory.CreateDirectory(Path.Combine(oldBare, ConfigRoot.LegacyPublishedRootDirName));
            string bothBare = At("bothbare", "bin");
            Directory.CreateDirectory(Path.Combine(bothBare, ConfigRoot.LegacyPublishedRootDirName));
            Directory.CreateDirectory(Path.Combine(bothBare, ConfigRoot.PublishedRootDirName));
            string ten = Path.Combine(new[] { marked }.Concat(Enumerable.Range(1, 10).Select(i => i.ToString())).ToArray());
            string nine = Path.Combine(new[] { marked }.Concat(Enumerable.Range(1, 9).Select(i => i.ToString())).ToArray());
            foreach (string directory in new[] { bundle, ten, nine, At("bare", "bin"), Path.Combine(checkout, "deep", "er"), At("Other.bundle", "Contents", "MacOS"), At("x.APP", "contents", "macos") })
                Directory.CreateDirectory(directory);

            var bases = new List<string>
            {
                marked, marked + Path.DirectorySeparatorChar, checkout, bundle, bundle + Path.DirectorySeparatorChar, ten, nine, At("bare", "bin"), At("absent", "bin"),
                Path.Combine(checkout, "deep", "er"), Path.Combine(checkout, "deep", "er", "..", "..", "out"), At("Other.bundle", "Contents", "MacOS"),
                At("x.APP", "contents", "macos"), At("checkout", "out", "EmuSen.app", "Contents"), Path.GetPathRoot(_temp)!, "relative", ".",
                old, At("old", "lib", "EmuSen"), oldBare, bothBare,
            };
            bases.AddRange(Paths(300, seed: 7).Where(p => !EffectivelyEmpty(p)));

            foreach (string from in bases)
            {
                foreach (bool macOS in new[] { false, true })
                {
                    GSweep("roots", $"root for [{from}], macOS {macOS}", Caught(() => GalaxiaNative.RootFor(from, macOS, home)));
                    GSweep("roots", $"seed for [{from}], macOS {macOS}", Caught(() => GalaxiaNative.SeedDirectoryFor(from, macOS)));
                }
                GSweep("roots", $"bundle for [{from}]", Caught(() => GalaxiaNative.BundleContentsFor(from)));
                GSweep("roots", $"Mac data for [{from}]", Caught(() => GalaxiaNative.MacDataDirectoryFor(from)));
                GSweep("roots", $"legacy root for [{from}]", Caught(() => GalaxiaNative.LegacyRootFor(from)));
            }

            // The cases are not vacuous: each branch of the rule is among them.
            Assert.Equal(marked, GalaxiaNative.RootFor(nine, false, home));
            Assert.Equal(Path.Combine(ten, ConfigRoot.PublishedRootDirName), GalaxiaNative.RootFor(ten, false, home));
            Assert.Equal(checkout, GalaxiaNative.RootFor(bundle, false, home));
            Assert.Equal(Path.Combine(home, "Library", "Application Support", "EmuSen"), GalaxiaNative.RootFor(bundle, true, home));
            Assert.Equal(checkout, GalaxiaNative.RootFor(Path.Combine(checkout, "deep", "er"), false, home));

            // A tree published before the shell was renamed: its marker and its fallback folder are still found, and the new folder wins where both are there.
            Assert.Equal(old, GalaxiaNative.RootFor(At("old", "lib", "EmuSen"), false, home));
            Assert.Equal(Path.Combine(oldBare, ConfigRoot.LegacyPublishedRootDirName), GalaxiaNative.RootFor(oldBare, false, home));
            Assert.Equal(Path.Combine(bothBare, ConfigRoot.PublishedRootDirName), GalaxiaNative.RootFor(bothBare, false, home));
            Assert.Equal(Path.Combine(At("bare", "bin"), ConfigRoot.PublishedRootDirName), GalaxiaNative.RootFor(At("bare", "bin"), false, home));
        }

        private static string Mode(string path) => OperatingSystem.IsWindows() ? "" : File.GetUnixFileMode(path).ToString();

        [Theory]
        [InlineData(0)]
        [InlineData(1)]
        [InlineData(63)]
        [InlineData(64)]
        [InlineData(65)]
        [InlineData(4096)]
        [InlineData((1 << 20) + 3)]
        public void A_file_written_is_the_file_dotnet_writes_and_reads_back(int length)
        {
            _case = $"{nameof(A_file_written_is_the_file_dotnet_writes_and_reads_back)}.{length}";
            byte[] contents = new byte[length];
            new Random(length).NextBytes(contents);
            string native = At("native", "Saves", "SNES", "game.srm"), dotnet = At("dotnet", "Saves", "SNES", "game.srm");

            foreach (byte[] bytes in new[] { new byte[] { 9, 9, 9 }, contents })
                Assert.True(GalaxiaNative.FileWrite(native, bytes));
            Directory.CreateDirectory(Path.GetDirectoryName(dotnet)!);
            File.WriteAllBytes(dotnet, contents);

            Assert.Equal(contents, File.ReadAllBytes(native));
            G("mode", Mode(native));
            G("folder's mode", Mode(Path.GetDirectoryName(native)!));
            Assert.False(File.Exists(native + AtomicFile.TempSuffix));
            Assert.Equal(contents, GalaxiaNative.FileRead(native));
            Assert.Equal(contents, GalaxiaNative.FileRead(dotnet));
            Assert.Empty(Reported());
        }

        [Fact]
        public void A_write_that_cannot_land_fails_and_leaves_what_the_csharp_left()
        {
            Put(At("native", "blocker"), "a file where a directory is wanted");
            Directory.CreateDirectory(At("native", "taken.srm"));
            Put(At("native", "kept.srm"), "old");

            foreach (string name in new[] { Path.Combine("blocker", "game.srm"), "taken.srm" })
            {
                Assert.False(GalaxiaNative.FileWrite(At("native", name), new byte[] { 1 }));
                string message = Assert.Single(Reported());
                Assert.StartsWith(At("native", name) + ": ", message);
                Assert.EndsWith(" The previous file is unchanged.", message);
                G($"temp file left by {name}", File.Exists(At("native", name) + AtomicFile.TempSuffix));
            }
            // A write over a directory strands its temp file, as the C#'s did, which the next successful write of that name would replace.
            Assert.True(File.Exists(At("native", "taken.srm") + AtomicFile.TempSuffix));
            Assert.Equal("old", File.ReadAllText(At("native", "kept.srm")));
        }

        // The old tree both sides are given: every directory the migration takes, the two it must not, and each awkward entry.
        private void BuildLegacy(string side)
        {
            string legacy = At(side, "legacy"), home = At(side, "home");
            Put(Path.Combine(legacy, "Saves", "ALTTP.srm"), "save");
            Put(Path.Combine(legacy, "Saves", "Save States", "ALTTP.state"), "state");
            Put(Path.Combine(legacy, "Saves", "Save States", "ALTTP.slot2.state"), "state two");
            Put(Path.Combine(legacy, "Firmware", "dsp1.rom"), "fw");
            Put(Path.Combine(legacy, "Cheats", "SNES", "é 日本.cht"), "cheat");
            Put(Path.Combine(legacy, "Logs", "Venus", "run.txt"), "log");
            Put(Path.Combine(legacy, "Logs", "a", "b", "c", "d", "deep.txt"), "deep");
            Put(Path.Combine(legacy, "Other", "not.migrated"), "other");
            Put(Path.Combine(legacy, "Games", "SNES", "ALTTP.smc"), "rom");
            Put(Path.Combine(legacy, "Roms", "SMW.smc"), "rom");
            Directory.CreateDirectory(Path.Combine(legacy, "Cheats", "empty"));
            Put(Path.Combine(legacy, "Saves", "kept.srm"), "old");
            Put(Path.Combine(home, "Saves", "kept.srm"), "newer");
            Put(Path.Combine(legacy, "Saves", "blocked.srm"), "blocked");
            Directory.CreateDirectory(Path.Combine(home, "Saves", "blocked.srm"));
            if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(Path.Combine(legacy, "Firmware", "dsp1.rom"), UnixFileMode.UserRead | UnixFileMode.UserWrite);
            if (Links.Allowed)
            {
                Put(At(side, "outside", "inner", "linked.txt"), "linked");
                Directory.CreateSymbolicLink(Path.Combine(legacy, "Saves", "linked"), At(side, "outside"));
                File.CreateSymbolicLink(Path.Combine(legacy, "Logs", "to-file.txt"), Path.Combine(legacy, "Firmware", "dsp1.rom"));
                File.CreateSymbolicLink(Path.Combine(legacy, "Logs", "dangling.txt"), At(side, "nowhere"));
            }
            Stamp(At(side));
        }

        // Gives every file under a directory a time fixed by its place, so two trees built a moment apart are written at the same instants.
        private static void Stamp(string root)
        {
            var files = Directory.EnumerateFiles(root, "*", new EnumerationOptions { RecurseSubdirectories = true, AttributesToSkip = FileAttributes.ReparsePoint })
                .OrderBy(f => Path.GetRelativePath(root, f), StringComparer.Ordinal)
                .ToList();
            for (int i = 0; i < files.Count; i++)
                File.SetLastWriteTimeUtc(files[i], new DateTime(2020, 1, 2, 3, 4, 5, DateTimeKind.Utc).AddTicks(i * 12_345_677L));
        }

        // Every file under a directory with what a copy is answerable for: its bytes, its permissions and when it was last written.
        private static List<string> Listing(string root) =>
            !Directory.Exists(root) ? new List<string>() : Directory.EnumerateFiles(root, "*", SearchOption.AllDirectories)
                .Select(f => $"{Path.GetRelativePath(root, f)} | {Entry(f)}")
                .OrderBy(line => line, StringComparer.Ordinal)
                .ToList();

        // A link is listed by what it names and not by its own time, which is when the test made it; one that names nothing has no bytes at all.
        private static string Entry(string file)
        {
            if (new FileInfo(file).LinkTarget is not { } target) return $"{Convert.ToHexString(File.ReadAllBytes(file))} | {Mode(file)} | {File.GetLastWriteTimeUtc(file).Ticks}";
            string named = Path.GetFullPath(target, Path.GetDirectoryName(file)!);
            return File.Exists(named) ? $"a link to {Convert.ToHexString(File.ReadAllBytes(named))}" : "a dangling link";
        }

        private List<string> Told(string side) => Reported().Select(m => m.Replace(At(side), "<root>")).OrderBy(m => m, StringComparer.Ordinal).ToList();

        [Fact]
        public void A_migration_copies_the_files_the_csharp_copied_the_same_way_and_leaves_the_same_ones()
        {
            BuildLegacy("native");
            List<string> romsBefore = Listing(At("native", "legacy", "Games")).Concat(Listing(At("native", "legacy", "Roms"))).ToList();

            int native = G("copied", GalaxiaNative.Migrate(GalaxiaMigrate.Between, At("native", "legacy"), At("native", "home")));
            List<string> nativeTold = Told("native");

            Assert.True(native >= 7, $"{native} copied");
            G("home", Listing(At("native", "home")));
            G("legacy", Listing(At("native", "legacy")));
            Assert.Equal(romsBefore, Listing(At("native", "legacy", "Games")).Concat(Listing(At("native", "legacy", "Roms"))).ToList());
            Assert.False(Directory.Exists(At("native", "home", "Games")));
            Assert.False(Directory.Exists(At("native", "home", "Roms")));
            Assert.False(Directory.Exists(At("native", "home", "Other")));
            Assert.Equal("newer", File.ReadAllText(At("native", "home", "Saves", "kept.srm")));

            // The files the C# left, each said once; only .NET's words for why differed from the library's.
            Assert.Equal(Links.Allowed ? 2 : 1, nativeTold.Count);
            G("left", nativeTold.Select(n => n[..n.IndexOf(": ", StringComparison.Ordinal)]));
            Assert.All(nativeTold, n => Assert.EndsWith(" Left in place.", n));

            G("copied again", GalaxiaNative.Migrate(GalaxiaMigrate.Between, At("native", "legacy"), At("native", "home")));
            G("home again", Listing(At("native", "home")));
            Reported();

            G("remaining", GalaxiaNative.RemainingLibrary(At("native", "legacy")).Select(p => p.Replace(At("native"), "<root>")));
            Assert.Equal(2, GalaxiaNative.RemainingLibrary(At("native", "legacy")).Count);
            Assert.Empty(Reported());
        }

        [Fact]
        public void A_directory_that_will_not_list_stops_a_migration_with_the_csharp_s_exception()
        {
            if (OperatingSystem.IsWindows() || Environment.UserName == "root") return;
            foreach (string side in new[] { "native" })
            {
                Put(At(side, "tree", "open", "a.txt"), "a");
                Directory.CreateDirectory(At(side, "tree", "shut"));
                File.SetUnixFileMode(At(side, "tree", "shut"), UnixFileMode.None);
            }
            try
            {
                Assert.Throws<UnauthorizedAccessException>(() => GalaxiaNative.Migrate(GalaxiaMigrate.Tree, At("native", "tree"), At("native", "copy")));
            }
            finally
            {
                foreach (string side in new[] { "native" })
                    File.SetUnixFileMode(At(side, "tree", "shut"), UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
            }
        }

        // The process's own old tree and config, with both destinations redirected so nothing is written outside the test's folder.
        [Fact]
        public void The_programs_own_migrations_are_the_explicit_ones_they_were_made_of()
        {
            // Their sources are this checkout's and this machine's, not the test's, so each is held to the explicit migrations the C# composed it of, whose answers are recorded above - see EmuSen_Backend.md §40.2.
            string legacy = GalaxiaNative.Directory(GalaxiaDirectory.LegacyRoot)!;
            foreach (GalaxiaMigrate what in new[] { GalaxiaMigrate.Own, GalaxiaMigrate.OwnConfig, GalaxiaMigrate.OwnSeed })
            {
                int Into(string side, Func<int> migrate)
                {
                    DataStore.OverrideDirectory = At(side, what.ToString(), "home");
                    ConfigStore.OverrideDirectory = At(side, what.ToString(), "etc");
                    return migrate();
                }
                int Config() => GalaxiaNative.Migrate(GalaxiaMigrate.Tree, ConfigStore.PreviousDirectory, ConfigStore.Directory);
                int own = Into("own", () => GalaxiaNative.Migrate(what));
                int spelled = Into("spelled", () => what switch
                {
                    GalaxiaMigrate.Own => GalaxiaNative.Migrate(GalaxiaMigrate.Between, legacy, DataStore.UsrHome) + Config(),
                    GalaxiaMigrate.OwnConfig => Config(),
                    _ => GalaxiaNative.Migrate(GalaxiaMigrate.Seed, ConfigRoot.SeedDirectory, DataStore.UsrHome),
                });
                Assert.Equal(spelled, own);
                Assert.Equal(Listing(At("spelled", what.ToString())), Listing(At("own", what.ToString())));
            }
            Assert.Equal(GalaxiaNative.RemainingLibrary(legacy), GalaxiaNative.RemainingLibrary(null));
            Reported();
        }

        [Theory]
        [InlineData(0)]
        [InlineData(1)]
        [InlineData(55)]
        [InlineData(56)]
        [InlineData(64)]
        [InlineData((1 << 16) - 1)]
        [InlineData(1 << 16)]
        [InlineData((1 << 16) + 1)]
        [InlineData(3_000_000)]
        public void A_roms_hash_is_the_same_text(int length)
        {
            byte[] contents = new byte[length];
            new Random(length + 1).NextBytes(contents);
            string rom = At("hash", $"rom {length} é.bin");
            Directory.CreateDirectory(Path.GetDirectoryName(rom)!);
            File.WriteAllBytes(rom, contents);

            Assert.Equal(Convert.ToHexStringLower(System.Security.Cryptography.MD5.HashData(contents)), GalaxiaNative.RomMd5(rom));
        }

        [Fact]
        public void A_rom_that_cannot_be_hashed_throws_what_it_threw()
        {
            Directory.CreateDirectory(At("hash"));
            foreach (string path in new[] { At("hash", "absent.bin"), At("no", "such", "folder", "rom.bin"), At("hash") })
                G(path, Assert.ThrowsAny<Exception>(() => GalaxiaNative.RomMd5(path)).GetType().Name);
        }

        [Fact]
        public void The_library_has_every_export_the_facade_calls_and_no_other()
        {
            string header = File.ReadAllText(Path.Combine(Repo, "Platform", "include", "emusen_platform.h"));
            foreach (string export in GalaxiaNative.Exports) Assert.Contains(export + "(", header);
            Assert.Contains("emusen_platform_abi_version(", header);
            // Interface 45's settings rows serve the Qt front end, which the C# half never calls (EmuSen_RustPlatform.md §52).
            string[] frontEndOnly = { "emusen_galaxia_settings_rows" };
            foreach (string export in frontEndOnly) Assert.Contains(export + "(", header);
            int declared = System.Text.RegularExpressions.Regex.Matches(header, @"^\w+ \*?emusen_(galaxia|platform)_\w+\(", System.Text.RegularExpressions.RegexOptions.Multiline).Count;
            Assert.Equal(GalaxiaNative.Exports.Length + 1 + frontEndOnly.Length, declared);
            Assert.Contains($"#define EMUSEN_PLATFORM_ABI_VERSION {PlatformLibrary.InterfaceVersion}u", header);
        }
    }
}

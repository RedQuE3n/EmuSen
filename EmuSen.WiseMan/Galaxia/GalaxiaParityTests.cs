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
    // Galaxia's C# rules and the platform library's, asked the same things side by side - see EmuSen_RustPlatform.md §6.3 and §10.5.
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
            ConfigDiagnostics.Sink = null;
            ConfigDiagnostics.Reset();
            ConfigStore.OverrideDirectory = null;
            ConfigStore.OverrideLegacyDirectory = null;
            DataStore.OverrideDirectory = null;
            ErrorLog.DirectoryOverride = _logOverride;
            ErrorLog.Redactor = _redactor;
            if (Directory.Exists(_temp)) Directory.Delete(_temp, recursive: true);
        }

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

        // A run that says which of the two it used can only prove parity if the facade really took the side the variable names: state 2, the library unless 0.
        [Fact]
        public void The_switch_is_what_the_variable_says()
        {
            Assert.Equal(Environment.GetEnvironmentVariable(GalaxiaNative.Variable) != "0", GalaxiaNative.Active);
            Assert.Contains($"interface {PlatformLibrary.InterfaceVersion}", PlatformLibrary.Report);
        }

        // A folder without the library: the C#, one line to the sink and the error log however often the switch is asked, and no line for 0 - see EmuSen_RustPlatform.md §13.
        [Fact]
        public void A_missing_library_falls_back_to_the_csharp_and_says_so_once_in_the_log()
        {
            string logs = At("logs");
            ErrorLog.DirectoryOverride = logs;
            string empty = Directory.CreateDirectory(At("no library")).FullName;
            int loads = 0;
            foreach (string? variable in new[] { null, "1", "0" })
            {
                var chosen = new PlatformSwitch(() => variable, () => { loads++; var opened = PlatformLibrary.Open(empty); return (opened.Handle != 0, opened.Report); }, GalaxiaNative.Announce);
                Assert.All(Enumerable.Range(0, 5), _ => Assert.False(chosen.Active));
            }
            Assert.Equal(2, loads);
            string expected = $"{PlatformLibrary.FileName} is not in use ({PlatformLibrary.FileName} not found beside the assemblies); Galaxia runs on its C# implementation.";
            Assert.Equal(new[] { expected, expected }, Reported());
            string written = string.Concat(Directory.GetFiles(logs).Select(File.ReadAllText));
            Assert.Equal(2, written.Split(expected).Length - 1);
            Assert.Contains("platform", written);

            var thrown = new PlatformSwitch(() => null, () => throw new DllNotFoundException("refused"), GalaxiaNative.Announce);
            Assert.False(thrown.Active);
            Assert.Equal(new[] { $"{PlatformLibrary.FileName} is not in use (DllNotFoundException: refused); Galaxia runs on its C# implementation." }, Reported());
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
        public void Every_path_rule_gives_the_string_dotnet_gives()
        {
            string[] paths = Paths(3000, seed: 20261007).ToArray();
            for (int i = 0; i < paths.Length; i++)
            {
                string a = paths[i], b = paths[(i * 7 + 3) % paths.Length];
                Assert.Equal(Path.Combine(a, b), GalaxiaNative.PathRule(GalaxiaPathRule.Combine, a, b));
                Assert.Equal(Path.GetFileName(a), GalaxiaNative.PathRule(GalaxiaPathRule.FileName, a));
                Assert.Equal(Path.GetFileNameWithoutExtension(a), GalaxiaNative.PathRule(GalaxiaPathRule.FileNameWithoutExtension, a));
                Assert.Equal(Path.ChangeExtension(a, ".png"), GalaxiaNative.PathRule(GalaxiaPathRule.ChangeExtension, a, ".png"));
                Assert.Equal(Path.ChangeExtension(a, "png"), GalaxiaNative.PathRule(GalaxiaPathRule.ChangeExtension, a, "png"));
                Assert.Equal(Path.ChangeExtension(a, ""), GalaxiaNative.PathRule(GalaxiaPathRule.ChangeExtension, a, ""));
                Assert.Equal(Path.GetDirectoryName(a), GalaxiaNative.PathRule(GalaxiaPathRule.DirectoryName, a));
                Assert.Equal(Path.TrimEndingDirectorySeparator(a), GalaxiaNative.PathRule(GalaxiaPathRule.TrimEndingSeparator, a));
                if (a.Length > 0) Assert.Equal(Path.GetFullPath(a), GalaxiaNative.PathRule(GalaxiaPathRule.FullPath, a));
            }
        }

        [Fact]
        public void Blank_text_and_caseless_equality_follow_dotnet_for_every_character()
        {
            Assert.True(GalaxiaNative.TestRule(GalaxiaTestRule.IsBlank, null));
            Assert.True(GalaxiaNative.TestRule(GalaxiaTestRule.IsBlank, ""));
            var different = new List<string>();
            for (int c = 1; c < 0x10000; c++)
            {
                if (char.IsSurrogate((char)c)) continue;
                string text = ((char)c).ToString();
                if (string.IsNullOrWhiteSpace(text) != GalaxiaNative.TestRule(GalaxiaTestRule.IsBlank, text)) different.Add($"blank U+{c:X4}");
                foreach (string other in new[] { char.ToUpperInvariant((char)c).ToString(), char.ToLowerInvariant((char)c).ToString(), "A" })
                {
                    if (string.Equals("x" + text, "X" + other, StringComparison.OrdinalIgnoreCase) != GalaxiaNative.TestRule(GalaxiaTestRule.EqualsIgnoreCase, "x" + text, "X" + other))
                        different.Add($"equals U+{c:X4} U+{(int)other[0]:X4}");
                    if (("x" + text).EndsWith(other, StringComparison.OrdinalIgnoreCase) != GalaxiaNative.TestRule(GalaxiaTestRule.EndsWithIgnoreCase, "x" + text, other))
                        different.Add($"ends U+{c:X4} U+{(int)other[0]:X4}");
                }
            }
            for (int c = 0x10000; c < 0x20000; c++)
            {
                string text = char.ConvertFromUtf32(c), upper = text.ToUpperInvariant(), lower = text.ToLowerInvariant();
                foreach (string other in new[] { upper, lower })
                    if (string.Equals(text, other, StringComparison.OrdinalIgnoreCase) != GalaxiaNative.TestRule(GalaxiaTestRule.EqualsIgnoreCase, text, other))
                        different.Add($"equals U+{c:X5} U+{char.ConvertToUtf32(other, 0):X5}");
            }
            Assert.True(different.Count == 0, $"{different.Count} differ: {string.Join(", ", different.Take(40))}");
        }

        [Fact]
        public void A_file_and_a_directory_exist_for_both_or_for_neither()
        {
            string file = Put(At("exists", "file.bin"));
            var cases = new List<string?> { null, "", file, file + Path.DirectorySeparatorChar, At("exists"), At("exists") + Path.DirectorySeparatorChar, At("absent"), At("exists", "file.bin", "under") };
            if (!OperatingSystem.IsWindows())
            {
                File.CreateSymbolicLink(At("exists", "to-file"), file);
                Directory.CreateSymbolicLink(At("exists", "to-directory"), At("exists"));
                File.CreateSymbolicLink(At("exists", "dangling"), At("nowhere"));
                cases.AddRange(new[] { At("exists", "to-file"), At("exists", "to-directory"), At("exists", "dangling"), At("exists", "to-directory") + "/" });
            }
            foreach (string? path in cases)
            {
                Assert.True(File.Exists(path) == GalaxiaNative.TestRule(GalaxiaTestRule.FileExists, path), $"File.Exists({path})");
                Assert.True(Directory.Exists(path) == GalaxiaNative.TestRule(GalaxiaTestRule.DirectoryExists, path), $"Directory.Exists({path})");
            }
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
            string ten = Path.Combine(new[] { marked }.Concat(Enumerable.Range(1, 10).Select(i => i.ToString())).ToArray());
            string nine = Path.Combine(new[] { marked }.Concat(Enumerable.Range(1, 9).Select(i => i.ToString())).ToArray());
            foreach (string directory in new[] { bundle, ten, nine, At("bare", "bin"), Path.Combine(checkout, "deep", "er"), At("Other.bundle", "Contents", "MacOS"), At("x.APP", "contents", "macos") })
                Directory.CreateDirectory(directory);

            var bases = new List<string>
            {
                marked, marked + Path.DirectorySeparatorChar, checkout, bundle, bundle + Path.DirectorySeparatorChar, ten, nine, At("bare", "bin"), At("absent", "bin"),
                Path.Combine(checkout, "deep", "er"), Path.Combine(checkout, "deep", "er", "..", "..", "out"), At("Other.bundle", "Contents", "MacOS"),
                At("x.APP", "contents", "macos"), At("checkout", "out", "EmuSen.app", "Contents"), Path.GetPathRoot(_temp)!, "relative", ".",
            };
            bases.AddRange(Paths(300, seed: 7).Where(p => p.Length > 0));

            foreach (string from in bases)
            {
                foreach (bool macOS in new[] { false, true })
                {
                    Assert.Equal(ConfigRoot.Managed.ComputeFor(from, macOS, home), GalaxiaNative.RootFor(from, macOS, home));
                    Assert.Equal(ConfigRoot.Managed.SeedDirectoryFor(from, macOS), GalaxiaNative.SeedDirectoryFor(from, macOS));
                }
                Assert.Equal(ConfigRoot.Managed.BundleContentsFor(from), GalaxiaNative.BundleContentsFor(from));
                Assert.Equal(ConfigRoot.Managed.MacDataDirectoryFor(from), GalaxiaNative.MacDataDirectoryFor(from));
                Assert.Equal(DataMigration.Managed.LegacyRootFor(from), GalaxiaNative.LegacyRootFor(from));
            }

            // The cases are not vacuous: each branch of the rule is among them.
            Assert.Equal(marked, GalaxiaNative.RootFor(nine, false, home));
            Assert.Equal(Path.Combine(ten, ConfigRoot.PublishedRootDirName), GalaxiaNative.RootFor(ten, false, home));
            Assert.Equal(checkout, GalaxiaNative.RootFor(bundle, false, home));
            Assert.Equal(Path.Combine(home, "Library", "Application Support", "EmuSen"), GalaxiaNative.RootFor(bundle, true, home));
            Assert.Equal(checkout, GalaxiaNative.RootFor(Path.Combine(checkout, "deep", "er"), false, home));
        }

        private static readonly (GalaxiaDirectory Which, string Name)[] Named =
        {
            (GalaxiaDirectory.Library, "Library"), (GalaxiaDirectory.Screenshots, "Screenshots"), (GalaxiaDirectory.Artwork, "Artwork"),
            (GalaxiaDirectory.Media, "Media"), (GalaxiaDirectory.Firmware, "Firmware"), (GalaxiaDirectory.Shaders, "Shaders"),
            (GalaxiaDirectory.Themes, "Themes"), (GalaxiaDirectory.Cheats, "Cheats"), (GalaxiaDirectory.Games, "Games"),
        };

        private static void AssertTheTreeAgrees()
        {
            Assert.Equal(ConfigRoot.Managed.Directory, GalaxiaNative.Directory(GalaxiaDirectory.Root));
            Assert.Equal(ConfigRoot.Managed.SeedDirectory, GalaxiaNative.Directory(GalaxiaDirectory.Seed));
            Assert.Equal(ConfigStore.Managed.Directory, GalaxiaNative.Directory(GalaxiaDirectory.Config));
            Assert.Equal(ConfigStore.Managed.PreviousDirectory, GalaxiaNative.Directory(GalaxiaDirectory.ConfigPrevious));
            Assert.Equal(ConfigStore.Managed.LegacyDirectory, GalaxiaNative.Directory(GalaxiaDirectory.ConfigLegacy));
            Assert.Equal(DataStore.Managed.UsrHome, GalaxiaNative.Directory(GalaxiaDirectory.Home));
            Assert.Equal(DataStore.Managed.Logs, GalaxiaNative.Directory(GalaxiaDirectory.Logs));
            Assert.Equal(DataStore.Managed.Saves, GalaxiaNative.Directory(GalaxiaDirectory.Saves));
            Assert.Equal(DataStore.Managed.SaveStates, GalaxiaNative.Directory(GalaxiaDirectory.SaveStates));
            Assert.Equal(DataMigration.Managed.LegacyRoot, GalaxiaNative.Directory(GalaxiaDirectory.LegacyRoot));
            foreach ((GalaxiaDirectory which, string name) in Named)
                Assert.Equal(Path.Combine(DataStore.Managed.UsrHome, name), GalaxiaNative.Directory(which));
            Assert.Equal(ErrorLog.Managed.DefaultRoot, GalaxiaNative.Directory(GalaxiaDirectory.LogDefault));
            Assert.Equal(Enum.GetValues<GalaxiaDirectory>().Length, Named.Length + 11);

            foreach (string file in new[] { "appsettings.json", "é.json", "a b.json", "" })
            {
                Assert.Equal(ConfigStore.Managed.For(file), GalaxiaNative.ConfigPath(null, file));
                Assert.Equal(ConfigStore.Managed.For("cheats", file), GalaxiaNative.ConfigPath("cheats", file));
                Assert.Equal(ConfigStore.Managed.For("", file), GalaxiaNative.ConfigPath("", file));
                Assert.Equal(ConfigStore.Managed.LegacyPathFor(file), GalaxiaNative.LegacyConfigPath(file));
            }
        }

        [Fact]
        public void Every_directory_agrees_under_every_combination_of_overrides()
        {
            string?[] choices = { null, At("redirect"), At("redirect") + Path.DirectorySeparatorChar, "relative", At("é 日本") };
            foreach (string? config in choices)
                foreach (string? legacy in choices)
                    foreach (string? data in choices)
                    {
                        ConfigStore.OverrideDirectory = config;
                        ConfigStore.OverrideLegacyDirectory = legacy;
                        DataStore.OverrideDirectory = data;
                        AssertTheTreeAgrees();
                    }

            // A redirected config with no redirected legacy directory reaches no real one, on either side.
            ConfigStore.OverrideDirectory = At("redirect");
            ConfigStore.OverrideLegacyDirectory = null;
            Assert.Null(GalaxiaNative.LegacyConfigPath("appsettings.json"));
            ConfigStore.OverrideDirectory = null;
            Assert.NotNull(GalaxiaNative.LegacyConfigPath("appsettings.json"));
        }

        [Fact]
        public void Every_save_is_named_alike()
        {
            var roms = new List<string>
            {
                "/roms/SNES/Zelda (U).sfc", "a.b.c.nes", "noext", ".hidden", "trailing.", "dir.with.dot/file", "日本語.gb", "with space.n64", @"C:\win\style.sfc",
                "a/b/", "", "x.y/", "/", "é.sfc", "..", "state.slot2.state",
            };
            roms.AddRange(Paths(400, seed: 11));
            string?[] directories = { null, "", "  ", "\t", At("elsewhere"), At("elsewhere") + Path.DirectorySeparatorChar, "relative", " padded " };
            string[] consoles = { "SNES", "NES", "N64", "GB", "", "Game Boy Color", ".." };
            int[] slots = { 1, 2, 3, 10, 0, -1, int.MaxValue, int.MinValue };

            foreach (string? data in new[] { null, At("data") })
            {
                DataStore.OverrideDirectory = data;
                foreach (string rom in roms)
                {
                    Assert.Equal(SaveLibrary.Managed.FlatSramPathFor(rom), GalaxiaNative.SavePath(GalaxiaSave.FlatSram, rom));
                    Assert.Equal(SaveLibrary.Managed.PicturePathFor(rom), GalaxiaNative.SavePath(GalaxiaSave.Picture, rom));
                    foreach (string console in consoles)
                        Assert.Equal(SaveLibrary.Managed.SramPathFor(rom, console), GalaxiaNative.SavePath(GalaxiaSave.Sram, rom, console));
                    foreach (string? directory in directories)
                    {
                        Assert.Equal(SaveLibrary.Managed.ResumeStatePathFor(rom, directory), GalaxiaNative.SavePath(GalaxiaSave.ResumeState, rom, directory));
                        foreach (int slot in slots)
                            Assert.Equal(SaveLibrary.Managed.StatePathFor(rom, slot, directory), GalaxiaNative.SavePath(GalaxiaSave.State, rom, directory, slot));
                    }
                }
            }
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
        public void A_file_written_by_one_side_is_the_file_the_other_writes_and_reads(int length)
        {
            byte[] contents = new byte[length];
            new Random(length).NextBytes(contents);
            string managed = At("managed", "Saves", "SNES", "game.srm"), native = At("native", "Saves", "SNES", "game.srm");

            foreach (byte[] bytes in new[] { new byte[] { 9, 9, 9 }, contents })
            {
                Assert.True(AtomicFile.Managed.Write(managed, bytes));
                Assert.True(GalaxiaNative.FileWrite(native, bytes));
            }

            Assert.Equal(contents, File.ReadAllBytes(managed));
            Assert.Equal(contents, File.ReadAllBytes(native));
            Assert.Equal(Mode(managed), Mode(native));
            Assert.Equal(Mode(Path.GetDirectoryName(managed)!), Mode(Path.GetDirectoryName(native)!));
            Assert.False(File.Exists(managed + AtomicFile.TempSuffix));
            Assert.False(File.Exists(native + AtomicFile.TempSuffix));
            Assert.Equal(contents, AtomicFile.Managed.TryRead(native));
            Assert.Equal(contents, GalaxiaNative.FileRead(managed));
            Assert.Empty(Reported());
        }

        [Fact]
        public void What_is_not_a_readable_file_is_absent_for_both_and_said_by_both_or_by_neither()
        {
            string file = Put(At("read", "save.srm"), "whole");
            Put(file + AtomicFile.TempSuffix, "half a sa");
            var quiet = new List<string> { At("read", "none.srm"), At("read"), "", At("read", "save.srm", "under"), At("read", "other.srm") + AtomicFile.TempSuffix };
            var loud = new List<string>();
            if (!OperatingSystem.IsWindows())
            {
                File.CreateSymbolicLink(At("read", "dangling.srm"), At("nowhere"));
                loud.Add(At("read", "dangling.srm"));
                if (Environment.UserName != "root")
                {
                    File.SetUnixFileMode(Put(At("read", "locked.srm")), UnixFileMode.None);
                    loud.Add(At("read", "locked.srm"));
                }
            }

            Assert.Equal(AtomicFile.Managed.TryRead(file), GalaxiaNative.FileRead(file));
            Assert.Empty(Reported());
            foreach (string path in quiet)
            {
                Assert.Null(AtomicFile.Managed.TryRead(path));
                Assert.Null(GalaxiaNative.FileRead(path));
                Assert.Empty(Reported());
            }
            foreach (string path in loud)
            {
                Assert.Null(AtomicFile.Managed.TryRead(path));
                string managed = Assert.Single(Reported());
                Assert.Null(GalaxiaNative.FileRead(path));
                string native = Assert.Single(Reported());
                foreach (string message in new[] { managed, native })
                {
                    Assert.StartsWith(path + ": ", message);
                    Assert.EndsWith(" Treating it as absent.", message);
                }
            }
        }

        [Fact]
        public void A_write_that_cannot_land_fails_alike_and_leaves_alike()
        {
            foreach (string side in new[] { "managed", "native" })
            {
                Put(At(side, "blocker"), "a file where a directory is wanted");
                Directory.CreateDirectory(At(side, "taken.srm"));
                Put(At(side, "kept.srm"), "old");
            }
            bool Write(string side, string name) =>
                side == "managed" ? AtomicFile.Managed.Write(At(side, name), new byte[] { 1 }) : GalaxiaNative.FileWrite(At(side, name), new byte[] { 1 });

            foreach (string name in new[] { Path.Combine("blocker", "game.srm"), "taken.srm" })
            {
                foreach (string side in new[] { "managed", "native" })
                {
                    Assert.False(Write(side, name));
                    string message = Assert.Single(Reported());
                    Assert.StartsWith(At(side, name) + ": ", message);
                    Assert.EndsWith(" The previous file is unchanged.", message);
                }
                Assert.Equal(File.Exists(At("managed", name) + AtomicFile.TempSuffix), File.Exists(At("native", name) + AtomicFile.TempSuffix));
            }
            // A write over a directory strands its temp file on both sides, which the next successful write of that name would replace.
            Assert.True(File.Exists(At("native", "taken.srm") + AtomicFile.TempSuffix));
            Assert.Equal("old", File.ReadAllText(At("managed", "kept.srm")));
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
            if (!OperatingSystem.IsWindows())
            {
                File.SetUnixFileMode(Path.Combine(legacy, "Firmware", "dsp1.rom"), UnixFileMode.UserRead | UnixFileMode.UserWrite);
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

        // Two listings compared whole, with the lines only one of them has named in full.
        private static void AssertSameListing(List<string> managed, List<string> native) =>
            Assert.True(managed.SequenceEqual(native), $"only the C#:\n{string.Join("\n", managed.Except(native))}\nonly the library:\n{string.Join("\n", native.Except(managed))}");

        private List<string> Told(string side) => Reported().Select(m => m.Replace(At(side), "<root>")).OrderBy(m => m, StringComparer.Ordinal).ToList();

        [Fact]
        public void A_migration_copies_the_same_files_the_same_way_and_leaves_the_same_ones()
        {
            BuildLegacy("managed");
            BuildLegacy("native");
            List<string> romsBefore = Listing(At("native", "legacy", "Games")).Concat(Listing(At("native", "legacy", "Roms"))).ToList();

            int managed = DataMigration.Managed.Run(At("managed", "legacy"), At("managed", "home"));
            List<string> managedTold = Told("managed");
            int native = GalaxiaNative.Migrate(GalaxiaMigrate.Between, At("native", "legacy"), At("native", "home"));
            List<string> nativeTold = Told("native");

            Assert.Equal(managed, native);
            Assert.True(managed >= 7, $"{managed} copied");
            AssertSameListing(Listing(At("managed", "home")), Listing(At("native", "home")));
            AssertSameListing(Listing(At("managed", "legacy")), Listing(At("native", "legacy")));
            Assert.Equal(romsBefore, Listing(At("native", "legacy", "Games")).Concat(Listing(At("native", "legacy", "Roms"))).ToList());
            Assert.False(Directory.Exists(At("native", "home", "Games")));
            Assert.False(Directory.Exists(At("native", "home", "Roms")));
            Assert.False(Directory.Exists(At("native", "home", "Other")));
            Assert.Equal("newer", File.ReadAllText(At("native", "home", "Saves", "kept.srm")));

            // The same files were left, each said once; only .NET's words for why differ from the library's.
            Assert.Equal(managedTold.Count, nativeTold.Count);
            Assert.Equal(OperatingSystem.IsWindows() ? 1 : 2, nativeTold.Count);
            foreach ((string m, string n) in managedTold.Zip(nativeTold))
            {
                Assert.Equal(m[..m.IndexOf(": ", StringComparison.Ordinal)], n[..n.IndexOf(": ", StringComparison.Ordinal)]);
                Assert.EndsWith(" Left in place.", m);
                Assert.EndsWith(" Left in place.", n);
            }

            Assert.Equal(DataMigration.Managed.Run(At("managed", "legacy"), At("managed", "home")), GalaxiaNative.Migrate(GalaxiaMigrate.Between, At("native", "legacy"), At("native", "home")));
            AssertSameListing(Listing(At("managed", "home")), Listing(At("native", "home")));
            Reported();

            Assert.Equal(
                DataMigration.Managed.RemainingLibraryDirectories(At("managed", "legacy")).Select(p => p.Replace(At("managed"), "<root>")),
                GalaxiaNative.RemainingLibrary(At("native", "legacy")).Select(p => p.Replace(At("native"), "<root>")));
            Assert.Equal(2, GalaxiaNative.RemainingLibrary(At("native", "legacy")).Count);
            Assert.Empty(Reported());
        }

        [Fact]
        public void The_other_migrations_agree_on_what_they_decline_and_what_they_copy()
        {
            foreach (string side in new[] { "managed", "native" })
            {
                Put(At(side, "tree", "Saves", "a.srm"), "a");
                Put(At(side, "tree", "Documents", "manual.md"), "page");
                Put(At(side, "seeded", "Documents", "mine.md"), "mine");
                Directory.CreateDirectory(At(side, "TREE"));
                Directory.CreateDirectory(At(side, "library", "Games", "SNES", "empty"));
                Put(At(side, "library", "Roms", "deep", "er", "game.sfc"), "rom");
                Stamp(At(side));
            }
            int Between(string side, string from, string to) =>
                side == "managed" ? DataMigration.Managed.Run(from, to) : GalaxiaNative.Migrate(GalaxiaMigrate.Between, from, to);
            int Tree(string side, string from, string to) =>
                side == "managed" ? DataMigration.Managed.CopyTree(from, to) : GalaxiaNative.Migrate(GalaxiaMigrate.Tree, from, to);
            int Seed(string side, string? from, string to) =>
                side == "managed" ? DataMigration.Managed.SeedFromBundle(from, to) : GalaxiaNative.Migrate(GalaxiaMigrate.Seed, from, to);
            (string Name, Func<string, int> Run)[] cases =
            {
                ("onto itself", side => Between(side, At(side, "tree"), At(side, "tree"))),
                ("onto itself, spelled in another case", side => Between(side, At(side, "tree"), At(side, "TREE"))),
                ("onto itself, by a longer way round", side => Between(side, At(side, "tree"), At(side, "x", "..", "tree"))),
                ("onto itself, with a separator after it", side => Between(side, At(side, "tree"), At(side, "tree") + Path.DirectorySeparatorChar)),
                ("from nowhere", side => Between(side, At(side, "absent"), At(side, "to"))),
                ("from a file", side => Between(side, At(side, "tree", "Saves", "a.srm"), At(side, "to"))),
                ("a tree from nowhere", side => Tree(side, At(side, "absent"), At(side, "to"))),
                ("no seed", side => Seed(side, null, At(side, "seeded"))),
                ("a seed", side => Seed(side, At(side, "tree"), At(side, "seeded"))),
                ("the seed again", side => Seed(side, At(side, "tree"), At(side, "seeded"))),
                ("a whole tree", side => Tree(side, At(side, "tree"), At(side, "copy"))),
            };
            foreach ((string name, Func<string, int> run) in cases)
                Assert.True(run("managed") == run("native"), name);
            Assert.Equal(2, Seed("native", At("native", "tree"), At("native", "seeded-again")));
            AssertSameListing(Listing(At("managed", "seeded")), Listing(At("native", "seeded")));
            AssertSameListing(Listing(At("managed", "copy")), Listing(At("native", "copy")));
            Assert.Equal("mine", File.ReadAllText(At("native", "seeded", "Documents", "mine.md")));
            Assert.Equal(new[] { At("native", "library", "Roms") }, GalaxiaNative.RemainingLibrary(At("native", "library")));
            Assert.Equal(DataMigration.Managed.RemainingLibraryDirectories(At("managed", "library")).Count, GalaxiaNative.RemainingLibrary(At("native", "library")).Count);
            Assert.Empty(Reported());
        }

        [Fact]
        public void A_directory_that_will_not_list_stops_both_migrations_with_the_same_exception()
        {
            if (OperatingSystem.IsWindows() || Environment.UserName == "root") return;
            foreach (string side in new[] { "managed", "native" })
            {
                Put(At(side, "tree", "open", "a.txt"), "a");
                Directory.CreateDirectory(At(side, "tree", "shut"));
                File.SetUnixFileMode(At(side, "tree", "shut"), UnixFileMode.None);
            }
            try
            {
                Assert.Throws<UnauthorizedAccessException>(() => DataMigration.Managed.CopyTree(At("managed", "tree"), At("managed", "copy")));
                Assert.Throws<UnauthorizedAccessException>(() => GalaxiaNative.Migrate(GalaxiaMigrate.Tree, At("native", "tree"), At("native", "copy")));
            }
            finally
            {
                foreach (string side in new[] { "managed", "native" })
                    File.SetUnixFileMode(At(side, "tree", "shut"), UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
            }
        }

        // The process's own old tree and config, with both destinations redirected so nothing is written outside the test's folder.
        [Fact]
        public void The_programs_own_migrations_agree_with_their_destinations_redirected()
        {
            (GalaxiaMigrate What, Func<int> Managed)[] own =
            {
                (GalaxiaMigrate.Own, DataMigration.Managed.Run), (GalaxiaMigrate.OwnConfig, DataMigration.Managed.RunConfig), (GalaxiaMigrate.OwnSeed, DataMigration.Managed.SeedFromBundle),
            };
            foreach ((GalaxiaMigrate what, Func<int> managed) in own)
            {
                DataStore.OverrideDirectory = At("managed", what.ToString(), "home");
                ConfigStore.OverrideDirectory = At("managed", what.ToString(), "etc");
                int expected = managed();
                DataStore.OverrideDirectory = At("native", what.ToString(), "home");
                ConfigStore.OverrideDirectory = At("native", what.ToString(), "etc");
                Assert.Equal(expected, GalaxiaNative.Migrate(what));
                AssertSameListing(Listing(At("managed", what.ToString())), Listing(At("native", what.ToString())));
            }
            Assert.Equal(DataMigration.Managed.RemainingLibraryDirectories(DataMigration.Managed.LegacyRoot), GalaxiaNative.RemainingLibrary(null));
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

            Assert.Equal(RomHash.Managed.Md5(rom), GalaxiaNative.RomMd5(rom));
            Assert.Equal(32, GalaxiaNative.RomMd5(rom).Length);
        }

        [Fact]
        public void A_rom_that_cannot_be_hashed_throws_what_it_threw()
        {
            Directory.CreateDirectory(At("hash"));
            foreach (string path in new[] { At("hash", "absent.bin"), At("no", "such", "folder", "rom.bin"), At("hash") })
            {
                Exception managed = Assert.ThrowsAny<Exception>(() => RomHash.Managed.Md5(path));
                Exception native = Assert.ThrowsAny<Exception>(() => GalaxiaNative.RomMd5(path));
                Assert.True(managed.GetType() == native.GetType(), $"{path}: {managed.GetType().Name} and {native.GetType().Name}");
            }
        }

        [Fact]
        public void The_library_has_every_export_the_facade_calls_and_no_other()
        {
            string header = File.ReadAllText(Path.Combine(ConfigRoot.Managed.Directory, "Platform", "include", "emusen_platform.h"));
            foreach (string export in GalaxiaNative.Exports) Assert.Contains(export + "(", header);
            Assert.Contains("emusen_platform_abi_version(", header);
            int declared = System.Text.RegularExpressions.Regex.Matches(header, @"^\w+ \*?emusen_(galaxia|platform)_\w+\(", System.Text.RegularExpressions.RegexOptions.Multiline).Count;
            Assert.Equal(GalaxiaNative.Exports.Length + 1, declared);
            Assert.Contains($"#define EMUSEN_PLATFORM_ABI_VERSION {PlatformLibrary.InterfaceVersion}u", header);
        }
    }
}

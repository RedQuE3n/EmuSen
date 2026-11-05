using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // An installed libretro entry laid out as Cores lays one out: the project's proxy and libretro host copied in, the test core beside them, approved by its hash, its sidecar written once - see EmuSen_LibretroHost.md §10d.
    public sealed class LibretroEntryFixture : IDisposable
    {
        [DllImport("libc", EntryPoint = "setenv")] private static extern int SetEnv(string name, string value, int overwrite);
        [DllImport("libc", EntryPoint = "unsetenv")] private static extern int UnsetEnv(string name);

        // The runner the proxy starts, named in the process's own environment, which .NET's SetEnvironmentVariable does not reach.
        public static void Environment(string name, string? value) => _ = value is null ? UnsetEnv(name) : SetEnv(name, value, 1);

        public string Root { get; } = Path.Combine(Path.GetTempPath(), "EmuSenLibretro_" + Guid.NewGuid().ToString("N"));
        public string Cores => Path.Combine(Root, "Cores");
        public string Stem { get; }
        public string Entry => Path.Combine(Cores, LibretroEntries.Folder, Stem);
        public string CoreFile => Path.Combine(Entry, $"{Stem}_libretro.so");
        public string Sha256 { get; }
        public DiscoveredCore Found { get; }

        public LibretroEntryFixture() : this("") { }

        // An entry whose information file names the libretro system, so that its machine reports the systems table's console; under another stem, that core's profile applies.
        internal LibretroEntryFixture(string systemId, string stem = "testcore", string extensions = "", string corename = "TestCore")
        {
            Stem = stem;
            DataStore.OverrideDirectory = Path.Combine(Root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(Root, "Config");
            Lay(Entry, systemId, stem, extensions, corename);
            Environment("EMUSEN_CORE_RUNNER", Path.Combine(AppContext.BaseDirectory, "emusen-core-runner"));
            Sha256 = CoreSidecar.HashOf(CoreFile);
            Approve(Sha256);
            CoreDiscovery.UseDirectories(new[] { AppContext.BaseDirectory });
            CoreDiscovery.UsePlayerDirectories(new[] { Cores });
            CoreSidecar.Write(LibretroEntries.ProxyPath(Entry), expect: Sha256);
            CoreDiscovery.Rescan();
            Found = CoreDiscovery.Found.Single(c => c.Entry == Entry);
        }

        // An entry's files: the two copies from the installation, the test core named on its stem, its information file, with the extensions a real core of that stem lists when given.
        public static void Lay(string folder, string systemId = "", string stem = "testcore", string extensions = "", string corename = "TestCore")
        {
            Directory.CreateDirectory(folder);
            File.Copy(CoreAdapterTests.LibraryPath(LibretroEntries.ProxyLibrary), LibretroEntries.ProxyPath(folder), overwrite: true);
            File.Copy(CoreAdapterTests.LibraryPath(LibretroEntries.HostLibrary), LibretroEntries.AdapterPath(folder), overwrite: true);
            File.Copy(CoreAdapterTests.LibraryPath("emusen_libretro_testcore"), Path.Combine(folder, $"{stem}_libretro.so"), overwrite: true);
            File.WriteAllText(Path.Combine(folder, $"{stem}_libretro.info"), $"display_name = \"EmuSen - Test Core\"\ncorename = \"{corename}\"\nlicense = \"GPLv3\"\nauthors = \"EmuSen\"\nsystemid = \"{systemId}\"\ndatabase = \"\"\n" + (extensions.Length > 0 ? $"supported_extensions = \"{extensions}\"\n" : ""));
        }

        public static void Approve(string sha256)
        {
            string path = CoreTrust.ApprovalsPath;
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            string text = File.Exists(path) ? File.ReadAllText(path) : "";
            if (CoreTrust.WithApproval(text, sha256, "testcore_libretro.so") is { } approved) File.WriteAllText(path, approved);
            CoreDiscovery.Rescan();
        }

        public static void Revoke(string sha256)
        {
            string path = CoreTrust.ApprovalsPath;
            if (File.Exists(path) && CoreTrust.WithoutApproval(File.ReadAllText(path), sha256) is { } text) File.WriteAllText(path, text);
            CoreDiscovery.Rescan();
        }

        public void Dispose()
        {
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UsePlayerDirectories(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(Root, recursive: true); } catch (IOException) { } catch (UnauthorizedAccessException) { }
        }
    }

    // The test core brokered through the C# host, headless: trust checked before any runner starts, a game's picture, input, image path and named files, and the broker's fault and sandbox proofs under .NET - see EmuSen_LibretroHost.md §3.5, §10d.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class LibretroHostTests(LibretroEntryFixture entry) : IClassFixture<LibretroEntryFixture>
    {
        // emusen-libretro-testcore's layout.rs: the frame counter, the pad's mask and what the core's fault did.
        private const int FrameCounter = 0x20, Pad = 0x30, Fault = 0x5C;

        private string Game(string name = "game.tcg")
        {
            string path = Path.Combine(entry.Root, name);
            File.WriteAllBytes(path, "TCG1 a test game"u8.ToArray());
            return path;
        }

        private CoreEngine Engine(params (string Key, string Value)[] settings)
        {
            var engine = new CoreEngine(entry.Found.Open() ?? throw new Xunit.Sdk.XunitException(entry.Found.Report)) { Options = TestCores.NoBattery };
            foreach (var (key, value) in settings) engine.Set(key, value);
            return engine;
        }

        private static byte Ram(CoreEngine engine, int address)
        {
            Span<byte> b = stackalloc byte[1];
            Assert.True(engine.Machine.ReadSpace(0, address, b));
            return b[0];
        }

        [Fact]
        public void The_entry_is_trusted_by_its_libretro_cores_hash_and_this_hosts_obligations()
        {
            Assert.Equal(("approved", "testcore_libretro.so", entry.Sha256), (entry.Found.Standing, entry.Found.Info.Libretro!.File, entry.Found.Info.Libretro.Sha256));
            CoreLibrary library = entry.Found.Open()!;
            Assert.True(library.Available, library.Report);
            Assert.Contains("named_files", library.Info.HostRequires);
            Assert.True(library.Has(CoreInterface.CapImagePath));
        }

        // §1.3: an unapproved core, another build of it, or a copy that is not this installation's starts nothing, the proxy never loaded.
        [Fact]
        public void Nothing_of_an_entry_starts_before_its_core_is_approved_and_its_copies_are_the_installations()
        {
            string Copy(string name, Action<string>? change = null)
            {
                string folder = Path.Combine(entry.Root, "Elsewhere", LibretroEntries.Folder, name);
                LibretroEntryFixture.Lay(folder);
                File.Copy(CoreSidecar.PathFor(LibretroEntries.ProxyPath(entry.Entry)), CoreSidecar.PathFor(LibretroEntries.ProxyPath(folder)), overwrite: true);
                change?.Invoke(folder);
                return folder;
            }
            static void Grow(string file) => File.AppendAllText(file, "x");
            string rebuilt = Copy("rebuilt", f => Grow(Path.Combine(f, "testcore_libretro.so")));
            string stale = Copy("stale", f => Grow(LibretroEntries.AdapterPath(f)));
            var refusals = new[] { rebuilt, stale }.Select(folder => new DiscoveredCore(CoreSidecar.Read(CoreSidecar.PathFor(LibretroEntries.ProxyPath(folder)))!) { Player = true, Entry = folder }).ToArray();
            Assert.Null(refusals[0].Open());
            Assert.True(refusals[0].AwaitsApproval);
            Assert.Contains("is not the build that was approved", refusals[0].Report);
            Assert.Equal("needs_approval", refusals[0].Standing);
            Assert.Null(refusals[1].Open());
            Assert.False(refusals[1].AwaitsApproval);
            Assert.Contains("needs refresh", refusals[1].Report);
            Assert.Equal("needs_refresh", refusals[1].Standing);

            string unapproved = Copy("unapproved");
            LibretroEntryFixture.Revoke(entry.Sha256);
            try
            {
                var waiting = new DiscoveredCore(CoreSidecar.Read(CoreSidecar.PathFor(LibretroEntries.ProxyPath(unapproved)))!) { Player = true, Entry = unapproved };
                Assert.Null(waiting.Open());
                Assert.Contains("has not been approved", waiting.Report);
            }
            finally { LibretroEntryFixture.Approve(entry.Sha256); }
            Assert.All(new[] { rebuilt, stale, unapproved }, f => Assert.False(CoreLibrary.IsOpen(LibretroEntries.ProxyPath(f))));
        }

        // §10d.1: a core file changed after the host hashed it and before the proxy was loaded is refused by the proxy itself, through emusen_core_expect, and no runner starts.
        [Fact]
        public void A_core_changed_between_the_hosts_hash_and_the_load_never_runs()
        {
            string folder = Path.Combine(entry.Root, "Swapped", LibretroEntries.Folder, "testcore");
            LibretroEntryFixture.Lay(folder);
            string core = Path.Combine(folder, "testcore_libretro.so");
            string approved = CoreSidecar.HashOf(core);
            File.AppendAllText(core, "a rebuilt core");
            string actual = CoreSidecar.HashOf(core);
            int before = Runners();
            CoreLibrary library = CoreLibrary.Open(LibretroEntries.ProxyPath(folder), approved);
            Assert.False(library.Available);
            Assert.Contains("is not the build that was approved", library.Report);
            Assert.Contains(approved, library.Report);
            Assert.Contains(actual, library.Report);
            Assert.Equal(before, Runners());
            Assert.NotNull(entry.Found.Open());
            Assert.Contains("under another approval", CoreLibrary.Open(LibretroEntries.ProxyPath(entry.Entry), new string('0', 64)).Report);
        }

        // The runners this process started that are alive, read from /proc.
        private static int Runners()
        {
            string me = Environment.ProcessId.ToString();
            int count = 0;
            foreach (string dir in Directory.EnumerateDirectories("/proc"))
            {
                string stat;
                try { stat = File.ReadAllText(Path.Combine(dir, "stat")); } catch (IOException) { continue; } catch (UnauthorizedAccessException) { continue; }
                int open = stat.IndexOf('('), close = stat.LastIndexOf(')');
                if (open < 0 || close < open) continue;
                string[] rest = stat[(close + 2)..].Split(' ');
                if (stat[(open + 1)..close].StartsWith("emusen-core-run", StringComparison.Ordinal) && rest.Length > 1 && rest[1] == me) count++;
            }
            return count;
        }

        // A game's picture, its pad and its unaltered file's path reach the brokered core; a patched or unpacked image has no path to give.
        [Fact]
        public void A_game_runs_with_its_picture_its_pad_and_its_files_path()
        {
            using var engine = Engine();
            string game = Game();
            engine.LoadRom(game);
            engine.SetButton(0, PadButton.Start, true);
            for (int i = 0; i < 10; i++) engine.RunFrame();
            Assert.Equal((320, 10), (engine.Machine.FrameInfo.Width, (int)Ram(engine, FrameCounter)));
            Assert.Equal(1 << 3, Ram(engine, Pad));
            Assert.Contains(engine.RecentLog, l => l.Contains($"game path: {game}"));
        }

        // §4.5: the save folder's files stored by name under the console's save folder, passed back at create, written when changed, and a removed one's last copy kept.
        [Fact]
        public void Named_files_are_kept_by_name_passed_back_and_a_removed_ones_last_copy_kept()
        {
            using var engine = Engine(("tc_files", "on"));
            engine.Options = CoreOptions.Default;
            string game = Game("files.tcg");
            engine.LoadRom(game);
            string folder = SaveLibrary.NamedFilesFolder(engine.Console, consoleKind: true, engine.Info.Id);
            Directory.CreateDirectory(Path.Combine(folder, "kept"));
            File.WriteAllBytes(Path.Combine(folder, "kept", "earlier.bin"), new byte[] { 1, 2, 3 });
            engine.LoadRom(game);
            string data = Path.Combine(entry.Cores, LibretroEntries.Folder, ".data", "testcore", "save");
            Assert.Equal(new byte[] { 1, 2, 3 }, File.ReadAllBytes(Path.Combine(data, "kept", "earlier.bin")));
            for (int i = 0; i < 15; i++) engine.RunFrame();
            engine.SaveSram();
            Assert.Equal("first notes\n", File.ReadAllText(Path.Combine(folder, "testcore", "notes.txt")));
            Assert.Contains(engine.Machine.Info.Battery, b => b.Name == "kept/earlier.bin" && b.Which >= CoreInterface.FileNamed);
            for (int i = 0; i < 15; i++) engine.RunFrame();
            engine.SaveSram();
            Assert.False(File.Exists(Path.Combine(folder, "testcore", "notes.txt")));
            Assert.Equal("first notes\n", File.ReadAllText(Path.Combine(folder + SaveLibrary.RemovedSuffix, "testcore", "notes.txt")));
            Assert.Equal(new byte[] { 1, 2, 3 }, File.ReadAllBytes(Path.Combine(folder, "kept", "earlier.bin")));
        }

        // §3.4: a crash or an abort ends the core's process with -10 and its signal, and the host loads the game again.
        [Theory]
        [InlineData("segv", 11)]
        [InlineData("abort", 6)]
        public void A_crash_ends_the_core_and_not_the_host(string fault, int signal)
        {
            using var engine = Engine(("tc_fault", fault));
            engine.LoadRom(Game());
            engine.RunFrame();
            var ended = Assert.Throws<CoreRefusedException>(engine.RunFrame);
            Assert.Equal(-10, ended.Status);
            Assert.Contains($"signal {signal}", ended.Message);
            engine.LoadRom(Game());
            engine.RunFrame();
            Assert.Equal(1L, engine.TotalFrames);
        }

        [Fact]
        public void A_frame_that_never_ends_is_stopped_at_its_deadline()
        {
            using var engine = Engine(("tc_fault", "hang"), ("broker.deadline", "1"));
            engine.LoadRom(Game());
            engine.RunFrame();
            var clock = Stopwatch.StartNew();
            var stopped = Assert.Throws<CoreRefusedException>(engine.RunFrame);
            Assert.Equal(-11, stopped.Status);
            Assert.True(clock.Elapsed < TimeSpan.FromSeconds(5), clock.Elapsed.ToString());
            Assert.Contains("stopped answering within 1 s", stopped.Message);
        }

        // §3.3: Landlock refuses the core a write outside its folders and a network connection; the core reads its refusal back as 2.
        [Theory]
        [InlineData("write_outside")]
        [InlineData("socket")]
        public void The_kernel_refuses_a_write_outside_the_cores_folders_and_the_network(string fault)
        {
            using var engine = Engine(("tc_fault", fault));
            engine.LoadRom(Game());
            for (int i = 0; i < 3; i++) engine.RunFrame();
            Assert.Equal(2, Ram(engine, Fault));
        }

        // A core never runs unconfined: a kernel without the Landlock it needs fails the load in words.
        [Fact]
        public void A_kernel_that_cannot_confine_the_core_fails_the_load_in_words()
        {
            using var engine = Engine();
            LibretroEntryFixture.Environment("EMUSEN_BROKER_REQUIRE_ABI", "99");
            try
            {
                var refused = Assert.Throws<CoreRefusedException>(() => engine.LoadRom(Game()));
                Assert.Contains("never runs unconfined", refused.Message);
            }
            finally { LibretroEntryFixture.Environment("EMUSEN_BROKER_REQUIRE_ABI", null); }
        }
    }
}

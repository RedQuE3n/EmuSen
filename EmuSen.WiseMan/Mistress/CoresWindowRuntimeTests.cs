using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Cores;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Mistress.BigPicture.WindowFitAuditTests;

namespace EmuSen.WiseMan.Mistress
{
    // Cores and Mistress's question over the Rust runtime's own listing: an installed libretro core waiting for approval offered with its core's hash, approved through the frontend API and then an engine - see EmuSen_LibretroHost.md §10m.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class CoresWindowRuntimeTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session = HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CoresWindowRuntimeTests).GetTypeInfo().Assembly);
        private static readonly FieldInfo ConfirmField = typeof(CoresWindow).GetField("Confirm", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realConfirm = ConfirmField.GetValue(null)!;
        private readonly Func<string, IToolJob> _realStart = CoresPane.StartJob;
        private readonly Func<bool> _realAvailable = CoresPane.ToolAvailable;
        private readonly Func<string, (long Free, long Total)?> _realDisk = CoresPane.Disk;
        private readonly Func<CoresRuntime?, CoresRuntime?> _realRuntime = CoresPane.RuntimeFor;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCoresRuntime_" + Guid.NewGuid().ToString("N"));
        private string Cores => Path.Combine(_root, "Cores");
        private string Entry => Path.Combine(Cores, "libretro", "testcore");
        private string CoreFile => Path.Combine(Entry, "testcore_libretro.so");

        public CoresWindowRuntimeTests()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            Directory.CreateDirectory(Entry);
            File.Copy(CoreAdapterTests.LibraryPath("emusen_libretro_proxy"), Path.Combine(Entry, "emusen-core-broker.so"));
            File.Copy(CoreAdapterTests.LibraryPath("emusen_libretro_host"), Path.Combine(Entry, "emusen-libretro-host.so"));
            File.Copy(CoreAdapterTests.LibraryPath("emusen_libretro_testcore"), CoreFile);
            File.WriteAllText(Path.Combine(Entry, "testcore_libretro.info"), "display_name = \"EmuSen - Test Core\"\ncorename = \"TestCore\"\nlicense = \"GPLv3\"\nsystemid = \"\"\n");
            LibretroEntryFixture.Environment("EMUSEN_CORE_RUNNER", Path.Combine(AppContext.BaseDirectory, "emusen-core-runner"));
            CoresPane.ToolAvailable = () => true;
            CoresPane.Disk = _ => (410L << 30, 1024L << 30);
            CoresPane.StartJob = _ => new ScriptedJob([LibretroCore("testcore", "EmuSen - Test Core", "ps2", "installed", 1 << 20),
                new { @event = "done", op = "core-ls", counts = new System.Collections.Generic.Dictionary<string, int> { ["installed"] = 1 }, catalogue = "2026-11-01" }], ends: true);
        }

        public void Dispose()
        {
            CoresPane.StartJob = _realStart;
            CoresPane.ToolAvailable = _realAvailable;
            CoresPane.Disk = _realDisk;
            CoresPane.RuntimeFor = _realRuntime;
            ConfirmField.SetValue(null, _realConfirm);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private CrystalRuntime Runtime() => new(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { Cores } });

        // Mistress's own static, as the window calls it.
        private static object? Mistress(string method, params object[] args) =>
            typeof(MainWindow).GetMethod(method, BindingFlags.Static | BindingFlags.NonPublic)!.Invoke(null, args);

        private static string Sha256(string file) => Convert.ToHexStringLower(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(file)));

        [RustImplementationFact("the cores listing's libretro rows, 1.5")]
        public Task The_window_offers_a_waiting_libretro_core_with_its_cores_hash_and_its_approval_makes_it_an_engine() => Session.Dispatch(() =>
        {
            using CrystalRuntime runtime = Runtime();
            CoresPane.RuntimeFor = _ => (CoresRuntime)Mistress("CoresRuntimeOf", runtime)!;
            string? asked = null;
            ConfirmField.SetValue(null, (Func<Avalonia.Controls.Window, string, string, Task<bool>>)((_, _, message) => { asked = message; return Task.FromResult(true); }));
            var window = new CoresWindow(null, null) { Width = 1280, Height = 800 };
            window.Show();
            Settle(window);
            CoresPane pane = window.Pane;
            pane.Poll();
            CoreRow row = pane.Rows.Single(r => r.Stem == "testcore");
            Assert.Equal((Sha256(CoreFile), "needs_approval", (string?)null), (row.Sha256, row.Trust, row.Engine));
            pane.Table.Select(row);
            Settle(window);
            Assert.True(pane.Actions.First(a => a.Text == "Approve…").IsEnabled);
            Assert.True(pane.ApproveSelectedAsync().Result);
            Assert.Contains(Sha256(CoreFile), asked);
            pane.Refresh();
            pane.Poll();
            row = pane.Rows.Single(r => r.Stem == "testcore");
            Assert.Equal("approved", row.Trust);
            Assert.NotNull(row.Engine);
            window.Close();
        }, default);

        [RustImplementationFact("the cores listing's libretro rows, 1.5")]
        public void Mistress_asks_about_a_libretro_core_by_its_cores_file_and_hash()
        {
            using CrystalRuntime runtime = Runtime();
            runtime.CoreApprove(Sha256(CoreFile));
            runtime.CoreRevoke(Sha256(CoreFile));
            // A game of the test core's own system, which no native core runs.
            string game = Path.Combine(_root, "game.tcg");
            File.WriteAllBytes(game, "TCG1 a test game"u8.ToArray());
            object? core = Mistress("CoreAwaitingApproval", runtime, game);
            Assert.True(core is not null, runtime.Route(game));
            string Field(string name) => (string)core!.GetType().GetProperty(name)!.GetValue(core)!;
            Assert.Equal(("testcore_libretro.so", Sha256(CoreFile)), (Field("File"), Field("Sha256")));
            runtime.CoreApprove(Field("Sha256"));
            Assert.Null(Mistress("CoreAwaitingApproval", runtime, game));
        }
    }
}

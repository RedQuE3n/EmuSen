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
using Avalonia.VisualTree;
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
        private static readonly FieldInfo TellField = typeof(CoresWindow).GetField("Tell", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realTell = TellField.GetValue(null)!;
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
            File.Copy(CoreSidecars.LibraryPath("emusen_libretro_proxy"), Path.Combine(Entry, "emusen-core-broker.so"));
            File.Copy(CoreSidecars.LibraryPath("emusen_libretro_host"), Path.Combine(Entry, "emusen-libretro-host.so"));
            File.Copy(CoreSidecars.LibraryPath("emusen_libretro_testcore"), CoreFile);
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
            TellField.SetValue(null, _realTell);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private CrystalRuntime Runtime() => new(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { Cores } });

        // Mistress's own static, as the window calls it.
        private static object? Mistress(string method, params object[] args) =>
            typeof(MainWindow).GetMethod(method, BindingFlags.Static | BindingFlags.NonPublic)!.Invoke(null, args);

        private static string Sha256(string file) => Convert.ToHexStringLower(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(file)));

        [Fact]
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
            Assert.Contains(Sha256(CoreFile), CoresWindowTests.Ungrouped(asked));
            pane.Refresh();
            pane.Poll();
            row = pane.Rows.Single(r => r.Stem == "testcore");
            Assert.Equal("approved", row.Trust);
            Assert.NotNull(row.Engine);
            window.Close();
        }, default);

        // An entry as `cores get` lays one out: the copies, the core named on its stem, its information file, NOTICE and broker.target.
        private string Installed(string stem, string name)
        {
            string folder = Path.Combine(Cores, "libretro", stem);
            LibretroEntryFixture.Lay(folder, "", stem);
            string core = Path.Combine(folder, $"{stem}_libretro.so");
            File.WriteAllText(Path.Combine(folder, "NOTICE"), $"# Installed by EmuSen's Cores at the player's request; the core keeps its own licence.\nformat: 1\nstem: {stem}\nname: {name}\ncorename: {stem}\nauthors: Someone\nlicense: GPLv3\nsource: https://buildbot.libretro.com/nightly/linux/x86_64/latest/{stem}_libretro.so.zip\nbuild: 2026-11-01\nindex_crc32: d589c813\nsha256: {Sha256(core)}\nfetched: 2026-11-01\n");
            File.WriteAllText(Path.Combine(folder, "broker.target"), "emusen-libretro-host.so\n");
            return core;
        }

        // The handheld's state on 2026-11-01: PCEE2 and Flycast installed, neither approved, no approvals file; the big-screen form driven by the pad.
        [Fact]
        public Task The_big_screen_form_approves_an_installed_core_with_the_pad() => Session.Dispatch(() =>
        {
            Directory.Delete(Entry, recursive: true);
            string pcee2 = Installed("pcee2", "Sony - PlayStation 2 (PCEE2)");
            Installed("flycast", "Sega - Dreamcast/NAOMI (Flycast)");
            CoresPane.StartJob = _ => new ScriptedJob([LibretroCore("pcee2", "Sony - PlayStation 2 (PCEE2)", "ps2", "installed", 1 << 20), LibretroCore("flycast", "Sega - Dreamcast/NAOMI (Flycast)", "dc", "installed", 1 << 20),
                new { @event = "done", op = "core-ls", counts = new System.Collections.Generic.Dictionary<string, int> { ["installed"] = 2 }, catalogue = "2026-11-01" }], ends: true);
            using CrystalRuntime runtime = Runtime();
            CoresPane.RuntimeFor = _ => (CoresRuntime)Mistress("CoresRuntimeOf", runtime)!;
            string? asked = null;
            ConfirmField.SetValue(null, (Func<Avalonia.Controls.Window, string, string, Task<bool>>)((_, _, message) => { asked = message; return Task.FromResult(true); }));
            new EmuSen.Galaxia.Models.AppSettings { RomDirectory = Path.Combine(_root, "Roms"), BigScreen = true, LogDirectory = Path.Combine(_root, "Logs"), ResumeOnLaunch = EmuSen.Galaxia.Models.AppSettings.ResumeNever }.Save();
            Directory.CreateDirectory(Path.Combine(_root, "Roms"));
            var main = new MainWindow { Width = 1280, Height = 800 };
            main.Show();
            var pad = new PadDriver(main);
            Settle(main);
            typeof(MainWindow).GetMethod("ShowCores", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(main, null);
            Settle(main);
            var shown = (CoresWindow)Sheets(main).Current!;
            CoresPane pane = shown.Pane;
            pane.Poll();
            Settle(main);
            Assert.Equal(EmuSen.LunaP.Controls.AdaptiveForm.BigScreen, pane.Form);
            CoreRow row = pane.Rows.Single(r => r.Stem == "pcee2");
            Assert.Equal((Sha256(pcee2), "needs_approval"), (row.Sha256, row.Trust));
            pane.BigScreenTable.Select(row);
            pane.FocusRole(EmuSen.LunaP.Controls.AdaptiveRole.Content);
            Settle(main);
            // A on a core waiting for approval asks the question as the frame's card, with the hash in its words (EmuSen_Interface.md §15.5, §15.6).
            Assert.Equal("Approve…", pane.Hints[0].Label);
            pad.A();
            Settle(main);
            Assert.True(pane.IsPlanOpen, "A on the row asks the approval");
            Assert.StartsWith("Approve ", pane.QuestionTitle);
            Assert.Contains(Sha256(pcee2), CoresWindowTests.Ungrouped(pane.PlanText));
            Assert.True(pane.Confirm());
            Settle(main);
            Assert.Null(asked);
            Assert.Contains(Sha256(pcee2), File.ReadAllText(Path.Combine(_root, "Config", "cores.approved")));
            main.Close();
        }, default);

        // The tester's run on 2026-11-01: PCEE2 installed while Mistress ran, so the runtime's listing predated it; Approve on its page asked nothing until Cores listed the directories again. The real question, Dialogs.ConfirmAsync over the sheet-hosted window, answered by the pad.
        [Fact]
        public Task A_core_installed_while_Mistress_runs_is_approved_through_the_real_question_by_pad() => Session.Dispatch(() =>
        {
            Directory.Delete(Entry, recursive: true);
            using CrystalRuntime runtime = Runtime();
            string pcee2 = Installed("pcee2", "Sony - PlayStation 2 (PCEE2)");
            CoresPane.StartJob = _ => new ScriptedJob([LibretroCore("pcee2", "Sony - PlayStation 2 (PCEE2)", "ps2", "installed", 1 << 20),
                new { @event = "done", op = "core-ls", counts = new System.Collections.Generic.Dictionary<string, int> { ["installed"] = 1 }, catalogue = "2026-11-01" }], ends: true);
            CoresPane.RuntimeFor = _ => (CoresRuntime)Mistress("CoresRuntimeOf", runtime)!;
            new EmuSen.Galaxia.Models.AppSettings { RomDirectory = Path.Combine(_root, "Roms"), BigScreen = true, LogDirectory = Path.Combine(_root, "Logs"), ResumeOnLaunch = EmuSen.Galaxia.Models.AppSettings.ResumeNever }.Save();
            Directory.CreateDirectory(Path.Combine(_root, "Roms"));
            var main = new MainWindow { Width = 1280, Height = 800 };
            main.Show();
            var pad = new PadDriver(main);
            Settle(main);
            typeof(MainWindow).GetMethod("ShowCores", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(main, null);
            Settle(main);
            var shown = (CoresWindow)Sheets(main).Current!;
            CoresPane pane = shown.Pane;
            pane.Poll();
            Settle(main);
            CoreRow row = pane.Rows.Single(r => r.Stem == "pcee2");
            Assert.Equal((Sha256(pcee2), "needs_approval"), (row.Sha256, row.Trust));
            pane.BigScreenTable.Select(row);
            pane.FocusRole(EmuSen.LunaP.Controls.AdaptiveRole.Content);
            Settle(main);
            pad.A();
            Settle(main);
            Assert.True(pane.IsPlanOpen, "A on the row asks the approval as the frame's card");
            Assert.Same(shown, Sheets(main).Current);
            Assert.Contains(Sha256(pcee2), CoresWindowTests.Ungrouped(pane.PlanText));
            Assert.True(pane.Confirm());
            Settle(main);
            Assert.Same(shown, Sheets(main).Current);
            Assert.Contains(Sha256(pcee2), File.ReadAllText(Path.Combine(_root, "Config", "cores.approved")));
            main.Close();
        }, default);

        // A host that cannot list the directories again: the page's Approve asks with the hash the entry records, and the runtime's refusal is said, never dropped (EmuSen_Cores.md §3.5).
        [Fact]
        public Task Without_a_rescan_the_question_is_still_asked_and_the_refusal_said() => Session.Dispatch(() =>
        {
            Directory.Delete(Entry, recursive: true);
            using CrystalRuntime runtime = Runtime();
            string pcee2 = Installed("pcee2", "Sony - PlayStation 2 (PCEE2)");
            // The listing's row as the core tool reads it, with the hash its NOTICE records.
            CoresPane.StartJob = _ => new ScriptedJob([new { @event = "core", stem = "pcee2", name = "Sony - PlayStation 2 (PCEE2)", corename = "PCEE2", systems = new[] { "ps2" }, source = "libretro", version = "2.3.1", build = "2026-11-01", state = "installed", license = "GPLv3", authors = new[] { "Someone" }, size = 1L << 20, sha256 = Sha256(pcee2) },
                new { @event = "done", op = "core-ls", counts = new System.Collections.Generic.Dictionary<string, int> { ["installed"] = 1 }, catalogue = "2026-11-01" }], ends: true);
            var full = (CoresRuntime)Mistress("CoresRuntimeOf", runtime)!;
            CoresPane.RuntimeFor = _ => new CoresRuntime { Cores = full.Cores, Systems = full.Systems, Setting = full.Setting, SetSetting = full.SetSetting, Approve = full.Approve, Revoke = full.Revoke };
            string? asked = null, told = null;
            ConfirmField.SetValue(null, (Func<Avalonia.Controls.Window, string, string, Task<bool>>)((_, _, message) => { asked = message; return Task.FromResult(true); }));
            TellField.SetValue(null, (Func<Avalonia.Controls.Window, string, string, Task>)((_, _, message) => { told = message; return Task.CompletedTask; }));
            var window = new CoresWindow(null, null) { Width = 1280, Height = 800 };
            window.Show();
            Settle(window);
            CoresPane pane = window.Pane;
            pane.Poll();
            CoreRow row = pane.Rows.Single(r => r.Stem == "pcee2");
            Assert.Null(row.Sha256);
            pane.Table.Select(row);
            Settle(window);
            Assert.False(pane.ApproveSelectedAsync().Result);
            Assert.Contains(Sha256(pcee2), CoresWindowTests.Ungrouped(asked));
            Assert.StartsWith("PCEE2 could not be approved: no core library listed has the SHA-256", told);
            window.Close();
        }, default);

        // The tester's run on 2026-11-01: PCEE2 installed and waiting, Jak opened on Sapphire without a question. The route names the console's waiting default where no layer chose an engine, and Mistress asks about it first with the engine the game opens on otherwise (EmuSen_LibretroHost.md §10t).
        [Fact]
        public void The_consoles_waiting_default_is_asked_about_ahead_of_the_native_fallback()
        {
            Directory.Delete(Entry, recursive: true);
            string pcee2 = Installed("pcee2", "Sony - PlayStation 2 (PCEE2)");
            File.AppendAllText(Path.Combine(Cores, "libretro", "pcee2", "pcee2_libretro.info"), "supported_extensions = \"iso|chd|cso|zso|gz|bin|mdf|nrg|elf|irx\"\n");
            string game = SyntheticPs2Disc.Dvd().WriteSet(_root, "Amber Saga");
            using CrystalRuntime runtime = Runtime();
            using (var route = System.Text.Json.JsonDocument.Parse(runtime.Route(game)))
            {
                Assert.Equal("Sapphire", route.RootElement.GetProperty("engine").GetString());
                Assert.Equal(("pcee2", Sha256(pcee2)), (route.RootElement.GetProperty("waiting_default").GetProperty("stem").GetString(), route.RootElement.GetProperty("waiting_default").GetProperty("sha256").GetString()));
            }
            object? core = Mistress("CoreAwaitingApproval", runtime, game);
            Assert.True(core is not null, runtime.Route(game));
            string? Field(string name) => (string?)core!.GetType().GetProperty(name)!.GetValue(core);
            Assert.Equal(("pcee2_libretro.so", Sha256(pcee2), "Sony - PlayStation 2 (PCEE2)", "Sapphire"), (Field("File"), Field("Sha256"), Field("Name"), Field("Instead")));
            Assert.Equal("Sony - PlayStation 2 (PCEE2) waits for your approval, so this game opens on Sapphire.", Mistress("FallbackNotice", core!, "Sapphire"));
            runtime.SetSetting("PlayStation 2", CoresPane.EngineKey, "Sapphire");
            Assert.Contains("\"waiting_default\":null", runtime.Route(game));
            Assert.Null(Mistress("CoreAwaitingApproval", runtime, game));
        }

        // PCEE2 as the tester's home had it on 2026-11-01: installed by an earlier build, so its proxy copy is not this installation's, and taking the synthetic disc.
        private (string Core, string Game) EarlierBuildsPcee2()
        {
            Directory.Delete(Entry, recursive: true);
            string pcee2 = Installed("pcee2", "Sony - PlayStation 2 (PCEE2)");
            string folder = Path.GetDirectoryName(pcee2)!;
            string info = Path.Combine(folder, "pcee2_libretro.info");
            File.WriteAllText(info, File.ReadAllText(info).Replace("systemid = \"\"", "systemid = \"playstation2\"") + "supported_extensions = \"iso|chd|cso|zso|gz|bin|mdf|nrg|elf|irx\"\n");
            using (var proxy = new FileStream(Path.Combine(folder, "emusen-core-broker.so"), FileMode.Append)) proxy.Write("\0an earlier build"u8);
            return (pcee2, SyntheticPs2Disc.Dvd().WriteSet(_root, "Amber Saga"));
        }

        // The tester's run on 2026-11-01: PCEE2 approved in Cores mid-session, its copies an earlier build's, and Jak then opened on Sapphire with nothing said. The approval refreshes the copies and reads the entry, so the route gives the default (EmuSen_LibretroHost.md §10v).
        [Fact]
        public void The_default_approved_mid_session_over_an_earlier_builds_copies_is_the_routes_engine()
        {
            (string pcee2, string game) = EarlierBuildsPcee2();
            using CrystalRuntime runtime = Runtime();
            Assert.Contains("\"engine\":\"Sapphire\"", runtime.Route(game));
            runtime.CoreApprove(Sha256(pcee2));
            using var route = System.Text.Json.JsonDocument.Parse(runtime.Route(game));
            using var cores = System.Text.Json.JsonDocument.Parse(runtime.Cores());
            string? engine = cores.RootElement.EnumerateArray().Where(c => c.TryGetProperty("libretro", out var l) && l.ValueKind == System.Text.Json.JsonValueKind.Object && l.GetProperty("sha256").GetString() == Sha256(pcee2))
                .Select(c => c.GetProperty("engine").GetString()).SingleOrDefault();
            Assert.True(engine is not null, runtime.Cores());
            Assert.Equal(engine, route.RootElement.GetProperty("engine").GetString());
            Assert.Equal((System.Text.Json.JsonValueKind.Null, System.Text.Json.JsonValueKind.Null), (route.RootElement.GetProperty("waiting_default").ValueKind, route.RootElement.GetProperty("default_held").ValueKind));
            Assert.Equal(Sha256(CoreSidecars.LibraryPath("emusen_libretro_proxy")), Sha256(Path.Combine(Path.GetDirectoryName(pcee2)!, "emusen-core-broker.so")));
            Assert.Null(Mistress("CoreAwaitingApproval", runtime, game));
        }

        // An approved default whose copies cannot be refreshed: the game opens on the fallback with the reason said, never silently, and nothing is asked (EmuSen_LibretroHost.md §10v).
        [Fact]
        public void An_approved_default_that_cannot_be_read_gives_its_reason_and_no_silent_fallback()
        {
            (string pcee2, string game) = EarlierBuildsPcee2();
            string folder = Path.GetDirectoryName(pcee2)!;
            using CrystalRuntime runtime = Runtime();
            File.SetUnixFileMode(folder, UnixFileMode.UserRead | UnixFileMode.UserExecute);
            try { runtime.CoreApprove(Sha256(pcee2)); }
            finally { File.SetUnixFileMode(folder, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute); }
            using (var route = System.Text.Json.JsonDocument.Parse(runtime.Route(game)))
            {
                Assert.Equal("Sapphire", route.RootElement.GetProperty("engine").GetString());
                var held = route.RootElement.GetProperty("default_held");
                Assert.Equal(("pcee2", Sha256(pcee2)), (held.GetProperty("stem").GetString(), held.GetProperty("sha256").GetString()));
                Assert.Contains("needs refresh", held.GetProperty("reason").GetString());
            }
            object? core = Mistress("CoreAwaitingApproval", runtime, game);
            Assert.True(core is not null, runtime.Route(game));
            string? Field(string name) => (string?)core!.GetType().GetProperty(name)!.GetValue(core);
            Assert.Equal(("Sony - PlayStation 2 (PCEE2)", "Sapphire"), (Field("Name"), Field("Instead")));
            Assert.Contains("refreshing it failed", Field("Held"));
            Assert.StartsWith("Sony - PlayStation 2 (PCEE2) could not be used, so this game opens on Sapphire: ", (string)Mistress("HeldNotice", core!, "Sapphire", Field("Held")!)!);
        }

        [Fact]
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

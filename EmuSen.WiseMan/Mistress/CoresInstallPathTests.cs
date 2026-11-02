using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Reflection;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Cores;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.Scraping;
using static EmuSen.WiseMan.Mistress.BigPicture.WindowFitAuditTests;

namespace EmuSen.WiseMan.Mistress
{
    // The player's whole path from a home with no core: Cores, the List card's Check for Updates, PCEE2's Install, the approval, all by the pad, the real core tool fetching from a buildbot of the test's own; then a PlayStation 2 disc routes to PCEE2 - see EmuSen_Cores.md §3.6.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class CoresInstallPathTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session = HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CoresInstallPathTests).GetTypeInfo().Assembly);
        private readonly Func<string, IToolJob> _realStart = CoresPane.StartJob;
        private readonly Func<CoresRuntime?, CoresRuntime?> _realRuntime = CoresPane.RuntimeFor;
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenCoresInstall_" + Guid.NewGuid().ToString("N"));
        private readonly Buildbot _buildbot = new();
        private readonly ReplayServer _server;

        public CoresInstallPathTests()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            LibretroEntryFixture.Environment("EMUSEN_CORE_RUNNER", Path.Combine(AppContext.BaseDirectory, "emusen-core-runner"));
            _server = new ReplayServer(_buildbot);
            // The real job, its request carrying the test's server as the buildbot's address; nothing else differs from the player's.
            CoresPane.StartJob = request => LibraryToolJob.Start(Served(request));
        }

        public void Dispose()
        {
            CoresPane.StartJob = _realStart;
            CoresPane.RuntimeFor = _realRuntime;
            _server.Dispose();
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Served(string request)
        {
            JsonObject r = JsonNode.Parse(request)!.AsObject();
            r["test"] = new JsonObject { ["connect"] = _server.Address, ["roots"] = ReplayServer.RootPem };
            return r.ToJsonString();
        }

        // libretro's three files for one core: the information archive, this platform's index and the core's zip.
        private sealed class Buildbot : HttpMessageHandler
        {
            public readonly Dictionary<string, byte[]> Files = new();
            public readonly List<string> Asked = new();

            protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancel)
            {
                string path = request.RequestUri!.AbsolutePath;
                lock (Asked) Asked.Add(path);
                return Task.FromResult(Files.TryGetValue(path, out byte[]? body)
                    ? new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(body) }
                    : new HttpResponseMessage(HttpStatusCode.NotFound) { Content = new ByteArrayContent([]) });
            }
        }

        private static byte[] Zip(string name, byte[] bytes)
        {
            using var memory = new MemoryStream();
            using (var zip = new ZipArchive(memory, ZipArchiveMode.Create, leaveOpen: true))
            using (Stream entry = zip.CreateEntry(name).Open()) entry.Write(bytes);
            return memory.ToArray();
        }

        private static uint Crc32(byte[] bytes)
        {
            uint crc = 0xFFFFFFFF;
            foreach (byte b in bytes)
            {
                crc ^= b;
                for (int k = 0; k < 8; k++) crc = (crc >> 1) ^ (0xEDB88320 & (uint)-(int)(crc & 1));
            }
            return ~crc;
        }

        private const string Info = "display_name = \"Sony - PlayStation 2 (PCEE2)\"\ncorename = \"PCEE2\"\nauthors = \"PCSX2 Team|libretro\"\nlicense = \"GPLv3\"\ndisplay_version = \"v2.7.x\"\nsystemid = \"playstation2\"\ndatabase = \"Sony - PlayStation 2\"\nsupported_extensions = \"iso|chd|cue\"\n";

        // The synthetic core served as PCEE2's nightly build, its index line naming the CRC-32 its zip records.
        private byte[] Serve()
        {
            byte[] core = File.ReadAllBytes(CoreAdapterTests.LibraryPath("emusen_libretro_testcore"));
            _buildbot.Files["/assets/frontend/info.zip"] = Zip("pcee2_libretro.info", Encoding.ASCII.GetBytes(Info));
            _buildbot.Files["/nightly/linux/x86_64/latest/.index-extended"] = Encoding.ASCII.GetBytes($"2026-11-02 {Crc32(core):x8} pcee2_libretro.so.zip\n");
            _buildbot.Files["/nightly/linux/x86_64/latest/pcee2_libretro.so.zip"] = Zip("pcee2_libretro.so", core);
            return core;
        }

        private static void Wait(MainWindow main, CoresPane pane)
        {
            for (int i = 0; i < 1200 && pane.Busy; i++)
            {
                Thread.Sleep(25);
                pane.Poll();
                Settle(main);
            }
            Assert.False(pane.Busy, "the core tool's job ended");
            Settle(main);
        }

        private static IInputElement? Focused(Window w) => TopLevel.GetTopLevel(w)?.FocusManager?.GetFocusedElement();

        private static string Said(IInputElement? e) => e switch
        {
            ListBoxItem { DataContext: CoreRow row } => "row " + row.Name,
            ContentControl { Content: string s } => s,
            Control c when Avalonia.Automation.AutomationProperties.GetName(c) is { Length: > 0 } n => n,
            _ => e?.GetType().Name ?? "nothing",
        };

        private static bool Within(IInputElement? e, Visual v) => e is Visual at && (ReferenceEquals(at, v) || at.GetVisualAncestors().Contains(v));

        // Down on the pad until the focus is on what the test wants, every step said; false when it never gets there.
        private static bool DownTo(PadDriver pad, Window main, Func<IInputElement?, bool> there, List<string> path, int most = 8, Func<IInputElement?, bool>? stays = null)
        {
            for (int i = 0; i < most; i++)
            {
                if (there(Focused(main))) return true;
                pad.Down();
                Settle(main);
                path.Add(Said(Focused(main)));
                if (stays is not null) Assert.True(stays(Focused(main)), $"the focus left the card for {path[^1]} ({string.Join(" > ", path)})");
            }
            return there(Focused(main));
        }

        [RustImplementationFact("the cores listing's libretro rows, 1.5")]
        public Task A_fresh_home_installs_PCEE2_by_the_pad_from_the_buildbot_and_a_PlayStation_2_disc_routes_to_it() => Session.Dispatch(() =>
        {
            byte[] core = Serve();
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(Path.Combine(roms, "ps2"));
            string jak = Path.Combine(roms, "ps2", "Jak.iso");
            File.WriteAllBytes(jak, SyntheticPs2Disc.Dvd().Iso);
            using var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { DataStore.Cores } });
            CoresPane.RuntimeFor = _ => (CoresRuntime)typeof(MainWindow).GetMethod("CoresRuntimeOf", BindingFlags.Static | BindingFlags.NonPublic)!.Invoke(null, [runtime])!;
            new EmuSen.Galaxia.Models.AppSettings { RomDirectory = roms, BigScreen = true, LogDirectory = Path.Combine(_root, "Logs"), ResumeOnLaunch = EmuSen.Galaxia.Models.AppSettings.ResumeNever }.Save();
            var main = new MainWindow { Width = 1280, Height = 800 };
            main.Show();
            var pad = new PadDriver(main);
            Settle(main);
            typeof(MainWindow).GetMethod("ShowCores", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(main, null);
            Settle(main);
            var shown = (CoresWindow)Sheets(main).Current!;
            CoresPane pane = shown.Pane;
            Wait(main, pane);
            Assert.Equal(EmuSen.LunaP.Controls.AdaptiveForm.BigScreen, pane.Form);
            Assert.DoesNotContain(pane.Rows, r => r.Stem == "pcee2");
            Assert.StartsWith("No catalogue of libretro's cores yet", pane.Notice);
            var path = new List<string>();

            // Select opens the List card; the D-pad stays on it down to Check for Updates, whose A reads the catalogue.
            pad.Select();
            Settle(main);
            Assert.True(pane.IsListToolsOpen);
            Assert.True(DownTo(pad, main, e => Said(e) == "Check for Updates", path, stays: e => Within(e, pane.ListCard)), string.Join(" > ", path));
            pad.A();
            Settle(main);
            Wait(main, pane);
            Assert.StartsWith("Read libretro's catalogue on ", pane.Notice);
            // Only the systems table's console names, never a raw id such as Endou's gg; sections and rows in the table's order, each console's default first (EmuSen_Interface.md §15.3).
            Assert.DoesNotContain(pane.Rows.SelectMany(r => r.Consoles), c => c is "gg" or "naomi" or "atomiswave");
            Assert.DoesNotContain(pane.Sections, s => s.Key is "console:gg");
            var ranks = pane.Sections.Skip(1).Select(s => s.Key["console:".Length..]).ToList();
            Assert.Equal(pane.Rows.Where(r => r.Consoles.Count > 0).Select(r => r.Consoles.Min(c => ranks.IndexOf(c))), pane.Rows.Where(r => r.Consoles.Count > 0).Select(r => r.Consoles.Min(c => ranks.IndexOf(c))).Order());
            Assert.Contains("/assets/frontend/info.zip", _buildbot.Asked);

            // The PlayStation 2 on the shoulders, then PCEE2's row by the D-pad.
            for (int i = 0; i < pane.Sections.Count && pane.SelectedSection != "console:PlayStation 2"; i++) { pad.R1(); Settle(main); }
            Assert.Equal("console:PlayStation 2", pane.SelectedSection);
            path.Clear();
            Assert.True(DownTo(pad, main, e => Said(e) == "row PCEE2", path), "PCEE2's row is reached by the D-pad: " + string.Join(" > ", path));
            Assert.Equal("Install…", pane.Hints[0].Label);

            // A asks the plan; its answers are reached by the D-pad and A on Apply runs the real download.
            pad.A();
            Settle(main);
            Assert.True(pane.IsPlanOpen);
            path.Clear();
            Assert.True(DownTo(pad, main, e => Said(e) is "Apply" or "Yes", path, stays: Uncovered), string.Join(" > ", path));
            pad.A();
            Settle(main);
            Wait(main, pane);
            string installed = Path.Combine(DataStore.Cores, "libretro", "pcee2", "pcee2_libretro.so");
            Assert.True(File.Exists(installed), pane.Notice);
            Assert.Equal(core, File.ReadAllBytes(installed));
            Assert.StartsWith("Installed pcee2.", pane.Notice);
            Assert.Equal("row PCEE2", Said(Focused(main)));
            CoreRow row = pane.Rows.Single(r => r.Stem == "pcee2");
            Assert.True(row.AwaitsApproval);

            // The question follows on A, answered Yes by the pad; the approval is the runtime's and the disc then routes to PCEE2.
            Assert.Equal("Approve…", pane.Hints[0].Label);
            pad.A();
            Settle(main);
            Assert.True(pane.IsPlanOpen);
            Assert.StartsWith("Approve ", pane.QuestionTitle);
            path.Clear();
            Assert.True(DownTo(pad, main, e => Said(e) is "Yes" or "Approve", path, stays: Uncovered), string.Join(" > ", path));
            pad.A();
            Settle(main);
            Wait(main, pane);
            string sha = Convert.ToHexStringLower(System.Security.Cryptography.SHA256.HashData(core));
            Assert.Contains(sha, File.ReadAllText(Path.Combine(_root, "Config", "cores.approved")));
            row = pane.Rows.Single(r => r.Stem == "pcee2");
            Assert.Equal("approved", row.Trust);
            Assert.Null(pane.OutcomeOf(row));
            Assert.Equal("row PCEE2", Said(Focused(main)));
            Assert.Equal("PCEE2", pane.Shown.First().Name);
            using JsonDocument routed = JsonDocument.Parse(runtime.Route(jak));
            Assert.Equal(row.Engine, routed.RootElement.GetProperty("engine").GetString());
            main.Close();
        }, default);

        // A buildbot that answers nothing: Check for Updates by the pad says why on the screen and in the error log, never silently (EmuSen_Cores.md §3.6).
        [RustImplementationFact("the cores listing's libretro rows, 1.5")]
        public Task A_failed_fetch_is_said_on_the_screen_and_in_the_error_log() => Session.Dispatch(() =>
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            new EmuSen.Galaxia.Models.AppSettings { RomDirectory = roms, BigScreen = true, LogDirectory = Path.Combine(_root, "Logs"), ResumeOnLaunch = EmuSen.Galaxia.Models.AppSettings.ResumeNever }.Save();
            var main = new MainWindow { Width = 1280, Height = 800 };
            main.Show();
            var pad = new PadDriver(main);
            Settle(main);
            typeof(MainWindow).GetMethod("ShowCores", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(main, null);
            Settle(main);
            CoresPane pane = ((CoresWindow)Sheets(main).Current!).Pane;
            Wait(main, pane);
            var path = new List<string>();
            pad.Select();
            Settle(main);
            Assert.True(DownTo(pad, main, e => Said(e) == "Check for Updates", path, stays: e => Within(e, pane.ListCard)), string.Join(" > ", path));
            pad.A();
            Settle(main);
            Wait(main, pane);
            Assert.Contains("/assets/frontend/info.zip", _buildbot.Asked);
            Assert.Contains("404", pane.Notice);
            string log = ErrorLog.PathFor(DateTime.Now);
            Assert.True(File.Exists(log), log);
            Assert.Contains(pane.Notice!, File.ReadAllText(log));
            main.Close();
        }, default);

        private static bool Uncovered(IInputElement? e) => e is Control c && !EmuSen.LunaP.Windowing.SheetLayer.GetIsCovered(c);
    }
}

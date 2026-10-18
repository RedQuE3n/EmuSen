using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // VenusRT in Mistress through discovery alone: the SNES's only engine since VenusRT_Native.md §68, a game played on the generic adapter with rewind, a stored Venus (C#) read as VenusRT, and a build without the library refusing in words.
    [Collection(TestCollections.ProcessGlobals)]
    public class VenusRtEngineTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(VenusRtEngineTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenVenusRtEngineTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public VenusRtEngineTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            Directory.CreateDirectory(DataStore.Firmware);
            _rom = Path.Combine(roms, "Adapter.sfc");
            File.WriteAllBytes(_rom, VenusRtCoreAbiTests.PadToBackdropRom());
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDirectories(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static bool Found => CoreDiscovery.Found.Any(c => c.Info.Id == "venusrt");

        // The SNES tab has no Engine row since the C# core was retired.
        [Fact]
        public Task The_SNES_tab_has_no_engine_row() => Session.Dispatch(() =>
        {
            if (!Found) return;
            var window = new GraphicsSettingsWindow(new GraphicsConfig(), _ => { }, "SNES");
            window.Show();
            Assert.Empty(window.GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == $"SNES.{CoreCatalog.EngineKey}"));
            window.Close();
        }, default);

        // A fresh config runs an .sfc on VenusRT on the adapter with rewind, and a stored Venus (C#) runs VenusRT with no notice and one line on the log.
        [Fact]
        public Task A_game_runs_on_venusrt_and_a_stored_venus_choice_runs_it_too() => Session.Dispatch(() =>
        {
            if (!Found) return;
            Assert.Null(ConsolesConfig.Load().Value("SNES", CoreCatalog.EngineKey));
            Assert.Null(CoreFactory.ConfiguredEngine(_rom));
            foreach (bool stored in new[] { false, true })
            {
                if (stored)
                {
                    ConsolesConfig config = ConsolesConfig.Load();
                    config.SetValue("SNES", CoreCatalog.EngineKey, "Venus (C#)");
                    config.Save();
                    Assert.Equal(VenusRtCoreAbiTests.Engine, CoreFactory.ConfiguredEngine(_rom));
                }
                MainWindow window = Start();
                WindowGame game = Game(window);
                var core = Assert.IsType<CoreEngine>(game.Core);
                Assert.Equal("venusrt", core.Info.Id);
                Assert.Null(game.EngineNotice);
                WaitFor(() => game.TotalFrames > 60 && Rewind(window).Depth > 0);
                window.Close();
            }
        }, default);

        // Without VenusRT's library an SNES game does not load, and the status line says why.
        [Fact]
        public Task Without_the_library_an_snes_game_does_not_load_and_the_status_says_why() => Session.Dispatch(() =>
        {
            if (!Found) return;
            CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "NoCores") });
            var window = new MainWindow();
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!.Invoke(window, new object[] { _rom, "Adapter.sfc" });
            Assert.Null(typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window));
            Assert.Equal("Failed to load Adapter.sfc: This build has no SNES core: VenusRT (Rust) is missing (no sidecar for it was found beside the program).",
                window.FindControl<TextBlock>("StatusText")!.Text);
            window.Close();
        }, default);

        // An empty firmware folder: neither an ordinary game nor a DSP-1 cartridge asks for anything, and the cartridge runs on the replacement with no notice (VenusRT_Native.md §66).
        [Fact]
        public Task An_ordinary_game_on_venusrt_prompts_for_no_firmware() => Session.Dispatch(async () =>
        {
            if (!Found) return;
            GraphicsConfig config = GraphicsConfig.Load();
            Assert.Empty(Directory.GetFiles(DataStore.Firmware));
            string dsp = Path.Combine(_root, "Roms", "Pilot.sfc");
            File.WriteAllBytes(dsp, SyntheticRom.BuildNecDsp("PILOTWINGS"));
            var window = new MainWindow();
            window.Show();
            TextBlock status = window.FindControl<TextBlock>("StatusText")!;
            MethodInfo prompt = typeof(MainWindow).GetMethod("PromptForMissingFirmwareAsync", BindingFlags.Instance | BindingFlags.NonPublic)!;
            string? before = status.Text;
            await (Task)prompt.Invoke(window, new object[] { _rom })!;
            Assert.Equal(before, status.Text);
            await (Task)prompt.Invoke(window, new object[] { dsp })!;
            Assert.Equal(before, status.Text);
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!.Invoke(window, new object[] { dsp, "Pilot.sfc" });
            WaitFor(() => Game(window).TotalFrames > 20);
            Assert.StartsWith("Running: ", status.Text);
            Assert.DoesNotContain("replacement", status.Text);
            window.Close();
        }, default);

        private MainWindow Start()
        {
            var window = new MainWindow();
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!.Invoke(window, new object[] { _rom, "Adapter.sfc" });
            WaitFor(() => Game(window).TotalFrames > 20);
            return window;
        }

        // Pumps the UI thread's queue, since the emulation thread reports back by posting to it.
        private static void WaitFor(Func<bool> condition)
        {
            var clock = Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out waiting for the emulation thread");
                Thread.Sleep(2);
            }
        }

        private static WindowGame Game(MainWindow window) => new(window);

        private static RewindBuffer Rewind(MainWindow window) => MistressGame.Rewind(window);
    }
}

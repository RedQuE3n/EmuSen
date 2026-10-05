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
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // VenusRT in Mistress through discovery alone: the SNES tab's engine row, a game played on the generic adapter with rewind, and VenusRT the default since VenusRT_Native.md §65.
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

        [Fact]
        public Task The_SNES_tab_offers_venusrt_and_a_choice_is_stored_for_the_next_game() => Session.Dispatch(() =>
        {
            if (!Found) return;
            var config = new GraphicsConfig();
            string? told = null;
            var window = new GraphicsSettingsWindow(config, console => told = console, "SNES");
            window.Show();
            Dropdown engine = window.GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == $"SNES.{CoreCatalog.EngineKey}").Distinct().Single();
            Assert.Equal(new[] { VenusRtCoreAbiTests.Engine, CoreCatalog.VenusEngine }, engine.Items.Cast<object>().Select(o => o.ToString()));
            Assert.Equal(VenusRtCoreAbiTests.Engine, engine.SelectedItem);

            engine.SelectedItem = CoreCatalog.VenusEngine;
            Dispatcher.UIThread.RunJobs();
            Assert.Equal("SNES", told);
            Assert.Equal(CoreCatalog.VenusEngine, GraphicsConfig.Load().Value("SNES", CoreCatalog.EngineKey));
            window.Close();
        }, default);

        // A fresh config runs an .sfc on VenusRT on the adapter with rewind, and a stored Venus (C#) choice is honoured.
        [Fact]
        public Task A_game_runs_on_venusrt_by_default_and_on_venus_once_venus_is_chosen() => Session.Dispatch(() =>
        {
            if (!Found) return;
            Assert.Null(GraphicsConfig.Load().Value("SNES", CoreCatalog.EngineKey));
            Assert.Equal(VenusRtCoreAbiTests.Engine, CoreFactory.ConfiguredEngine(_rom));
            MainWindow window = Start();
            EmulatorSession game = Game(window);
            var core = Assert.IsType<CoreEngine>(game.Core);
            Assert.Equal("venusrt", core.Info.Id);
            Assert.Null(game.EngineNotice);
            WaitFor(() => game.TotalFrames > 60 && Rewind(window).Depth > 0);
            window.Close();

            GraphicsConfig config = GraphicsConfig.Load();
            config.SetValue("SNES", CoreCatalog.EngineKey, CoreCatalog.VenusEngine);
            config.Save();
            Assert.Equal(CoreCatalog.VenusEngine, CoreFactory.ConfiguredEngine(_rom));
            window = Start();
            Assert.IsType<VenusCore>(Game(window).Core);
            Assert.Null(Game(window).EngineNotice);
            window.Close();
        }, default);

        // An empty firmware folder and VenusRT chosen: neither an ordinary game nor a DSP-1 cartridge asks for anything, and the cartridge runs on the replacement with no notice (VenusRT_Native.md §66).
        [Fact]
        public Task An_ordinary_game_on_venusrt_prompts_for_no_firmware() => Session.Dispatch(async () =>
        {
            if (!Found) return;
            GraphicsConfig config = GraphicsConfig.Load();
            config.SetValue("SNES", CoreCatalog.EngineKey, VenusRtCoreAbiTests.Engine);
            config.Save();
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

        // A state's record names the engine that wrote it, and a state another SNES engine wrote is offered to the running one, not called a newer build's (EmuSen_Galaxia.md §5.3b).
        [Fact]
        public Task A_states_record_names_its_engine_and_another_engines_version_is_not_compared() => Session.Dispatch(() =>
        {
            if (!Found) return;
            GraphicsConfig config = GraphicsConfig.Load();
            config.SetValue("SNES", CoreCatalog.EngineKey, CoreCatalog.VenusEngine);
            config.Save();
            MainWindow window = Start();
            EmulatorSession venus = Game(window);
            string statePath = Path.Combine(_root, "venus.state");
            File.WriteAllBytes(statePath, new byte[] { 1, 2, 3 });
            typeof(MainWindow).GetMethod("WriteStateRecord", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { venus, statePath, _rom });
            var records = (EmuSen.Mistress.Library.FileRecords)typeof(MainWindow).GetField("_fileRecords", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            StateRecord written = records.ReadState(statePath)!;
            Assert.Equal(CoreCatalog.VenusEngine, written.Core);
            MethodInfo refusal = typeof(MainWindow).GetMethod("Refusal", BindingFlags.Static | BindingFlags.NonPublic)!;
            StateRecord fromVenusRt = written with { Core = VenusRtCoreAbiTests.Engine, StateVersion = 18 };
            Assert.Null(refusal.Invoke(null, new object?[] { fromVenusRt, _rom, venus }));
            Assert.StartsWith("That state was saved by a newer build", (string)refusal.Invoke(null, new object?[] { written with { StateVersion = 99 }, _rom, venus })!);
            Assert.StartsWith("That state was saved by a newer build", (string)refusal.Invoke(null, new object?[] { written with { Core = "SNES", StateVersion = 18 }, _rom, venus })!);
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

        private static EmulatorSession Game(MainWindow window) =>
            (EmulatorSession)typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static RewindBuffer Rewind(MainWindow window) =>
            (RewindBuffer)typeof(MainWindow).GetField("_rewind", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
    }
}

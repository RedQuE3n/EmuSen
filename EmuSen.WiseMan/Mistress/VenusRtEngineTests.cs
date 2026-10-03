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
    // VenusRT in Mistress through discovery alone: the SNES tab's engine row, a game played on the generic adapter with rewind, and Venus (C#) the default - see VenusRT_Native.md §33.
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
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, "spc700.rom"), VenusRtTestRomRunnerTests.IdleIpl());
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
            Assert.Equal(new[] { CoreCatalog.VenusEngine, VenusRtCoreAbiTests.Engine }, engine.Items.Cast<object>().Select(o => o.ToString()));
            Assert.Equal(CoreCatalog.VenusEngine, engine.SelectedItem);

            engine.SelectedItem = VenusRtCoreAbiTests.Engine;
            Dispatcher.UIThread.RunJobs();
            Assert.Equal("SNES", told);
            Assert.Equal(VenusRtCoreAbiTests.Engine, GraphicsConfig.Load().Value("SNES", CoreCatalog.EngineKey));
            window.Close();
        }, default);

        [Fact]
        public Task A_game_runs_on_venus_until_venusrt_is_chosen_and_then_on_the_adapter_with_rewind() => Session.Dispatch(() =>
        {
            if (!Found) return;
            MainWindow window = Start();
            Assert.IsType<VenusCore>(Game(window).Core);
            window.Close();

            GraphicsConfig config = GraphicsConfig.Load();
            config.SetValue("SNES", CoreCatalog.EngineKey, VenusRtCoreAbiTests.Engine);
            config.Save();
            window = Start();
            EmulatorSession game = Game(window);
            var core = Assert.IsType<CoreEngine>(game.Core);
            Assert.Equal("venusrt", core.Info.Id);
            Assert.Null(game.EngineNotice);
            WaitFor(() => game.TotalFrames > 60 && Rewind(window).Depth > 0);
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

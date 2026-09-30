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
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // MoonRT chosen in the graphics window and played in Mistress: frames, rewind, a state the C# Moon reads, and the fallback - see Moon_Native.md §8.3.
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonRtEngineTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MoonRtEngineTests).GetTypeInfo().Assembly);

        public const string FallbackMarkVariable = "EMUSEN_MOONRT_FALLBACK_MARK";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMoonRtEngineTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public MoonRtEngineTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "Program.nes");
            File.WriteAllBytes(_rom, BoardPrograms.Build(1, 8, cycleIrq: false));
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
            CoreOptions.BatteryRamDisabled = false;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static void Choose(string engine)
        {
            GraphicsConfig config = GraphicsConfig.Load();
            config.SetValue("NES", CoreCatalog.EngineKey, engine);
            config.Save();
        }

        [Fact]
        public Task The_NES_tab_offers_the_engine_with_Moon_the_default_and_a_choice_is_stored() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            string? told = null;
            var window = new GraphicsSettingsWindow(config, console => told = console, "NES");
            window.Show();

            Dropdown engine = window.GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == $"NES.{CoreCatalog.EngineKey}").Distinct().Single();
            Assert.Equal(new[] { CoreCatalog.MoonEngine, CoreCatalog.MoonRtEngine }, engine.Items.Cast<object>().Select(o => o.ToString()));
            Assert.Equal(CoreCatalog.MoonEngine, engine.SelectedItem);

            engine.SelectedItem = CoreCatalog.MoonRtEngine;
            Dispatcher.UIThread.RunJobs();

            Assert.Equal("NES", told);
            Assert.Equal(CoreCatalog.MoonRtEngine, GraphicsConfig.Load().Value("NES", CoreCatalog.EngineKey));
            window.Close();
        }, default);

        [Fact]
        public Task A_game_runs_on_Moon_until_MoonRT_is_chosen_and_then_on_MoonRT_with_rewind_and_its_states_Moons() => Session.Dispatch(() =>
        {
            MainWindow window = Start();
            Assert.IsType<MoonCore>(Game(window).Core);
            window.Close();

            Assert.True(MoonRtCore.Available, MoonNative.Report);
            Choose(CoreCatalog.MoonRtEngine);
            window = Start();
            EmulatorSession game = Game(window);
            Assert.IsType<MoonRtCore>(game.Core);
            Assert.Null(game.EngineNotice);
            Assert.DoesNotContain("not available", Status(window).Text ?? "");
            WaitFor(() => game.TotalFrames > 90 && Rewind(window).Depth > 0);

            Status(window).Text = "";
            Invoke(window, "SaveState");
            WaitFor(() => Status(window).Text?.StartsWith("State saved") == true);
            var moon = new MoonCore();
            moon.LoadRom(_rom);
            moon.LoadState(StatePath(window));
            Assert.True(moon.TotalFrames > 60);
            window.Close();
        }, default);

        // The library is loaded once per process, so a process started with it off is the only honest test of the fallback - see Mars_Native.md §5.5.
        [Fact]
        public void With_the_library_turned_off_a_game_asking_for_MoonRT_runs_on_Moon_and_says_why()
        {
            string mark = Path.Combine(_root, "fallback.txt");
            var start = new ProcessStartInfo("dotnet")
            {
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
            };
            foreach (string argument in new[] { "test", typeof(MoonRtEngineTests).Assembly.Location, "--filter", $"FullyQualifiedName={typeof(MoonRtEngineTests).FullName}.{nameof(TheFallbackInAProcessWithTheLibraryOff)}" })
                start.ArgumentList.Add(argument);
            start.Environment[MoonNative.Variable] = "0";
            start.Environment[FallbackMarkVariable] = mark;

            using Process child = Process.Start(start)!;
            Task<string> output = child.StandardOutput.ReadToEndAsync();
            Task<string> errors = child.StandardError.ReadToEndAsync();
            Assert.True(child.WaitForExit(240_000), "the child test run did not finish");
            Assert.True(child.ExitCode == 0, output.Result + errors.Result);
            Assert.True(File.Exists(mark), "the child ran nothing:\n" + output.Result);

            string[] found = File.ReadAllLines(mark);
            Assert.Equal(nameof(MoonCore), found[0]);
            Assert.Contains("turned off by EMUSEN_MOON_NATIVE=0", found[1]);
            Assert.Contains(CoreCatalog.MoonRtEngine + " is not available", found[2]);
        }

        // Run only by the test above, in its own process; it asserts nothing unless that process has the library off.
        [Fact]
        public Task TheFallbackInAProcessWithTheLibraryOff() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable(FallbackMarkVariable) is not { } mark || Environment.GetEnvironmentVariable(MoonNative.Variable) != "0") return;

            Assert.False(MoonNative.Available);
            Assert.False(MoonRtCore.Available);
            Choose(CoreCatalog.MoonRtEngine);
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "Program.nes");
            string? status = Status(window).Text;
            EmulatorSession game = Game(window);
            WaitFor(() => game.TotalFrames > 20);

            Assert.IsType<MoonCore>(game.Core);
            Assert.Contains("turned off by EMUSEN_MOON_NATIVE=0", game.EngineNotice);
            Assert.Contains(game.EngineNotice!, status);
            File.WriteAllLines(mark, new[] { game.Core!.GetType().Name, MoonNative.Report, game.EngineNotice! });
            window.Close();
        }, default);

        private MainWindow Start()
        {
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "Program.nes");
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

        private static EmuSen.Common.RewindBuffer Rewind(MainWindow window) =>
            (EmuSen.Common.RewindBuffer)typeof(MainWindow).GetField("_rewind", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static string StatePath(MainWindow window) =>
            (string)typeof(MainWindow).GetProperty("CurrentStatePath", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}

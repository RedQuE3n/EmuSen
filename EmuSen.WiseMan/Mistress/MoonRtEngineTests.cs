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
    // MoonRT, the NES's one engine, played in Mistress: frames, rewind and states, a stored Moon (C#) running there quietly, and a build without the library - see Moon_Native.md §9.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonRtEngineTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MoonRtEngineTests).GetTypeInfo().Assembly);

        public const string FallbackMarkVariable = "EMUSEN_MOONRT_FALLBACK_MARK";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMoonRtEngineTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public MoonRtEngineTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "Program.nes");
            File.WriteAllBytes(_rom, BoardPrograms.Build(1, 8, cycleIrq: false));
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static void Choose(string engine)
        {
            ConsolesConfig config = ConsolesConfig.Load();
            config.SetValue("NES", CoreCatalog.EngineKey, engine);
            config.Save();
        }

        // The NES has one engine, so its tab offers no choice of engine.
        [Fact]
        public Task The_NES_tab_offers_no_engine_choice() => Session.Dispatch(() =>
        {
            var window = new GraphicsSettingsWindow(new GraphicsConfig(), _ => { }, "NES");
            window.Show();
            Assert.Empty(window.GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == $"NES.{CoreCatalog.EngineKey}"));
            window.Close();
        }, default);

        // A stored Moon (C#) runs the game on MoonRT with no notice; frames, rewind and a state saved and loaded all work there.
        [Fact]
        public Task A_stored_csharp_choice_plays_on_MoonRT_with_rewind_and_its_states() => Session.Dispatch(() =>
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            Choose("Moon (C#)");
            MainWindow window = Start();
            EmulatorSession game = Game(window);
            Assert.IsType<MoonRtCore>(game.Core);
            Assert.Null(game.EngineNotice);
            Assert.DoesNotContain("not available", Status(window).Text ?? "");
            WaitFor(() => game.TotalFrames > 90 && Rewind(window).Depth > 0);

            Status(window).Text = "";
            Invoke(window, "SaveState");
            WaitFor(() => Status(window).Text?.StartsWith("State saved") == true);
            using var reader = new MoonRtCore();
            reader.LoadRom(_rom);
            reader.LoadState(StatePath(window));
            Assert.True(reader.TotalFrames > 60);
            window.Close();
        }, default);

        // The library is loaded once per process, so a process started with it off is the only honest test of the fallback - see Mars_Native.md §5.5.
        [Fact]
        public void With_the_library_turned_off_an_nes_game_does_not_load_and_the_status_says_why()
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
            Assert.Equal("no core", found[0]);
            Assert.Contains("turned off by EMUSEN_MOON_NATIVE=0", found[1]);
            Assert.Contains("This build has no NES core: " + CoreCatalog.MoonRtEngine + " is missing (turned off by EMUSEN_MOON_NATIVE=0)", found[2]);
        }

        // Run only by the test above, in its own process; it asserts nothing unless that process has the library off.
        [Fact]
        public Task TheFallbackInAProcessWithTheLibraryOff() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable(FallbackMarkVariable) is not { } mark || Environment.GetEnvironmentVariable(MoonNative.Variable) != "0") return;

            Assert.False(MoonNative.Available);
            Assert.False(MoonRtCore.Available);
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "Program.nes");
            string status = Status(window).Text ?? "";

            Assert.Null(typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window));
            Assert.StartsWith("Failed to load Program.nes: ", status);
            File.WriteAllLines(mark, new[] { "no core", MoonNative.Report, status });
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

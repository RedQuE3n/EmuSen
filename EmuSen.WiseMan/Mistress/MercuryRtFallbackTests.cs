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
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // MercuryRT chosen in the graphics window with its library turned off: Mistress runs Mercury (C#) and says why - see Mercury_Native.md §8.7.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryRtFallbackTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MercuryRtFallbackTests).GetTypeInfo().Assembly);

        public const string FallbackMarkVariable = "EMUSEN_MERCURYRT_FALLBACK_MARK";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMercuryRtFallbackTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public MercuryRtFallbackTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "Loop.gb");
            File.WriteAllBytes(_rom, SyntheticGbRom.Build(patches: (0, new byte[] { 0x18, 0xFE })));
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
            config.SetValue("GB", CoreCatalog.EngineKey, engine);
            config.Save();
        }

        // The library is loaded once per process, so a process started with it off is the only honest test of the fallback - see Mars_Native.md §5.5.
        [Fact]
        public void With_the_library_turned_off_a_game_asking_for_MercuryRT_runs_on_Mercury_and_says_why()
        {
            string mark = Path.Combine(_root, "fallback.txt");
            var start = new ProcessStartInfo("dotnet")
            {
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
            };
            foreach (string argument in new[] { "test", typeof(MercuryRtFallbackTests).Assembly.Location, "--filter", $"FullyQualifiedName={typeof(MercuryRtFallbackTests).FullName}.{nameof(TheFallbackInAProcessWithTheLibraryOff)}" })
                start.ArgumentList.Add(argument);
            start.Environment[MercuryNative.Variable] = "0";
            start.Environment[FallbackMarkVariable] = mark;

            using Process child = Process.Start(start)!;
            Task<string> output = child.StandardOutput.ReadToEndAsync();
            Task<string> errors = child.StandardError.ReadToEndAsync();
            Assert.True(child.WaitForExit(240_000), "the child test run did not finish");
            Assert.True(child.ExitCode == 0, output.Result + errors.Result);
            Assert.True(File.Exists(mark), "the child ran nothing:\n" + output.Result);

            string[] found = File.ReadAllLines(mark);
            Assert.Equal(nameof(MercuryCore), found[0]);
            Assert.Contains("turned off by EMUSEN_MERCURY_NATIVE=0", found[1]);
            Assert.Contains(CoreCatalog.MercuryRtEngine + " is not available", found[2]);
        }

        // Run only by the test above, in its own process; it asserts nothing unless that process has the library off.
        [Fact]
        public Task TheFallbackInAProcessWithTheLibraryOff() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable(FallbackMarkVariable) is not { } mark || Environment.GetEnvironmentVariable(MercuryNative.Variable) != "0") return;

            Assert.False(MercuryNative.Available);
            Assert.False(MercuryRtCore.Available);
            Choose(CoreCatalog.MercuryRtEngine);
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "Loop.gb");
            string? status = Status(window).Text;
            EmulatorSession game = Game(window);
            WaitFor(() => game.TotalFrames > 20);

            Assert.IsType<MercuryCore>(game.Core);
            Assert.Contains("turned off by EMUSEN_MERCURY_NATIVE=0", game.EngineNotice);
            Assert.Contains(game.EngineNotice!, status);
            File.WriteAllLines(mark, new[] { game.Core!.GetType().Name, MercuryNative.Report, game.EngineNotice! });
            window.Close();
        }, default);

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

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}

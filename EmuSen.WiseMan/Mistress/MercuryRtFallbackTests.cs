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
    // A build whose MercuryRT library is missing or turned off: Mistress loads no Game Boy game and says plainly why - see Mercury_Native.md §15.4.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryRtFallbackTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MercuryRtFallbackTests).GetTypeInfo().Assembly);

        public const string FallbackMarkVariable = "EMUSEN_MERCURYRT_FALLBACK_MARK";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMercuryRtFallbackTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public MercuryRtFallbackTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "Loop.gb");
            File.WriteAllBytes(_rom, SyntheticGbRom.Build(patches: (0, new byte[] { 0x18, 0xFE })));
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // The library is loaded once per process, so a process started with it off is the only honest test of the fallback - see Mars_Native.md §5.5.
        [Fact]
        public void With_the_library_turned_off_a_game_boy_game_does_not_load_and_the_status_says_why()
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
            Assert.Equal("no core", found[0]);
            Assert.Contains("turned off by EMUSEN_MERCURY_NATIVE=0", found[1]);
            Assert.Contains("This build has no Game Boy core: " + CoreCatalog.MercuryRtEngine + " is missing (turned off by EMUSEN_MERCURY_NATIVE=0)", found[2]);
        }

        // Run only by the test above, in its own process; it asserts nothing unless that process has the library off.
        [Fact]
        public Task TheFallbackInAProcessWithTheLibraryOff() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable(FallbackMarkVariable) is not { } mark || Environment.GetEnvironmentVariable(MercuryNative.Variable) != "0") return;

            Assert.False(MercuryNative.Available);
            Assert.False(MercuryRtCore.Available);
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "Loop.gb");
            string status = Status(window).Text ?? "";

            Assert.Null(typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window));
            Assert.StartsWith("Failed to load Loop.gb: ", status);
            File.WriteAllLines(mark, new[] { "no core", MercuryNative.Report, status });
            window.Close();
        }, default);

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}

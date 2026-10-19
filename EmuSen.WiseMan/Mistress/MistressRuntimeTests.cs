using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Crystal.Client;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Mistress on either implementation of the frontend API: its game log names the one serving it, and its DianaSh console refuses the C# shell over the Rust runtime in words - see EmuSen_Backend.md §29.29.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class MistressRuntimeTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MistressRuntimeTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMistressRuntime", Guid.NewGuid().ToString("N"));
        private readonly string _logs;
        private readonly string _rom;

        public MistressRuntimeTests()
        {
            string roms = Path.Combine(_root, "Roms");
            _logs = Path.Combine(_root, "Logs");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States"), LogDirectory = _logs }.Save();
            // The boot stub, then a loop at $00:8005 the CPU runs for ever.
            _rom = Path.Combine(roms, "Loop.sfc");
            File.WriteAllBytes(_rom, SyntheticRom.Build((5, new byte[] { 0x4C, 0x05, 0x80 })));
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static void Pump(Func<bool> until, int ms = 10_000)
        {
            var clock = Stopwatch.StartNew();
            while (!until() && clock.ElapsedMilliseconds < ms)
            {
                Dispatcher.UIThread.RunJobs();
                Thread.Sleep(5);
            }
        }

        private void Load(MainWindow window) =>
            typeof(MainWindow).GetMethod("LoadRom", Hidden, null, new[] { typeof(string), typeof(string) }, null)!.Invoke(window, new object[] { _rom, "Loop.sfc" });

        private MainWindow Started()
        {
            var window = new MainWindow().WithoutBattery();
            window.Show();
            Load(window);
            Assert.NotNull(MistressGame.Live(window));
            return window;
        }

        private static DianaShConsoleWindow Console(MainWindow window)
        {
            typeof(MainWindow).GetMethod("ShowShellConsole", Hidden)!.Invoke(window, null);
            return window.OwnedWindows.OfType<DianaShConsoleWindow>().Single();
        }

        private static string Output(DianaShConsoleWindow console) =>
            ((ConsolePane)typeof(DianaShConsoleWindow).GetField("_console", Hidden)!.GetValue(console)!).OutputText;

        private static void Type(DianaShConsoleWindow console, string line) =>
            typeof(DianaShConsoleWindow).GetMethod("Submit", Hidden)!.Invoke(console, new object[] { line });

        private static string RefusalLine => $"[ERROR] {DianaShInterpreter.ForeignTableRefusal}";

        [Fact]
        public Task A_game_s_log_names_the_implementation_that_serves_it() => Session.Dispatch(() =>
        {
            MainWindow window = Started();
            window.Close();

            string log = Directory.GetFiles(_logs, "general.log", SearchOption.AllDirectories).Single();
            Assert.Contains($"[ROM] Runtime: {CrystalRuntime.Implementation}", File.ReadAllLines(log));
        }, default);

        // Run at each pairing of EMUSEN_CRYSTAL_NATIVE and EMUSEN_DIANASH_NATIVE: only the C# shell over the Rust runtime is refused.
        [Fact]
        public Task The_console_over_a_running_game_refuses_only_the_csharp_shell_over_the_rust_runtime_in_words() => Session.Dispatch(() =>
        {
            MainWindow window = Started();
            DianaShConsoleWindow console = Console(window);
            bool refused = CrystalRuntime.Native && !DianaShNative.Active;

            if (refused)
            {
                Assert.Contains(RefusalLine, Output(console).Split('\n'));
                Type(console, "pause");
                Pump(() => window.IsPaused);
                Assert.True(window.IsPaused, "the window's own commands stopped working with the refusal");

                Load(window);
                Assert.Equal(2, Output(console).Split('\n').Count(l => l == RefusalLine));
            }
            else
            {
                Assert.DoesNotContain("[ERROR]", Output(console));
                TextBlock status = window.GetControl<TextBlock>("StatusText");
                Type(console, "bp add 008005");
                Pump(() => status.Text?.Contains("BREAKPOINT", StringComparison.Ordinal) == true);
                Assert.Contains("BREAKPOINT", status.Text);
            }
            window.Close();
        }, default);
    }
}

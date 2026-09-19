using System;
using System.IO;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The state hotkeys against a game that keeps running: every state is one instant of the machine - see EmuSen_Settings_Reference.md §4.21a.
    [Collection(TestCollections.ProcessGlobals)]
    public class MainWindowSaveStateThreadTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MainWindowSaveStateThreadTests).GetTypeInfo().Assembly);

        private const int Zero = 0, A1 = 5, T0 = 8;

        // Where the program stores its counter after every increment.
        private const uint Counter = 0x3F0;

        private readonly string _root;
        private readonly string _romPath;

        public MainWindowSaveStateThreadTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenSaveStateThreadTests", Guid.NewGuid().ToString("N"));
            string romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Data");
            new AppSettings { RomDirectory = romDir, LogDirectory = Path.Combine(_root, "Logs"), StateDirectory = Path.Combine(_root, "States") }.Save();

            _romPath = Path.Combine(romDir, "Counter.z64");
            File.WriteAllBytes(_romPath, SyntheticN64Rom.BuildRunningFromRdram(new MipsAssembler()
                .Lui(A1, 0x8000)
                .Addiu(T0, T0, 1)
                .Sw(T0, A1, (short)Counter)
                .Beq(Zero, Zero, -3)
                .Nop()
                .ToArray()));
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // Saved from the UI thread while the emulation thread ran, the registers and the memory came from different frames.
        [Fact]
        public Task A_state_saved_by_the_hotkey_while_the_game_runs_is_one_instant_of_it() => Session.Dispatch(() =>
        {
            MainWindow window = Start();

            for (int i = 0; i < 12; i++)
            {
                SaveThroughTheHotkey(window);
                AssertOneInstant(Load(StatePath(window)));
                Thread.Sleep(7);
            }

            window.Close();
        }, default);

        // A paused emulation thread is woken for the request, and runs no frame while it serves it.
        [Fact]
        public Task A_state_saved_while_paused_is_written_without_the_game_advancing() => Session.Dispatch(() =>
        {
            MainWindow window = Start();
            window.PauseEmulation();
            long frames = WaitUntilParked(window);

            SaveThroughTheHotkey(window);

            Assert.Equal(frames, Game(window).TotalFrames);
            AssertOneInstant(Load(StatePath(window)));

            window.Close();
        }, default);

        [Fact]
        public Task A_state_loaded_by_the_hotkey_while_the_game_runs_takes_the_machine_back_to_it() => Session.Dispatch(() =>
        {
            MainWindow window = Start();
            SaveThroughTheHotkey(window);
            long saved = Load(StatePath(window)).TotalFrames;

            WaitFor(() => Game(window).TotalFrames > saved + 60);

            Status(window).Text = "";
            Invoke(window, "LoadState");
            WaitFor(() => Status(window).Text?.StartsWith("State loaded") == true);
            window.PauseEmulation();
            WaitUntilParked(window);

            Assert.InRange(Game(window).TotalFrames, saved, saved + 60);
            AssertOneInstant((MarsCore)Game(window).Core!);

            window.Close();
        }, default);

        private MainWindow Start()
        {
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _romPath, "Counter.z64");
            WaitFor(() => Game(window).TotalFrames > 20);
            return window;
        }

        private static void SaveThroughTheHotkey(MainWindow window)
        {
            Status(window).Text = "";
            Invoke(window, "SaveState");
            WaitFor(() => Status(window).Text?.StartsWith("State saved") == true);
        }

        private MarsCore Load(string path)
        {
            var core = new MarsCore();
            core.LoadRom(_romPath);
            core.LoadState(path);
            return core;
        }

        // Between the add and the store the stored copy is one behind; any other difference is two instants in one state.
        private static void AssertOneInstant(MarsCore core)
        {
            uint counter = (uint)core.Cpu!.Gpr[T0];
            uint stored = core.Bus!.Read32(Counter);
            Assert.True(stored == counter || stored + 1 == counter, $"the register holds {counter} and memory {stored}: the state is torn");
        }

        private static long WaitUntilParked(MainWindow window)
        {
            long frames;
            do
            {
                frames = Game(window).TotalFrames;
                Thread.Sleep(250);
            }
            while (Game(window).TotalFrames != frames);

            return frames;
        }

        // Pumps the UI thread's queue, since the emulation thread reports back by posting to it.
        private static void WaitFor(Func<bool> condition)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out waiting for the emulation thread");
                Thread.Sleep(2);
            }
        }

        private static EmulatorSession Game(MainWindow window) =>
            (EmulatorSession)typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static string StatePath(MainWindow window) =>
            (string)typeof(MainWindow).GetProperty("CurrentStatePath", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}

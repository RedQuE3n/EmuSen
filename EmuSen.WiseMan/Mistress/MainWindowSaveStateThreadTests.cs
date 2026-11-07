using EmuSen.WiseMan.Bindings.Systems.N64;
using System;
using System.IO;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Crystal;
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
                AssertOneInstant(Read(StatePath(window)));
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
            AssertOneInstant(Read(StatePath(window)));

            window.Close();
        }, default);

        [Fact]
        public Task A_state_loaded_by_the_hotkey_while_the_game_runs_takes_the_machine_back_to_it() => Session.Dispatch(() =>
        {
            MainWindow window = Start();
            SaveThroughTheHotkey(window);
            long saved;
            saved = Read(StatePath(window)).Frames;

            WaitFor(() => Game(window).TotalFrames > saved + 60);

            Status(window).Text = "";
            Invoke(window, "LoadState");
            WaitFor(() => Status(window).Text?.StartsWith("State loaded") == true);
            window.PauseEmulation();
            WaitUntilParked(window);

            Assert.InRange(Game(window).TotalFrames, saved, saved + 60);
            // The parked machine saved through the session's own call and read back by a session of a runtime of the test's, outside the window's.
            string parked = Path.Combine(_root, "parked.state");
            MistressGame.Of(window).Crystal.SaveState(parked);
            AssertOneInstant(Read(parked));

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

        // A state read back by a session of a runtime of the test's: the counter register, the copy the program stored, and the frames.
        private (uint Counter, uint Stored, long Frames) Read(string path)
        {
            using var runtime = new EmuSen.Crystal.Client.CrystalRuntime();
            using var session = runtime.Open(new EmuSen.Crystal.Client.CrystalOpen { Path = _romPath, NoBattery = true });
            session.LoadState(path);
            using var registers = System.Text.Json.JsonDocument.Parse(session.Telemetry(EmuSen.Crystal.Client.Contract.CrystalSessionAbi.TelemetryCpu));
            uint counter = (uint)registers.RootElement.EnumerateArray().First(r => r.GetProperty("name").GetString() == "t0").GetProperty("value").GetUInt64();
            byte[] word = new byte[4];
            session.ReadSpace("CPU", 0x80000000 | Counter, word);
            using var info = System.Text.Json.JsonDocument.Parse(session.Info());
            return (counter, System.Buffers.Binary.BinaryPrimitives.ReadUInt32BigEndian(word), info.RootElement.GetProperty("total_frames").GetInt64());
        }

        // Between the add and the store the stored copy is one behind; any other difference is two instants in one state.
        private static void AssertOneInstant((uint Counter, uint Stored, long Frames) state)
        {
            Assert.True(state.Stored == state.Counter || state.Stored + 1 == state.Counter, $"the register holds {state.Counter} and memory {state.Stored}: the state is torn");
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

        private static WindowGame Game(MainWindow window) => new(window);

        private static string StatePath(MainWindow window) =>
            (string)typeof(MainWindow).GetProperty("CurrentStatePath", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}

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
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // MarsRT chosen in the graphics window and played in Mistress: frames, states, the battery save, cheats, rewind, and the fallback - see EmuSen_Settings_Reference.md §4.44.
    [Collection(TestCollections.ProcessGlobals)]
    public class MarsRtEngineTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MarsRtEngineTests).GetTypeInfo().Assembly);

        public const string FallbackMarkVariable = "EMUSEN_MARSRT_FALLBACK_MARK";

        // Block 5 of the 4 Kbit EEPROM, which the synthetic system never writes; it writes block 2.
        private const int Marked = 5 * 8, Written = 2 * 8;
        private static readonly byte[] Marker = "MARSRT!!"u8.ToArray();

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMarsRtEngineTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public MarsRtEngineTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "System.z64");
            File.WriteAllBytes(_rom, SyntheticN64System.Build(rsp: true));
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();

            // This collection runs alone, so the process-wide switch can be held off here and put back.
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
            config.SetValue("N64", CoreCatalog.EngineKey, engine);
            config.Save();
        }

        [Fact]
        public Task The_N64_tab_offers_the_engine_and_a_choice_is_stored_for_the_next_game() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            string? told = null;
            var window = new GraphicsSettingsWindow(config, console => told = console, "N64");
            window.Show();

            Dropdown engine = window.GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == $"N64.{CoreCatalog.EngineKey}").Distinct().Single();
            Assert.Equal(new[] { CoreCatalog.MarsEngine, CoreCatalog.MarsRtEngine }, engine.Items.Cast<object>().Select(o => o.ToString()));
            Assert.Equal(CoreCatalog.MarsEngine, engine.SelectedItem);
            Assert.DoesNotContain(window.GetLogicalDescendants().OfType<Control>(), c => c.Name is string name && name.EndsWith("." + CoreCatalog.EngineKey) && name != $"N64.{CoreCatalog.EngineKey}" && name != $"GB.{CoreCatalog.EngineKey}");

            engine.SelectedItem = CoreCatalog.MarsRtEngine;
            Dispatcher.UIThread.RunJobs();

            Assert.Equal("N64", told);
            Assert.Equal(CoreCatalog.MarsRtEngine, GraphicsConfig.Load().Value("N64", CoreCatalog.EngineKey));
            window.Close();
        }, default);

        [Fact]
        public Task A_game_runs_on_Mars_until_MarsRT_is_chosen_and_then_on_MarsRT_with_rewind_off() => Session.Dispatch(() =>
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            MainWindow window = Start();
            Assert.IsType<MarsCore>(Game(window).Core);
            window.Close();

            Choose(CoreCatalog.MarsRtEngine);
            window = Start();
            EmulatorSession game = Game(window);
            Assert.IsType<MarsRtCore>(game.Core);
            Assert.Null(game.EngineNotice);
            Assert.DoesNotContain("not available", Status(window).Text ?? "");

            long? serial = game.FrameSerial;
            WaitFor(() => game.TotalFrames > 60 && game.FrameSerial != serial);
            Assert.Equal(0, Rewind(window).Depth);
            Assert.Equal("true", ((ICoreSettings)game.Core!).Get("ExpansionPak"));
            window.Close();
        }, default);

        [Fact]
        public Task A_state_saved_and_loaded_by_the_hotkeys_on_MarsRT_is_a_Mars_state() => Session.Dispatch(() =>
        {
            Choose(CoreCatalog.MarsRtEngine);
            MainWindow window = Start();
            EmulatorSession game = Game(window);

            Status(window).Text = "";
            Invoke(window, "SaveState");
            WaitFor(() => Status(window).Text?.StartsWith("State saved") == true);
            string state = StatePath(window);

            var mars = new MarsCore(expansionPak: true, batteryRamDisabled: true);
            mars.LoadRom(_rom);
            mars.LoadState(state);
            long saved = mars.TotalFrames;
            Assert.True(saved > 20);

            WaitFor(() => game.TotalFrames > saved + 60);
            Status(window).Text = "";
            Invoke(window, "LoadState");
            WaitFor(() => Status(window).Text?.StartsWith("State loaded") == true);
            window.PauseEmulation();
            WaitUntilParked(window);

            Assert.IsType<MarsRtCore>(game.Core);
            Assert.InRange(game.TotalFrames, saved, saved + 60);
            window.Close();
        }, default);

        // The mark is in a block the game never writes, so it is in the file afterwards only if the start read it - see Mars_Save.md §7.
        [Fact]
        public Task A_battery_save_is_read_when_MarsRT_starts_a_game_and_written_when_it_stops() => Session.Dispatch(() =>
        {
            Choose(CoreCatalog.MarsRtEngine);
            string save = SaveLibrary.SramPathFor(_rom);
            byte[] original = Enumerable.Repeat((byte)0xFF, EmuSen.Cores.Nintendo.Mars.Memory.Eeprom.Size).ToArray();
            Marker.CopyTo(original, Marked);
            Directory.CreateDirectory(Path.GetDirectoryName(save)!);
            File.WriteAllBytes(save, original);

            MainWindow window = Start();
            WaitFor(() => Game(window).TotalFrames > 40);
            Invoke(window, "ShowLibrary");

            byte[] file = File.ReadAllBytes(save);
            Assert.Equal(Marker, file[Marked..(Marked + 8)]);
            Assert.NotEqual(original[Written..(Written + 8)], file[Written..(Written + 8)]);

            var mars = new MarsCore(batteryRamDisabled: false);
            mars.LoadRom(_rom);
            Assert.Equal(file, mars.Bus!.Save.Contents);
            window.Close();
        }, default);

        [Fact]
        public Task A_cheat_imported_from_a_cht_file_reaches_MarsRT_at_the_frame_s_end() => Session.Dispatch(() =>
        {
            Choose(CoreCatalog.MarsRtEngine);
            MainWindow window = Start();
            EmulatorSession game = Game(window);
            string cht = Path.Combine(_root, "System.cht");
            File.WriteAllText(cht, "cheats = 1\n\ncheat0_desc = \"Poke\"\ncheat0_code = \"80000300 0042\"\ncheat0_enable = false\n");

            var cheats = (CheatRegistry)typeof(MainWindow).GetField("_cheats", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            Assert.Same(cheats, game.DebugTarget!.Cheats);
            CheatImport.FromChtFile(cheats, cht, game.CheatAutoDetectCodec, replace: true);
            cheats.SetEnabled(cheats.GetCheats().Single().Id, true);

            var core = (MarsRtCore)game.Core!;
            WaitFor(() => core.Peek(MarsRtSpace.Rdram, 0x300) == 0x42);
            window.Close();
        }, default);

        // The library is loaded once per process, so a process started with it off is the only honest test of the fallback - see Mars_Native.md §5.5.
        [Fact]
        public void With_the_library_turned_off_a_game_asking_for_MarsRT_runs_on_Mars_and_says_why()
        {
            string mark = Path.Combine(_root, "fallback.txt");
            var start = new ProcessStartInfo("dotnet")
            {
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
            };
            foreach (string argument in new[] { "test", typeof(MarsRtEngineTests).Assembly.Location, "--filter", $"FullyQualifiedName={typeof(MarsRtEngineTests).FullName}.{nameof(TheFallbackInAProcessWithTheLibraryOff)}" })
                start.ArgumentList.Add(argument);
            start.Environment[MarsNative.Variable] = "0";
            start.Environment[FallbackMarkVariable] = mark;

            using Process child = Process.Start(start)!;
            Task<string> output = child.StandardOutput.ReadToEndAsync();
            Task<string> errors = child.StandardError.ReadToEndAsync();
            Assert.True(child.WaitForExit(240_000), "the child test run did not finish");
            Assert.True(child.ExitCode == 0, output.Result + errors.Result);
            Assert.True(File.Exists(mark), "the child ran nothing:\n" + output.Result);

            string[] found = File.ReadAllLines(mark);
            Assert.Equal(nameof(MarsCore), found[0]);
            Assert.Contains("turned off by EMUSEN_MARS_NATIVE=0", found[1]);
            Assert.Contains(CoreCatalog.MarsRtEngine + " is not available", found[2]);
        }

        // Run only by the test above, in its own process; it asserts nothing unless that process has the library off.
        [Fact]
        public Task TheFallbackInAProcessWithTheLibraryOff() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable(FallbackMarkVariable) is not { } mark || Environment.GetEnvironmentVariable(MarsNative.Variable) != "0") return;

            Assert.False(MarsNative.Available);
            Assert.False(MarsRtCore.Available);
            Choose(CoreCatalog.MarsRtEngine);
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "System.z64");
            string? status = Status(window).Text;
            EmulatorSession game = Game(window);
            WaitFor(() => game.TotalFrames > 20);

            Assert.IsType<MarsCore>(game.Core);
            Assert.Contains("turned off by EMUSEN_MARS_NATIVE=0", game.EngineNotice);
            Assert.Contains(game.EngineNotice!, status);
            File.WriteAllLines(mark, new[] { game.Core!.GetType().Name, MarsNative.Report, game.EngineNotice! });
            window.Close();
        }, default);

        private MainWindow Start()
        {
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "System.z64");
            WaitFor(() => Game(window).TotalFrames > 20);
            return window;
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

        private static string StatePath(MainWindow window) =>
            (string)typeof(MainWindow).GetProperty("CurrentStatePath", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}

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
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Crystal.Cheats;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // MarsRT, the N64's one engine, played in Mistress: frames, states, the battery save, cheats, rewind, and a build without its library - see EmuSen_Settings_Reference.md §4.44.
    [Collection(TestCollections.ProcessGlobals)]
    public class MarsRtEngineTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MarsRtEngineTests).GetTypeInfo().Assembly);

        public const string FallbackMarkVariable = "EMUSEN_MARSRT_FALLBACK_MARK";

        // Block 5 of the 4 Kbit EEPROM, which the synthetic system never writes; it writes block 2.
        private const int Marked = 5 * 8, Written = 2 * 8;

        // The largest EEPROM a cartridge carries, 16 kilobits - see Mars_Save.md §2.
        private const int EepromSize = 0x800;
        private static readonly byte[] Marker = "MARSRT!!"u8.ToArray();

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMarsRtEngineTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public MarsRtEngineTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "System.z64");
            File.WriteAllBytes(_rom, SyntheticN64System.Build(rsp: true));
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs") }.Save();

            // This collection runs alone, so the process-wide switch can be held off here and put back.
        }

        public void Dispose()
        {
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

        // The N64 has one engine since Mars (C#) was retired, so its tab offers no choice of one - see Mars_Native.md §6.7.
        [Fact]
        public Task The_N64_tab_offers_no_engine() => Session.Dispatch(() =>
        {
            var window = new GraphicsSettingsWindow(new GraphicsConfig(), _ => { }, "N64");
            window.Show();

            Assert.DoesNotContain(window.GetLogicalDescendants().OfType<Control>(), c => c.Name == $"N64.{CoreCatalog.EngineKey}");
            Assert.DoesNotContain(window.GetLogicalDescendants().OfType<Control>(), c => c.Name is string name && name.EndsWith("." + CoreCatalog.EngineKey) && name != $"GB.{CoreCatalog.EngineKey}" && name != $"NES.{CoreCatalog.EngineKey}" && name != $"SNES.{CoreCatalog.EngineKey}");
            window.Close();
        }, default);

        [Fact]
        public Task A_game_runs_on_MarsRT_with_rewind_and_still_does_when_the_retired_Mars_is_stored() => Session.Dispatch(() =>
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            MainWindow window = Start();
            EmulatorSession game = Game(window);
            Assert.IsType<MarsRtCore>(game.Core);
            Assert.Null(game.EngineNotice);
            Assert.DoesNotContain("not available", Status(window).Text ?? "");

            long? serial = game.FrameSerial;
            WaitFor(() => game.TotalFrames > 60 && game.FrameSerial != serial);
            Assert.True(Rewind(window).Depth > 0, "MarsRT's frames left no rewind history");
            Assert.Equal("true", ((ICoreSettings)game.Core!).Get("ExpansionPak"));
            window.Close();

            Choose("Mars (C#)");
            window = Start();
            Assert.IsType<MarsRtCore>(Game(window).Core);
            Assert.Null(Game(window).EngineNotice);
            WaitFor(() => Game(window).TotalFrames > 60 && Rewind(window).Depth > 0);
            window.Close();
        }, default);

        // The hotkey held on MarsRT with its workers and deferred picture: the game goes back, then plays on when let go - see Mars_Native.md §6.6.3.
        [Fact]
        public Task Holding_rewind_on_MarsRT_steps_the_game_back_and_letting_go_plays_it_on() => Session.Dispatch(() =>
        {
            Choose(CoreCatalog.MarsRtEngine);
            MainWindow window = Start();
            EmulatorSession game = Game(window);
            WaitFor(() => game.TotalFrames > 120 && Rewind(window).Depth > 20);
            var core = Assert.IsType<MarsRtCore>(game.Core);
            Assert.True(core.ThreadedRdp && core.DeferredPresentation, "Mistress did not give MarsRT its workers and the deferred picture");

            long reached = game.TotalFrames;
            FieldInfo held = typeof(MainWindow).GetField("_rewindHeld", BindingFlags.Instance | BindingFlags.NonPublic)!;
            held.SetValue(window, true);
            WaitFor(() => game.TotalFrames < reached - 40);
            held.SetValue(window, false);
            long back = game.TotalFrames;
            WaitFor(() => game.TotalFrames > back + 60);
            window.Close();
        }, default);

        [Fact]
        public Task A_state_saved_and_loaded_by_the_hotkeys_on_MarsRT_takes_the_game_back_to_it() => Session.Dispatch(() =>
        {
            Choose(CoreCatalog.MarsRtEngine);
            MainWindow window = Start();
            EmulatorSession game = Game(window);

            Status(window).Text = "";
            Invoke(window, "SaveState");
            WaitFor(() => Status(window).Text?.StartsWith("State saved") == true);
            string state = StatePath(window);

            long saved = BitConverter.ToInt64(File.ReadAllBytes(state), 12);
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
            string save = SaveLibrary.SramPathFor(_rom, EmuSen.Cores.BatterySave.N64);
            byte[] original = Enumerable.Repeat((byte)0xFF, EepromSize).ToArray();
            Marker.CopyTo(original, Marked);
            Directory.CreateDirectory(Path.GetDirectoryName(save)!);
            File.WriteAllBytes(save, original);

            MainWindow window = Start();
            WaitFor(() => Game(window).TotalFrames > 40);
            Invoke(window, "ShowLibrary");

            byte[] file = File.ReadAllBytes(save);
            Assert.Equal(Marker, file[Marked..(Marked + 8)]);
            Assert.NotEqual(original[Written..(Written + 8)], file[Written..(Written + 8)]);
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

        // The library is loaded once per process, so a process started with it off is the only honest test of a build without it - see Mars_Native.md §6.7.
        [Fact]
        public void With_the_library_turned_off_an_n64_game_says_the_core_is_not_in_this_build()
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
            Assert.Contains("The N64 core is not in this build (turned off by EMUSEN_MARS_NATIVE=0)", found[0]);
        }

        // Run only by the test above, in its own process; it asserts nothing unless that process has the library off.
        [Fact]
        public Task TheFallbackInAProcessWithTheLibraryOff() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable(FallbackMarkVariable) is not { } mark || Environment.GetEnvironmentVariable(MarsNative.Variable) != "0") return;

            Assert.False(MarsNative.Available);
            Assert.False(MarsRtCore.Available);
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "System.z64");
            string? status = Status(window).Text;

            Assert.Null(typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window));
            Assert.StartsWith("Failed to load System.z64: ", status);
            File.WriteAllLines(mark, new[] { status! });
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

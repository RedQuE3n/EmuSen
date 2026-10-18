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
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // §3.12's C1, C2 and C5 in Mistress's own window: a resume typed while paused, a breakpoint set from the console, and Apply Cheats while paused - see EmuSen_Backend.md §14.13.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class MistressRunControlTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MistressRunControlTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMistressRunControl", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public MistressRunControlTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs") }.Save();
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

        private MainWindow Started()
        {
            var window = new MainWindow().WithoutBattery();
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", Hidden, null, new[] { typeof(string), typeof(string) }, null)!.Invoke(window, new object[] { _rom, "Loop.sfc" });
            Pump(() => Frames(window) > 10);
            Assert.True(Frames(window) > 10, "the game did not run");
            return window;
        }

        private static long Frames(MainWindow window) => MistressGame.TotalFrames(window);

        // The console window opened as the menu opens it, and a line typed into it.
        private static DianaShConsoleWindow Console(MainWindow window)
        {
            typeof(MainWindow).GetMethod("ShowShellConsole", Hidden)!.Invoke(window, null);
            return window.OwnedWindows.OfType<DianaShConsoleWindow>().Single();
        }

        private static void Type(DianaShConsoleWindow console, string line) =>
            typeof(DianaShConsoleWindow).GetMethod("Submit", Hidden)!.Invoke(console, new object[] { line });

        // Frames that stay where they are for a while: nothing advanced.
        private static bool Held(MainWindow window)
        {
            long before = Frames(window);
            Pump(() => false, 400);
            return Frames(window) == before;
        }

        // C1: a resume typed in the console while the game is paused resumes it.
        [Fact]
        public Task C1_a_resume_typed_while_paused_resumes_the_game() => Session.Dispatch(() =>
        {
            MainWindow window = Started();
            DianaShConsoleWindow console = Console(window);
            window.PauseEmulation();
            Assert.True(Held(window), "the pause did not hold the game");

            Type(console, "resume");
            Pump(() => !window.IsPaused);

            Assert.False(window.IsPaused, "the typed resume left the game paused");
            Assert.False(Held(window), "the typed resume did not run the game");
            window.Close();
        }, default);

        // C2: a breakpoint set from the console stops the game, and the window says so; removed and resumed, the game runs on.
        [Fact]
        public Task C2_a_breakpoint_set_from_the_console_stops_the_game() => Session.Dispatch(() =>
        {
            MainWindow window = Started();
            DianaShConsoleWindow console = Console(window);
            TextBlock status = window.GetControl<TextBlock>("StatusText");

            Type(console, "bp add 008005");
            Pump(() => status.Text?.Contains("BREAKPOINT", StringComparison.Ordinal) == true);

            Assert.True(Held(window), "the breakpoint did not stop the game");
            Assert.Contains("BREAKPOINT", status.Text);

            Type(console, "bp remove 1");
            Type(console, "resume");
            Pump(() => !Held(window), 5_000);
            Assert.False(Held(window), "the game did not run on after the breakpoint was removed");
            window.Close();
        }, default);

        // C5: Apply Cheats while paused pokes nothing outside a frame's end; the poke lands at the next frame boundary.
        [Fact]
        public Task C5_apply_while_paused_pokes_nothing_until_a_frame_ends() => Session.Dispatch(() =>
        {
            MainWindow window = Started();
            window.PauseEmulation();
            Assert.True(Held(window), "the pause did not hold the game");
            byte before = Peek(window, 0x7E0100);
            Assert.NotEqual(0x42, before);

            MistressGame.Cheats(window).Import(new CheatFile { Cheats = { new CheatFileEntry { Kind = "RamPoke", Description = "poke", Writes = { new CheatFileWrite { Space = "CpuBus", Address = "7E0100", Value = "42" } } } } });
            typeof(MainWindow).GetMethod("ShowActiveCheats", Hidden)!.Invoke(window, null);
            var cheats = window.OwnedWindows.OfType<ActiveCheatsWindow>().Single();
            cheats.GetControl<Button>("ApplyButton").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            Pump(() => false, 300);

            Assert.Equal(before, Peek(window, 0x7E0100));
            window.ResumeEmulation();
            Pump(() => Peek(window, 0x7E0100) == 0x42);
            Assert.Equal(0x42, Peek(window, 0x7E0100));
            window.Close();
        }, default);

        private static byte Peek(MainWindow window, int address) =>
            MistressGame.Target(window).GetMemorySpaces().First(s => s.Name == "CpuBus").Read(address);
    }
}

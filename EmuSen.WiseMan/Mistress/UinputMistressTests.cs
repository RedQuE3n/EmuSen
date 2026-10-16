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
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Input;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // A kernel pad pulled and plugged back under Mistress's own window and its real SDL devices, in the library, in a big-screen session and in a game - see EmuSen_Input.md §8.12.
    [Collection(TestCollections.ProcessGlobals)]
    public class UinputMistressTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(UinputMistressTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private static readonly MethodInfo PadTick = typeof(MainWindow).GetMethod("PadTick", Hidden)!;
        private static readonly MethodInfo PollGamepad = typeof(MainWindow).GetMethod("PollGamepad", Hidden)!;

        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenUinputMistressTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public UinputMistressTests(ITestOutputHelper output)
        {
            _output = output;
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreOptions.BatteryRamDisabled = true;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static GamepadManager Gamepad(MainWindow w) => (GamepadManager)typeof(MainWindow).GetField("_gamepad", Hidden)!.GetValue(w)!;

        private static string? Notice(MainWindow w) => w.GetControl<NoticeLayer>("PadNotice").Current;

        private static bool GameOn(MainWindow w) => w.GetControl<Control>("GameFrame").IsVisible;

        // One 16 ms tick of the window's timers: the pad timer always, and the game's poll while a game is on screen.
        private static void Tick(MainWindow w)
        {
            PadTick.Invoke(w, null);
            if (GameOn(w)) PollGamepad.Invoke(w, null);
            Dispatcher.UIThread.RunJobs();
            Thread.Sleep(16);
        }

        private static bool TickUntil(MainWindow w, Func<bool> condition, double seconds = 4)
        {
            var until = Stopwatch.StartNew();
            while (until.Elapsed.TotalSeconds < seconds)
            {
                Tick(w);
                if (condition()) return true;
            }
            return false;
        }

        private static ConnectedPad? Open(MainWindow w, UinputPad pad) => Gamepad(w).Pads.FirstOrDefault(p => p.Path == pad.EventPath);

        private static ushort SnesPlayer1(MainWindow w)
        {
            var session = (EmulatorSession)typeof(MainWindow).GetField("_session", Hidden)!.GetValue(w)!;
            return SyntheticRom.Pads(session.Core!).One;
        }

        public static TheoryData<string> Screens => new() { "library", "big picture", "game" };

        [UinputTheory, MemberData(nameof(Screens))]
        public Task A_pad_pulled_and_plugged_back_is_player_1_again_and_steers(string screen) => Session.Dispatch(() =>
        {
            for (int i = 0; i < 3; i++) File.WriteAllBytes(Path.Combine(_romDir, $"Game {i}.sfc"), SyntheticRom.BuildPadReader());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = screen == "big picture" }.Save();

            var pad = new UinputPad("EmuSen uinput pad");
            MainWindow? window = null;
            try
            {
                window = new MainWindow { Width = 1280, Height = 800 };
                window.Show();
                ((DispatcherTimer?)typeof(MainWindow).GetField("_padTimer", Hidden)!.GetValue(window))?.Stop();
                Gamepad(window).Start();
                var list = window.GetControl<ListBox>("LibraryList");
                list.SelectedIndex = 0;
                Assert.True(TickUntil(window, () => Open(window, pad) is not null), "the pad present at start");

                if (screen == "game")
                {
                    pad.Press(UinputPad.South, true);
                    Assert.True(TickUntil(window, () => GameOn(window)), "the pad's A starts the game");
                    pad.Press(UinputPad.South, false);
                    typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(window, null);
                    TickUntil(window, () => SnesPlayer1(window) == 0, 1);
                }

                pad.Dispose();
                Assert.True(TickUntil(window, () => Open(window, pad) is null), "the pull");
                _output.WriteLine($"{screen}, pulled: {Notice(window)}");
                Assert.Equal("Controller disconnected: Xbox 360 Controller (Player 1)", Notice(window));

                pad = new UinputPad("EmuSen uinput pad");
                Assert.True(TickUntil(window, () => Open(window, pad) is not null), "the pad plugged back");
                _output.WriteLine($"{screen}, plugged back: {Notice(window)}");
                Assert.Equal("Controller connected: Xbox 360 Controller (Player 1)", Notice(window));
                Assert.Equal(1, Gamepad(window).Players.PlayerOf(Open(window, pad)!));

                if (screen == "game")
                {
                    pad.Press(UinputPad.South, true);
                    Assert.True(TickUntil(window, () => SnesPlayer1(window) == 0x8000, 1), "its B reaches the game's port 1");
                    pad.Press(UinputPad.South, false);
                }
                else
                {
                    pad.HatY(1);
                    Assert.True(TickUntil(window, () => list.SelectedIndex == 1, 1), "its d-pad moves the library");
                    pad.HatY(0);
                }
            }
            finally
            {
                pad.Dispose();
                window?.Close();
            }
        }, default);
    }
}

using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // A second, third and fourth pad playing as players 2 to 4 through Mistress's own frame loop, hot-plug and the slot rule included - see EmuSen_Input.md §8.
    [Collection(TestCollections.ProcessGlobals)]
    public class MultiplayerTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MultiplayerTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMultiplayerTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public MultiplayerTests()
        {
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

        // One game in the library, started from the first pad, then held still so the test runs the frames.
        private (MainWindow Window, PadDriver First) Playing(string file, byte[] image, Action<AppSettings>? settings = null, bool bigScreen = false)
        {
            File.WriteAllBytes(Path.Combine(_romDir, file), image);
            var app = new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen };
            settings?.Invoke(app);
            app.Save();

            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var first = new PadDriver(window) { Pad = { Name = "First Pad" } };
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            first.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(window, null);
            return (window, first);
        }

        private static EmulatorSession SessionOf(MainWindow w) => (EmulatorSession)typeof(MainWindow).GetField("_session", Hidden)!.GetValue(w)!;

        private static void Poll(MainWindow w) => typeof(MainWindow).GetMethod("PollGamepad", Hidden)!.Invoke(w, null);

        // The SNES auto-read's two words, as the game would read them at $4218 and $421A.
        private static (ushort One, ushort Two) SnesPads(MainWindow w)
        {
            var venus = Assert.IsType<VenusCore>(SessionOf(w).Core);
            venus.Bus!.Input.LatchAutoJoypad();
            return ((ushort)(venus.Bus.Input.ReadJoy1Low() | venus.Bus.Input.ReadJoy1High() << 8), (ushort)(venus.Bus.Input.ReadJoy2Low() | venus.Bus.Input.ReadJoy2High() << 8));
        }

        private const ushort SnesB = 0x8000, SnesA = 0x0080, SnesStart = 0x1000;

        private static string? Notice(MainWindow w) => w.GetControl<NoticeLayer>("PadNotice").Current;

        [Fact]
        public Task A_second_pad_plays_as_player_2_and_its_slot_waits_while_it_is_unplugged() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = Playing("Two.sfc", SyntheticRom.BuildBlank());
            Assert.Equal(2, SessionOf(window).ControllerPorts);
            PadDriver second = first.Plug("Second Pad");
            Poll(window);
            Assert.Equal("Controller connected: Second Pad (Player 2)", Notice(window));

            second.Pad.Press(SDL.GamepadButton.South);
            first.Pad.Press(SDL.GamepadButton.East);
            Poll(window);
            Assert.Equal((SnesA, SnesB), SnesPads(window));

            // Pulled out mid-game: its buttons are let go, player 1 plays on, nothing pauses, and the notice names the seat kept.
            second.Unplug();
            Poll(window);
            Assert.Equal((SnesA, (ushort)0), SnesPads(window));
            Assert.Equal("Controller disconnected: Second Pad (Player 2)", Notice(window));
            Assert.False(window.IsPaused);
            Assert.False(first.Gamepad.Players.SeatOf(2)!.IsOpen);

            second.Replug();
            Poll(window);
            Assert.Equal("Controller connected: Second Pad (Player 2)", Notice(window));
            Assert.Equal((SnesA, SnesB), SnesPads(window));
            window.Close();
        }, default);

        // Player 1's pad going leaves player 2 where it is; a spare pad plugged in then is player 1.
        [Fact]
        public Task Player_1s_pad_going_moves_nobody_up_and_a_spare_takes_its_seat() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = Playing("Spare.sfc", SyntheticRom.BuildBlank());
            PadDriver second = first.Plug("Second Pad");
            Poll(window);
            first.Unplug();
            Poll(window);

            second.Pad.Press(SDL.GamepadButton.Start);
            Poll(window);
            Assert.Equal(((ushort)0, SnesStart), SnesPads(window));

            PadDriver spare = first.Plug("Spare Pad");
            Poll(window);
            Assert.Equal("Controller connected: Spare Pad (Player 1)", Notice(window));
            spare.Pad.Press(SDL.GamepadButton.Start);
            Poll(window);
            Assert.Equal((SnesStart, SnesStart), SnesPads(window));
            window.Close();
        }, default);

        // The keyboard moved to player 2 presses the second port, beside player 1's pad on the first.
        [Fact]
        public Task The_keyboard_can_play_as_player_2() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = Playing("Keys.sfc", SyntheticRom.BuildBlank(), a => a.KeyboardPlayer = 2);
            window.KeyPress(Key.Enter, RawInputModifiers.None, PhysicalKey.Enter, null);
            first.Pad.Press(SDL.GamepadButton.South);
            Poll(window);
            Assert.Equal((SnesB, SnesStart), SnesPads(window));
            window.KeyRelease(Key.Enter, RawInputModifiers.None, PhysicalKey.Enter, null);
            Assert.Equal((SnesB, (ushort)0), SnesPads(window));
            window.Close();
        }, default);

        // Four pads on the N64, each port plugged in as its player's pad is, and a fifth pad, which no port hears.
        [Fact]
        public Task Four_pads_play_the_n64s_four_ports_in_a_big_screen_session() => Session.Dispatch(() =>
        {
            GraphicsConfig graphics = GraphicsConfig.Load();
            graphics.SetValue("N64", CoreCatalog.EngineKey, CoreCatalog.MarsEngine);
            graphics.Save();
            (MainWindow window, PadDriver first) = Playing("Four.z64", SyntheticN64Rom.Build(patches: (0, new byte[] { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 })), bigScreen: true);
            var mars = Assert.IsType<MarsCore>(SessionOf(window).Core);
            var ports = mars.Bus!.Si.Controllers;
            Poll(window);
            Assert.Equal(new[] { true, false, false, false }, ports.Select(p => p.Present));

            PadDriver[] others = { first.Plug("Pad 2"), first.Plug("Pad 3"), first.Plug("Pad 4"), first.Plug("Pad 5") };
            Poll(window);
            Assert.Equal(new[] { true, true, true, true }, ports.Select(p => p.Present));

            for (int i = 0; i < others.Length; i++) others[i].Pad.Press(SDL.GamepadButton.East);
            others[2].Pad.SetAxis(SDL.GamepadAxis.LeftX, 1);
            Poll(window);
            Assert.Equal(new ushort[] { 0, 0x8000, 0x8000, 0x8000 }, ports.Select(p => p.Buttons));
            Assert.Equal(new sbyte[] { 0, 0, 0, 127 }, ports.Select(p => p.StickX));
            Assert.Equal(5, first.Gamepad.Players.PlayerOf(first.Gamepad.Pads[4]));
            window.Close();
        }, default);
    }
}

using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using System.Collections.Generic;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // Settings ▸ Controllers ▸ Players: which pad is which, a press lighting its row, trading seats, the keyboard's player and a seat kept, on the desktop and by pad alone in big picture - see EmuSen_Input.md §8.9.
    [Collection(TestCollections.ProcessGlobals)]
    public class PlayerPreferencesTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PlayerPreferencesTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPlayerPreferencesTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PlayerPreferencesTests()
        {
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private (MainWindow Window, PadDriver First) Desktop()
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            return (window, new PadDriver(window) { Pad = { Name = "First Pad" } });
        }

        // The Players page in the desktop's Settings window.
        private static void OpenPlayers(MainWindow window)
        {
            SettingsDesk.At(window, EmuSen.Mistress.Pages.SettingsScreen.Controllers, "players");
            Dispatcher.UIThread.RunJobs();
        }

        private static void Poll(PadDriver pad)
        {
            pad.Tick();
            Dispatcher.UIThread.RunJobs();
        }

        [Fact]
        public Task The_players_page_names_each_pads_player_and_trades_seats() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = Desktop();
            PadDriver second = first.Plug("Second Pad");
            first.Tick();
            OpenPlayers(window);

            Assert.Equal("First Pad", SettingsDesk.Row(window, "pad0").Label);
            Assert.Equal("Second Pad", SettingsDesk.Row(window, "pad1").Label);
            Assert.Equal("Player 1", SettingsDesk.Value(window, "pad0"));
            Assert.Equal("Player 2", SettingsDesk.Value(window, "pad1"));
            Assert.Equal("Player 1", SettingsDesk.Value(window, "keyboardplayer"));

            SettingsDesk.Pick(window, "pad1", "Player 1");
            Assert.Same(first.Gamepad.Pads[1], first.Gamepad.Players.PadFor(1));
            Assert.Equal("Player 2", SettingsDesk.Value(window, "pad0"));
            Assert.Equal("Player 1", SettingsDesk.Value(window, "pad1"));

            SettingsDesk.Pick(window, "keyboardplayer", "Player 2");
            Assert.Equal(2, AppSettings.Load().KeyboardPlayer);

            SettingsDesk.Pick(window, "pad0", EmuSen.Mistress.Pages.SettingsScreen.NoPlayer);
            Assert.Equal(0, first.Gamepad.Players.PlayerOf(first.Gamepad.Pads[0]));
            Assert.StartsWith("Plays as no one", SettingsDesk.Row(window, "pad0").Sentence);
            window.Close();
        }, default);

        // Any button held on a pad lights its row, and only its row; two of one name are numbered.
        [Fact]
        public Task A_press_on_a_pad_lights_its_row_and_identical_pads_are_numbered() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = Desktop();
            PadDriver twin1 = first.Plug("Twin Pad"), twin2 = first.Plug("Twin Pad");
            first.Tick();
            OpenPlayers(window);
            Assert.Equal(("1 · Twin Pad", "2 · Twin Pad"), (SettingsDesk.Row(window, "pad1").Label, SettingsDesk.Row(window, "pad2").Label));
            bool[] Lit() => Enumerable.Range(0, 3).Select(i => SettingsDesk.Value(window, $"pad{i}")!.StartsWith("● ", StringComparison.Ordinal)).ToArray();

            twin2.Pad.Press(SDL.GamepadButton.LeftShoulder);
            Poll(first);
            Assert.Equal(new[] { false, false, true }, Lit());
            twin2.Pad.Release(SDL.GamepadButton.LeftShoulder);
            twin1.Pad.SetAxis(SDL.GamepadAxis.RightTrigger, 1);
            Poll(first);
            Assert.Equal(new[] { false, true, false }, Lit());
            window.Close();
        }, default);

        // A pad pulled out leaves its seat kept and named, and A on it gives it up; one plugged in while the page is open gets a row.
        [Fact]
        public Task A_seat_kept_for_a_pad_gone_is_shown_and_can_be_forgotten() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = Desktop();
            PadDriver second = first.Plug("Second Pad");
            first.Tick();
            OpenPlayers(window);

            second.Unplug();
            Poll(first);
            Assert.DoesNotContain("pad1", SettingsDesk.Keys(window));
            Assert.Equal("Player 2: Second Pad, disconnected", SettingsDesk.Row(window, "kept2").Label);
            SettingsDesk.Choose(window, "kept2");
            Assert.Null(first.Gamepad.Players.SeatOf(2));
            Assert.DoesNotContain("kept2", SettingsDesk.Keys(window));

            first.Plug("Third Pad");
            Poll(first);
            Assert.Equal("Third Pad", SettingsDesk.Row(window, "pad1").Label);
            Assert.Equal(2, first.Gamepad.Players.PlayerOf(first.Gamepad.Pads[1]));
            window.Close();
        }, default);

        // In a big-screen session the players are menu rows: reached, changed and lit with the pad alone.
        [Fact]
        public Task In_big_picture_the_pad_alone_changes_a_pads_player() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(1280, 800);
            PadDriver second = s.Pad.Plug("Second Pad");
            s.Pad.Tick();
            // Settings ▸ Controllers ▸ Players: a row per pad, stepped with the pad, a held button marking its row.
            SettingsLayout layout = SettingsPad.Open(s.Window, s.Pad);
            SettingsPad.To(s.Window, s.Pad, "players");
            s.Pad.A();
            Assert.Equal("Second Pad", layout.Rows.Single(r => r.Key == "pad1").Label);
            SettingsPad.To(s.Window, s.Pad, "pad1");
            s.Pad.Right();
            Assert.Equal(3, s.Pad.Gamepad.Players.PlayerOf(s.Pad.Gamepad.Pads[1]));
            Assert.Equal("Player 3", SettingsPad.Value(s.Window, "pad1"));

            second.Pad.Press(SDL.GamepadButton.LeftShoulder);
            s.Pad.Tick();
            s.Settle();
            Assert.Equal("● Player 3", SettingsPad.Value(s.Window, "pad1"));
            second.Pad.Release(SDL.GamepadButton.LeftShoulder);
            s.Pad.Tick();
            s.Settle();
            Assert.Equal("Player 3", SettingsPad.Value(s.Window, "pad1"));
            Assert.Empty(FitAudit.Check(layout, []));
            s.Pad.B();
            s.Pad.B();
            Assert.Null(s.Window.SettingsShown);
        }, default);
    }
}

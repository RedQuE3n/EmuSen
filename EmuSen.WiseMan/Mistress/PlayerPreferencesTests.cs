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
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // Preferences ▸ Controllers' players: which pad is which, a press lighting its row, trading seats, the keyboard's player and a seat kept, on the desktop and by pad alone in big picture - see EmuSen_Input.md §8.9.
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

        private static PreferencesWindow OpenControllers(MainWindow window)
        {
            typeof(MainWindow).GetMethod("ShowPreferencesAt", Hidden)!.Invoke(window, new object?[] { PreferencesWindow.ControllersTab });
            Dispatcher.UIThread.RunJobs();
            return window.GetControl<SheetLayer>("Sheets").Current as PreferencesWindow ?? window.OwnedWindows.OfType<PreferencesWindow>().Last();
        }

        private static T Named<T>(Control root, string name) where T : Control =>
            root.GetLogicalDescendants().OfType<T>().First(c => c.Name == name);

        private static T? Maybe<T>(Control root, string name) where T : Control =>
            root.GetLogicalDescendants().OfType<T>().FirstOrDefault(c => c.Name == name);

        [Fact]
        public Task The_controllers_tab_names_each_pads_player_and_trades_seats() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = Desktop();
            PadDriver second = first.Plug("Second Pad");
            first.Tick();
            PreferencesWindow prefs = OpenControllers(window);

            Assert.Equal("First Pad", Named<FieldRow>(prefs, "PadPlayerRow0").Label);
            Assert.Equal("Second Pad", Named<FieldRow>(prefs, "PadPlayerRow1").Label);
            Assert.Equal("Player 1", Named<Dropdown>(prefs, "PadPlayerDropdown0").SelectedItem);
            Assert.Equal("Player 2", Named<Dropdown>(prefs, "PadPlayerDropdown1").SelectedItem);
            Assert.Equal("Player 1", Named<Dropdown>(prefs, "KeyboardPlayerDropdown").SelectedItem);

            Named<Dropdown>(prefs, "PadPlayerDropdown1").SelectedItem = "Player 1";
            Assert.Same(first.Gamepad.Pads[1], first.Gamepad.Players.PadFor(1));
            Assert.Equal("Player 2", Named<Dropdown>(prefs, "PadPlayerDropdown0").SelectedItem);
            Assert.Equal("Player 1", Named<Dropdown>(prefs, "PadPlayerDropdown1").SelectedItem);

            Named<Dropdown>(prefs, "KeyboardPlayerDropdown").SelectedItem = "Player 2";
            Assert.Equal(2, AppSettings.Load().KeyboardPlayer);

            Named<Dropdown>(prefs, "PadPlayerDropdown0").SelectedItem = PlayerPreferencesRows.NoPlayer;
            Assert.Equal(0, first.Gamepad.Players.PlayerOf(first.Gamepad.Pads[0]));
            Assert.StartsWith("Plays as no one", Named<FieldRow>(prefs, "PadPlayerRow0").Hint);
            prefs.Close();
            window.Close();
        }, default);

        // Any button held on a pad lights its row, and only its row; two of one name are numbered.
        [Fact]
        public Task A_press_on_a_pad_lights_its_row_and_identical_pads_are_numbered() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = Desktop();
            PadDriver twin1 = first.Plug("Twin Pad"), twin2 = first.Plug("Twin Pad");
            first.Tick();
            PreferencesWindow prefs = OpenControllers(window);
            Assert.Equal(("1 · Twin Pad", "2 · Twin Pad"), (Named<FieldRow>(prefs, "PadPlayerRow1").Label, Named<FieldRow>(prefs, "PadPlayerRow2").Label));

            twin2.Pad.Press(SDL.GamepadButton.LeftShoulder);
            first.Tick();
            Assert.Equal(new[] { false, false, true }, Enumerable.Range(0, 3).Select(i => Named<Dropdown>(prefs, $"PadPlayerDropdown{i}").Classes.Contains("identify")));
            twin2.Pad.Release(SDL.GamepadButton.LeftShoulder);
            twin1.Pad.SetAxis(SDL.GamepadAxis.RightTrigger, 1);
            first.Tick();
            Assert.Equal(new[] { false, true, false }, Enumerable.Range(0, 3).Select(i => Named<Dropdown>(prefs, $"PadPlayerDropdown{i}").Classes.Contains("identify")));
            prefs.Close();
            window.Close();
        }, default);

        // A pad pulled out leaves its seat kept and named, and Forget gives it up; one plugged in while the sheet is open gets a row.
        [Fact]
        public Task A_seat_kept_for_a_pad_gone_is_shown_and_can_be_forgotten() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = Desktop();
            PadDriver second = first.Plug("Second Pad");
            first.Tick();
            PreferencesWindow prefs = OpenControllers(window);

            second.Unplug();
            first.Tick();
            Assert.Null(Maybe<FieldRow>(prefs, "PadPlayerRow1"));
            Assert.Equal("Player 2: Second Pad, disconnected", Named<FieldRow>(prefs, "KeptPlayerRow2").Label);
            Named<Button>(prefs, "ForgetPlayerButton2").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            Assert.Null(first.Gamepad.Players.SeatOf(2));
            Assert.Null(Maybe<FieldRow>(prefs, "KeptPlayerRow2"));

            first.Plug("Third Pad");
            first.Tick();
            // The tab's content is reached twice in the logical tree, through its tab and through the selected-content presenter.
            Assert.Equal("PadPlayerRow2", prefs.GetLogicalDescendants().OfType<FieldRow>().Distinct().Single(r => r.Label == "Third Pad").Name);
            Assert.Equal(2, first.Gamepad.Players.PlayerOf(first.Gamepad.Pads[1]));
            prefs.Close();
            window.Close();
        }, default);

        // In a big-screen session the players are menu rows: reached, changed and lit with the pad alone.
        [Fact]
        public Task In_big_picture_the_pad_alone_changes_a_pads_player() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(1280, 800);
            PadDriver second = s.Pad.Plug("Second Pad");
            s.Pad.Tick();
            ThemedLibraryFlowTests.Choose(s, "Preferences");
            s.Settle();
            SheetLayer sheets = s.Window.GetControl<SheetLayer>("Sheets");
            var prefs = Assert.IsType<PreferencesWindow>(sheets.Current);
            Control sheet = sheets.SheetOf(prefs)!;
            PadAudit.Reach(sheet, s.Pad, e => e is Control { Name: "BigMenuPage4" });
            s.Pad.A();
            Assert.Equal("Controllers", prefs.Form!.Menu.Title);

            Dropdown choice = Named<Dropdown>(sheet, "PadPlayerDropdown1");
            Assert.Equal("Second Pad", MenuRows.GetLabel(choice));
            PadAudit.Reach(sheet, s.Pad, e => ReferenceEquals(e, choice));
            s.Pad.Right();
            Assert.Equal(3, s.Pad.Gamepad.Players.PlayerOf(s.Pad.Gamepad.Pads[1]));
            Assert.Equal("Player 3", choice.SelectedItem);

            second.Pad.Press(SDL.GamepadButton.LeftShoulder);
            s.Pad.Tick();
            Assert.True(choice.Classes.Contains("identify"));
            Assert.NotNull(MenuRows.GetValueColor(choice));
            second.Pad.Release(SDL.GamepadButton.LeftShoulder);
            s.Pad.Tick();
            Assert.False(choice.Classes.Contains("identify"));
            Assert.Empty(FitAudit.Check(sheet, WindowAllowances.For(prefs)));
            ThemedSwitchesTests.PutAway(s);
        }, default);
    }
}

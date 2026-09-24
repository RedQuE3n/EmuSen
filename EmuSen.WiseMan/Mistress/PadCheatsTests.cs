using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // Cheats from the pad during play: the menu entry, a code typed on the on-screen keyboard, the tick, Apply, and the poke landing - see EmuSen_Settings_Reference.md §4.45.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class PadCheatsTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PadCheatsTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPadCheatsTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PadCheatsTests(ITestOutputHelper output)
        {
            _out = output;
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

        private (MainWindow Window, PadDriver Pad) GameModeWithAGame()
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = true }.Save();

            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            return (window, pad);
        }

        private static void Choose(MainWindow window, PadDriver pad, string entry)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            string[] lines = window.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();
            int at = Array.FindIndex(lines, l => l.StartsWith(entry, StringComparison.Ordinal));
            Assert.True(at >= 0, $"No '{entry}' in the pad menu: {string.Join(", ", lines)}");
            pad.Down(at);
            pad.A();
        }

        private static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");
        private static Control Sheet(MainWindow w) => Sheets(w).SheetOf(Sheets(w).Current!)!;
        private static InputElement Reach(MainWindow w, PadDriver pad, Func<InputElement, bool> target) => PadAudit.Reach(Sheet(w), pad, target);
        private static void Picture(MainWindow w, string name) => UiTest.Dump("pad-" + name, UiTest.Capture(w));
        private static CheatRegistry Cheats(MainWindow w) => (CheatRegistry)typeof(MainWindow).GetField("_cheats", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(w)!;

        private static byte Peek(MainWindow w, int address)
        {
            var target = (IDebugTarget)typeof(MainWindow).GetField("_debugTarget", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(w)!;
            return target.GetMemorySpaces().First(s => s.Name == "CpuBus").Read(address);
        }

        private static void Stop(MainWindow window) =>
            typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        // Each character reached on the keys with the d-pad, rows first, the short way round a row, and pressed with A.
        internal static void TypeByPad(PadDriver pad, OnScreenKeyboard keyboard, string text)
        {
            (int Row, int Column) Where(string key)
            {
                var rows = keyboard.Layout.Rows;
                for (int r = 0; r < rows.Count; r++)
                    for (int c = 0; c < rows[r].Count; c++)
                        if (rows[r][c].Key == key) return (r, c);
                throw new InvalidOperationException($"No key {key} on {keyboard.Layout.Name}.");
            }

            foreach (char ch in text)
            {
                string key = ch == ' ' ? KeyboardLayout.Space : ch.ToString();
                (int row, int column) = Where(key);
                for (int guard = 0; guard < 40 && keyboard.CurrentKey != key; guard++)
                {
                    (int r, int c) = Where(keyboard.CurrentKey);
                    int count = keyboard.Layout.Rows[r].Count;
                    if (r < row) pad.Down();
                    else if (r > row) pad.Up();
                    else if ((column - c + count) % count <= count / 2) pad.Right();
                    else pad.Left();
                }
                Assert.Equal(key, keyboard.CurrentKey);
                pad.A();
            }
        }

        [Fact]
        public Task A_code_is_typed_added_ticked_off_and_on_and_applied_all_from_the_pad_and_it_lands_in_memory() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Cheats");

            var cheats = Assert.IsType<ActiveCheatsWindow>(Sheets(window).Current);
            Assert.Empty(window.OwnedWindows);
            Assert.True(window.IsPaused);
            var tabs = Sheet(window).GetVisualDescendants().OfType<TabControl>().First();
            Assert.Equal("SNES", (tabs.SelectedItem as TabItem)?.Header);

            var code = (TextBox)Reach(window, pad, e => e is TextBox { Name: "SNESCodeBox" });
            pad.A();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(window)!;
            Assert.Same(code, keyboard.Target);
            Assert.Same(KeyboardLayout.Code, keyboard.Layout);
            TypeByPad(pad, keyboard, "7E010042");
            Picture(window, "cheats-keyboard");
            pad.Start();
            Assert.Null(OnScreenKeyboard.OpenOver(window));
            Assert.Equal("7E010042", code.Text);
            Assert.Same(code, window.FocusManager!.GetFocusedElement());

            var description = (TextBox)Reach(window, pad, e => e is TextBox { Name: "SNESDescriptionBox" });
            pad.A();
            keyboard = OnScreenKeyboard.OpenOver(window)!;
            Assert.Same(KeyboardLayout.Letters, keyboard.Layout);
            TypeByPad(pad, keyboard, "lives");
            pad.Start();
            Assert.Equal("lives", description.Text);

            Reach(window, pad, e => e is Button { Name: "SNESAddButton" });
            pad.A();
            CheatInfo added = Assert.Single(Cheats(window).GetCheats());
            Assert.Equal("lives", added.Description);
            Assert.True(added.Enabled);

            // A on the cheat's row ticks its box, and the table writes it through to the registry.
            Reach(window, pad, e => e is ListBoxItem);
            pad.A();
            Assert.False(Cheats(window).GetCheats().Single().Enabled);
            pad.A();
            Assert.True(Cheats(window).GetCheats().Single().Enabled);
            Picture(window, "cheats-list");

            byte before = Peek(window, 0x7E0100);
            Assert.NotEqual(0x42, before);
            Reach(window, pad, e => e is Button { Name: "ApplyButton" });
            pad.A();
            Assert.Equal(0x42, Peek(window, 0x7E0100));

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Assert.False(window.IsPaused);
            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task The_database_opens_over_the_cheats_and_B_comes_back_to_them() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Cheats");
            var cheats = Sheets(window).Current;

            Reach(window, pad, e => e is Button { Name: "DatabaseButton" });
            pad.A();
            Assert.IsType<CheatDatabaseWindow>(Sheets(window).Current);
            Picture(window, "cheat-database");

            pad.B();
            Assert.Same(cheats, Sheets(window).Current);
            Assert.True(window.IsPaused);
            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Stop(window);
            window.Close();
        }, default);

        [Theory]
        [InlineData("Cheats")]
        [InlineData("Cheat Database")]
        public Task Every_control_of_the_cheat_sheets_is_reached_by_the_pad(string which) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Cheats(window).AddRamPoke("CpuBus", 0x7E0100, 0x42, "one");
            Cheats(window).AddRamPoke("CpuBus", 0x7E0101, 0x43, "two");
            Choose(window, pad, "Cheats");
            if (which == "Cheat Database")
            {
                Reach(window, pad, e => e is Button { Name: "DatabaseButton" });
                pad.A();
            }

            Control sheet = Sheet(window);
            TabControl? tabs = sheet.GetVisualDescendants().OfType<TabControl>().FirstOrDefault();
            var missing = new System.Collections.Generic.List<string>();
            for (int page = 0; page < (tabs?.ItemCount ?? 1); page++)
            {
                if (page > 0) pad.R1();
                window.UpdateLayout();
                var reached = PadAudit.Reachable(sheet, pad);
                missing.AddRange(PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(c => $"[{(tabs?.SelectedItem as TabItem)?.Header}] {PadAudit.Describe(c)}"));
            }
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);

            Stop(window);
            window.Close();
        }, default);

        // Two games under the console's folder: the pad picks the system, moves to the games, loads one, and ticks its cheat on.
        [Fact]
        public Task A_game_s_cheats_come_from_the_database_by_pad_and_one_is_ticked_on() => Session.Dispatch(() =>
        {
            string db = Path.Combine(_root, "Cheats", "Nintendo - Super Nintendo Entertainment System");
            Directory.CreateDirectory(db);
            File.WriteAllText(Path.Combine(db, "Alpha (USA).cht"), "cheats = 1\ncheat0_desc = \"Alpha lives\"\ncheat0_code = \"7E010042\"\ncheat0_enable = false\n");
            File.WriteAllText(Path.Combine(db, "Beta (USA).cht"), "cheats = 1\ncheat0_desc = \"Beta lives\"\ncheat0_code = \"7E010143\"\ncheat0_enable = false\n");
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            var settings = (AppSettings)typeof(MainWindow).GetField("_appSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            settings.CheatDatabaseDirectory = Path.Combine(_root, "Cheats");

            Choose(window, pad, "Cheats");
            Reach(window, pad, e => e is Button { Name: "DatabaseButton" });
            pad.A();
            var database = Assert.IsType<CheatDatabaseWindow>(Sheets(window).Current);

            Reach(window, pad, e => e is ListBoxItem item && item.FindAncestorOfType<ListBox>()?.Name == "SystemsList");
            pad.A();
            Reach(window, pad, e => e is ListBoxItem item && item.FindAncestorOfType<ListBox>()?.Name == "GamesList" && item.Content?.ToString()?.Contains("Beta") == true);
            Picture(window, "cheat-database-games");
            pad.A();

            CheatInfo loaded = Assert.Single(Cheats(window).GetCheats());
            Assert.Equal("Beta lives", loaded.Description);
            Assert.False(loaded.Enabled);

            // The database stays over the cheats until B; the cheats' list then has the game's code, ticked on with A.
            pad.B();
            Assert.IsType<ActiveCheatsWindow>(Sheets(window).Current);
            Reach(window, pad, e => e is ListBoxItem item && item.FindAncestorOfType<ListBox>()?.Name == "PART_Rows");
            pad.A();
            Assert.True(Cheats(window).GetCheats().Single().Enabled);

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Stop(window);
            window.Close();
        }, default);
    }
}

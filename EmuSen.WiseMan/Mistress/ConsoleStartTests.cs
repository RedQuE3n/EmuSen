using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Mistress.Library;

namespace EmuSen.WiseMan.Mistress
{
    // Mistress's entries for a console started with nothing in it, the session they start, and the drive's rows in it - see Diamond_Shell.md §1.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class ConsoleStartTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ConsoleStartTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenConsoleStart_" + Guid.NewGuid().ToString("N"));

        public ConsoleStartTests()
        {
            Directory.CreateDirectory(Path.Combine(_root, "Roms"));
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            new AppSettings { BigPictureInterface = { MenuOpeningEffect = BigPictureInterface.OpeningNone }, RomDirectory = Path.Combine(_root, "Roms"), StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs"), CheatDatabaseDirectory = Path.Combine(_root, "Cheats") }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static object? Call(MainWindow w, string method, params object[] args) =>
            typeof(MainWindow).GetMethods(Hidden | BindingFlags.Public).First(m => m.Name == method && m.GetParameters().Length == args.Length).Invoke(w, args);

        private static T Field<T>(MainWindow w, string name) => (T)typeof(MainWindow).GetField(name, Hidden)!.GetValue(w)!;

        private static IReadOnlyList<string> Rows(object? entries) => ((IEnumerable<PadMenuEntry>)entries!).Select(e => e.Text()).ToList();

        // Start Console's page for one system of the build's table, by its id.
        private static PadMenuEntry PageRow(MainWindow w, string id) =>
            w.StartConsoleEntries(EmuSen.Mistress.Library.RuntimeCatalog.Systems.Where(s => s.Id == id)).Single();

        // The firmware line under the page's sentences: found, not found as Cores words it, or none for a console that needs none - see EmuSen_BigPicture.md §51.8.
        [Fact]
        public Task Start_Console_s_pane_says_whether_the_firmware_was_found() => Session.Dispatch(() =>
        {
            string folder = Path.Combine(_root, "MyFirmware");
            Directory.CreateDirectory(folder);
            AppSettings settings = AppSettings.Load();
            settings.FirmwareDirectory = folder;
            settings.Save();
            var window = new MainWindow();
            try
            {
                Assert.Equal(("✗ Not found: add it in Settings → Firmware", false), PageRow(window, "ps2").PaneSentences!().Flag);
                File.WriteAllBytes(Path.Combine(folder, "ps2-0230a-20080220.bin"), new byte[4 << 20]);
                Assert.Equal(("✓ Firmware found.", true), PageRow(window, "ps2").PaneSentences!().Flag);
                // The PlayStation's BIOS has an open replacement, so with none in the folders it needs none, and nothing is said.
                PadMenuEntry psx = PageRow(window, "psx");
                Assert.Null(psx.PaneSentences!().Flag);
                Assert.Equal("Starts a PlayStation with no disc in it, on Diamond.", psx.PaneSentences!().Sentences[0]);
            }
            finally { window.Close(); }
        }, default);

        // A console that starts empty but that no engine runs is offered dimmed as not supported yet, and A on it starts nothing.
        [Fact]
        public Task A_console_no_engine_runs_is_dimmed_as_not_supported_yet_and_starts_nothing() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            try
            {
                var psx = EmuSen.Mistress.Library.RuntimeCatalog.Systems.First(s => s.Id == "psx");
                var saturn = psx with { Id = "saturn", Name = "Saturn", Console = "Saturn", Engines = [], DisplayName = "Saturn", StartsEmpty = true, Manufacturer = "Sega", ReleaseYear = 1994 };
                List<PadMenuEntry> rows = window.StartConsoleEntries([psx, saturn]);
                PadMenuEntry row = rows.Single(r => r.Text() == "Saturn");
                Assert.True(row.Dimmed);
                Assert.Equal(MainWindow.NotSupportedYet, row.Value!());
                Assert.False(row.Closes);
                Assert.False(rows.Single(r => r.Text() == "PlayStation").Dimmed);
                row.Accept();
                Assert.Null(Field<object?>(window, "_session"));
                Assert.False(window.GetControl<Control>("GameFrame").IsVisible);
            }
            finally { window.Close(); }
        }, default);

        // The pad menu's Library > Start a Console, and the desktop library's button while the PlayStation's shelf is shown.
        [Fact]
        public Task The_entries_offer_the_consoles_that_start_empty() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            try
            {
                Assert.Contains("Start PlayStation", Rows(Call(window, "StartConsoleRows")));
                Assert.Contains(MainWindow.StartConsoleMenu, Rows(Call(window, "HomeRows")));
                var button = window.GetControl<Button>("StartConsoleButton");
                Field<AppSettings>(window, "_appSettings").SelectedCore = RuntimeCatalog.Shelves.First(s => s.Core.Console == "PlayStation").Name;
                Call(window, "UpdateStartConsoleButton");
                Assert.Equal((true, (object?)"Start PlayStation"), (button.IsVisible, button.Content));
                Field<AppSettings>(window, "_appSettings").SelectedCore = RuntimeCatalog.Shelves.First(s => s.Core.Console == "SNES").Name;
                Call(window, "UpdateStartConsoleButton");
                Assert.False(button.IsVisible);
            }
            finally
            {
                window.Close();
            }
        }, default);

        // Started from the menu: running under the console's own name, its states filed under it, the drive's rows offered.
        [Fact]
        public Task A_console_started_empty_runs_under_its_own_name_with_the_drives_rows() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            try
            {
                Call(window, "StartConsole", "PlayStation");
                Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
                Assert.StartsWith("Running: PlayStation (no disc)", window.GetControl<TextBlock>("StatusText").Text);
                Assert.Equal("PlayStation", Field<string>(window, "_consoleSession"));
                Assert.Null(typeof(MainWindow).GetField("_currentRomPath", Hidden)!.GetValue(window));
                string state = (string)typeof(MainWindow).GetProperty("CurrentStatePath", Hidden)!.GetValue(window)!;
                Assert.Equal("PlayStation (no disc).state", Path.GetFileName(state));
                Assert.Equal(new[] { MainWindow.ChangeDiscMenu }, Rows(Call(window, "DiscRows")));
                Assert.Contains(MainWindow.ChangeDiscMenu, Rows(Call(window, "GameMenuRows")));
                Assert.Equal(("psx", System.Text.Json.JsonValueKind.Null), (MistressGame.Info(window).GetProperty("system").GetString(), MistressGame.Info(window).GetProperty("game").ValueKind));
            }
            finally
            {
                typeof(MainWindow).GetMethod("ShowLibrary", Hidden)!.Invoke(window, null);
                window.Close();
            }
        }, default);

        // Change Disc as a page of the card: the console's games and any file on the desktop; A puts one in and the console sees it, and Open the tray takes it out (EmuSen_Interface.md §18.5).
        [Fact]
        public Task Change_disc_puts_a_library_game_in_and_the_console_sees_it() => Session.Dispatch(() =>
        {
            SyntheticPsxDisc.Build().WriteSet(Path.Combine(_root, "Roms"), "Amber Saga");
            var window = new MainWindow();
            window.Show();
            try
            {
                var pad = new PadDriver(window);
                Call(window, "StartConsole", "PlayStation");
                EmuSen.Mistress.GameSession session = MistressGame.Of(window);
                Assert.Equal(-1, session.Discs.Index);
                pad.Chord(SDL3.SDL.GamepadButton.Back, SDL3.SDL.GamepadButton.Start);
                PadMenu.Choose(window, pad, MainWindow.ChangeDiscMenu, exact: true);
                Assert.Equal(2, PadMenu.Depth(window));
                Assert.Equal(new[] { "Amber Saga", MainWindow.AnyFileRow }, PadMenu.Lines(window));
                Assert.True(window.IsPaused);
                BigPicture.WindowFitAuditTests.Settle(window);
                Assert.Equal("cover", PadMenu.Pane(window).Showing);
                Assert.Equal(["Put in", "Back", "Screenshot"], PadMenu.Big(window).Hints!.Select(h => h.Label));

                pad.A();
                Assert.False(PadMenu.IsOpen(window));
                WaitFor(() => session.Discs is { Index: 0, LidOpen: false }, "the disc in the drive and the lid closed");
                WaitFor(() => window.GetControl<TextBlock>("StatusText").Text == "Amber Saga is in", "the status");

                pad.Chord(SDL3.SDL.GamepadButton.Back, SDL3.SDL.GamepadButton.Start);
                PadMenu.Choose(window, pad, MainWindow.ChangeDiscMenu, exact: true);
                Assert.Contains(MainWindow.OpenTrayRow, PadMenu.Lines(window));
                PadMenu.Choose(window, pad, MainWindow.OpenTrayRow, exact: true);
                WaitFor(() => session.Discs.Index == -1, "the tray open");
            }
            finally
            {
                typeof(MainWindow).GetMethod("ShowLibrary", Hidden)!.Invoke(window, null);
                window.Close();
            }
        }, default);

        // Game Options ▸ Memory Cards on a disc console: the slots, the mode and the drawer, Settings' rows; a slot's card changed is kept for the console and goes in when the game next starts (EmuSen_Interface.md §20.9).
        [Fact]
        public Task Memory_cards_change_the_card_a_slot_starts_the_game_with() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();
            try
            {
                var pad = new PadDriver(window);
                Call(window, "StartConsole", "PlayStation");
                void Open()
                {
                    pad.Chord(SDL3.SDL.GamepadButton.Back, SDL3.SDL.GamepadButton.Start);
                    PadMenu.Choose(window, pad, MainWindow.GameOptionsMenu, exact: true);
                    PadMenu.Choose(window, pad, MainWindow.MemoryCardsMenu, exact: true);
                }
                Open();
                Assert.Equal(new[] { "Slot 1", "Slot 2", "Mode", "Cards" }, PadMenu.Entries(window).Select(e => e.Label!()));
                Assert.Equal(["Change", "Card", "Back"], PadMenu.Big(window).Hints!.Select(h => h.Label).Take(3));
                string before = PadMenu.Entries(window)[0].Value!()!;
                pad.Right();
                string after = PadMenu.Entries(window)[0].Value!()!;
                Assert.NotEqual(before, after);
                Assert.Contains(PadMenu.Entries(window)[0].PaneScreen!()!.Lines, l => l.Text.Contains("when the game next starts", StringComparison.Ordinal));
                pad.Start();
                Open();
                Assert.Equal(after, PadMenu.Entries(window)[0].Value!());
            }
            finally
            {
                typeof(MainWindow).GetMethod("ShowLibrary", Hidden)!.Invoke(window, null);
                window.Close();
            }
        }, default);

        private static void WaitFor(Func<bool> condition, string what)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail($"timed out waiting for {what}");
                System.Threading.Thread.Sleep(5);
            }
        }

        // With Change Disc and a second pad the Pause Card's first page would reach nine rows, so Swap Players is left to Game Options ▸ Controllers, and Memory Cards shows for the disc console - see EmuSen_BigPicture.md §48.3.
        [Fact]
        public Task A_ninth_row_sends_swap_players_under_controllers() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();
            try
            {
                var pad = new PadDriver(window);
                pad.Plug("Xbox Wireless Controller");
                pad.Tick();
                Call(window, "StartConsole", "PlayStation");
                Assert.True(pad.Gamepad.CanSwapPlayers);
                IReadOnlyList<string> first = Rows(Call(window, "GameMenuRows"));
                Assert.Equal(8, first.Count);
                Assert.DoesNotContain(MainWindow.SwapPlayersRow, first);
                var controllers = new EmuSen.Mistress.Pages.SettingsScreen(window);
                controllers.OpenGame(EmuSen.Mistress.Pages.SettingsScreen.GameControllers);
                Assert.Contains("Swap Players", controllers.Rows.Select(r => r.Label));
                Assert.Contains(MainWindow.MemoryCardsMenu, Rows(Call(window, "GameOptionsRows")));
            }
            finally
            {
                typeof(MainWindow).GetMethod("ShowLibrary", Hidden)!.Invoke(window, null);
                window.Close();
            }
        }, default);
    }
}

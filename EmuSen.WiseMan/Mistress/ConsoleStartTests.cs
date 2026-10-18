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
            CoreDiscovery.UseDevelopment(false);
            new AppSettings { RomDirectory = Path.Combine(_root, "Roms"), StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs"), CheatDatabaseDirectory = Path.Combine(_root, "Cheats") }.Save();
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static object? Call(MainWindow w, string method, params object[] args) =>
            typeof(MainWindow).GetMethods(Hidden | BindingFlags.Public).First(m => m.Name == method && m.GetParameters().Length == args.Length).Invoke(w, args);

        private static T Field<T>(MainWindow w, string name) => (T)typeof(MainWindow).GetField(name, Hidden)!.GetValue(w)!;

        private static IReadOnlyList<string> Rows(object? entries) => ((IEnumerable<PadMenuEntry>)entries!).Select(e => e.Text()).ToList();

        // The pad menu's Library > Start a Console, and the desktop library's button while the PlayStation's shelf is shown.
        [Fact]
        public Task The_entries_offer_the_consoles_that_start_empty() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            try
            {
                Assert.Contains("Start PlayStation", Rows(Call(window, "StartConsoleRows")));
                Assert.Contains(MainWindow.StartConsoleMenu, Rows(Call(window, "LibraryRows")));
                var button = window.GetControl<Button>("StartConsoleButton");
                Field<AppSettings>(window, "_appSettings").SelectedCore = CoreCatalog.ShelvesInReleaseOrder.First(s => s.Core.Console == "PlayStation").Name;
                Call(window, "UpdateStartConsoleButton");
                Assert.Equal((true, (object?)"Start PlayStation"), (button.IsVisible, button.Content));
                Field<AppSettings>(window, "_appSettings").SelectedCore = CoreCatalog.ShelvesInReleaseOrder.First(s => s.Core.Console == "SNES").Name;
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
                Assert.Equal(new[] { "Insert Disc..." }, Rows(Call(window, "DriveRows")));
                Assert.Contains("Insert Disc...", Rows(Call(window, "GameMenuRows")));
                Assert.Equal("psx", MistressGame.Engine(window)!.StartedEmpty);
            }
            finally
            {
                typeof(MainWindow).GetMethod("ShowLibrary", Hidden)!.Invoke(window, null);
                window.Close();
            }
        }, default);
    }
}

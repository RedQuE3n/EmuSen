using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // A console's page in Settings: a row per console in the Consoles section, the N64's rows showing the config's values, and a change saved at once - see EmuSen_BigPicture.md §52.
    [Collection(TestCollections.ProcessGlobals)]
    public class ConsoleSettingsPageTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ConsoleSettingsPageTests).GetTypeInfo().Assembly);

        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenConsolePage_" + Guid.NewGuid().ToString("N"));

        public ConsoleSettingsPageTests()
        {
            ConfigStore.OverrideDirectory = _dir;
            DataStore.OverrideDirectory = Path.Combine(_dir, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        [Fact]
        public Task A_row_per_console_and_each_runs_one_opens_its_page() => Session.Dispatch(() =>
        {
            MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles);
            Assert.Equal(RuntimeCatalog.Systems.Select(s => s.Console).Distinct(StringComparer.OrdinalIgnoreCase).Select(c => "console:" + c).Order(), SettingsDesk.Keys(window).Where(k => k.StartsWith("console:", StringComparison.Ordinal)).Order());
            SettingsDesk.Choose(window, "console:N64");
            Assert.Contains("RenderScale", SettingsDesk.Keys(window));
            window.Close();
        }, default);

        // A page longer than the window: its last row is reached by the keys, as a pad does.
        [Fact]
        public Task A_page_taller_than_the_window_has_its_last_row_reached() => Session.Dispatch(() =>
        {
            MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:N64");
            string last = SettingsDesk.Keys(window)[^1];
            SettingsDesk.To(window, last);
            Assert.Equal("reset", last);
            window.Close();
        }, default);

        [Fact]
        public Task The_N64s_rows_show_the_configs_values_and_the_defaults_where_it_has_none() => Session.Dispatch(() =>
        {
            var consoles = new ConsolesConfig();
            consoles.SetValue("N64", "RdpWorkers", "2");
            consoles.SetValue("N64", "SkipRepeatedScans", "false");
            consoles.Save();

            MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:N64");
            Assert.Equal("2", SettingsDesk.Value(window, "RdpWorkers"));
            Assert.Equal(SettingsScreen.Off, SettingsDesk.Value(window, "SkipRepeatedScans"));
            Assert.Equal(SettingsScreen.On, SettingsDesk.Value(window, "ThreadedRdp"));
            Assert.Equal(SettingsScreen.On, SettingsDesk.Value(window, "DeferredPresentation"));
            window.Close();
        }, default);

        [Fact]
        public Task A_change_is_saved_at_once_in_the_consoles_settings_and_not_the_graphics() => Session.Dispatch(() =>
        {
            MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:N64");
            SettingsDesk.Pick(window, "ThreadedRdp", SettingsScreen.Off);
            SettingsDesk.Pick(window, "RdpWorkers", "5");

            Assert.Equal("false", ConsolesConfig.Load().Value("N64", "ThreadedRdp"));
            Assert.Equal("5", ConsolesConfig.Load().Value("N64", "RdpWorkers"));
            Assert.Null(GraphicsConfig.Load().Value("N64", "RdpWorkers"));
            window.Close();
        }, default);

        // The Antialiasing row's note is MarsRT's, read from a running machine through SETTING_NOTES; with no game running the page offers the row and no note - see EmuSen_Backend.md §20 (Q15).
        [Fact]
        public Task With_no_game_running_the_antialiasing_row_has_no_note() => Session.Dispatch(() =>
        {
            var consoles = new ConsolesConfig();
            consoles.SetValue("N64", "RenderScale", "3");
            consoles.SetValue("N64", "Antialiasing", "3x");
            consoles.Save();
            MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:N64");
            SettingRow row = RuntimeCatalog.SettingsFor("N64").Single(r => r.Key == "Antialiasing");
            Assert.Contains("Antialiasing", SettingsDesk.Keys(window));
            Assert.Equal(row.Hint, SettingsDesk.Row(window, "Antialiasing").Sentence);
            window.Close();
        }, default);
    }
}

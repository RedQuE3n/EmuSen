using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.LogicalTree;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The screen filter, one per console, chosen in the graphics window and drawn by the frame control - see EmuSen_Settings_Reference.md §4.40.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScreenFilterSettingTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScreenFilterSettingTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScreenFilterTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public ScreenFilterSettingTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "Game.sfc");
            File.WriteAllBytes(_rom, SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static Dropdown Filter(Window w, string console) =>
            w.GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == $"{console}.{GraphicsSettingsWindow.ScreenFilterKey}").Distinct().Single();

        [Fact]
        public Task Every_console_s_tab_offers_the_filters_and_a_choice_is_saved_for_that_console_alone() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            string? told = null;
            var window = new GraphicsSettingsWindow(config, console => told = console, null);
            window.Show();

            foreach (string console in CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console))
            {
                Dropdown filter = Filter(window, console);
                Assert.Equal(ScreenFilters.Names, filter.Items.Cast<object>().Select(o => o.ToString()));
                Assert.Equal(ScreenFilters.None, filter.SelectedItem);
            }

            Filter(window, "SNES").SelectedItem = "Scanlines";
            Dispatcher.UIThread.RunJobs();

            Assert.Equal("SNES", told);
            Assert.Equal("Scanlines", GraphicsConfig.Load().Value("SNES", GraphicsSettingsWindow.ScreenFilterKey));
            Assert.Null(GraphicsConfig.Load().Value("NES", GraphicsSettingsWindow.ScreenFilterKey));
            window.Close();
        }, default);

        [Fact]
        public Task A_game_starts_with_its_console_s_filter_and_a_change_reaches_it_at_once() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("SNES", GraphicsSettingsWindow.ScreenFilterKey, "Simple CRT");
            config.SetValue("NES", GraphicsSettingsWindow.ScreenFilterKey, "Scanlines");
            config.Save();

            var window = new MainWindow();
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!
                .Invoke(window, new object[] { _rom, "Game.sfc" });
            GameFrameControl frame = window.GetControl<GameFrameControl>("GameFrame");
            Assert.Equal(ShaderEffect.Crt, frame.ActiveEffect);

            typeof(MainWindow).GetMethod("ShowGraphicsSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            GraphicsSettingsWindow settings = window.OwnedWindows.OfType<GraphicsSettingsWindow>().Single();
            Filter(settings, "NES").SelectedItem = ScreenFilters.None;
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(ShaderEffect.Crt, frame.ActiveEffect);

            Filter(settings, "SNES").SelectedItem = ScreenFilters.None;
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(ShaderEffect.None, frame.ActiveEffect);
            settings.Close();
            window.Close();
        }, default);

        [Theory]
        [InlineData(null)]
        [InlineData("A filter from a newer build")]
        public void An_unknown_filter_name_draws_no_filter(string? name)
        {
            Assert.Equal(ShaderEffect.None, ScreenFilters.ByName(name));
        }
    }
}

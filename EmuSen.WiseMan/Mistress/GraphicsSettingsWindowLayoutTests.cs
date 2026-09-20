using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress
{
    // One tab per console, the N64's controls showing the config's values, and a change saved and reported - see EmuSen_Settings_Reference.md §4.26.
    public class GraphicsSettingsWindowLayoutTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GraphicsSettingsWindowLayoutTests).GetTypeInfo().Assembly);

        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenGraphics_" + Guid.NewGuid().ToString("N"));

        public GraphicsSettingsWindowLayoutTests() => ConfigStore.OverrideDirectory = _dir;

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private static T ByName<T>(Window w, string name) where T : Control =>
            w.GetVisualDescendants().OfType<T>().First(c => c.Name == name);

        [Fact]
        public Task A_tab_per_console_in_release_order_and_the_running_consoles_tab_selected() => Session.Dispatch(() =>
        {
            var window = new GraphicsSettingsWindow(new GraphicsConfig(), null, "N64");
            window.Show();
            window.CaptureRenderedFrame();

            var tabs = ByName<Tabs>(window, "ConsoleTabs");
            Assert.Equal(CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console), tabs.Items.Cast<TabItem>().Select(t => (string)t.Header!));
            Assert.Equal("N64", (string)((TabItem)tabs.Items[tabs.SelectedIndex]!).Header!);
        }, default);

        [Fact]
        public Task The_N64s_controls_show_the_configs_values_and_the_defaults_where_it_has_none() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("N64", "RdpWorkers", "2");
            config.SetValue("N64", "SkipRepeatedScans", "false");

            var window = new GraphicsSettingsWindow(config, null, "N64");
            window.Show();
            window.CaptureRenderedFrame();

            Assert.Equal("2", ByName<Dropdown>(window, "N64.RdpWorkers").SelectedItem);
            Assert.False(ByName<LunaSwitch>(window, "N64.SkipRepeatedScans").IsChecked);
            Assert.True(ByName<LunaSwitch>(window, "N64.ThreadedRdp").IsChecked);
            Assert.True(ByName<LunaSwitch>(window, "N64.DeferredPresentation").IsChecked);
        }, default);

        [Fact]
        public Task A_change_is_saved_at_once_and_the_console_reported() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            var reported = new List<string>();
            var window = new GraphicsSettingsWindow(config, reported.Add, "N64");
            window.Show();
            window.CaptureRenderedFrame();

            ByName<LunaSwitch>(window, "N64.ThreadedRdp").IsChecked = false;
            ByName<Dropdown>(window, "N64.RdpWorkers").SelectedItem = "5";

            Assert.Equal("false", config.Value("N64", "ThreadedRdp"));
            Assert.Equal("5", config.Value("N64", "RdpWorkers"));
            Assert.Equal(new[] { "N64", "N64" }, reported);
            Assert.Equal("5", GraphicsConfig.Load().Value("N64", "RdpWorkers"));
        }, default);
    }
}

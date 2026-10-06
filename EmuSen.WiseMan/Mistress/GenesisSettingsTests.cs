using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The Genesis's settings through the per-console path: its pack names the console, its engine's schema gives the rows, and a stored value reaches the machine at create - see EmuSen_Settings_Reference.md §4.90.
    [Collection(TestCollections.ProcessGlobals)]
    public class GenesisSettingsTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GenesisSettingsTests).GetTypeInfo().Assembly);

        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenGenesisSettings_" + Guid.NewGuid().ToString("N"));

        public GenesisSettingsTests()
        {
            Directory.CreateDirectory(_dir);
            ConfigStore.OverrideDirectory = _dir;
            DataStore.OverrideDirectory = Path.Combine(_dir, "Home");
            CoreDiscovery.UseDevelopment(true);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private static T ByName<T>(Window w, string name) where T : Control =>
            w.GetVisualDescendants().OfType<T>().First(c => c.Name == name);

        [Fact]
        public void The_packs_name_the_consoles_only_the_discovered_engine_runs()
        {
            Assert.Equal(new[] { "Genesis", "Sega CD", "32X" }, CoreCatalog.DiscoveredConsoles);
            Assert.Equal(("Genesis", "Sega CD", "32X"), (CoreCatalog.DiscoveredConsoleForSystem("md"), CoreCatalog.DiscoveredConsoleForSystem("mcd"), CoreCatalog.DiscoveredConsoleForSystem("32x")));
            Assert.Null(CoreCatalog.DiscoveredConsoleForSystem("snes"));
            CoreDiscovery.UseDevelopment(false);
            Assert.Empty(CoreCatalog.DiscoveredConsoles);
        }

        [Fact]
        public void The_genesis_offers_its_model_region_and_two_pads_in_words()
        {
            var settings = CoreCatalog.SettingsFor("Genesis");
            Assert.Equal(new[] { "model", "region", "pad1", "pad2" }, settings.Select(s => s.Key));
            var region = settings.Single(s => s.Key == "region");
            Assert.Equal(new[] { "auto", "md.us", "md.eu", "md.jp", "md.asia" }, region.Choices);
            Assert.Equal(new[] { "From the cartridge", "Americas (NTSC)", "Europe (PAL)", "Japan (NTSC)", "Asia (PAL)" }, region.ChoiceLabels);
            Assert.Equal(new[] { "3-Button Control Pad", "6-Button Arcade Pad" }, settings.Single(s => s.Key == "pad1").ChoiceLabels);
            Assert.Equal("md.model1", settings.Single(s => s.Key == "model").Default);
        }

        [Fact]
        public Task The_genesis_tab_shows_each_choice_in_words_and_stores_its_value() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("Genesis", "pad2", "md.pad6");
            string? changed = null;
            var window = new GraphicsSettingsWindow(config, c => changed = c, "Genesis");
            window.Show();
            window.CaptureRenderedFrame();

            Assert.Equal("Genesis", (string)((TabItem)ByName<Tabs>(window, "ConsoleTabs").Items[ByName<Tabs>(window, "ConsoleTabs").SelectedIndex]!).Header!);
            Assert.Equal("From the cartridge", ByName<Dropdown>(window, "Genesis.region").SelectedItem);
            Assert.Equal("6-Button Arcade Pad", ByName<Dropdown>(window, "Genesis.pad2").SelectedItem);
            ByName<Dropdown>(window, "Genesis.region").SelectedItem = "Europe (PAL)";
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(("md.eu", "Genesis"), (config.Value("Genesis", "region"), changed));
            window.Close();
        }, default);

        [Fact]
        public void A_stored_region_model_and_pad_reach_the_machine_when_it_is_made()
        {
            var config = new GraphicsConfig();
            config.SetValue("Genesis", "region", "md.eu");
            config.SetValue("Genesis", "pad1", "md.pad6");
            config.Save();
            string rom = Path.Combine(_dir, "game.md");
            File.WriteAllBytes(rom, SyntheticMdRom.Cartridge(region: "U"));
            var bundle = CoreFactory.Load(rom);
            var core = Assert.IsType<CoreEngine>(bundle.Core);
            Assert.Equal("pal", core.Machine.Info.Region);
            Assert.InRange(core.FrameRateHz, 49.70, 49.71);
            Assert.Equal("md.pad6", core.Machine.Info.Ports[0].Controller);
            Assert.Equal(("md.eu", "md.pad3"), (core.Get("region"), core.Get("pad2")));
            core.Dispose();
        }
    }
}

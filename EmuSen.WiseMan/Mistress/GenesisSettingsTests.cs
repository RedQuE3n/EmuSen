using System;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Mistress.Library;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Mistress.Pages;

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
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private static T ByName<T>(Window w, string name) where T : Control =>
            w.GetVisualDescendants().OfType<T>().First(c => c.Name == name);

        [Fact]
        public void The_packs_name_the_consoles_only_the_discovered_engine_runs_and_its_systems_in_development_have_no_tab()
        {
            Assert.Equal(new[] { "SG-1000", "Master System", "Genesis", "Sega CD", "PlayStation", "PlayStation 2" }, RuntimeCatalog.ConsoleNames.Where(c => c is not ("NES" or "GB" or "SNES" or "N64")));
            Assert.Equal(("Genesis", "Sega CD", "32X"), (RuntimeCatalog.ConsoleOfSystem("md"), RuntimeCatalog.ConsoleOfSystem("mcd"), RuntimeCatalog.ConsoleOfSystem("32x")));
        }

        [Fact]
        public void The_genesis_offers_its_model_region_and_two_pads_in_words()
        {
            var settings = RuntimeCatalog.SettingsFor("Genesis");
            // The console's page lists the engine's choice first and its save media after; the engine's own rows are between.
            Assert.Equal(new[] { EngineNames.EngineKey, "model", "region", "pad1", "pad2" }, settings.Select(s => s.Key).Take(5));
            var region = settings.Single(s => s.Key == "region");
            Assert.Equal(new[] { "auto", "md.us", "md.eu", "md.jp", "md.asia" }, region.Choices);
            Assert.Equal(new[] { "From the cartridge", "Americas (NTSC)", "Europe (PAL)", "Japan (NTSC)", "Asia (PAL)" }, region.ChoiceLabels);
            string[] pads = { "3-Button Control Pad", "6-Button Arcade Pad", "Team Player with 3-Button Control Pads", "Team Player with 6-Button Arcade Pads" };
            Assert.Equal(pads.Concat(new[] { "4 Way Play with 3-Button Control Pads", "4 Way Play with 6-Button Arcade Pads" }), settings.Single(s => s.Key == "pad1").ChoiceLabels);
            Assert.Equal(pads, settings.Single(s => s.Key == "pad2").ChoiceLabels);
            Assert.Equal("md.model1", settings.Single(s => s.Key == "model").Default);
        }

        [Fact]
        public Task The_genesis_page_shows_each_choice_in_words_and_stores_its_value() => Session.Dispatch(() =>
        {
            var consoles = new ConsolesConfig();
            consoles.SetValue("Genesis", "pad2", "md.pad6");
            consoles.Save();
            MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:Genesis");

            Assert.Equal("From the cartridge", SettingsDesk.Value(window, "region"));
            Assert.Equal("6-Button Arcade Pad", SettingsDesk.Value(window, "pad2"));
            SettingsDesk.Pick(window, "region", "Europe (PAL)");
            Assert.Equal("md.eu", ConsolesConfig.Load().Value("Genesis", "region"));
            window.Close();
        }, default);

        [Fact]
        public void A_stored_region_model_and_pad_reach_the_machine_when_it_is_made()
        {
            var config = new ConsolesConfig();
            config.SetValue("Genesis", "region", "md.eu");
            config.SetValue("Genesis", "pad1", "md.pad6");
            config.Save();
            string rom = Path.Combine(_dir, "game.md");
            File.WriteAllBytes(rom, SyntheticMdRom.Cartridge(region: "U"));
            using var runtime = new CrystalRuntime();
            using CrystalSession session = runtime.Open(new CrystalOpen { Path = rom, NoBattery = true });
            var info = new CrystalFrameInfo();
            session.Frame(ref info);
            Assert.InRange(info.frame_rate_hz, 49.70, 49.71);
            using var shown = JsonDocument.Parse(session.Info());
            JsonElement ports = shown.RootElement.GetProperty("ports");
            Assert.Equal(("md.pad6", "md.pad3"), (ports[0].GetProperty("controller").GetString(), ports[1].GetProperty("controller").GetString()));
            Assert.Equal("md.eu", session.Setting("region"));
        }
    }
}

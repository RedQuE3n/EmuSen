using System.Collections.Generic;
using System.Linq;
using Avalonia.VisualTree;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The PlayStation 2 tab from a hosted core's options schema, its categories as sections, with the CRT filter's held rows locked - see EmuSen_LibretroHost.md §10i.
    public partial class WindowFitAuditTests
    {
        // The rows a running hosted PlayStation 2 game gives with console_picture on, read from the test core's session through the C# frontend API.
        internal static IReadOnlyList<SettingRow> HostedRows()
        {
            string? config = ConfigStore.OverrideDirectory, data = DataStore.OverrideDirectory;
            try
            {
                using var entry = new LibretroEntryFixture(systemId: "playstation2");
                var disc = SyntheticPs2Disc.Dvd();
                "TCG1"u8.CopyTo(disc.Iso);
                using var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { System.AppContext.BaseDirectory }, PlayerCores = new[] { entry.Cores } });
                using CrystalSession s = runtime.Open(new CrystalOpen { Path = disc.WriteSet(entry.Root, "Hosted"), Engine = entry.Found.EngineName, NoBattery = true });
                s.SetSetting(ConsolePicture.Key, "true", game: false);
                return RuntimeCatalog.SettingsOf(s);
            }
            finally
            {
                ConfigStore.OverrideDirectory = config;
                DataStore.OverrideDirectory = data;
            }
        }

        // The hosted core's categories as the console page's headings in its order, every option a row, a held row shown with why and taking no choice, console_picture not drawn - see EmuSen_LibretroHost.md §10i.
        [Fact]
        public System.Threading.Tasks.Task A_hosted_cores_page_has_its_categories_as_headings_and_its_held_rows_said() => Session.Dispatch(() =>
        {
            ConfigStore.OverrideDirectory = System.IO.Path.Combine(_root, "Config");
            IReadOnlyList<SettingRow> rows = HostedRows();
            var graphics = new GraphicsConfig();
            graphics.SetValue("PlayStation 2", ConsolePicture.Key, "true");
            var host = new SettingsHostFake { Graphics = graphics, Rows = c => c == "PlayStation 2" ? rows : RuntimeCatalog.SettingsFor(c), RunningConsole = "PlayStation 2" };
            var screen = new EmuSen.Mistress.Pages.SettingsScreen(host, EmuSen.Mistress.Pages.SettingsScreen.Consoles);
            screen.OpenPath(EmuSen.Mistress.Pages.SettingsScreen.Consoles, "console:PlayStation 2");
            var page = screen.Rows;
            Assert.Equal(new[] { "Picture", "Sound", "Memory", "States", "Probe" }.Select(g => "group:" + g), page.Where(r => r.Key.StartsWith("group:", System.StringComparison.Ordinal)).Select(r => r.Key));
            Assert.All(page.Where(r => r.Key.StartsWith("group:", System.StringComparison.Ordinal)), r => Assert.False(r.IsEnabled));
            foreach (string key in new[] { "tc_format", "tc_rate", "tc_hw" }) Assert.True(page.Single(r => r.Key == key).IsEnabled, key);
            Assert.DoesNotContain(page, r => r.Key == ConsolePicture.Key);
            var held = page.Single(r => r.Key == "tc_deinterlace");
            Assert.False(held.IsEnabled);
            Assert.Contains("Held at off (label) while the CRT filter is on", held.Sentence);
        }, default);
    }
}

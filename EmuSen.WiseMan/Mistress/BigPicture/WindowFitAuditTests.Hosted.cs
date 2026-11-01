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

        // The hosted core's categories as the tab's sections in its order, every option a row, the held rows and console_picture locked.
        [Fact]
        public System.Threading.Tasks.Task A_hosted_cores_tab_has_its_categories_as_sections_and_its_held_rows_locked() => Session.Dispatch(() =>
        {
            ConfigStore.OverrideDirectory = System.IO.Path.Combine(_root, "Config");
            IReadOnlyList<SettingRow> rows = HostedRows();
            var config = new GraphicsConfig();
            config.SetValue("PlayStation 2", ConsolePicture.Key, "true");
            var sheet = new GraphicsSettingsWindow(config, null, "PlayStation 2", runningSettings: c => c == "PlayStation 2" ? rows : null);
            try
            {
                sheet.Show();
                Settle(sheet);
                var controls = sheet.GetVisualDescendants().OfType<Avalonia.Controls.Control>().ToArray();
                string[] sections = controls.Select(c => c.Name ?? "").Where(n => n.StartsWith("PlayStation 2.Group.", System.StringComparison.Ordinal)).ToArray();
                Assert.Equal(new[] { "Picture", "Sound", "Memory", "States", "Probe" }.Select(g => $"PlayStation 2.Group.{g}"), sections);
                foreach (string key in new[] { "tc_format", "tc_rate", "tc_hw", ConsolePicture.Key })
                    Assert.True(controls.Single(c => c.Name == $"PlayStation 2.{key}").IsVisible == (key is "tc_format" or "tc_rate" or "tc_hw"), key);
                Assert.False(controls.Single(c => c.Name == "PlayStation 2.tc_deinterlace").IsVisible);
                Assert.Contains("Held at off (label) while the CRT filter is on", ((EmuSen.LunaP.Controls.HintText)controls.Single(c => c.Name == "PlayStation 2.tc_deinterlace.Note")).Text);
            }
            finally { sheet.Close(); }
        }, default);

        internal static void ShowHostedTab(MainWindow window)
        {
            IReadOnlyList<SettingRow> rows = HostedRows();
            Assert.Contains(rows, r => r.Key == "tc_scale" && r.Locked && r.Group == "Picture");
            var graphics = (GraphicsConfig)typeof(MainWindow).GetField("_graphics", Hidden)!.GetValue(window)!;
            var sheet = new GraphicsSettingsWindow(graphics, null, "PlayStation 2", runningSettings: c => c == "PlayStation 2" ? rows : null);
            _ = EmuSen.LunaP.Windowing.SheetLayer.Show(sheet, window);
            Settle(window);
            SelectTab(window, "PlayStation 2");
            Avalonia.Controls.Control scale = Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<Avalonia.Controls.Control>().Single(c => c.Name == "PlayStation 2.tc_scale");
            Assert.False(scale.IsVisible, "the held row is locked, its note alone shown");
        }
    }
}

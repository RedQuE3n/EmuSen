using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.VisualTree;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The library with the PlayStation shelf present, a lone .bin routed to it by its content, at 1280 by 800 and 1920 by 1200, with development cores shown since Diamond stays marked until G9 - see EmuSen_Settings_Reference.md §4.105.
    public partial class WindowFitAuditTests
    {
        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_sidebar_with_the_playstation_shelf_is_whole_and_nothing_on_it_overlaps(int width, int height) => Session.Dispatch(() =>
        {
            CoreDiscovery.UseDevelopment(true);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            File.WriteAllBytes(Path.Combine(_romDir, "Cobalt Harbor (Synthetic).sfc"), SyntheticRom.BuildBlank());
            File.WriteAllBytes(Path.Combine(_romDir, "Granite Choir (Synthetic).md"), SyntheticMdRom.Cartridge());
            File.WriteAllBytes(Path.Combine(_romDir, "Ruby Lantern (Synthetic).bin"), SyntheticPsxDisc.Build().DataTrack);
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
            var window = new MainWindow { Width = width, Height = height };
            try
            {
                window.Show();
                Settle(window);
                SourceList sidebar = window.GetControl<SourceList>("LibrarySidebar");
                string[] rows = sidebar.GetVisualDescendants().OfType<TextBlock>().Select(t => t.Text ?? "").ToArray();
                Assert.Contains("PlayStation", rows);
                List<string> faults = FitAudit.Check(sidebar, null, smallest: 0);
                foreach (string f in faults) _out.WriteLine($"Sidebar-PlayStation-{width}x{height}: {f}");
                SavePicture(window, $"Sidebar-PlayStation-{width}x{height}");
                Assert.Empty(faults);
            }
            finally
            {
                window.Close();
                CoreDiscovery.UseDevelopment(null);
            }
        }, default);

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_system_carousel_with_the_playstation_shelf_is_whole_on_every_system(int width, int height) => Session.Dispatch(() =>
        {
            CoreDiscovery.UseDevelopment(true);
            try
            {
                using var theme = new SyntheticTheme();
                ThemedSession.Write(theme);
                File.Copy(SceneAssets.Halves("session-psx", 120, 240, Avalonia.Media.Colors.DarkSlateGray, Avalonia.Media.Colors.LightSteelBlue), theme.PathOf("art/psx.png"), overwrite: true);
                using var s = new ThemedSession(width, height, themeDirectory: theme.Root, roms: dir => File.WriteAllBytes(Path.Combine(dir, "Ruby Lantern (Synthetic).bin"), SyntheticPsxDisc.Build().DataTrack));
                var seen = new List<string>();
                var faults = new List<string>();
                for (int i = 0; i < 6 && !seen.Contains(s.System ?? ""); i++)
                {
                    seen.Add(s.System ?? "");
                    Assert.Equal("system", s.View);
                    foreach (string f in FitAudit.Check(s.Themed.Root, null, smallest: 0)) faults.Add($"{s.System}: {f}");
                    SavePicture(s.Window, $"Carousel-{s.System}-{width}x{height}");
                    s.Pad.Right();
                    s.Run(600);
                    s.Settle();
                }
                foreach (string f in faults) _out.WriteLine($"Carousel-PlayStation-{width}x{height}: {f}");
                Assert.Contains("psx", seen);
                Assert.Empty(faults);
            }
            finally
            {
                CoreDiscovery.UseDevelopment(null);
            }
        }, default);
    }
}

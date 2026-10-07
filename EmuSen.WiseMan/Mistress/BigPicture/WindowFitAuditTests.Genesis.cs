using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Threading;
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
    // The library with the Genesis shelf present, at 1280 by 800 and 1920 by 1200: the desktop's sidebar and big picture's system carousel - see EmuSen_Settings_Reference.md §4.92.
    public partial class WindowFitAuditTests
    {
        public static TheoryData<int, int> Sizes()
        {
            var data = new TheoryData<int, int>();
            foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(width, height);
            return data;
        }

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_sidebar_with_the_genesis_shelf_is_whole_and_nothing_on_it_overlaps(int width, int height) => Session.Dispatch(() =>
        {
            CoreDiscovery.UseDevelopment(false);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            File.WriteAllBytes(Path.Combine(_romDir, "Cobalt Harbor (Synthetic).sfc"), SyntheticRom.BuildBlank());
            File.WriteAllBytes(Path.Combine(_romDir, "Granite Choir (Synthetic).md"), SyntheticMdRom.Cartridge());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
            var window = new MainWindow { Width = width, Height = height };
            try
            {
                window.Show();
                Settle(window);
                SourceList sidebar = window.GetControl<SourceList>("LibrarySidebar");
                string[] rows = sidebar.GetVisualDescendants().OfType<TextBlock>().Select(t => t.Text ?? "").ToArray();
                Assert.Contains("Genesis", rows);
                List<string> faults = FitAudit.Check(sidebar, null, smallest: 0);
                foreach (string f in faults) _out.WriteLine($"Sidebar-Genesis-{width}x{height}: {f}");
                SavePicture(window, $"Sidebar-Genesis-{width}x{height}");
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
        public Task The_system_carousel_with_the_genesis_shelf_is_whole_on_every_system(int width, int height) => Session.Dispatch(() =>
        {
            CoreDiscovery.UseDevelopment(false);
            try
            {
                // The session's own theme with a picture for the genesis system too, as a theme that knows the console has.
                using var theme = new SyntheticTheme();
                ThemedSession.Write(theme);
                File.Copy(SceneAssets.Halves("session-genesis", 120, 240, Avalonia.Media.Colors.MidnightBlue, Avalonia.Media.Colors.DodgerBlue), theme.PathOf("art/genesis.png"), overwrite: true);
                using var s = new ThemedSession(width, height, themeDirectory: theme.Root, roms: dir => File.WriteAllBytes(Path.Combine(dir, "Granite Harbor (Synthetic).md"), SyntheticMdRom.Cartridge()));
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
                foreach (string f in faults) _out.WriteLine($"Carousel-Genesis-{width}x{height}: {f}");
                Assert.Contains("genesis", seen);
                Assert.Empty(faults);
            }
            finally
            {
                CoreDiscovery.UseDevelopment(null);
            }
        }, default);
    }
}

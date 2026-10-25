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
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The library with packages beside it at 1280 by 800 and 1920 by 1200: a packaged game listed once, a converted disc's package as its game, nothing cut off - see EmuSen_GamePackages.md §3.6 and §14.
    public partial class WindowFitAuditTests
    {
        private const string PackagedTitle = "Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)";

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_library_with_packages_lists_each_game_once_and_nothing_is_cut_off(int width, int height) => Session.Dispatch(() =>
        {
            CoreDiscovery.UseDevelopment(false);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            File.WriteAllBytes(Path.Combine(_romDir, "Cobalt Harbor (Synthetic).sfc"), SyntheticRom.BuildBlank());
            Directory.CreateDirectory(Path.Combine(_romDir, "nes"));
            File.WriteAllBytes(Path.Combine(_romDir, "nes", PackagedTitle + ".nes"), SyntheticNesRom.Build());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
            CrystalParityTests.PackageFor(Path.Combine(DataStore.Packages, "NES", PackagedTitle + ".nes"), _romDir, $"nes/{PackagedTitle}.nes", "nes");
            string chd = Path.Combine(DataStore.Compressed, "psx", "Ruby Lantern (Synthetic).chd");
            Directory.CreateDirectory(Path.GetDirectoryName(chd)!);
            File.WriteAllBytes(chd, new byte[64]);
            string disc = Path.Combine(DataStore.Packages, "PlayStation", "Ruby Lantern (Synthetic).chd");
            Directory.CreateDirectory(disc);
            File.WriteAllText(Path.Combine(disc, "package.json"), "{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"psx\", \"mode\": \"linked\", \"game\": {\"role\": \"image\", \"where\": \"compressed\", \"path\": \"psx/Ruby Lantern (Synthetic).chd\"}}");
            var window = new MainWindow { Width = width, Height = height };
            try
            {
                window.Show();
                Settle(window);
                var list = (LunaList<RomEntry>)window.GetControl<ListBox>("LibraryList");
                for (int i = 0; i < 400 && list.Models.Count() < 3; i++)
                {
                    Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                    System.Threading.Thread.Sleep(25);
                }
                Settle(window);
                List<RomEntry> entries = list.Models.ToList();
                Assert.Equal(3, entries.Count);
                Assert.Single(entries, e => e.Title == PackagedTitle && e.Package is not null);
                Assert.Single(entries, e => e.FullPath == chd && e.Shelf.StartsWith("PlayStation", StringComparison.Ordinal));
                var faults = new List<string>();
                faults.AddRange(FitAudit.Check(list, null, smallest: 0).Select(f => $"list: {f}"));
                faults.AddRange(FitAudit.Check(window.GetControl<SourceList>("LibrarySidebar"), null, smallest: 0).Select(f => $"sidebar: {f}"));
                foreach (string f in faults) _out.WriteLine($"Library-Packages-{width}x{height}: {f}");
                SavePicture(window, $"Library-Packages-{width}x{height}");
                Assert.Empty(faults);
            }
            finally
            {
                window.Close();
                CoreDiscovery.UseDevelopment(null);
            }
        }, default);
    }
}

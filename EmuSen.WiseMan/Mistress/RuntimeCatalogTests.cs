using System.IO;
using System.Linq;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Mistress's catalog read from the frontend API holds the facts the C# catalog answered, recorded before it was deleted - see EmuSen_Backend.md §14.13 and §40.10.
    [Collection(TestCollections.ProcessGlobals)]
    public class RuntimeCatalogTests
    {
        public RuntimeCatalogTests() => RuntimeCatalog.Forget();

        [Fact]
        public void The_shelves_and_consoles_are_the_catalog_s()
        {
            var golden = new ParityGolden("RuntimeCatalog.Shelves");
            golden.Add("shelves", string.Join("|", RuntimeCatalog.Shelves.Select(s => s.Name)));
            golden.Add("consoles", string.Join("|", RuntimeCatalog.ConsoleNames));
            golden.Add("display names", string.Join("|", RuntimeCatalog.Consoles.Select(c => c.DisplayName)));
            golden.Add("filter choices", string.Join("|", RuntimeCatalog.FilterChoices));
            golden.Add("extensions", string.Join("|", RuntimeCatalog.RomExtensions.Order(System.StringComparer.Ordinal)));
            golden.Add("cheat systems", string.Join("|", RuntimeCatalog.SupportedCheatSystems.Order(System.StringComparer.Ordinal)));
            golden.Check(least: 6);
        }

        [Fact]
        public void Each_console_s_host_pad_and_empty_start_are_the_catalog_s()
        {
            var golden = new ParityGolden("RuntimeCatalog.Consoles");
            foreach (string console in RuntimeCatalog.ConsoleNames)
            {
                golden.Add($"{console} host", RuntimeCatalog.HostOf(console));
                golden.Add($"{console} controls", string.Join("|", RuntimeCatalog.ControlsFor(console)));
                golden.Add($"{console} starts empty", RuntimeCatalog.StartsEmpty(console).ToString());
            }
            golden.Add("consoles that start empty", string.Join("|", RuntimeCatalog.ConsolesThatStartEmpty));
            golden.Check(least: 3 * RuntimeCatalog.ConsoleNames.Count() + 1);
        }

        // The N64's pad, read from MarsRT's sidecar since §18.10, keeps both sticks in Mistress's bindings.
        [Fact]
        public void An_n64_pad_lists_its_sticks()
        {
            Assert.Contains(EmuSen.Galaxia.Input.PadAxis.LeftX, RuntimeCatalog.AxesFor("N64"));
            Assert.Contains(EmuSen.Galaxia.Input.PadControl.LeftStickUp, RuntimeCatalog.ControlsFor("N64"));
            Assert.Contains(EmuSen.Galaxia.Input.PadControl.RightStickLeft, RuntimeCatalog.ControlsFor("N64"));
        }

        [Fact]
        public void A_file_s_console_and_shelf_are_routing_s()
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenRuntimeCatalog_" + System.Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            try
            {
                string snes = Path.Combine(dir, "A.sfc"), nes = Path.Combine(dir, "B.nes"), gbc = Path.Combine(dir, "C.gbc");
                File.WriteAllBytes(snes, SyntheticRom.BuildBlank());
                File.WriteAllBytes(nes, SyntheticNesRom.Build());
                File.WriteAllBytes(gbc, new byte[0x8000]);
                var golden = new ParityGolden("RuntimeCatalog.Routes");
                foreach (string file in new[] { snes, nes, gbc })
                {
                    string name = Path.GetFileName(file);
                    golden.Add($"{name} console", RuntimeCatalog.ConsoleForRom(file) ?? "(none)");
                    golden.Add($"{name} shelf", RuntimeCatalog.ShelfFor(file) ?? "(none)");
                    golden.Add($"{name} core", RuntimeCatalog.CoreForRom(file)?.DisplayName ?? "(none)");
                }
                golden.Check(least: 9);
            }
            finally { Directory.Delete(dir, recursive: true); }
        }
    }
}

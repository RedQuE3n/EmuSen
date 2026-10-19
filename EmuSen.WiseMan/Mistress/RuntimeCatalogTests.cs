using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Mistress's catalog read from the frontend API holds the facts the catalog behind it answers - see EmuSen_Backend.md §14.13.
    [Collection(TestCollections.ProcessGlobals)]
    public class RuntimeCatalogTests
    {
        public RuntimeCatalogTests() => RuntimeCatalog.Forget();

        [Fact]
        public void The_shelves_and_consoles_are_the_catalog_s()
        {
            Assert.Equal(CoreCatalog.ShelvesInReleaseOrder.Select(s => s.Name), RuntimeCatalog.Shelves.Select(s => s.Name));
            Assert.Equal(CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console), RuntimeCatalog.ConsoleNames);
            Assert.Equal(CoreCatalog.ConsolesInReleaseOrder.Select(c => c.DisplayName), RuntimeCatalog.Consoles.Select(c => c.DisplayName));
            Assert.Equal(new[] { RuntimeCatalog.AllConsoles }.Concat(CoreCatalog.ShelvesInReleaseOrder.Select(s => s.Name)), RuntimeCatalog.FilterChoices);
            Assert.Equal(CoreCatalog.RomExtensions.Order(System.StringComparer.Ordinal), RuntimeCatalog.RomExtensions.Order(System.StringComparer.Ordinal));
            Assert.Equal(CoreCatalog.SupportedCheatSystems.Order(System.StringComparer.Ordinal), RuntimeCatalog.SupportedCheatSystems.Order(System.StringComparer.Ordinal));
        }

        [Fact]
        public void Each_console_s_host_pad_and_empty_start_are_the_catalog_s()
        {
            foreach (string console in RuntimeCatalog.ConsoleNames)
            {
                Assert.Equal(CoreCatalog.HostOf(console), RuntimeCatalog.HostOf(console));
                Assert.Equal(CoreCatalog.ControlsFor(console), RuntimeCatalog.ControlsFor(console));
                Assert.Equal(CoreCatalog.EmptyStartFor(console) is not null, RuntimeCatalog.StartsEmpty(console));
            }
            Assert.Equal(CoreCatalog.ConsolesThatStartEmpty.Select(e => e.Console), RuntimeCatalog.ConsolesThatStartEmpty);
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
                foreach (string file in new[] { snes, nes, gbc })
                {
                    Assert.Equal(CoreCatalog.ConsoleForRom(file), RuntimeCatalog.ConsoleForRom(file));
                    Assert.Equal(CoreCatalog.ShelfFor(file), RuntimeCatalog.ShelfFor(file));
                    Assert.Equal(CoreCatalog.CoreForRom(file)?.DisplayName, RuntimeCatalog.CoreForRom(file)?.DisplayName);
                }
            }
            finally { Directory.Delete(dir, recursive: true); }
        }
    }
}

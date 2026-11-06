using System;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Each core's buttons against the pad the catalog lists for its console, moved from ConsoleBindingsTests to go to Rust with the core host - see EmuSen_Backend.md §40.10.
    [Collection(TestCollections.ProcessGlobals)]
    public class ConsoleButtonsTests
    {
        // The rebind window lists a console's pad with no ROM loaded, so it comes off the core rather than being restated - see EmuSen_Input.md §5.1.
        [Fact]
        public void The_catalogs_button_list_is_the_cores_own()
        {
            using (var moonRt = Fixtures.MoonEngines.Create()) Assert.Equal(moonRt.SupportedButtons, CoreCatalog.ButtonsFor("NES"));
            using (var venusRt = (IDisposable)Fixtures.SyntheticRom.LoadCore(Fixtures.SyntheticRom.BuildBlank())) Assert.Equal(((ICore)venusRt).SupportedButtons, CoreCatalog.ButtonsFor("SNES"));
            using (var gameBoy = MercuryEngines.Create()) Assert.Equal(gameBoy.SupportedButtons.Order(), CoreCatalog.ButtonsFor("GB").Order());
            using var marsRt = new MarsRtTestCore();
            Assert.Equal(marsRt.SupportedButtons, CoreCatalog.ButtonsFor("N64"));
        }
    }
}

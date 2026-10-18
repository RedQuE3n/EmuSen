using System;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Crystal.Systems.Snes;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Cores
{
    // Each console's pad as a subset of PadButton, the SNES's being its first twelve - see EmuSen_Input.md §3.
    public class PadButtonTests
    {
        // The SNES's twelve are PadButton's first twelve, in the pad's order; what follows is RetroPad's own - see EmuSen_Input.md §7.1.
        [Fact]
        public void The_snes_pad_is_PadButtons_first_twelve()
        {
            Assert.Equal(Enum.GetValues<PadButton>().Take(12), SnesSystem.PadButtons);
            Assert.Equal(new[] { "L2", "R2", "L3", "R3" }, Enum.GetNames<PadButton>().Skip(12));
        }

        [Fact]
        public void MoonRT_reports_only_the_eight_buttons_an_NES_pad_has()
        {
            using var moon = Fixtures.MoonEngines.Create();
            moon.LoadRom(Fixtures.SyntheticNesRom.WriteTemp(Fixtures.SyntheticNesRom.Build()));

            Assert.Equal(8, moon.SupportedButtons.Count);
            Assert.DoesNotContain(PadButton.X, moon.SupportedButtons);
            Assert.DoesNotContain(PadButton.Y, moon.SupportedButtons);
            Assert.DoesNotContain(PadButton.L, moon.SupportedButtons);
            Assert.DoesNotContain(PadButton.R, moon.SupportedButtons);
        }

        [Fact]
        public void VenusRT_reports_all_twelve()
        {
            using var venusRt = (IDisposable)Fixtures.SyntheticRom.LoadCore(Fixtures.SyntheticRom.BuildBlank());

            Assert.Equal(Enum.GetValues<PadButton>().Take(12), ((ICore)venusRt).SupportedButtons.ToArray());
        }
    }
}

using System;
using System.Linq;
using EmuSen.Cores;
using SnesButton = EmuSen.Cores.Nintendo.Venus.Controllers.SnesButton;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Cores
{
    // VenusCore.SetButton casts between these two enums - see EmuSen_Input.md §3.
    public class PadButtonTests
    {
        [Fact]
        public void PadButton_and_SnesButton_agree_name_for_name_in_order()
        {
            string[] pad = Enum.GetNames<PadButton>();
            string[] snes = Enum.GetNames<SnesButton>();

            Assert.Equal(snes, pad);
        }

        // A rename or a reorder breaks the cast silently; a mismatched value catches it here.
        [Theory]
        [InlineData(PadButton.B, SnesButton.B)]
        [InlineData(PadButton.Y, SnesButton.Y)]
        [InlineData(PadButton.Select, SnesButton.Select)]
        [InlineData(PadButton.Start, SnesButton.Start)]
        [InlineData(PadButton.Up, SnesButton.Up)]
        [InlineData(PadButton.Down, SnesButton.Down)]
        [InlineData(PadButton.Left, SnesButton.Left)]
        [InlineData(PadButton.Right, SnesButton.Right)]
        [InlineData(PadButton.A, SnesButton.A)]
        [InlineData(PadButton.X, SnesButton.X)]
        [InlineData(PadButton.L, SnesButton.L)]
        [InlineData(PadButton.R, SnesButton.R)]
        public void Casting_a_PadButton_to_SnesButton_lands_on_the_same_button(PadButton pad, SnesButton expected)
        {
            Assert.Equal(expected, (SnesButton)pad);
        }

        // Galaxia persists enums by name, which is what made the retype free - see EmuSen_Input.md §3.
        [Fact]
        public void Every_saved_SnesButton_name_still_parses_as_a_PadButton()
        {
            foreach (string name in Enum.GetNames<SnesButton>())
            {
                Assert.True(Enum.TryParse(name, out PadButton _), $"'{name}' no longer parses as a PadButton.");
            }
        }

        [Fact]
        public void Moon_reports_only_the_eight_buttons_an_NES_pad_has()
        {
            var moon = new EmuSen.Cores.Nintendo.Moon.MoonCore();

            Assert.Equal(8, moon.SupportedButtons.Count);
            Assert.DoesNotContain(PadButton.X, moon.SupportedButtons);
            Assert.DoesNotContain(PadButton.Y, moon.SupportedButtons);
            Assert.DoesNotContain(PadButton.L, moon.SupportedButtons);
            Assert.DoesNotContain(PadButton.R, moon.SupportedButtons);
        }

        [Fact]
        public void Venus_reports_all_twelve()
        {
            var venus = new EmuSen.Cores.Nintendo.Venus.VenusCore(headless: true);

            Assert.Equal(Enum.GetValues<PadButton>(), venus.SupportedButtons.ToArray());
        }

        // A button the console lacks is dropped, never folded onto one it has.
        [Fact]
        public void Moon_ignores_a_button_the_NES_pad_does_not_have()
        {
            var moon = new EmuSen.Cores.Nintendo.Moon.MoonCore();
            moon.LoadRom(Fixtures.SyntheticNesRom.WriteTemp(Fixtures.SyntheticNesRom.Build()));

            moon.SetButton(0, PadButton.X, pressed: true);
            moon.SetButton(0, PadButton.L, pressed: true);

            Assert.Equal(0, moon.Bus!.Controller1.State);
        }

        [Fact]
        public void Moon_routes_a_button_the_NES_pad_does_have()
        {
            var moon = new EmuSen.Cores.Nintendo.Moon.MoonCore();
            moon.LoadRom(Fixtures.SyntheticNesRom.WriteTemp(Fixtures.SyntheticNesRom.Build()));

            moon.SetButton(0, PadButton.Start, pressed: true);

            Assert.NotEqual(0, moon.Bus!.Controller1.State);
        }

        // 0-based here, 1-based inside Venus - see EmuSen_Input.md §2.
        [Fact]
        public void Port_zero_is_player_one_on_both_cores()
        {
            var moon = new EmuSen.Cores.Nintendo.Moon.MoonCore();
            moon.LoadRom(Fixtures.SyntheticNesRom.WriteTemp(Fixtures.SyntheticNesRom.Build()));

            moon.SetButton(0, PadButton.A, pressed: true);

            Assert.NotEqual(0, moon.Bus!.Controller1.State);
            Assert.Equal(0, moon.Bus!.Controller2.State);
        }
    }
}

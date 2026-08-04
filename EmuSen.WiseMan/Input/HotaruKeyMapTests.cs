using Avalonia.Input;
using EmuSen.Cores;
using EmuSen.Hotaru.Input;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Input
{
    // HotaruKeyMap's default bindings - asserted directly against the
    // scheme the now-deleted EmuSen/Settings/InputBindings.cs used to
    // define (Z/X/A/S for B/A/Y/X, Q/W for L/R, Enter/RightShift for
    // Start/Select, arrow keys for the d-pad), since HotaruKeyMap is a
    // faithful port of that scheme onto Avalonia's Key enum rather than a
    // redesign.
    public class HotaruKeyMapTests
    {
        [Theory]
        [InlineData(PadButton.Up, Key.Up)]
        [InlineData(PadButton.Down, Key.Down)]
        [InlineData(PadButton.Left, Key.Left)]
        [InlineData(PadButton.Right, Key.Right)]
        [InlineData(PadButton.B, Key.Z)]
        [InlineData(PadButton.A, Key.X)]
        [InlineData(PadButton.Y, Key.A)]
        [InlineData(PadButton.X, Key.S)]
        [InlineData(PadButton.L, Key.Q)]
        [InlineData(PadButton.R, Key.W)]
        [InlineData(PadButton.Start, Key.Enter)]
        [InlineData(PadButton.Select, Key.RightShift)]
        public void Default_binding_matches_the_console_builds_old_scheme(PadButton button, Key expectedKey)
        {
            Assert.Equal(expectedKey, HotaruKeyMap.ButtonToKey[button]);
        }

        [Fact]
        public void Every_snes_button_has_exactly_one_binding()
        {
            Assert.Equal(System.Enum.GetValues<PadButton>().Length, HotaruKeyMap.ButtonToKey.Count);
        }

        [Theory]
        [InlineData(Key.Up, PadButton.Up)]
        [InlineData(Key.Z, PadButton.B)]
        [InlineData(Key.Enter, PadButton.Start)]
        public void Reverse_lookup_resolves_back_to_the_same_button(Key key, PadButton expectedButton)
        {
            Assert.True(HotaruKeyMap.TryGetButton(key, out PadButton button));
            Assert.Equal(expectedButton, button);
        }

        [Fact]
        public void Unbound_key_fails_reverse_lookup()
        {
            Assert.False(HotaruKeyMap.TryGetButton(Key.F12, out _));
        }
    }
}

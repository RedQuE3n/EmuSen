using Avalonia.Input;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Hotaru.Input;

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
        [InlineData(SnesButton.Up, Key.Up)]
        [InlineData(SnesButton.Down, Key.Down)]
        [InlineData(SnesButton.Left, Key.Left)]
        [InlineData(SnesButton.Right, Key.Right)]
        [InlineData(SnesButton.B, Key.Z)]
        [InlineData(SnesButton.A, Key.X)]
        [InlineData(SnesButton.Y, Key.A)]
        [InlineData(SnesButton.X, Key.S)]
        [InlineData(SnesButton.L, Key.Q)]
        [InlineData(SnesButton.R, Key.W)]
        [InlineData(SnesButton.Start, Key.Enter)]
        [InlineData(SnesButton.Select, Key.RightShift)]
        public void Default_binding_matches_the_console_builds_old_scheme(SnesButton button, Key expectedKey)
        {
            Assert.Equal(expectedKey, HotaruKeyMap.ButtonToKey[button]);
        }

        [Fact]
        public void Every_snes_button_has_exactly_one_binding()
        {
            Assert.Equal(System.Enum.GetValues<SnesButton>().Length, HotaruKeyMap.ButtonToKey.Count);
        }

        [Theory]
        [InlineData(Key.Up, SnesButton.Up)]
        [InlineData(Key.Z, SnesButton.B)]
        [InlineData(Key.Enter, SnesButton.Start)]
        public void Reverse_lookup_resolves_back_to_the_same_button(Key key, SnesButton expectedButton)
        {
            Assert.True(HotaruKeyMap.TryGetButton(key, out SnesButton button));
            Assert.Equal(expectedButton, button);
        }

        [Fact]
        public void Unbound_key_fails_reverse_lookup()
        {
            Assert.False(HotaruKeyMap.TryGetButton(Key.F12, out _));
        }
    }
}

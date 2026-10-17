using System;
using System.Collections.Generic;
using EmuSen.Endymion.Input;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Input;
using SDL3;

namespace EmuSen.WiseMan.Input
{
    // Player 1 to the first pad pressed in a game, an external pad ahead of a handheld's own, and the swap - see EmuSen_Input.md §8.13.
    public class PlayerClaimsTests
    {
        private sealed class Ports
        {
            public readonly Dictionary<(int Port, PadButton Button), bool> Buttons = new();
            public bool Held(int port, PadButton button) => Buttons.GetValueOrDefault((port, button));
        }

        private readonly bool[] _keys = new bool[Enum.GetValues<PadControl>().Length];

        // The Legion Go S's controls as SDL names them on the tester's handheld, and an Xbox Wireless Controller.
        private static SimulatedPad BuiltIn() => new() { Name = "Legion Go S", Vendor = 0x1A86, Product = 0xE310, Type = SDL.GamepadType.Xbox360 };
        private static SimulatedPad Xbox(string name = "Xbox Wireless Controller") => new() { Name = name, Vendor = 0x045E, Product = 0x0B13, Type = SDL.GamepadType.XboxOne };

        private (GamepadManager Manager, SimulatedPads Devices, PortRouter Router, Ports Out, List<PadConnection> Claims) Game(int ports, bool claim, bool externalFirst, params SimulatedPad[] pads)
        {
            SimulatedPads devices = SimulatedPads.With(pads);
            var manager = new GamepadManager(new GamepadBindingMap(), start: false, devices) { ClaimPlayersByPress = claim, ExternalControllerFirst = externalFirst };
            manager.Start();
            var output = new Ports();
            var router = new PortRouter(manager, (p, b, h) => output.Buttons[(p, b)] = h, (_, _, _) => { }, (_, _) => { }) { KeyboardHeld = c => _keys[(int)c] };
            var claims = new List<PadConnection>();
            manager.Claimed += claims.Add;
            router.Reset(ports, Array.Empty<PadAxis>());
            manager.BeginClaims();
            return (manager, devices, router, output, claims);
        }

        private static int PlayerOf(GamepadManager m, SimulatedPad pad) => m.Players.PlayerOf(Find(m, pad));

        private static ConnectedPad Find(GamepadManager m, SimulatedPad pad)
        {
            foreach (ConnectedPad open in m.Pads)
                if (open.IsOpen && open.Path == pad.Path) return open;
            throw new InvalidOperationException($"{pad.Name} is not open");
        }

        [Fact]
        public void A_handhelds_own_controls_are_known_by_vendor_and_product_and_others_are_not()
        {
            SimulatedPad builtIn = BuiltIn(), xbox = Xbox(), deck = new() { Name = "Steam Deck", Vendor = 0x28DE, Product = 0x1205 };
            SimulatedPad steamOs = new() { Name = "Legion Go S Controller", Vendor = 0x28DE, Product = 0x12FF }, steamVirtual = new() { Name = "Microsoft X-Box 360 pad 0", Vendor = 0x28DE, Product = 0x11FF };
            (GamepadManager m, _, _, _, _) = Game(2, claim: false, externalFirst: false, builtIn, xbox, deck, steamOs, steamVirtual);
            Assert.Equal(new[] { true, false, true, true, false }, Array.ConvertAll(new[] { builtIn, xbox, deck, steamOs, steamVirtual }, p => Find(m, p).IsBuiltIn));
            Assert.Equal((0x1A86, 0xE310), (Find(m, builtIn).Vendor, Find(m, builtIn).Product));
            Assert.True(m.HasBuiltInControls);
        }

        // The vendor and product cross the platform library as the C# reads them, and a pad that has gone keeps them.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void Both_halves_read_a_pads_vendor_and_product_at_opening(bool native)
        {
            if (native) Assert.True(DeviceNative.Ready, DeviceNative.Report);
            var pad = new SimulatedPad(native) { Name = "Legion Go S", Vendor = 0x1A86, Product = 0xE310 };
            Assert.Equal((0x1A86, 0xE310), (pad.Vendor, pad.Product));
            var devices = new SimulatedPads(native);
            devices.Connect(pad);
            var manager = new GamepadManager(new GamepadBindingMap(), start: true, devices, native);
            ConnectedPad open = Assert.Single(manager.Pads);
            Assert.Equal((0x1A86, 0xE310, true), (open.Vendor, open.Product, open.IsBuiltIn));
            devices.Disconnect(pad);
            manager.Poll();
            Assert.False(open.IsOpen);
            Assert.Equal((0x1A86, 0xE310, true), (open.Vendor, open.Product, open.IsBuiltIn));
            manager.Dispose();
        }

        // The report of 2026-10-17: the built-in controls opened first and the Xbox pad second, and the game heard only player 1.
        [Fact]
        public void With_a_built_in_pad_first_the_external_pads_first_press_is_player_1()
        {
            SimulatedPad builtIn = BuiltIn(), xbox = Xbox();
            (GamepadManager m, _, PortRouter router, Ports output, List<PadConnection> claims) = Game(2, claim: true, externalFirst: false, builtIn, xbox);
            Assert.Equal((1, 2), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));

            xbox.Press(SDL.GamepadButton.South);
            router.PollPads();
            Assert.True(output.Held(0, PadButton.B));
            Assert.False(output.Held(1, PadButton.B));
            Assert.Equal((2, 1), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));
            PadConnection claim = Assert.Single(claims);
            Assert.Equal(("Xbox Wireless Controller", 1), (claim.Pad.Name, claim.Player));
            Assert.Equal(0, xbox.PlayerIndex);

            // The built-in controls pressed next take player 2, where they already were, and nothing moves.
            builtIn.Press(SDL.GamepadButton.East);
            router.PollPads();
            Assert.True(output.Held(1, PadButton.A));
            Assert.Single(claims);
            Assert.Equal((2, 1), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));
        }

        // What the game heard before 2026-10-17, kept for a player who arranges the seats by hand.
        [Fact]
        public void With_claims_off_the_players_are_the_order_the_pads_connected()
        {
            SimulatedPad builtIn = BuiltIn(), xbox = Xbox();
            (GamepadManager m, _, PortRouter router, Ports output, List<PadConnection> claims) = Game(2, claim: false, externalFirst: false, builtIn, xbox);
            xbox.Press(SDL.GamepadButton.South);
            router.PollPads();
            Assert.True(output.Held(1, PadButton.B));
            Assert.False(output.Held(0, PadButton.B));
            Assert.Equal((1, 2), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));
            Assert.Empty(claims);
        }

        // The handheld setting, on and off, for pads present at start and a pad plugged in later.
        [Theory]
        [InlineData(true)]
        [InlineData(false)]
        public void External_first_seats_an_external_pad_ahead_of_the_built_in_controls(bool on)
        {
            SimulatedPad builtIn = BuiltIn(), xbox = Xbox(), second = Xbox("Xbox Wireless Controller (2)");
            second.Guid = "second";
            (GamepadManager m, SimulatedPads devices, _, _, _) = Game(4, claim: false, externalFirst: on, builtIn, xbox);
            Assert.Equal(on ? (2, 1) : (1, 2), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));

            devices.Connect(second);
            m.Poll();
            Assert.Equal(on ? (3, 1, 2) : (1, 2, 3), (PlayerOf(m, builtIn), PlayerOf(m, xbox), PlayerOf(m, second)));
            Assert.Equal(on ? 2 : 0, builtIn.PlayerIndex);
        }

        // Turned on while the pads are connected, it seats them at once; external pads alone are left in their order.
        [Fact]
        public void External_first_turned_on_rearranges_the_seats_and_leaves_external_pads_alone()
        {
            SimulatedPad builtIn = BuiltIn(), xbox = Xbox();
            (GamepadManager m, _, _, _, _) = Game(2, claim: false, externalFirst: false, builtIn, xbox);
            m.ExternalControllerFirst = true;
            Assert.Equal((2, 1), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));

            SimulatedPad one = Xbox("One"), two = Xbox("Two");
            (GamepadManager n, _, _, _, _) = Game(2, claim: false, externalFirst: true, one, two);
            Assert.Equal((1, 2), (PlayerOf(n, one), PlayerOf(n, two)));
            Assert.False(n.HasBuiltInControls);
        }

        // A built-in pad already pressed in the game keeps player 1 when an external pad connects after it.
        [Fact]
        public void External_first_does_not_take_a_player_claimed_in_the_game()
        {
            SimulatedPad builtIn = BuiltIn(), xbox = Xbox();
            (GamepadManager m, SimulatedPads devices, PortRouter router, Ports output, _) = Game(2, claim: true, externalFirst: true, builtIn);
            builtIn.Press(SDL.GamepadButton.South);
            router.PollPads();
            Assert.True(m.PlayerOneClaimed);

            devices.Connect(xbox);
            m.Poll();
            Assert.Equal((1, 2), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));
            xbox.Press(SDL.GamepadButton.Start);
            router.PollPads();
            Assert.True(output.Held(1, PadButton.Start));
            Assert.True(output.Held(0, PadButton.B));
        }

        // A claimed pad that drops and comes back is the same player, and the pad that stayed is not moved meanwhile.
        [Fact]
        public void A_claimed_pad_that_reconnects_is_the_same_player()
        {
            SimulatedPad builtIn = BuiltIn(), xbox = Xbox();
            (GamepadManager m, SimulatedPads devices, PortRouter router, Ports output, List<PadConnection> claims) = Game(2, claim: true, externalFirst: false, builtIn, xbox);
            xbox.Press(SDL.GamepadButton.South);
            router.PollPads();
            xbox.ReleaseAll();
            router.PollPads();

            devices.Disconnect(xbox);
            m.Poll();
            builtIn.Press(SDL.GamepadButton.Start);
            router.PollPads();
            Assert.True(output.Held(1, PadButton.Start));
            Assert.False(output.Held(0, PadButton.Start));

            devices.Connect(xbox);
            m.Poll();
            Assert.Equal((2, 1), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));
            xbox.Press(SDL.GamepadButton.East);
            router.PollPads();
            Assert.True(output.Held(0, PadButton.A));
            Assert.Single(claims);
        }

        // Two pads pressed in one poll take players in the order they connected; the pad nobody pressed follows.
        [Fact]
        public void Pads_pressed_together_take_players_in_the_order_they_connected()
        {
            SimulatedPad one = Xbox("One"), two = Xbox("Two"), three = Xbox("Three");
            (GamepadManager m, _, PortRouter router, Ports output, List<PadConnection> claims) = Game(4, claim: true, externalFirst: false, one, two, three);
            three.Press(SDL.GamepadButton.South);
            two.Press(SDL.GamepadButton.East);
            router.PollPads();
            Assert.Equal((3, 1, 2), (PlayerOf(m, one), PlayerOf(m, two), PlayerOf(m, three)));
            Assert.True(output.Held(0, PadButton.A));
            Assert.True(output.Held(1, PadButton.B));
            Assert.Equal(new[] { ("Two", 1), ("Three", 2) }, claims.ConvertAll(c => (c.Pad.Name, c.Player)));
        }

        // Swap Players trades 1 and 2, and the seats are the player's own until the next game, when the first press claims again.
        [Fact]
        public void Swapping_players_ends_the_claims_until_the_next_game()
        {
            SimulatedPad builtIn = BuiltIn(), xbox = Xbox();
            (GamepadManager m, _, PortRouter router, Ports output, List<PadConnection> claims) = Game(2, claim: true, externalFirst: false, builtIn, xbox);
            Assert.True(m.CanSwapPlayers);
            Assert.True(m.SwapPlayers());
            Assert.Equal((2, 1), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));
            Assert.True(m.PlayerOneClaimed);

            builtIn.Press(SDL.GamepadButton.South);
            router.PollPads();
            Assert.True(output.Held(1, PadButton.B));
            Assert.Empty(claims);

            builtIn.ReleaseAll();
            router.Reset(2, Array.Empty<PadAxis>());
            m.BeginClaims();
            builtIn.Press(SDL.GamepadButton.South);
            router.PollPads();
            Assert.Equal((1, 2), (PlayerOf(m, builtIn), PlayerOf(m, xbox)));
            Assert.True(output.Held(0, PadButton.B));
        }

        // With one pad there is no player 2 to swap with.
        [Fact]
        public void One_pad_cannot_be_swapped()
        {
            (GamepadManager m, _, _, _, _) = Game(2, claim: true, externalFirst: false, Xbox());
            Assert.False(m.CanSwapPlayers);
            Assert.False(m.SwapPlayers());
        }

        // A pad set to None stays out of the game when pressed, and First Controller hears the first pad pressed alone.
        [Fact]
        public void A_pad_with_no_player_does_not_claim_and_first_controller_hears_the_claimed_player_1_alone()
        {
            SimulatedPad one = Xbox("One"), two = Xbox("Two"), three = Xbox("Three");
            (GamepadManager m, _, PortRouter router, Ports output, _) = Game(2, claim: true, externalFirst: false, one, two, three);
            m.Assign(Find(m, three), 0);
            m.BeginClaims();
            m.FirstControllerOnly = true;
            three.Press(SDL.GamepadButton.Start);
            two.Press(SDL.GamepadButton.South);
            router.PollPads();
            Assert.Equal((0, 1), (PlayerOf(m, three), PlayerOf(m, two)));
            Assert.True(output.Held(0, PadButton.B));
            Assert.False(output.Held(0, PadButton.Start));
            one.Press(SDL.GamepadButton.East);
            router.PollPads();
            Assert.Equal(2, PlayerOf(m, one));
            Assert.False(output.Held(1, PadButton.A));
        }

        // The keyboard stays the player it was given, and a key claims nothing.
        [Fact]
        public void The_keyboard_keeps_its_player_and_claims_nothing()
        {
            SimulatedPad builtIn = BuiltIn(), xbox = Xbox();
            (GamepadManager m, _, PortRouter router, Ports output, List<PadConnection> claims) = Game(2, claim: true, externalFirst: false, builtIn, xbox);
            router.KeyboardPlayer = 2;
            _keys[(int)PadControl.Start] = true;
            router.PollPads();
            Assert.True(output.Held(1, PadButton.Start));
            Assert.Empty(claims);
            Assert.False(m.PlayerOneClaimed);
            xbox.Press(SDL.GamepadButton.South);
            router.PollPads();
            Assert.True(output.Held(0, PadButton.B));
            Assert.True(output.Held(1, PadButton.Start));
        }
    }
}

using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Input;
using SDL3;

namespace EmuSen.WiseMan.Input
{
    // Each player's pad and the keyboard to the game's ports, nothing past them, and a port plugged in while its player has a pad - see EmuSen_Input.md §8.5 and §8.6.
    public class PortRouterTests
    {
        private sealed class Ports
        {
            public readonly Dictionary<(int Port, PadButton Button), bool> Buttons = new();
            public readonly Dictionary<(int Port, PadAxis Axis), double> Axes = new();
            public readonly Dictionary<int, bool> Connected = new();
            public readonly List<(int Port, PadButton Button, bool Held)> Sent = new();

            public bool Held(int port, PadButton button) => Buttons.GetValueOrDefault((port, button));
        }

        private readonly bool[] _keys = new bool[Enum.GetValues<PadControl>().Length];

        private (GamepadManager Manager, SimulatedPads Devices, PortRouter Router, Ports Out) Start(int ports, IReadOnlyList<PadAxis>? axes, params SimulatedPad[] pads)
        {
            SimulatedPads devices = SimulatedPads.With(pads);
            var manager = new GamepadManager(new GamepadBindingMap(), start: true, devices);
            var output = new Ports();
            var router = new PortRouter(manager,
                (p, b, h) => { output.Buttons[(p, b)] = h; output.Sent.Add((p, b, h)); },
                (p, a, v) => output.Axes[(p, a)] = v,
                (p, c) => output.Connected[p] = c)
            {
                KeyboardHeld = c => _keys[(int)c],
            };
            router.Reset(ports, axes ?? Array.Empty<PadAxis>());
            return (manager, devices, router, output);
        }

        [Fact]
        public void Each_players_pad_reaches_its_own_port()
        {
            SimulatedPad one = new() { Name = "One" }, two = new() { Name = "Two" }, three = new() { Name = "Three" }, four = new() { Name = "Four" };
            (_, _, PortRouter router, Ports output) = Start(4, null, one, two, three, four);

            two.Press(SDL.GamepadButton.South);
            four.Press(SDL.GamepadButton.Start);
            router.PollPads();

            Assert.True(output.Held(1, PadButton.B));
            Assert.True(output.Held(3, PadButton.Start));
            Assert.Equal(new[] { (1, PadButton.B, true), (3, PadButton.Start, true) }, output.Sent);

            two.Release(SDL.GamepadButton.South);
            router.PollPads();
            Assert.False(output.Held(1, PadButton.B));
            Assert.Equal(3, output.Sent.Count);
        }

        // A two-port console never hears player 3, whose pad is still seated.
        [Fact]
        public void Nothing_is_sent_past_the_games_ports()
        {
            SimulatedPad one = new(), two = new(), three = new();
            (GamepadManager manager, _, PortRouter router, Ports output) = Start(2, new[] { PadAxis.LeftX }, one, two, three);
            Assert.Equal(3, manager.Players.Highest);

            three.Press(SDL.GamepadButton.South);
            three.SetAxis(SDL.GamepadAxis.LeftX, 1);
            router.PollPads();

            Assert.DoesNotContain(output.Buttons.Keys, k => k.Port >= 2);
            Assert.DoesNotContain(output.Axes.Keys, k => k.Port >= 2);
            Assert.DoesNotContain(output.Connected.Keys, p => p >= 2);
        }

        // The pad's buttons are let go on its port, the port stays plugged in for its return, and it plays again once back.
        [Fact]
        public void A_pad_pulled_out_mid_game_lets_go_of_its_buttons_and_its_port_waits()
        {
            SimulatedPad one = new() { Name = "One" }, two = new() { Name = "Two" };
            (GamepadManager manager, SimulatedPads devices, PortRouter router, Ports output) = Start(2, null, one, two);
            one.Press(SDL.GamepadButton.East);
            two.Press(SDL.GamepadButton.South);
            router.PollPads();
            Assert.True(output.Held(1, PadButton.B));
            Assert.True(output.Connected[1]);

            devices.Disconnect(two);
            manager.Poll();
            router.PollPads();
            Assert.False(output.Held(1, PadButton.B));
            Assert.True(output.Held(0, PadButton.A));
            Assert.True(output.Connected[1]);

            devices.Connect(two);
            manager.Poll();
            router.PollPads();
            Assert.True(output.Held(1, PadButton.B));
        }

        [Fact]
        public void The_keyboard_plays_as_its_player_beside_that_players_pad()
        {
            SimulatedPad one = new(), two = new();
            (_, _, PortRouter router, Ports output) = Start(2, null, one, two);

            _keys[(int)PadControl.Start] = true;
            router.KeysChanged();
            Assert.True(output.Held(0, PadButton.Start));

            router.KeyboardPlayer = 2;
            router.KeysChanged();
            Assert.False(output.Held(0, PadButton.Start));
            Assert.True(output.Held(1, PadButton.Start));

            one.Press(SDL.GamepadButton.South);
            router.PollPads();
            Assert.True(output.Held(0, PadButton.B));
        }

        // A key change reads the pads as the last poll found them, as the frontends did before.
        [Fact]
        public void A_key_change_uses_the_pads_as_last_polled()
        {
            SimulatedPad one = new();
            (_, _, PortRouter router, Ports output) = Start(1, null, one);
            one.Press(SDL.GamepadButton.South);
            _keys[(int)PadControl.Up] = true;
            router.KeysChanged();
            Assert.True(output.Held(0, PadButton.Up));
            Assert.False(output.Held(0, PadButton.B));
            router.PollPads();
            Assert.True(output.Held(0, PadButton.B));
        }

        // Mirroring sends player 1 to the second port as well, beside player 2's own pad.
        [Fact]
        public void The_mirror_adds_player_1_to_the_second_port()
        {
            SimulatedPad one = new(), two = new();
            (_, _, PortRouter router, Ports output) = Start(2, new[] { PadAxis.LeftX }, one, two);
            router.MirrorPlayer1ToPlayer2 = true;

            one.Press(SDL.GamepadButton.South);
            two.Press(SDL.GamepadButton.Start);
            one.SetAxis(SDL.GamepadAxis.LeftX, -0.6);
            two.SetAxis(SDL.GamepadAxis.LeftX, 0.3);
            router.PollPads();

            Assert.True(output.Held(1, PadButton.B));
            Assert.True(output.Held(1, PadButton.Start));
            Assert.False(output.Held(0, PadButton.Start));
            Assert.Equal(-0.6, output.Axes[(1, PadAxis.LeftX)], 3);
            Assert.Equal(-0.6, output.Axes[(0, PadAxis.LeftX)], 3);
        }

        [Fact]
        public void Each_port_gets_its_own_players_axes_on_every_poll()
        {
            SimulatedPad one = new(), two = new();
            (_, _, PortRouter router, Ports output) = Start(2, new[] { PadAxis.LeftX, PadAxis.LeftY }, one, two);
            two.SetAxis(SDL.GamepadAxis.LeftY, 1);
            router.PollPads();
            Assert.Equal(0, output.Axes[(0, PadAxis.LeftY)]);
            Assert.Equal(1, output.Axes[(1, PadAxis.LeftY)], 3);

            output.Axes.Clear();
            router.PollPads();
            Assert.Equal(4, output.Axes.Count);
        }

        // A port past the first holds a controller while its player has a pad seated or the keyboard; the first always does.
        [Fact]
        public void A_port_is_plugged_in_while_its_player_has_something_to_play_with()
        {
            SimulatedPad one = new() { Name = "One" }, two = new() { Name = "Two" };
            (GamepadManager manager, _, PortRouter router, Ports output) = Start(4, null, one, two);
            router.PollPads();
            Assert.Equal(new[] { (1, true), (2, false), (3, false) }, output.Connected.OrderBy(c => c.Key).Select(c => (c.Key, c.Value)));

            router.KeyboardPlayer = 4;
            router.KeysChanged();
            Assert.True(output.Connected[3]);

            manager.Players.Clear();
            router.PollPads();
            Assert.False(output.Connected[1]);
            Assert.True(router.Connected(0));

            manager.FirstControllerOnly = true;
            Assert.False(router.Connected(1));
        }

        // A new game starts with nothing held, so a button already down is sent again.
        [Fact]
        public void A_reset_forgets_what_was_sent()
        {
            SimulatedPad one = new();
            (_, _, PortRouter router, Ports output) = Start(1, null, one);
            one.Press(SDL.GamepadButton.South);
            router.PollPads();
            router.Reset(1, Array.Empty<PadAxis>());
            router.PollPads();
            Assert.Equal(2, output.Sent.Count(s => s.Button == PadButton.B && s.Held));
            Assert.True(router.PadHeld(1, PadButton.B));
        }
    }
}

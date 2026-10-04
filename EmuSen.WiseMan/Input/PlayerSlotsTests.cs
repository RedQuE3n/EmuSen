using System.Collections.Generic;
using System.Linq;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Input;
using SDL3;

namespace EmuSen.WiseMan.Input
{
    // The rule that seats pads as players, and its edge cases - see EmuSen_Input.md §8.2 and §8.3.
    public class PlayerSlotsTests
    {
        private static (GamepadManager Manager, SimulatedPads Devices, List<PadConnection> Changes) Start(params SimulatedPad[] pads)
        {
            SimulatedPads devices = SimulatedPads.With(pads);
            var manager = new GamepadManager(new GamepadBindingMap(), start: true, devices);
            var changes = new List<PadConnection>();
            manager.PadChanged += changes.Add;
            return (manager, devices, changes);
        }

        private static int PlayerOf(GamepadManager manager, SimulatedPad pad) =>
            manager.Players.Seated().Where(s => s.Pad.IsOpen && s.Pad.Name == pad.Name && s.Pad.Path == pad.Path).Select(s => s.Player).SingleOrDefault();

        private static string[] Seats(GamepadManager manager) =>
            Enumerable.Range(1, 4).Select(p => manager.Players.SeatOf(p) is { } pad ? (pad.IsOpen ? pad.Name : pad.Name + " (gone)") : "-").ToArray();

        [Fact]
        public void Pads_take_players_in_the_order_they_connect_and_each_hears_only_its_own()
        {
            SimulatedPad a = new() { Name = "A" }, b = new() { Name = "B" }, c = new() { Name = "C" };
            (GamepadManager manager, SimulatedPads devices, List<PadConnection> changes) = Start(a, b);
            devices.Connect(c);
            manager.Poll();

            Assert.Equal(new[] { "A", "B", "C", "-" }, Seats(manager));
            Assert.Equal(3, changes.Single().Player);

            b.Press(SDL.GamepadButton.South);
            c.SetAxis(SDL.GamepadAxis.LeftX, 1);
            Assert.False(manager.IsPressed(PadButton.B, 1));
            Assert.True(manager.IsPressed(PadButton.B, 2));
            Assert.False(manager.IsPressed(PadButton.B, 3));
            Assert.Equal(0, manager.Axis(PadAxis.LeftX, 2));
            Assert.Equal(1, manager.Axis(PadAxis.LeftX, 3), 3);
            Assert.False(manager.IsPressed(PadButton.B, 4));
        }

        // The seat is kept while its pad is gone; nobody moves up, and the pad finds it again.
        [Fact]
        public void A_pad_that_goes_keeps_its_player_and_gets_it_back()
        {
            SimulatedPad a = new() { Name = "A" }, b = new() { Name = "B" }, c = new() { Name = "C" };
            (GamepadManager manager, SimulatedPads devices, List<PadConnection> changes) = Start(a, b, c);

            devices.Disconnect(b);
            manager.Poll();
            Assert.Equal(new[] { "A", "B (gone)", "C", "-" }, Seats(manager));
            Assert.Equal((false, 2), (changes.Single().Connected, changes.Single().Player));
            c.Press(SDL.GamepadButton.South);
            Assert.True(manager.IsPressed(PadButton.B, 3));
            Assert.False(manager.IsPressed(PadButton.B, 2));

            devices.Connect(b);
            manager.Poll();
            Assert.Equal(new[] { "A", "B", "C", "-" }, Seats(manager));
            Assert.Equal((true, 2), (changes[1].Connected, changes[1].Player));
        }

        // Two of one model share a GUID; the path tells them apart, whichever comes back first.
        [Fact]
        public void Two_identical_pads_are_told_apart_by_where_they_are_plugged_in()
        {
            SimulatedPad left = new() { Name = "Twin", Path = "/dev/input/event10" }, right = new() { Name = "Twin", Path = "/dev/input/event11" };
            (GamepadManager manager, SimulatedPads devices, _) = Start(left, right);
            Assert.Equal(manager.Pads[0].Guid, manager.Pads[1].Guid);

            devices.Disconnect(left);
            devices.Disconnect(right);
            manager.Poll();
            devices.Connect(right);
            manager.Poll();
            devices.Connect(left);
            manager.Poll();

            Assert.Equal(1, PlayerOf(manager, left));
            Assert.Equal(2, PlayerOf(manager, right));
        }

        // With no path, as SDL's virtual pads have, the lowest seat held for the model is taken.
        [Fact]
        public void Identical_pads_with_no_path_take_their_models_seats_lowest_first()
        {
            SimulatedPad left = new() { Name = "Twin", Path = null }, right = new() { Name = "Twin", Path = null }, other = new() { Name = "Other" };
            (GamepadManager manager, SimulatedPads devices, _) = Start(left, other, right);

            devices.Disconnect(left);
            devices.Disconnect(right);
            manager.Poll();
            devices.Connect(right);
            manager.Poll();

            Assert.Equal(new[] { "Twin", "Other", "Twin (gone)", "-" }, Seats(manager));
        }

        // A new pad takes the lowest seat without a pad connected, a reserved one included: a lone player's spare pad is player 1.
        [Fact]
        public void A_new_pad_takes_the_lowest_seat_with_no_pad_connected()
        {
            SimulatedPad first = new() { Name = "First" }, spare = new() { Name = "Spare" };
            (GamepadManager manager, SimulatedPads devices, _) = Start(first);

            devices.Disconnect(first);
            manager.Poll();
            devices.Connect(spare);
            manager.Poll();
            Assert.Equal(new[] { "Spare", "-", "-", "-" }, Seats(manager));

            // The first pad back finds its seat taken, so it takes the lowest one free.
            devices.Connect(first);
            manager.Poll();
            Assert.Equal(new[] { "Spare", "First", "-", "-" }, Seats(manager));
        }

        [Fact]
        public void Choosing_a_player_for_a_pad_trades_seats_and_none_takes_it_out_of_the_game()
        {
            SimulatedPad a = new() { Name = "A" }, b = new() { Name = "B" }, c = new() { Name = "C" };
            (GamepadManager manager, SimulatedPads devices, _) = Start(a, b, c);
            int changed = 0;
            manager.Players.Changed += () => changed++;

            manager.Assign(manager.Pads[2], 1);
            Assert.Equal(new[] { "C", "B", "A", "-" }, Seats(manager));

            manager.Assign(manager.Pads[1], 0);
            Assert.Equal(new[] { "C", "-", "A", "-" }, Seats(manager));
            b.Press(SDL.GamepadButton.South);
            Assert.False(Enumerable.Range(1, PlayerSlots.MaxPlayers).Any(p => manager.IsPressed(PadButton.B, p)));

            // A pad with no seat moved into one held by a connected pad sends that pad to the lowest seat free.
            manager.Assign(manager.Pads[1], 3);
            Assert.Equal(new[] { "C", "A", "B", "-" }, Seats(manager));
            Assert.Equal(3, changed);
        }

        // A reservation trades seats as a pad does; Forget lets one go.
        [Fact]
        public void A_reservation_trades_seats_like_a_pad_and_is_let_go_by_forget()
        {
            SimulatedPad a = new() { Name = "A" }, b = new() { Name = "B" }, c = new() { Name = "C" };
            (GamepadManager manager, SimulatedPads devices, _) = Start(a, b, c);
            devices.Disconnect(b);
            devices.Disconnect(c);
            manager.Poll();

            manager.Players.Forget(3);
            Assert.Equal(new[] { "A", "B (gone)", "-", "-" }, Seats(manager));
            manager.Players.Forget(1);
            Assert.Equal(new[] { "A", "B (gone)", "-", "-" }, Seats(manager));

            manager.Assign(manager.Pads[0], 2);
            Assert.Equal(new[] { "B (gone)", "A", "-", "-" }, Seats(manager));
        }

        // More pads than seats: the ninth is no player, and still steers the interface.
        [Fact]
        public void A_pad_past_the_last_seat_is_no_player_and_still_steers_the_interface()
        {
            SimulatedPad[] pads = Enumerable.Range(1, PlayerSlots.MaxPlayers + 1).Select(i => new SimulatedPad { Name = $"Pad {i}" }).ToArray();
            (GamepadManager manager, _, _) = Start(pads);

            Assert.Equal(PlayerSlots.MaxPlayers, manager.Players.Highest);
            Assert.Equal(0, manager.Players.PlayerOf(manager.Pads[^1]));
            Assert.Equal(PlayerSlots.MaxPlayers + 1, manager.FrontendPadCount);
            pads[^1].Press(SDL.GamepadButton.North);
            Assert.Equal(SDL.GamepadButton.North, manager.GetAnyPressedButton());
        }

        // A pad that registers twice is ES-DE's case for the first controller alone, which now quiets the game's other players too.
        [Fact]
        public void With_the_first_controller_alone_the_game_hears_player_1_only()
        {
            SimulatedPad real = new() { Name = "Pad" }, twin = new() { Name = "Pad (virtual)" };
            (GamepadManager manager, _, _) = Start(real, twin);
            twin.Press(SDL.GamepadButton.South);
            Assert.True(manager.IsPressed(PadButton.B, 2));

            manager.FirstControllerOnly = true;
            Assert.False(manager.IsPressed(PadButton.B, 2));
            Assert.Null(manager.PlayerPad(2));
            real.Press(SDL.GamepadButton.South);
            Assert.True(manager.IsPressed(PadButton.B, 1));
        }

        [Fact]
        public void Each_player_reads_through_its_own_bindings()
        {
            SimulatedPad a = new() { Name = "A" }, b = new() { Name = "B" };
            (GamepadManager manager, _, _) = Start(a, b);
            var second = new GamepadBindingMap();
            second.Rebind(PadButton.B, SDL.GamepadButton.North);
            manager.PlayerBindings = p => p == 2 ? second : null;

            a.Press(SDL.GamepadButton.South);
            b.Press(SDL.GamepadButton.South);
            Assert.True(manager.IsPressed(PadButton.B, 1));
            Assert.False(manager.IsPressed(PadButton.B, 2));
            b.Press(SDL.GamepadButton.North);
            Assert.True(manager.IsPressed(PadButton.B, 2));
            Assert.Same(manager.Bindings, manager.BindingsFor(3));
        }

        // SDL lights the number on a pad that has lights; a pad with no seat is told -1.
        [Fact]
        public void Each_pad_is_told_its_player_number()
        {
            SimulatedPad a = new() { Name = "A" }, b = new() { Name = "B" };
            (GamepadManager manager, _, _) = Start(a, b);
            Assert.Equal((0, 1), (a.PlayerIndex, b.PlayerIndex));

            manager.Assign(manager.Pads[1], 1);
            Assert.Equal((1, 0), (a.PlayerIndex, b.PlayerIndex));
            manager.Assign(manager.Pads[0], 0);
            Assert.Equal((-1, 0), (a.PlayerIndex, b.PlayerIndex));
        }

        // Other devices start the seating again, as the window's pads are a different set.
        [Fact]
        public void Other_devices_clear_every_seat()
        {
            SimulatedPad a = new() { Name = "A" };
            (GamepadManager manager, SimulatedPads devices, _) = Start(a);
            devices.Disconnect(a);
            manager.Poll();
            manager.UseDevices(SimulatedPads.With(new SimulatedPad { Name = "Z" }));
            Assert.Equal(new[] { "Z", "-", "-", "-" }, Seats(manager));
        }
    }
}

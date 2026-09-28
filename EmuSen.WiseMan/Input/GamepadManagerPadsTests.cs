using System.Collections.Generic;
using System.Linq;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Input;
using SDL3;

namespace EmuSen.WiseMan.Input
{
    // Every pad opened, hot-plugged and let go; player 1 is the first opened - see EmuSen_Settings_Reference.md §4.61.
    public class GamepadManagerPadsTests
    {
        private static (GamepadManager Manager, SimulatedPads Devices, List<PadConnection> Changes) Start(params SimulatedPad[] pads)
        {
            SimulatedPads devices = SimulatedPads.With(pads);
            var manager = new GamepadManager(new GamepadBindingMap(), start: true, devices);
            var changes = new List<PadConnection>();
            manager.PadChanged += changes.Add;
            return (manager, devices, changes);
        }

        [Fact]
        public void Every_pad_present_at_start_is_opened_in_order_and_none_is_announced()
        {
            SimulatedPad first = new() { Name = "First" }, second = new() { Name = "Second" };
            (GamepadManager manager, SimulatedPads devices, List<PadConnection> changes) = Start(first, second);
            manager.Poll();

            Assert.Equal(new[] { "First", "Second" }, manager.Pads.Select(p => p.Name));
            Assert.Equal("First", manager.ControllerName);
            Assert.Equal(2, devices.OpenHandles);
            Assert.Empty(changes);
        }

        [Fact]
        public void A_pad_plugged_in_is_opened_and_announced_and_one_pulled_out_is_closed_and_announced()
        {
            SimulatedPad first = new() { Name = "First" }, second = new() { Name = "Second", Type = SDL.GamepadType.PS5 };
            (GamepadManager manager, SimulatedPads devices, List<PadConnection> changes) = Start(first);

            devices.Connect(second);
            manager.Poll();
            Assert.Equal(2, manager.Pads.Count);
            Assert.Equal(("Second", true), (changes.Single().Pad.Name, changes.Single().Connected));

            devices.Disconnect(second);
            manager.Poll();
            Assert.Single(manager.Pads);
            Assert.Equal(1, devices.OpenHandles);
            Assert.Equal(("Second", false, SDL.GamepadType.PS5), (changes[1].Pad.Name, changes[1].Connected, changes[1].Pad.Type));
            Assert.False(changes[1].Pad.IsOpen);
        }

        // Player 1 is the first pad opened: the game's buttons and axes are read from it alone, as before any second pad existed.
        [Fact]
        public void The_game_reads_the_first_pad_alone_and_the_next_one_once_the_first_goes()
        {
            SimulatedPad first = new() { Name = "First" }, second = new() { Name = "Second" };
            (GamepadManager manager, SimulatedPads devices, _) = Start(first, second);

            second.Press(SDL.GamepadButton.South);
            second.SetAxis(SDL.GamepadAxis.LeftX, 1);
            Assert.False(manager.IsPressed(PadButton.B));
            Assert.Equal(0, manager.Axis(PadAxis.LeftX));
            Assert.False(manager.IsRawPressed(SDL.GamepadButton.South));

            first.Press(SDL.GamepadButton.South);
            Assert.True(manager.IsPressed(PadButton.B));

            devices.Disconnect(first);
            manager.Poll();
            Assert.Equal("Second", manager.ControllerName);
            Assert.True(manager.IsPressed(PadButton.B));
            Assert.Equal(1, manager.Axis(PadAxis.LeftX), 3);
        }

        [Fact]
        public void The_rebind_capture_hears_any_pad_the_interface_reads()
        {
            SimulatedPad first = new(), second = new();
            (GamepadManager manager, _, _) = Start(first, second);

            second.Press(SDL.GamepadButton.North);
            Assert.Equal(SDL.GamepadButton.North, manager.GetAnyPressedButton());

            manager.FirstControllerOnly = true;
            Assert.Null(manager.GetAnyPressedButton());
        }

        // §15.14's lesson for pads: each handle opened is closed, whether the pad goes or the manager does.
        [Fact]
        public void Every_handle_is_closed_when_its_pad_goes_and_when_the_manager_is_disposed()
        {
            SimulatedPad first = new(), second = new(), third = new();
            (GamepadManager manager, SimulatedPads devices, _) = Start(first, second);
            devices.Connect(third);
            manager.Poll();
            Assert.Equal(3, devices.OpenHandles);

            devices.Disconnect(second);
            manager.Poll();
            Assert.Equal(2, devices.OpenHandles);

            manager.Dispose();
            Assert.Equal(0, devices.OpenHandles);
            Assert.Equal(devices.Opens, devices.Closes);
            Assert.False(devices.Initialized);
        }

        [Fact]
        public void A_pad_pulled_out_and_plugged_back_is_opened_again()
        {
            SimulatedPad pad = new() { Name = "Only" };
            (GamepadManager manager, SimulatedPads devices, List<PadConnection> changes) = Start(pad);

            devices.Disconnect(pad);
            for (int i = 0; i < 100; i++) manager.Poll();
            Assert.False(manager.IsConnected);

            devices.Connect(pad);
            manager.Poll();
            Assert.True(manager.IsConnected);
            Assert.Equal(new[] { false, true }, changes.Select(c => c.Connected));
            Assert.Equal(1, devices.OpenHandles);
        }
    }
}

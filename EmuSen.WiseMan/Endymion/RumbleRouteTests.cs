using System;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Endymion
{
    // A port's motors reach the pad of the player the port hears, through the library's devices - see EmuSen_Input.md §9.
    public class RumbleRouteTests
    {
        [Fact]
        public void A_ports_motors_reach_its_players_pad_and_nothing_past_the_games_ports()
        {
            var devices = new SimulatedPads();
            var (first, second) = (new SimulatedPad(), new SimulatedPad());
            devices.Connect(first);
            devices.Connect(second);
            using var manager = new GamepadManager(new GamepadBindingMap(), true, devices);
            manager.Assign(manager.Pads[0], 1);
            manager.Assign(manager.Pads[1], 2);
            var router = new PortRouter(manager, (_, _, _) => { }, (_, _, _) => { }, null);
            router.Reset(2, Array.Empty<PadAxis>());

            router.Rumble(1, 40000, 7);
            Assert.Equal(((ushort)0, (ushort)0), first.Rumble);
            Assert.Equal(((ushort)40000, (ushort)7), second.Rumble);
            router.Rumble(0, 65535, 65535);
            Assert.Equal(((ushort)65535, (ushort)65535), first.Rumble);

            router.Rumble(2, 1, 1);
            router.Rumble(-1, 1, 1);
            Assert.Equal(((ushort)65535, (ushort)65535), first.Rumble);
            Assert.Equal(((ushort)40000, (ushort)7), second.Rumble);

            manager.FirstControllerOnly = true;
            router.Rumble(1, 2, 2);
            Assert.Equal(((ushort)40000, (ushort)7), second.Rumble);
        }
    }
}

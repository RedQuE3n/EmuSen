using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Input
{
    // SDL's own virtual joysticks beneath the real device layer: their GUIDs and paths as SDL reports them, the seats, a port, and a pad pulled and replaced - see EmuSen_Input.md §8.8.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class SdlVirtualPadsTests : IDisposable
    {
        // The real layer, showing only the pads this test attached, so a pad on the desk takes no seat.
        private sealed class VirtualOnly(SdlPadDevices sdl, HashSet<uint> mine) : IPadDevices
        {
            public bool Init() => sdl.Init();
            public void Quit() => sdl.Quit();
            public uint[] Attached() => sdl.Attached().Where(mine.Contains).ToArray();
            public IntPtr Open(uint id) => sdl.Open(id);
            public void Close(IntPtr pad) => sdl.Close(pad);
            public bool IsAttached(IntPtr pad) => sdl.IsAttached(pad);
            public bool DevicesChanged() => sdl.DevicesChanged();
            public void Update() => sdl.Update();
            public bool Button(IntPtr pad, SDL.GamepadButton button) => sdl.Button(pad, button);
            public short Axis(IntPtr pad, SDL.GamepadAxis axis) => sdl.Axis(pad, axis);
            public string? Name(IntPtr pad) => sdl.Name(pad);
            public SDL.GamepadType Type(IntPtr pad) => sdl.Type(pad);
            public SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button) => sdl.Label(pad, button);
            public string Guid(IntPtr pad) => sdl.Guid(pad);
            public string? Path(IntPtr pad) => sdl.Path(pad);
            public void SetPlayerIndex(IntPtr pad, int index) => sdl.SetPlayerIndex(pad, index);
        }

        private readonly HashSet<uint> _mine = new();
        private readonly List<IntPtr> _names = new();
        private readonly Dictionary<uint, IntPtr> _joysticks = new();
        private readonly bool _sdl;

        public SdlVirtualPadsTests() => _sdl = SDL.InitSubSystem(SDL.InitFlags.Gamepad);

        public void Dispose()
        {
            foreach (IntPtr joystick in _joysticks.Values) SDL.CloseJoystick(joystick);
            foreach (uint id in _mine.ToArray()) SDL.DetachVirtualJoystick(id);
            foreach (IntPtr name in _names) Marshal.FreeHGlobal(name);
            if (_sdl) SDL.QuitSubSystem(SDL.InitFlags.Gamepad);
        }

        // A virtual pad with SDL's whole gamepad layout, under a USB vendor and product, which SDL folds into its GUID.
        private uint Attach(string name, ushort product)
        {
            IntPtr text = Marshal.StringToHGlobalAnsi(name);
            _names.Add(text);
            var desc = new SDL.VirtualJoystickDesc
            {
                Version = (uint)Marshal.SizeOf<SDL.VirtualJoystickDesc>(),
                Type = SDL.JoystickType.Gamepad,
                NAxes = (ushort)SDL.GamepadAxis.Count,
                NButtons = (ushort)SDL.GamepadButton.Count,
                VendorID = 0x1209,
                ProductID = product,
                Name = text,
            };
            uint id = SDL.AttachVirtualJoystick(in desc);
            Assert.True(id != 0, SDL.GetError());
            _mine.Add(id);
            _joysticks[id] = SDL.OpenJoystick(id);
            return id;
        }

        private void Detach(uint id)
        {
            SDL.CloseJoystick(_joysticks[id]);
            _joysticks.Remove(id);
            Assert.True(SDL.DetachVirtualJoystick(id), SDL.GetError());
            _mine.Remove(id);
        }

        private void Press(uint id, SDL.GamepadButton button, bool down) => SDL.SetJoystickVirtualButton(_joysticks[id], (int)button, down);

        private static void Poll(GamepadManager manager)
        {
            SDL.PumpEvents();
            manager.Poll();
        }

        [Fact]
        public void Sdls_virtual_pads_take_seats_reach_their_ports_and_a_replacement_finds_the_seat_kept()
        {
            Assert.True(_sdl, SDL.GetError());
            uint a = Attach("EmuSen Test Pad", 0x0001), b = Attach("EmuSen Test Pad", 0x0001), c = Attach("EmuSen Other Pad", 0x0002);
            using var manager = new GamepadManager(new GamepadBindingMap(), start: true, new VirtualOnly(new SdlPadDevices(), _mine));
            var changes = new List<PadConnection>();
            manager.PadChanged += changes.Add;
            Poll(manager);

            Assert.Equal(3, manager.Pads.Count);
            Assert.Equal(new[] { 1, 2, 3 }, manager.Pads.Select(manager.Players.PlayerOf));
            Assert.Equal(manager.Pads[0].Guid, manager.Pads[1].Guid);
            Assert.NotEqual(manager.Pads[0].Guid, manager.Pads[2].Guid);
            Assert.Equal(32, manager.Pads[0].Guid.Length);

            var sent = new Dictionary<(int, PadButton), bool>();
            var router = new PortRouter(manager, (p, btn, held) => sent[(p, btn)] = held, (_, _, _) => { });
            router.Reset(4, Array.Empty<PadAxis>());

            Press(b, SDL.GamepadButton.South, true);
            Press(c, SDL.GamepadButton.Start, true);
            Poll(manager);
            router.PollPads();
            Assert.True(manager.IsPressed(PadButton.B, 2));
            Assert.False(manager.IsPressed(PadButton.B, 1));
            Assert.Equal(new Dictionary<(int, PadButton), bool> { [(1, PadButton.B)] = true, [(2, PadButton.Start)] = true }, sent);

            // Player 2's pad pulled out: its button let go, the seat kept, and another of the same model plugged in takes it.
            Detach(b);
            Poll(manager);
            router.PollPads();
            Assert.Equal((false, 2), (changes.Last().Connected, changes.Last().Player));
            Assert.False(sent[(1, PadButton.B)]);
            Assert.True(sent[(2, PadButton.Start)]);
            Assert.False(manager.Players.SeatOf(2)!.IsOpen);

            uint replacement = Attach("EmuSen Test Pad", 0x0001);
            for (int i = 0; i < 3 && manager.Pads.Count < 3; i++) Poll(manager);
            Assert.Equal((true, 2), (changes.Last().Connected, changes.Last().Player));
            Press(replacement, SDL.GamepadButton.South, true);
            Poll(manager);
            router.PollPads();
            Assert.True(sent[(1, PadButton.B)]);
            _ = a;
        }
    }
}

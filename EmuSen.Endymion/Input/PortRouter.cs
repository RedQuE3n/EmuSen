using System;
using System.Collections.Generic;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Input;

namespace EmuSen.Endymion.Input
{
    // Every player's pad and the keyboard routed to the running game's controller ports, shared by both frontends - see EmuSen_Input.md §8.5.
    public sealed class PortRouter
    {
        private static readonly PadButton[] Buttons = Enum.GetValues<PadButton>();
        private static readonly PadControl[] Controls = Enum.GetValues<PadControl>();

        private readonly NativeHandle _native;

        private readonly GamepadManager _pads;
        private readonly Action<int, PadButton, bool> _setButton;
        private readonly Action<int, PadAxis, double> _setAxis;
        private readonly Action<int, bool>? _setConnected;

        // The axes the running game reads, as the library was told them, in their order; the pads are read for these.
        private PadAxis[] _axes = Array.Empty<PadAxis>();
        private Func<PadControl, bool> _keyboardHeld = _ => false;

        // The library's rules - see EmuSen_RustPlatform.md §12.2.
        public unsafe PortRouter(GamepadManager pads, Action<int, PadButton, bool> setButton, Action<int, PadAxis, double> setAxis, Action<int, bool>? setConnected = null)
        {
            EndymionNative.Up();
            _pads = pads;
            _setButton = setButton;
            _setAxis = setAxis;
            _setConnected = setConnected;
            _native = new NativeHandle(EndymionNative.RouterNew(), EndymionNative.RouterFree);
        }

        // A port's motors to the pad of the player it hears, port 0 player 1; nothing past the game's ports - see EmuSen_Input.md §9.
        public void Rumble(int port, ushort strong, ushort weak)
        {
            if (port >= 0 && port < Ports) _pads.Rumble(port + 1, strong, weak);
        }

        // How many controller ports the running game has; nothing is sent past them.
        public int Ports => Get(EndymionNative.RouterPorts);

        // 1-based; the keyboard's controls count as this player's pad.
        public int KeyboardPlayer
        {
            get => Get(EndymionNative.RouterKeyboardPlayer);
            set => Set(EndymionNative.RouterKeyboardPlayer, value);
        }

        public bool MirrorPlayer1ToPlayer2
        {
            get => Get(EndymionNative.RouterMirror) != 0;
            set => Set(EndymionNative.RouterMirror, value ? 1 : 0);
        }

        public Func<PadControl, bool> KeyboardHeld
        {
            get => _keyboardHeld;
            set => _keyboardHeld = value;
        }

        // A game just started: its ports and the axes it reads, with nothing held and nothing yet sent.
        public unsafe void Reset(int ports, IReadOnlyList<PadAxis> axes)
        {
            _axes = new PadAxis[axes.Count];
            for (int i = 0; i < _axes.Length; i++) _axes[i] = axes[i];
            uint[] values = Array.ConvertAll(_axes, a => (uint)a);
            fixed (uint* v = values) EndymionNative.RouterReset(_native.Value, ports, v, (nuint)values.Length);
            GC.KeepAlive(_native);
        }

        // The running game's ports changed in number: a port taken away lets go of its buttons, the rest keep what they were sent - see EmuSen_Input.md §8.11.
        public unsafe void Resize(int ports)
        {
            RouterChange[] room = new RouterChange[Math.Max(1, (Ports - Math.Max(0, ports)) * Buttons.Length)];
            long made;
            fixed (RouterChange* o = room) made = EndymionNative.RouterResize(_native.Value, ports, o, (nuint)room.Length);
            Apply(room, made);
        }

        // A pad pressed for the first time claims its player, then the pad of each player a port hears is read and what changed sent with every axis.
        public unsafe void PollPads()
        {
            _pads.ClaimPressed();
            int players = Math.Min(PlayerSlots.MaxPlayers, Ports);
            ushort[] held = new ushort[players];
            double[] axes = new double[players * _axes.Length];
            for (int player = 1; player <= players; player++)
            {
                foreach (PadButton button in Buttons)
                    if (_pads.IsPressed(button, player)) held[player - 1] |= (ushort)(1 << (int)button);
                for (int k = 0; k < _axes.Length; k++) axes[(player - 1) * _axes.Length + k] = _pads.Axis(_axes[k], player);
            }
            uint keyboard = Keyboard(), seated = Seated();
            RouterChange[] room = new RouterChange[Bound()];
            long made;
            fixed (ushort* h = held)
            fixed (double* a = axes)
            fixed (RouterChange* o = room)
                made = EndymionNative.RouterPoll(_native.Value, h, (nuint)held.Length, a, (nuint)axes.Length, keyboard, seated, o, (nuint)room.Length);
            Apply(room, made);
        }

        // A player's pad as the last poll read it, through that player's bindings.
        public unsafe bool PadHeld(int player, PadButton button)
        {
            bool held = EndymionNative.RouterPadHeld(_native.Value, player, (uint)button) != 0;
            GC.KeepAlive(_native);
            return held;
        }

        // After a key goes down or up: the keyboard as it is now, the pads as last read.
        public unsafe void KeysChanged()
        {
            uint keyboard = Keyboard(), seated = Seated();
            RouterChange[] room = new RouterChange[Bound()];
            long made;
            fixed (RouterChange* o = room) made = EndymionNative.RouterSend(_native.Value, keyboard, seated, o, (nuint)room.Length);
            Apply(room, made);
        }

        // A port past the first holds a controller while its player has a pad seated, connected or reserved, or the keyboard - see EmuSen_Input.md §8.6.
        public unsafe bool Connected(int port)
        {
            bool connected = EndymionNative.RouterConnected(_native.Value, port, Seated()) != 0;
            GC.KeepAlive(_native);
            return connected;
        }

        // The most changes one send can make: each port's controller past the first, and each port's buttons and axes.
        private int Bound() => Math.Max(1, Math.Max(0, Ports - 1) + Ports * (Buttons.Length + _axes.Length));

        // The keyboard as it is now, every control asked once.
        private uint Keyboard()
        {
            uint mask = 0;
            foreach (PadControl control in Controls)
                if (_keyboardHeld(control)) mask |= 1u << (int)control;
            return mask;
        }

        // The players whose pad the game may hear: seated, connected or kept, and not cut off by the first controller alone.
        private uint Seated()
        {
            uint mask = 0;
            for (int player = 1; player <= PlayerSlots.MaxPlayers; player++)
                if (!(_pads.FirstControllerOnly && player > 1) && _pads.Players.SeatOf(player) is not null) mask |= 1u << (player - 1);
            return mask;
        }

        // The library's changes, made to the core in its order; ones that did not fit the room are taken first.
        private unsafe void Apply(RouterChange[] room, long made)
        {
            if (made > room.Length)
            {
                room = new RouterChange[made];
                fixed (RouterChange* o = room) EndymionNative.RouterTake(_native.Value, o, (nuint)room.Length);
            }
            GC.KeepAlive(_native);
            for (int i = 0; i < made; i++)
            {
                RouterChange change = room[i];
                switch (change.Kind)
                {
                    case EndymionNative.ChangeConnected: _setConnected?.Invoke(change.Port, change.On != 0); break;
                    case EndymionNative.ChangeButton: _setButton(change.Port, (PadButton)change.Which, change.On != 0); break;
                    default: _setAxis(change.Port, (PadAxis)change.Which, change.Value); break;
                }
            }
        }

        private unsafe int Get(uint which)
        {
            int value = EndymionNative.RouterGet(_native.Value, which);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe void Set(uint which, int value)
        {
            EndymionNative.RouterSet(_native.Value, which, value);
            GC.KeepAlive(_native);
        }
    }
}

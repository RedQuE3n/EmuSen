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

        private readonly Managed? _managed;
        private readonly NativeHandle? _native;

        private readonly GamepadManager _pads;
        private readonly Action<int, PadButton, bool> _setButton;
        private readonly Action<int, PadAxis, double> _setAxis;
        private readonly Action<int, bool>? _setConnected;

        // The axes the running game reads, as the library was told them, in their order; the pads are read for these.
        private PadAxis[] _axes = Array.Empty<PadAxis>();
        private Func<PadControl, bool> _keyboardHeld = _ => false;

        public PortRouter(GamepadManager pads, Action<int, PadButton, bool> setButton, Action<int, PadAxis, double> setAxis, Action<int, bool>? setConnected = null)
            : this(pads, setButton, setAxis, setConnected, EndymionNative.Active) { }

        // The library's rules or the C#'s, chosen for the instance; a parity test asks for each - see EmuSen_RustPlatform.md §12.2.
        internal unsafe PortRouter(GamepadManager pads, Action<int, PadButton, bool> setButton, Action<int, PadAxis, double> setAxis, Action<int, bool>? setConnected, bool native)
        {
            _pads = pads;
            _setButton = setButton;
            _setAxis = setAxis;
            _setConnected = setConnected;
            if (native) _native = new NativeHandle(EndymionNative.RouterNew(), EndymionNative.RouterFree);
            else _managed = new Managed(pads, setButton, setAxis, setConnected);
        }

        // How many controller ports the running game has; nothing is sent past them.
        public int Ports => _managed?.Ports ?? Get(EndymionNative.RouterPorts);

        // 1-based; the keyboard's controls count as this player's pad.
        public int KeyboardPlayer
        {
            get => _managed?.KeyboardPlayer ?? Get(EndymionNative.RouterKeyboardPlayer);
            set { if (_managed is not null) _managed.KeyboardPlayer = value; else Set(EndymionNative.RouterKeyboardPlayer, value); }
        }

        public bool MirrorPlayer1ToPlayer2
        {
            get => _managed?.MirrorPlayer1ToPlayer2 ?? Get(EndymionNative.RouterMirror) != 0;
            set { if (_managed is not null) _managed.MirrorPlayer1ToPlayer2 = value; else Set(EndymionNative.RouterMirror, value ? 1 : 0); }
        }

        public Func<PadControl, bool> KeyboardHeld
        {
            get => _managed?.KeyboardHeld ?? _keyboardHeld;
            set { if (_managed is not null) _managed.KeyboardHeld = value; else _keyboardHeld = value; }
        }

        // A game just started: its ports and the axes it reads, with nothing held and nothing yet sent.
        public unsafe void Reset(int ports, IReadOnlyList<PadAxis> axes)
        {
            if (_managed is not null)
            {
                _managed.Reset(ports, axes);
                return;
            }
            _axes = new PadAxis[axes.Count];
            for (int i = 0; i < _axes.Length; i++) _axes[i] = axes[i];
            uint[] values = Array.ConvertAll(_axes, a => (uint)a);
            fixed (uint* v = values) EndymionNative.RouterReset(_native!.Value, ports, v, (nuint)values.Length);
            GC.KeepAlive(_native);
        }

        // The running game's ports changed in number: a port taken away lets go of its buttons, the rest keep what they were sent - see EmuSen_Input.md §8.11.
        public unsafe void Resize(int ports)
        {
            if (_managed is not null)
            {
                _managed.Resize(ports);
                return;
            }
            RouterChange[] room = new RouterChange[Math.Max(1, (Ports - Math.Max(0, ports)) * Buttons.Length)];
            long made;
            fixed (RouterChange* o = room) made = EndymionNative.RouterResize(_native!.Value, ports, o, (nuint)room.Length);
            Apply(room, made);
        }

        // Reads the pad of each player a port hears, then sends what changed and every axis.
        public unsafe void PollPads()
        {
            if (_managed is not null)
            {
                _managed.PollPads();
                return;
            }
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
                made = EndymionNative.RouterPoll(_native!.Value, h, (nuint)held.Length, a, (nuint)axes.Length, keyboard, seated, o, (nuint)room.Length);
            Apply(room, made);
        }

        // A player's pad as the last poll read it, through that player's bindings.
        public unsafe bool PadHeld(int player, PadButton button)
        {
            if (_managed is not null) return _managed.PadHeld(player, button);
            bool held = EndymionNative.RouterPadHeld(_native!.Value, player, (uint)button) != 0;
            GC.KeepAlive(_native);
            return held;
        }

        // After a key goes down or up: the keyboard as it is now, the pads as last read.
        public unsafe void KeysChanged()
        {
            if (_managed is not null)
            {
                _managed.KeysChanged();
                return;
            }
            uint keyboard = Keyboard(), seated = Seated();
            RouterChange[] room = new RouterChange[Bound()];
            long made;
            fixed (RouterChange* o = room) made = EndymionNative.RouterSend(_native!.Value, keyboard, seated, o, (nuint)room.Length);
            Apply(room, made);
        }

        // A port past the first holds a controller while its player has a pad seated, connected or reserved, or the keyboard - see EmuSen_Input.md §8.6.
        public unsafe bool Connected(int port)
        {
            if (_managed is not null) return _managed.Connected(port);
            bool connected = EndymionNative.RouterConnected(_native!.Value, port, Seated()) != 0;
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
                fixed (RouterChange* o = room) EndymionNative.RouterTake(_native!.Value, o, (nuint)room.Length);
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
            int value = EndymionNative.RouterGet(_native!.Value, which);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe void Set(uint which, int value)
        {
            EndymionNative.RouterSet(_native!.Value, which, value);
            GC.KeepAlive(_native);
        }

        // The C# rules: the default, and what the library's are held to until Endymion's gate - see EmuSen_RustPlatform.md §3.9.
        internal sealed class Managed
        {
            private static readonly int AxisCount = Enum.GetValues<PadAxis>().Length;

            private readonly GamepadManager _pads;
            private readonly Action<int, PadButton, bool> _setButton;
            private readonly Action<int, PadAxis, double> _setAxis;
            private readonly Action<int, bool>? _setConnected;

            // The pads as the last poll read them, by player; a key change reads these rather than the pads.
            private readonly bool[,] _padHeld = new bool[PlayerSlots.MaxPlayers, Buttons.Length];
            private readonly double[,] _padAxes = new double[PlayerSlots.MaxPlayers, AxisCount];

            // What each port was last told, so a button goes to the core only when it changes.
            private bool[,] _sent = new bool[0, Buttons.Length];
            private IReadOnlyList<PadAxis> _axes = Array.Empty<PadAxis>();

            public Managed(GamepadManager pads, Action<int, PadButton, bool> setButton, Action<int, PadAxis, double> setAxis, Action<int, bool>? setConnected = null)
            {
                _pads = pads;
                _setButton = setButton;
                _setAxis = setAxis;
                _setConnected = setConnected;
            }

            public int Ports => _sent.GetLength(0);

            public int KeyboardPlayer { get; set; } = 1;

            public bool MirrorPlayer1ToPlayer2 { get; set; }

            public Func<PadControl, bool> KeyboardHeld { get; set; } = _ => false;

            public void Reset(int ports, IReadOnlyList<PadAxis> axes)
            {
                _sent = new bool[Math.Max(0, ports), Buttons.Length];
                _axes = axes;
                Array.Clear(_padHeld);
                Array.Clear(_padAxes);
            }

            public void Resize(int ports)
            {
                ports = Math.Max(0, ports);
                for (int port = ports; port < Ports; port++)
                    foreach (PadButton button in Buttons)
                        if (_sent[port, (int)button]) _setButton(port, button, false);
                var sent = new bool[ports, Buttons.Length];
                for (int port = 0; port < Math.Min(ports, Ports); port++)
                    foreach (PadButton button in Buttons) sent[port, (int)button] = _sent[port, (int)button];
                _sent = sent;
            }

            public void PollPads()
            {
                int players = Math.Min(PlayerSlots.MaxPlayers, Ports);
                for (int player = 1; player <= players; player++)
                {
                    foreach (PadButton button in Buttons) _padHeld[player - 1, (int)button] = _pads.IsPressed(button, player);
                    foreach (PadAxis axis in _axes) _padAxes[player - 1, (int)axis] = _pads.Axis(axis, player);
                }
                Send();
            }

            public bool PadHeld(int player, PadButton button) => player is >= 1 and <= PlayerSlots.MaxPlayers && _padHeld[player - 1, (int)button];

            public void KeysChanged() => Send();

            public bool Connected(int port) =>
                port == 0 || PlayerHasInput(port + 1) || (Mirrored(port) is > 0 and var also && PlayerHasInput(also));

            private bool PlayerHasInput(int player) =>
                player == KeyboardPlayer || (!(_pads.FirstControllerOnly && player > 1) && _pads.Players.SeatOf(player) is not null);

            private void Send()
            {
                // Every time, since a state loaded carries the ports it was saved with.
                for (int port = 1; port < Ports; port++) _setConnected?.Invoke(port, Connected(port));
                for (int port = 0; port < Ports; port++)
                {
                    foreach (PadButton button in Buttons)
                    {
                        bool held = Held(port, PadControls.From(button));
                        if (held == _sent[port, (int)button]) continue;
                        _sent[port, (int)button] = held;
                        _setButton(port, button, held);
                    }
                    int p = port;
                    foreach (PadAxis axis in _axes) _setAxis(port, axis, PadControls.Resolve(axis, RawAxis(port, axis), c => Held(p, c)));
                }
            }

            // A port hears its own player, and the second port player 1 as well while mirroring.
            private int Mirrored(int port) => port == 1 && MirrorPlayer1ToPlayer2 ? 1 : 0;

            private bool Held(int port, PadControl control)
            {
                if (HeldBy(port + 1, control)) return true;
                return Mirrored(port) is > 0 and var also && HeldBy(also, control);
            }

            private bool HeldBy(int player, PadControl control)
            {
                if (player == KeyboardPlayer && KeyboardHeld(control)) return true;
                return player <= PlayerSlots.MaxPlayers && PadControls.IsButton(control, out PadButton button) && _padHeld[player - 1, (int)button];
            }

            // Of the pads a port hears, the stick pushed furthest.
            private double RawAxis(int port, PadAxis axis)
            {
                double value = port < PlayerSlots.MaxPlayers ? _padAxes[port, (int)axis] : 0;
                if (Mirrored(port) is > 0 and var also && Math.Abs(_padAxes[also - 1, (int)axis]) > Math.Abs(value)) value = _padAxes[also - 1, (int)axis];
                return value;
            }
        }
    }
}

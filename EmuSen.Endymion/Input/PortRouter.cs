using System;
using System.Collections.Generic;
using EmuSen.Galaxia.Input;

namespace EmuSen.Endymion.Input
{
    // Every player's pad and the keyboard routed to the running game's controller ports, shared by both frontends - see EmuSen_Input.md §8.5.
    public sealed class PortRouter
    {
        private static readonly PadButton[] Buttons = Enum.GetValues<PadButton>();
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

        public PortRouter(GamepadManager pads, Action<int, PadButton, bool> setButton, Action<int, PadAxis, double> setAxis, Action<int, bool>? setConnected = null)
        {
            _pads = pads;
            _setButton = setButton;
            _setAxis = setAxis;
            _setConnected = setConnected;
        }

        // How many controller ports the running game has; nothing is sent past them.
        public int Ports => _sent.GetLength(0);

        // 1-based; the keyboard's controls count as this player's pad.
        public int KeyboardPlayer { get; set; } = 1;

        public bool MirrorPlayer1ToPlayer2 { get; set; }

        public Func<PadControl, bool> KeyboardHeld { get; set; } = _ => false;

        // A game just started: its ports and the axes it reads, with nothing held and nothing yet sent.
        public void Reset(int ports, IReadOnlyList<PadAxis> axes)
        {
            _sent = new bool[Math.Max(0, ports), Buttons.Length];
            _axes = axes;
            Array.Clear(_padHeld);
            Array.Clear(_padAxes);
        }

        // The running game's ports changed in number: a port taken away lets go of its buttons, the rest keep what they were sent - see EmuSen_Input.md §8.11.
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

        // Reads the pad of each player a port hears, then sends what changed and every axis.
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

        // A player's pad as the last poll read it, through that player's bindings.
        public bool PadHeld(int player, PadButton button) => player is >= 1 and <= PlayerSlots.MaxPlayers && _padHeld[player - 1, (int)button];

        // After a key goes down or up: the keyboard as it is now, the pads as last read.
        public void KeysChanged() => Send();

        // A port past the first holds a controller while its player has a pad seated, connected or reserved, or the keyboard - see EmuSen_Input.md §8.6.
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

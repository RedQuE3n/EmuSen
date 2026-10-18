using System;

namespace EmuSen.Cores.Native
{
    // The buttons and axes the player holds on each port, taken on any thread and handed to the machine on its own before each frame - see Diamond_Players.md §12.7.
    internal sealed class InputLatch
    {
        private readonly object _gate = new();
        private uint[] _held = new uint[2], _changed = new uint[2];
        private double[] _axes = Array.Empty<double>();
        private bool[] _moved = Array.Empty<bool>();
        private int _axesPerPort;

        // Whether each port holds a controller, every port full at a new machine, and which changed since the last send - see EmuSen_CoreAPI.md §6.36.
        private bool[] _present = { true, true }, _presenceChanged = new bool[2];

        // What Send copies out under the gate and passes to the machine outside it, kept so a frame allocates nothing.
        private uint[] _sendHeld = Array.Empty<uint>(), _sendChanged = Array.Empty<uint>();
        private double[] _sendAxes = Array.Empty<double>();
        private bool[] _sendMoved = Array.Empty<bool>();
        private bool[] _sendPresent = Array.Empty<bool>(), _sendPresenceChanged = Array.Empty<bool>();

        // Nothing held, for a new machine of ports ports whose axes are numbered below axesPerPort.
        public void Reset(int ports, int axesPerPort)
        {
            lock (_gate)
            {
                _held = new uint[ports];
                _changed = new uint[ports];
                _present = new bool[ports];
                Array.Fill(_present, true);
                _presenceChanged = new bool[ports];
                _axesPerPort = axesPerPort;
                _axes = new double[ports * axesPerPort];
                _moved = new bool[ports * axesPerPort];
            }
        }

        // A port added while the game runs is held from now.
        public void Grow(int ports)
        {
            lock (_gate)
            {
                if (ports <= _held.Length) return;
                int was = _held.Length;
                Array.Resize(ref _held, ports);
                Array.Resize(ref _changed, ports);
                Array.Resize(ref _present, ports);
                Array.Fill(_present, true, was, ports - was);
                Array.Resize(ref _presenceChanged, ports);
                Array.Resize(ref _axes, ports * _axesPerPort);
                Array.Resize(ref _moved, ports * _axesPerPort);
            }
        }

        public int Ports
        {
            get { lock (_gate) return _held.Length; }
        }

        public uint Buttons(int port)
        {
            lock (_gate) return port >= 0 && port < _held.Length ? _held[port] : 0;
        }

        // The bits of changed take mask's values, as set_buttons takes them; a port the machine lacks is ignored.
        public void Hold(int port, uint mask, uint changed)
        {
            lock (_gate)
            {
                if (port < 0 || port >= _held.Length) return;
                _held[port] = (_held[port] & ~changed) | (mask & changed);
                _changed[port] |= changed;
            }
        }

        // A controller plugged in or pulled out; only a change is sent.
        public void Connect(int port, bool connected)
        {
            lock (_gate)
            {
                if (port < 0 || port >= _held.Length || _present[port] == connected) return;
                _present[port] = connected;
                _presenceChanged[port] = !_presenceChanged[port];
            }
        }

        // An axis's latest value replaces any not yet sent.
        public void Move(int port, uint axis, double value)
        {
            lock (_gate)
            {
                if (port < 0 || port >= _held.Length || axis >= _axesPerPort) return;
                int at = port * _axesPerPort + (int)axis;
                _axes[at] = value;
                _moved[at] = true;
            }
        }

        // On the machine's own thread: every port's changed bits and every moved axis since the last send.
        public void Send(CoreMachine machine)
        {
            int ports, perPort;
            lock (_gate)
            {
                ports = _held.Length;
                perPort = _axesPerPort;
                if (_sendHeld.Length != ports) (_sendHeld, _sendChanged) = (new uint[ports], new uint[ports]);
                if (_sendAxes.Length != _axes.Length) (_sendAxes, _sendMoved) = (new double[_axes.Length], new bool[_axes.Length]);
                if (_sendPresent.Length != ports) (_sendPresent, _sendPresenceChanged) = (new bool[ports], new bool[ports]);
                _present.CopyTo(_sendPresent, 0);
                _presenceChanged.CopyTo(_sendPresenceChanged, 0);
                Array.Clear(_presenceChanged);
                _held.CopyTo(_sendHeld, 0);
                _changed.CopyTo(_sendChanged, 0);
                _axes.CopyTo(_sendAxes, 0);
                _moved.CopyTo(_sendMoved, 0);
                Array.Clear(_changed);
                Array.Clear(_moved);
            }
            for (int port = 0; port < ports; port++)
            {
                if (_sendPresenceChanged[port]) machine.SetPresent(port, _sendPresent[port]);
                if (_sendChanged[port] != 0) machine.SetButtons(port, _sendHeld[port], _sendChanged[port]);
                for (int axis = 0; axis < perPort; axis++)
                    if (_sendMoved[port * perPort + axis]) machine.SetAxis(port, (uint)axis, _sendAxes[port * perPort + axis]);
            }
        }
    }
}

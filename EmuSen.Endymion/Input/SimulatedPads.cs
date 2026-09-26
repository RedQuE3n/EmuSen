using System;
using System.Collections.Generic;
using System.Linq;
using SDL3;

namespace EmuSen.Endymion.Input
{
    // Pads with no device behind them, plugged in and pulled out by a test, and the handles GamepadManager holds on them - see EmuSen_Settings_Reference.md §4.61.
    public sealed class SimulatedPads : IPadDevices
    {
        private readonly Dictionary<uint, SimulatedPad> _attached = new();
        private readonly Dictionary<IntPtr, (uint Id, SimulatedPad Pad)> _open = new();
        private uint _nextId = 1;
        private long _nextHandle = 0x1000;
        private bool _changed;

        public static SimulatedPads With(params SimulatedPad[] pads)
        {
            var set = new SimulatedPads();
            foreach (SimulatedPad pad in pads) set.Connect(pad);
            return set;
        }

        // The first pad plugged in that is still attached.
        public SimulatedPad? First => _attached.OrderBy(p => p.Key).Select(p => p.Value).FirstOrDefault();

        // Handles opened and not yet closed: SDL's gamepads a real run would still hold.
        public int OpenHandles => _open.Count;

        public int Opens { get; private set; }
        public int Closes { get; private set; }

        public bool Initialized { get; private set; }

        public uint Connect(SimulatedPad pad)
        {
            uint id = _nextId++;
            _attached[id] = pad;
            _changed = true;
            return id;
        }

        public void Disconnect(SimulatedPad pad)
        {
            foreach (uint id in _attached.Where(p => ReferenceEquals(p.Value, pad)).Select(p => p.Key).ToList()) _attached.Remove(id);
            _changed = true;
        }

        public bool Init() => Initialized = true;

        public void Quit() => Initialized = false;

        public uint[] Attached() => _attached.Keys.Order().ToArray();

        public IntPtr Open(uint id)
        {
            if (!_attached.TryGetValue(id, out SimulatedPad? pad)) return IntPtr.Zero;
            var handle = new IntPtr(_nextHandle++);
            _open[handle] = (id, pad);
            Opens++;
            return handle;
        }

        public void Close(IntPtr pad)
        {
            if (_open.Remove(pad)) Closes++;
        }

        public bool IsAttached(IntPtr pad) => _open.TryGetValue(pad, out var p) && _attached.ContainsKey(p.Id);

        public bool DevicesChanged()
        {
            bool changed = _changed;
            _changed = false;
            return changed;
        }

        public void Update() { }

        private SimulatedPad? Pad(IntPtr handle) => _open.TryGetValue(handle, out var p) ? p.Pad : null;

        public bool Button(IntPtr pad, SDL.GamepadButton button) => Pad(pad)?.IsHeld(button) ?? false;

        public short Axis(IntPtr pad, SDL.GamepadAxis axis) => Pad(pad)?.Axis(axis) ?? 0;

        public string? Name(IntPtr pad) => Pad(pad)?.Name;

        public SDL.GamepadType Type(IntPtr pad) => Pad(pad)?.Type ?? SDL.GamepadType.Unknown;

        public SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button) => SDL.GamepadButtonLabel.Unknown;
    }
}

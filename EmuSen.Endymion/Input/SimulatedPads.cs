using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Endymion.Native;
using SDL3;

namespace EmuSen.Endymion.Input
{
    // Pads with no device behind them, plugged in and pulled out by a test, and the handles GamepadManager holds on them - see EmuSen_Settings_Reference.md §4.61.
    public sealed class SimulatedPads : IPadDevices
    {
        private readonly Managed? _managed;
        private readonly NativeHandle? _native;

        // The pads plugged into the library's set by the id it gave each, so First answers with the test's own object.
        private readonly Dictionary<uint, SimulatedPad> _plugged = new();

        public SimulatedPads() : this(DeviceNative.Active) { }

        // The library's set or the C#'s, chosen for the instance; each takes pads of its own kind - see EmuSen_RustPlatform.md §14.2.
        internal unsafe SimulatedPads(bool native)
        {
            if (native) _native = new NativeHandle(DeviceNative.SimSetNew(), DeviceNative.SimSetFree);
            else _managed = new Managed();
        }

        internal bool IsNative => _native is not null;

        internal nint NativeValue => _native!.Value;

        public static SimulatedPads With(params SimulatedPad[] pads)
        {
            var set = new SimulatedPads(pads.Length > 0 ? pads[0].IsNative : DeviceNative.Active);
            foreach (SimulatedPad pad in pads) set.Connect(pad);
            return set;
        }

        // The first pad plugged in that is still attached.
        public SimulatedPad? First => _managed is not null ? _managed.First : _plugged.GetValueOrDefault((uint)Get(DeviceNative.SetFirst));

        // Handles opened and not yet closed: SDL's gamepads a real run would still hold.
        public int OpenHandles => _managed?.OpenHandles ?? (int)Get(DeviceNative.SetOpenHandles);

        public int Opens => _managed?.Opens ?? (int)Get(DeviceNative.SetOpens);
        public int Closes => _managed?.Closes ?? (int)Get(DeviceNative.SetCloses);

        public bool Initialized => _managed?.Initialized ?? Get(DeviceNative.SetInitialized) != 0;

        public unsafe uint Connect(SimulatedPad pad)
        {
            if (pad.IsNative != IsNative) throw new ArgumentException("A simulated pad is plugged into a set of its own implementation.", nameof(pad));
            if (_managed is not null) return _managed.Connect(pad);
            uint id = (uint)DeviceNative.SimSetConnect(_native!.Value, pad.NativeValue);
            GC.KeepAlive(_native);
            GC.KeepAlive(pad);
            _plugged[id] = pad;
            return id;
        }

        public unsafe void Disconnect(SimulatedPad pad)
        {
            if (_managed is not null)
            {
                _managed.Disconnect(pad);
                return;
            }
            if (!pad.IsNative) return;
            DeviceNative.SimSetDisconnect(_native!.Value, pad.NativeValue);
            GC.KeepAlive(_native);
            GC.KeepAlive(pad);
            foreach (uint id in _plugged.Where(p => ReferenceEquals(p.Value, pad)).Select(p => p.Key).ToList()) _plugged.Remove(id);
        }

        public bool Init() => _managed?.Init() ?? Call(DeviceNative.DeviceInit) != 0;

        public void Quit()
        {
            if (_managed is not null) _managed.Quit();
            else Call(DeviceNative.DeviceQuit);
        }

        public unsafe uint[] Attached()
        {
            if (_managed is not null) return _managed.Attached();
            uint[] ids = new uint[Math.Max(1, _plugged.Count)];
            long count;
            fixed (uint* p = ids) count = DeviceNative.SimSetAttached(_native!.Value, p, (nuint)ids.Length);
            if (count > ids.Length)
            {
                ids = new uint[count];
                fixed (uint* p = ids) count = DeviceNative.SimSetAttached(_native.Value, p, (nuint)ids.Length);
            }
            GC.KeepAlive(_native);
            return ids.AsSpan(0, (int)Math.Min(count, ids.Length)).ToArray();
        }

        public IntPtr Open(uint id) => _managed?.Open(id) ?? (IntPtr)Call(DeviceNative.DeviceOpen, 0, (int)id);

        public void Close(IntPtr pad)
        {
            if (_managed is not null) _managed.Close(pad);
            else Call(DeviceNative.DeviceClose, pad);
        }

        public bool IsAttached(IntPtr pad) => _managed?.IsAttached(pad) ?? Call(DeviceNative.DeviceIsAttached, pad) != 0;

        public bool DevicesChanged() => _managed?.DevicesChanged() ?? Call(DeviceNative.DeviceChanged) != 0;

        public void Update() { }

        public bool Button(IntPtr pad, SDL.GamepadButton button) => _managed?.Button(pad, button) ?? Call(DeviceNative.DeviceButton, pad, (int)button) != 0;

        public short Axis(IntPtr pad, SDL.GamepadAxis axis) => _managed?.Axis(pad, axis) ?? (short)Call(DeviceNative.DeviceAxis, pad, (int)axis);

        public string? Name(IntPtr pad) => _managed is not null ? _managed.Name(pad) : Text(DeviceNative.DeviceName, pad);

        public SDL.GamepadType Type(IntPtr pad) => _managed?.Type(pad) ?? (SDL.GamepadType)Call(DeviceNative.DeviceKind, pad);

        public SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button) =>
            _managed?.Label(pad, button) ?? (SDL.GamepadButtonLabel)Call(DeviceNative.DeviceLabel, pad, (int)button);

        public string Guid(IntPtr pad) => _managed is not null ? _managed.Guid(pad) : Text(DeviceNative.DeviceGuid, pad)!;

        public string? Path(IntPtr pad) => _managed is not null ? _managed.Path(pad) : Text(DeviceNative.DevicePath, pad);

        public void SetPlayerIndex(IntPtr pad, int index)
        {
            if (_managed is not null) _managed.SetPlayerIndex(pad, index);
            else Call(DeviceNative.DeviceSetPlayerIndex, pad, index);
        }

        private unsafe long Get(uint which)
        {
            long value = DeviceNative.SimSetGet(_native!.Value, which);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe long Call(uint op, IntPtr pad = default, int arg = 0)
        {
            long value = DeviceNative.SimSetCall(_native!.Value, op, (ulong)(long)pad, arg);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe string? Text(uint op, IntPtr pad)
        {
            string? text = DeviceNative.Text((buffer, capacity) => DeviceNative.SimSetText(_native!.Value, op, (ulong)(long)pad, buffer, capacity));
            GC.KeepAlive(_native);
            return text;
        }

        // The C# set: the default, and what the library's is held to - see EmuSen_RustPlatform.md §14.
        internal sealed class Managed
        {
            private readonly Dictionary<uint, SimulatedPad> _attached = new();
            private readonly Dictionary<IntPtr, (uint Id, SimulatedPad Pad)> _open = new();
            private uint _nextId = 1;
            private long _nextHandle = 0x1000;
            private bool _changed;

            public SimulatedPad? First => _attached.OrderBy(p => p.Key).Select(p => p.Value).FirstOrDefault();

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

            private SimulatedPad? Pad(IntPtr handle) => _open.TryGetValue(handle, out var p) ? p.Pad : null;

            public bool Button(IntPtr pad, SDL.GamepadButton button) => Pad(pad)?.IsHeld(button) ?? false;

            public short Axis(IntPtr pad, SDL.GamepadAxis axis) => Pad(pad)?.Axis(axis) ?? 0;

            public string? Name(IntPtr pad) => Pad(pad)?.Name;

            public SDL.GamepadType Type(IntPtr pad) => Pad(pad)?.Type ?? SDL.GamepadType.Unknown;

            public SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button) => SDL.GamepadButtonLabel.Unknown;

            public string Guid(IntPtr pad) => Pad(pad) is { } p ? p.Guid ?? Convert.ToHexString(System.Security.Cryptography.MD5.HashData(System.Text.Encoding.UTF8.GetBytes(p.Name))).ToLowerInvariant() : "";

            public string? Path(IntPtr pad) => Pad(pad)?.Path;

            public void SetPlayerIndex(IntPtr pad, int index)
            {
                if (Pad(pad) is { } p) p.PlayerIndex = index;
            }
        }
    }
}

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
        private readonly NativeHandle _native;

        // The pads plugged into the library's set by the id it gave each, so First answers with the test's own object.
        private readonly Dictionary<uint, SimulatedPad> _plugged = new();

        // The library's set - see EmuSen_RustPlatform.md §14.2.
        public unsafe SimulatedPads()
        {
            DeviceNative.Up();
            _native = new NativeHandle(DeviceNative.SimSetNew(), DeviceNative.SimSetFree);
        }

        internal nint NativeValue => _native.Value;

        public static SimulatedPads With(params SimulatedPad[] pads)
        {
            var set = new SimulatedPads();
            foreach (SimulatedPad pad in pads) set.Connect(pad);
            return set;
        }

        // The first pad plugged in that is still attached.
        public SimulatedPad? First => _plugged.GetValueOrDefault((uint)Get(DeviceNative.SetFirst));

        // Handles opened and not yet closed: SDL's gamepads a real run would still hold.
        public int OpenHandles => (int)Get(DeviceNative.SetOpenHandles);

        public int Opens => (int)Get(DeviceNative.SetOpens);
        public int Closes => (int)Get(DeviceNative.SetCloses);

        public bool Initialized => Get(DeviceNative.SetInitialized) != 0;

        public unsafe uint Connect(SimulatedPad pad)
        {
            uint id = (uint)DeviceNative.SimSetConnect(_native.Value, pad.NativeValue);
            GC.KeepAlive(_native);
            GC.KeepAlive(pad);
            _plugged[id] = pad;
            return id;
        }

        public unsafe void Disconnect(SimulatedPad pad)
        {
            DeviceNative.SimSetDisconnect(_native.Value, pad.NativeValue);
            GC.KeepAlive(_native);
            GC.KeepAlive(pad);
            foreach (uint id in _plugged.Where(p => ReferenceEquals(p.Value, pad)).Select(p => p.Key).ToList()) _plugged.Remove(id);
        }

        public bool Init() => Call(DeviceNative.DeviceInit) != 0;

        public void Quit()
        {
            Call(DeviceNative.DeviceQuit);
        }

        public unsafe uint[] Attached()
        {
            uint[] ids = new uint[Math.Max(1, _plugged.Count)];
            long count;
            fixed (uint* p = ids) count = DeviceNative.SimSetAttached(_native.Value, p, (nuint)ids.Length);
            if (count > ids.Length)
            {
                ids = new uint[count];
                fixed (uint* p = ids) count = DeviceNative.SimSetAttached(_native.Value, p, (nuint)ids.Length);
            }
            GC.KeepAlive(_native);
            return ids.AsSpan(0, (int)Math.Min(count, ids.Length)).ToArray();
        }

        public IntPtr Open(uint id) => (IntPtr)Call(DeviceNative.DeviceOpen, 0, (int)id);

        public void Close(IntPtr pad)
        {
            Call(DeviceNative.DeviceClose, pad);
        }

        public bool IsAttached(IntPtr pad) => Call(DeviceNative.DeviceIsAttached, pad) != 0;

        public bool DevicesChanged() => Call(DeviceNative.DeviceChanged) != 0;

        public void Update() { }

        public bool Button(IntPtr pad, SDL.GamepadButton button) => Call(DeviceNative.DeviceButton, pad, (int)button) != 0;

        public short Axis(IntPtr pad, SDL.GamepadAxis axis) => (short)Call(DeviceNative.DeviceAxis, pad, (int)axis);

        public string? Name(IntPtr pad) => Text(DeviceNative.DeviceName, pad);

        public SDL.GamepadType Type(IntPtr pad) => (SDL.GamepadType)Call(DeviceNative.DeviceKind, pad);

        public SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button) =>
            (SDL.GamepadButtonLabel)Call(DeviceNative.DeviceLabel, pad, (int)button);

        public string Guid(IntPtr pad) => Text(DeviceNative.DeviceGuid, pad)!;

        public string? Path(IntPtr pad) => Text(DeviceNative.DevicePath, pad);

        public uint VendorProduct(IntPtr pad) => (uint)Call(DeviceNative.DeviceVendorProduct, pad);

        public void SetPlayerIndex(IntPtr pad, int index)
        {
            Call(DeviceNative.DeviceSetPlayerIndex, pad, index);
        }

        public void Rumble(IntPtr pad, ushort strong, ushort weak)
        {
            Call(DeviceNative.DeviceRumble, pad, (int)((uint)strong << 16 | weak));
        }

        private unsafe long Get(uint which)
        {
            long value = DeviceNative.SimSetGet(_native.Value, which);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe long Call(uint op, IntPtr pad = default, int arg = 0)
        {
            long value = DeviceNative.SimSetCall(_native.Value, op, (ulong)(long)pad, arg);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe string? Text(uint op, IntPtr pad)
        {
            string? text = DeviceNative.Text((buffer, capacity) => DeviceNative.SimSetText(_native.Value, op, (ulong)(long)pad, buffer, capacity));
            GC.KeepAlive(_native);
            return text;
        }
    }
}

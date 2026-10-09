using System;
using EmuSen.Endymion.Native;
using SDL3;

namespace EmuSen.Endymion.Input
{
    // One opened pad; once it is closed it reads as nothing held and keeps the name and type it had - see EmuSen_Settings_Reference.md §4.61.
    public sealed class ConnectedPad
    {
        private readonly IPadDevices? _devices;
        private string? _closedName;
        private SDL.GamepadType _closedType;

        // The library's pads and this pad's key in them, when the library's manager opened it - see EmuSen_RustPlatform.md §14.2.
        private readonly NativeHandle? _pads;
        private readonly ulong _key;

        internal ConnectedPad(IPadDevices devices, uint id, IntPtr handle)
        {
            _devices = devices;
            Id = id;
            Handle = handle;
            Guid = devices.Guid(handle);
            Path = devices.Path(handle);
        }

        internal unsafe ConnectedPad(NativeHandle pads, ulong key)
        {
            _pads = pads;
            _key = key;
            Id = (uint)Get(DeviceNative.PadsId);
            Guid = DeviceNative.Text((buffer, capacity) => DeviceNative.PadsText(pads.Value, key, DeviceNative.PadsGuid, buffer, capacity)) ?? "";
            Path = DeviceNative.Text((buffer, capacity) => DeviceNative.PadsText(pads.Value, key, DeviceNative.PadsPath, buffer, capacity));
            GC.KeepAlive(pads);
        }

        internal ulong Key => _key;

        public uint Id { get; }

        // Read once at opening, so a pad that has gone is still known by them - see EmuSen_Input.md §8.
        public string Guid { get; }
        public string? Path { get; }

        internal IntPtr Handle { get; private set; }

        public bool IsOpen => _pads is not null ? Get(DeviceNative.PadsIsOpen) != 0 : Handle != IntPtr.Zero;

        public string Name => _pads is not null ? NativeName() : !IsOpen ? _closedName ?? "Unknown controller" : _devices!.Name(Handle) is { Length: > 0 } name ? name : "Unknown controller";

        // SDL's own reading of what the pad is, from its vendor and product - see EmuSen_Settings_Reference.md §4.52.
        public SDL.GamepadType Type => _pads is not null ? (SDL.GamepadType)Get(DeviceNative.PadsKind) : IsOpen ? _devices!.Type(Handle) : _closedType;

        public bool IsRawPressed(SDL.GamepadButton button) => _pads is not null ? Get(DeviceNative.PadsRawPressed, (int)button) != 0 : IsOpen && _devices!.Button(Handle, button);

        internal short AxisValue(SDL.GamepadAxis axis) => _pads is not null ? (short)Get(DeviceNative.PadsAxisValue, (int)axis) : IsOpen ? _devices!.Axis(Handle, axis) : (short)0;

        public double RawAxis(SDL.GamepadAxis axis) => _pads is not null ? NativeRawAxis(axis) : Math.Clamp(AxisValue(axis) / (double)short.MaxValue, -1.0, 1.0);

        public string? ButtonLabel(SDL.GamepadButton button)
        {
            if (_pads is not null) return Get(DeviceNative.PadsLabel, (int)button) is var known and not 0 ? ((SDL.GamepadButtonLabel)known).ToString() : null;
            if (!IsOpen) return null;
            SDL.GamepadButtonLabel label = _devices!.Label(Handle, button);
            return label == SDL.GamepadButtonLabel.Unknown ? null : label.ToString();
        }

        internal void Close()
        {
            if (!IsOpen) return;
            _closedName = Name;
            _closedType = Type;
            _devices!.Close(Handle);
            Handle = IntPtr.Zero;
        }

        private unsafe long Get(uint which, int arg = 0)
        {
            long value = DeviceNative.PadsGet(_pads!.Value, _key, which, arg);
            GC.KeepAlive(_pads);
            return value;
        }

        private unsafe string NativeName()
        {
            string name = DeviceNative.Text((buffer, capacity) => DeviceNative.PadsText(_pads!.Value, _key, DeviceNative.PadsName, buffer, capacity))!;
            GC.KeepAlive(_pads);
            return name;
        }

        private unsafe double NativeRawAxis(SDL.GamepadAxis axis)
        {
            double value;
            DeviceNative.PadsAxis(_pads!.Value, _key, DeviceNative.AxisRaw, (uint)(int)axis, &value);
            GC.KeepAlive(_pads);
            return value;
        }
    }

    // A pad plugged in or pulled out, as GamepadManager.Poll found it.
    public readonly record struct PadConnection(ConnectedPad Pad, bool Connected)
    {
        // The player the pad took or keeps reserved, 1-based; 0 for a pad seated nowhere.
        public int Player { get; init; }
    }
}

using System;
using EmuSen.Endymion.Native;
using SDL3;

namespace EmuSen.Endymion.Input
{
    // One opened pad; once it is closed it reads as nothing held and keeps the name and type it had - see EmuSen_Settings_Reference.md §4.61.
    public sealed class ConnectedPad
    {
        // The library's pads and this pad's key in them - see EmuSen_RustPlatform.md §14.2.
        private readonly NativeHandle _pads;
        private readonly ulong _key;

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

        // SDL's vendor and product, read while open and kept once closed; under Steam Input, the controller behind the virtual pad - see EmuSen_Input.md §8.13.2.
        public ushort Vendor => (ushort)(VendorProduct >> 16);
        public ushort Product => (ushort)VendorProduct;

        private uint VendorProduct => (uint)Get(DeviceNative.PadsVendorProduct);

        // A handheld's own controls, known by their vendor and product - see EmuSen_Input.md §8.13.
        public bool IsBuiltIn => BuiltInControls.Contains(Vendor, Product);

        public bool IsOpen => Get(DeviceNative.PadsIsOpen) != 0;

        public unsafe string Name
        {
            get
            {
                string name = DeviceNative.Text((buffer, capacity) => DeviceNative.PadsText(_pads.Value, _key, DeviceNative.PadsName, buffer, capacity))!;
                GC.KeepAlive(_pads);
                return name;
            }
        }

        // SDL's own reading of what the pad is, from its vendor and product - see EmuSen_Settings_Reference.md §4.52.
        public SDL.GamepadType Type => (SDL.GamepadType)Get(DeviceNative.PadsKind);

        // Any button held, or a trigger past half its travel: what claims a player and lights a row - see EmuSen_Input.md §8.13.
        public bool AnyPressed()
        {
            for (var b = SDL.GamepadButton.South; b < SDL.GamepadButton.Count; b++)
                if (IsRawPressed(b)) return true;
            return RawAxis(SDL.GamepadAxis.LeftTrigger) > 0.5 || RawAxis(SDL.GamepadAxis.RightTrigger) > 0.5;
        }

        public bool IsRawPressed(SDL.GamepadButton button) => Get(DeviceNative.PadsRawPressed, (int)button) != 0;

        public unsafe double RawAxis(SDL.GamepadAxis axis)
        {
            double value;
            DeviceNative.PadsAxis(_pads.Value, _key, DeviceNative.AxisRaw, (uint)(int)axis, &value);
            GC.KeepAlive(_pads);
            return value;
        }

        public string? ButtonLabel(SDL.GamepadButton button) => Get(DeviceNative.PadsLabel, (int)button) is var known and not 0 ? ((SDL.GamepadButtonLabel)known).ToString() : null;

        // Drives the pad's motors for RumbleMs; a closed pad is left alone - see EmuSen_Input.md §9.
        public unsafe void Rumble(ushort strong, ushort weak)
        {
            DeviceNative.PadsRumble(_pads.Value, _key, strong, weak);
            GC.KeepAlive(_pads);
        }

        private unsafe long Get(uint which, int arg = 0)
        {
            long value = DeviceNative.PadsGet(_pads.Value, _key, which, arg);
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

using System;
using System.Collections.Generic;
using EmuSen.Endymion.Native;
using SDL3;

namespace EmuSen.Endymion.Input
{
    // A pad with no device behind it, read by GamepadManager in place of SDL's - see EmuSen_Settings_Reference.md §4.45.
    public sealed class SimulatedPad
    {
        private static int _made;

        private readonly NativeHandle _native;

        // The library's pad - see EmuSen_RustPlatform.md §14.2.
        public unsafe SimulatedPad()
        {
            DeviceNative.Up();
            string path = $"/dev/input/simulated{System.Threading.Interlocked.Increment(ref _made)}";
            byte[] bytes = DeviceNative.Utf8(path)!;
            fixed (byte* p = bytes) _native = new NativeHandle(DeviceNative.SimPadNew(p, (nuint)bytes.Length), DeviceNative.SimPadFree);
        }

        internal nint NativeValue => _native.Value;

        public string Name
        {
            get => GetText(DeviceNative.PadName)!;
            set => SetText(DeviceNative.PadName, value);
        }

        // SDL's GUID is the model's, so two pads of one name share one unless a test says otherwise.
        public string? Guid
        {
            get => GetText(DeviceNative.PadGuid);
            set => SetText(DeviceNative.PadGuid, value);
        }

        // One per pad, as a USB port's; a test changes it to plug the pad in somewhere else.
        public string? Path
        {
            get => GetText(DeviceNative.PadPath);
            set => SetText(DeviceNative.PadPath, value);
        }

        // The player number SDL was last asked to light on it, or -1.
        public int PlayerIndex
        {
            get => GetNumber(DeviceNative.PadPlayerIndex);
            set => SetNumber(DeviceNative.PadPlayerIndex, value);
        }

        // The motors as last driven, strong and weak.
        public (ushort Strong, ushort Weak) Rumble => ((ushort)((uint)GetNumber(DeviceNative.PadRumble) >> 16), (ushort)GetNumber(DeviceNative.PadRumble));

        // What SDL would say the pad is, from which the interface picks its button drawings - see EmuSen_Settings_Reference.md §4.52.
        public SDL.GamepadType Type
        {
            get => (SDL.GamepadType)GetNumber(DeviceNative.PadKind);
            set => SetNumber(DeviceNative.PadKind, (int)value);
        }

        // The USB vendor and product SDL would report, by which a handheld's own controls are known - see EmuSen_Input.md §8.13.
        public ushort Vendor
        {
            get => (ushort)((uint)GetNumber(DeviceNative.PadVendorProduct) >> 16);
            set => SetNumber(DeviceNative.PadVendorProduct, (int)((uint)value << 16 | Product));
        }

        public ushort Product
        {
            get => (ushort)GetNumber(DeviceNative.PadVendorProduct);
            set => SetNumber(DeviceNative.PadVendorProduct, (int)((uint)Vendor << 16 | value));
        }

        public void Press(SDL.GamepadButton button) => Hold((int)button, 1);

        public void Release(SDL.GamepadButton button) => Hold((int)button, 0);

        public void ReleaseAll() => Hold(0, 2);

        // -1 to 1 for a stick, 0 to 1 for a trigger, as GamepadManager.RawAxis reports them.
        public unsafe void SetAxis(SDL.GamepadAxis axis, double value)
        {
            DeviceNative.SimPadSetAxis(_native.Value, (int)axis, value);
            GC.KeepAlive(_native);
        }

        public unsafe bool IsHeld(SDL.GamepadButton button)
        {
            bool held = DeviceNative.SimPadHeld(_native.Value, (int)button) != 0;
            GC.KeepAlive(_native);
            return held;
        }

        public unsafe short Axis(SDL.GamepadAxis axis)
        {
            short value = (short)DeviceNative.SimPadAxis(_native.Value, (int)axis);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe void Hold(int button, uint down)
        {
            DeviceNative.SimPadPress(_native.Value, button, down);
            GC.KeepAlive(_native);
        }

        private unsafe string? GetText(uint which)
        {
            string? text = DeviceNative.Text((buffer, capacity) => DeviceNative.SimPadText(_native.Value, which, buffer, capacity));
            GC.KeepAlive(_native);
            return text;
        }

        private unsafe void SetText(uint which, string? value)
        {
            byte[]? bytes = DeviceNative.Utf8(value);
            int status;
            fixed (byte* p = bytes) status = DeviceNative.SimPadSetText(_native.Value, which, bytes is null ? null : DeviceNative.Pin(p), (nuint)(bytes?.Length ?? 0));
            GC.KeepAlive(_native);
            if (status == EndymionNative.Null) throw new ArgumentNullException(nameof(value));
        }

        private unsafe int GetNumber(uint which)
        {
            int value;
            DeviceNative.SimPadGet(_native.Value, which, &value);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe void SetNumber(uint which, int value)
        {
            DeviceNative.SimPadSet(_native.Value, which, value);
            GC.KeepAlive(_native);
        }
    }
}

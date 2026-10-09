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

        private readonly Managed? _managed;
        private readonly NativeHandle? _native;

        public SimulatedPad() : this(DeviceNative.Active) { }

        // The library's pad or the C#'s, chosen for the instance; a set takes pads of its own kind - see EmuSen_RustPlatform.md §14.2.
        internal unsafe SimulatedPad(bool native)
        {
            string path = $"/dev/input/simulated{System.Threading.Interlocked.Increment(ref _made)}";
            if (!native)
            {
                _managed = new Managed { Path = path };
                return;
            }
            byte[] bytes = DeviceNative.Utf8(path)!;
            fixed (byte* p = bytes) _native = new NativeHandle(DeviceNative.SimPadNew(p, (nuint)bytes.Length), DeviceNative.SimPadFree);
        }

        internal bool IsNative => _native is not null;

        internal nint NativeValue => _native!.Value;

        public string Name
        {
            get => _managed is not null ? _managed.Name : GetText(DeviceNative.PadName)!;
            set { if (_managed is not null) _managed.Name = value; else SetText(DeviceNative.PadName, value); }
        }

        // SDL's GUID is the model's, so two pads of one name share one unless a test says otherwise.
        public string? Guid
        {
            get => _managed is not null ? _managed.Guid : GetText(DeviceNative.PadGuid);
            set { if (_managed is not null) _managed.Guid = value; else SetText(DeviceNative.PadGuid, value); }
        }

        // One per pad, as a USB port's; a test changes it to plug the pad in somewhere else.
        public string? Path
        {
            get => _managed is not null ? _managed.Path : GetText(DeviceNative.PadPath);
            set { if (_managed is not null) _managed.Path = value; else SetText(DeviceNative.PadPath, value); }
        }

        // The player number SDL was last asked to light on it, or -1.
        public int PlayerIndex
        {
            get => _managed is not null ? _managed.PlayerIndex : GetNumber(DeviceNative.PadPlayerIndex);
            set { if (_managed is not null) _managed.PlayerIndex = value; else SetNumber(DeviceNative.PadPlayerIndex, value); }
        }

        // What SDL would say the pad is, from which the interface picks its button drawings - see EmuSen_Settings_Reference.md §4.52.
        public SDL.GamepadType Type
        {
            get => _managed is not null ? _managed.Type : (SDL.GamepadType)GetNumber(DeviceNative.PadKind);
            set { if (_managed is not null) _managed.Type = value; else SetNumber(DeviceNative.PadKind, (int)value); }
        }

        public void Press(SDL.GamepadButton button)
        {
            if (_managed is not null) _managed.Press(button);
            else Hold((int)button, 1);
        }

        public void Release(SDL.GamepadButton button)
        {
            if (_managed is not null) _managed.Release(button);
            else Hold((int)button, 0);
        }

        public void ReleaseAll()
        {
            if (_managed is not null) _managed.ReleaseAll();
            else Hold(0, 2);
        }

        // -1 to 1 for a stick, 0 to 1 for a trigger, as GamepadManager.RawAxis reports them.
        public unsafe void SetAxis(SDL.GamepadAxis axis, double value)
        {
            if (_managed is not null)
            {
                _managed.SetAxis(axis, value);
                return;
            }
            DeviceNative.SimPadSetAxis(_native!.Value, (int)axis, value);
            GC.KeepAlive(_native);
        }

        public unsafe bool IsHeld(SDL.GamepadButton button)
        {
            if (_managed is not null) return _managed.IsHeld(button);
            bool held = DeviceNative.SimPadHeld(_native!.Value, (int)button) != 0;
            GC.KeepAlive(_native);
            return held;
        }

        public unsafe short Axis(SDL.GamepadAxis axis)
        {
            if (_managed is not null) return _managed.Axis(axis);
            short value = (short)DeviceNative.SimPadAxis(_native!.Value, (int)axis);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe void Hold(int button, uint down)
        {
            DeviceNative.SimPadPress(_native!.Value, button, down);
            GC.KeepAlive(_native);
        }

        private unsafe string? GetText(uint which)
        {
            string? text = DeviceNative.Text((buffer, capacity) => DeviceNative.SimPadText(_native!.Value, which, buffer, capacity));
            GC.KeepAlive(_native);
            return text;
        }

        private unsafe void SetText(uint which, string? value)
        {
            byte[]? bytes = DeviceNative.Utf8(value);
            int status;
            fixed (byte* p = bytes) status = DeviceNative.SimPadSetText(_native!.Value, which, bytes is null ? null : DeviceNative.Pin(p), (nuint)(bytes?.Length ?? 0));
            GC.KeepAlive(_native);
            if (status == EndymionNative.Null) throw new ArgumentNullException(nameof(value));
        }

        private unsafe int GetNumber(uint which)
        {
            int value;
            DeviceNative.SimPadGet(_native!.Value, which, &value);
            GC.KeepAlive(_native);
            return value;
        }

        private unsafe void SetNumber(uint which, int value)
        {
            DeviceNative.SimPadSet(_native!.Value, which, value);
            GC.KeepAlive(_native);
        }

        // The C# pad: the default, and what the library's is held to - see EmuSen_RustPlatform.md §14.
        internal sealed class Managed
        {
            private readonly HashSet<SDL.GamepadButton> _held = new();
            private readonly Dictionary<SDL.GamepadAxis, short> _axes = new();

            public string Name { get; set; } = "Simulated pad";

            public string? Guid { get; set; }

            public string? Path { get; set; }

            public int PlayerIndex { get; set; } = -1;

            public SDL.GamepadType Type { get; set; } = SDL.GamepadType.Unknown;

            public void Press(SDL.GamepadButton button) => _held.Add(button);

            public void Release(SDL.GamepadButton button) => _held.Remove(button);

            public void ReleaseAll()
            {
                _held.Clear();
                _axes.Clear();
            }

            public void SetAxis(SDL.GamepadAxis axis, double value) =>
                _axes[axis] = (short)Math.Round(Math.Clamp(value, -1.0, 1.0) * short.MaxValue);

            public bool IsHeld(SDL.GamepadButton button) => _held.Contains(button);

            public short Axis(SDL.GamepadAxis axis) => _axes.TryGetValue(axis, out short value) ? value : (short)0;
        }
    }
}

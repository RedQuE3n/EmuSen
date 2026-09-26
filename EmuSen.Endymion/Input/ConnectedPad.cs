using System;
using SDL3;

namespace EmuSen.Endymion.Input
{
    // One opened pad; once it is closed it reads as nothing held and keeps the name and type it had - see EmuSen_Settings_Reference.md §4.61.
    public sealed class ConnectedPad
    {
        private readonly IPadDevices _devices;
        private string? _closedName;
        private SDL.GamepadType _closedType;

        internal ConnectedPad(IPadDevices devices, uint id, IntPtr handle)
        {
            _devices = devices;
            Id = id;
            Handle = handle;
        }

        public uint Id { get; }

        internal IntPtr Handle { get; private set; }

        public bool IsOpen => Handle != IntPtr.Zero;

        public string Name => !IsOpen ? _closedName ?? "Unknown controller" : _devices.Name(Handle) is { Length: > 0 } name ? name : "Unknown controller";

        // SDL's own reading of what the pad is, from its vendor and product - see EmuSen_Settings_Reference.md §4.52.
        public SDL.GamepadType Type => IsOpen ? _devices.Type(Handle) : _closedType;

        public bool IsRawPressed(SDL.GamepadButton button) => IsOpen && _devices.Button(Handle, button);

        internal short AxisValue(SDL.GamepadAxis axis) => IsOpen ? _devices.Axis(Handle, axis) : (short)0;

        public double RawAxis(SDL.GamepadAxis axis) => Math.Clamp(AxisValue(axis) / (double)short.MaxValue, -1.0, 1.0);

        public string? ButtonLabel(SDL.GamepadButton button)
        {
            if (!IsOpen) return null;
            SDL.GamepadButtonLabel label = _devices.Label(Handle, button);
            return label == SDL.GamepadButtonLabel.Unknown ? null : label.ToString();
        }

        internal void Close()
        {
            if (!IsOpen) return;
            _closedName = Name;
            _closedType = Type;
            _devices.Close(Handle);
            Handle = IntPtr.Zero;
        }
    }

    // A pad plugged in or pulled out, as GamepadManager.Poll found it.
    public readonly record struct PadConnection(ConnectedPad Pad, bool Connected);
}

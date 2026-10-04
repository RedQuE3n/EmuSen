using System;
using SDL3;

namespace EmuSen.Endymion.Input
{
    // What GamepadManager asks of SDL's gamepad API, so a test can stand a set of simulated pads in its place - see EmuSen_Settings_Reference.md §4.61.
    public interface IPadDevices
    {
        bool Init();
        void Quit();
        uint[] Attached();
        IntPtr Open(uint id);
        void Close(IntPtr pad);
        bool IsAttached(IntPtr pad);

        // True once for each batch of SDL's added and removed events, which it consumes.
        bool DevicesChanged();

        void Update();
        bool Button(IntPtr pad, SDL.GamepadButton button);
        short Axis(IntPtr pad, SDL.GamepadAxis axis);
        string? Name(IntPtr pad);
        SDL.GamepadType Type(IntPtr pad);
        SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button);

        // What a pad is and where it is plugged in, by which a pad back after a disconnect finds its player - see EmuSen_Input.md §8.
        string Guid(IntPtr pad);
        string? Path(IntPtr pad);

        // Lights the player number on a pad that has one; most have none, and SDL ignores them.
        void SetPlayerIndex(IntPtr pad, int index);
    }

    // The real devices, through SDL3.
    public sealed class SdlPadDevices : IPadDevices
    {
        // Gamepad subsystem only, and InitSubSystem rather than Init - see EmuSen_Settings_Reference.md §4.10.
        public bool Init() => SDL.InitSubSystem(SDL.InitFlags.Gamepad);

        public void Quit() => SDL.QuitSubSystem(SDL.InitFlags.Gamepad);

        public uint[] Attached() => SDL.GetGamepads(out int count) is { } ids ? ids.AsSpan(0, Math.Min(count, ids.Length)).ToArray() : [];

        public IntPtr Open(uint id) => SDL.OpenGamepad(id);

        public void Close(IntPtr pad) => SDL.CloseGamepad(pad);

        public bool IsAttached(IntPtr pad) => SDL.GamepadConnected(pad);

        public bool DevicesChanged()
        {
            bool changed = SDL.HasEvents((uint)SDL.EventType.GamepadAdded, (uint)SDL.EventType.GamepadRemoved);
            if (changed) SDL.FlushEvents((uint)SDL.EventType.GamepadAdded, (uint)SDL.EventType.GamepadRemoved);
            return changed;
        }

        public void Update() => SDL.UpdateGamepads();

        public bool Button(IntPtr pad, SDL.GamepadButton button) => SDL.GetGamepadButton(pad, button);

        public short Axis(IntPtr pad, SDL.GamepadAxis axis) => SDL.GetGamepadAxis(pad, axis);

        public string? Name(IntPtr pad) => SDL.GetGamepadName(pad);

        public SDL.GamepadType Type(IntPtr pad) => SDL.GetGamepadType(pad);

        public SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button) => SDL.GetGamepadButtonLabel(pad, button);

        // SDL3-CS's GUIDToString faults, so the sixteen bytes are written out here in SDL's own order.
        public string Guid(IntPtr pad) => GuidText(SDL.GetJoystickGUID(SDL.GetGamepadJoystick(pad)));

        public static string GuidText(SDL.GUID guid) =>
            Convert.ToHexString(System.Runtime.InteropServices.MemoryMarshal.AsBytes(System.Runtime.InteropServices.MemoryMarshal.CreateReadOnlySpan(ref guid, 1))).ToLowerInvariant();

        public string? Path(IntPtr pad) => SDL.GetGamepadPath(pad) is { Length: > 0 } path ? path : null;

        public void SetPlayerIndex(IntPtr pad, int index) => SDL.SetGamepadPlayerIndex(pad, index);
    }
}

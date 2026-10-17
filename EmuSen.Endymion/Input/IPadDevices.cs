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

        // True once for each batch of SDL's added and removed events; it consumes every pad event SDL queued - see EmuSen_Input.md §8.12.
        bool DevicesChanged();

        // Brings the pads' state up to date and lets the devices learn of a pad plugged in - see EmuSen_Input.md §8.12.
        void Update();
        bool Button(IntPtr pad, SDL.GamepadButton button);
        short Axis(IntPtr pad, SDL.GamepadAxis axis);
        string? Name(IntPtr pad);
        SDL.GamepadType Type(IntPtr pad);
        SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button);

        // What a pad is and where it is plugged in, by which a pad back after a disconnect finds its player - see EmuSen_Input.md §8.
        string Guid(IntPtr pad);
        string? Path(IntPtr pad);

        // The USB vendor in the high 16 bits and the product in the low; under Steam Input, the controller behind its virtual pad - see EmuSen_Input.md §8.13.
        uint VendorProduct(IntPtr pad);

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

        // SDL's joystick and gamepad events, 0x600 to 0x6FF; nothing else in the process reads them.
        private const uint FirstPadEvent = 0x600, LastPadEvent = 0x6FF;

        public bool DevicesChanged()
        {
            bool changed = SDL.HasEvents((uint)SDL.EventType.GamepadAdded, (uint)SDL.EventType.GamepadRemoved);
            SDL.FlushEvents(FirstPadEvent, LastPadEvent);
            return changed;
        }

        // PumpEvents, not UpdateGamepads: under udev SDL learns of a pad plugged in only when events are pumped - see EmuSen_Input.md §8.12.
        public void Update() => SDL.PumpEvents();

        public bool Button(IntPtr pad, SDL.GamepadButton button) => SDL.GetGamepadButton(pad, button);

        public short Axis(IntPtr pad, SDL.GamepadAxis axis) => SDL.GetGamepadAxis(pad, axis);

        // Steam's virtual pad by the controller Steam's file names behind it, which only the joystick's name carries - see EmuSen_Input.md §8.13.2.
        public string? Name(IntPtr pad)
        {
            IntPtr joystick = SDL.GetGamepadJoystick(pad);
            if (IsSteamVirtual(SDL.GetJoystickGUID(joystick)) && (SDL.GetGamepadVendor(pad), SDL.GetGamepadProduct(pad)) != (SteamVendor, SteamVirtualProduct)) return SDL.GetJoystickName(joystick);
            return SDL.GetGamepadName(pad);
        }

        private const ushort SteamVendor = 0x28DE, SteamVirtualProduct = 0x11FF;

        // Vendor and product little-endian at bytes 4 and 8 of SDL's GUID.
        internal static bool IsSteamVirtual(SDL.GUID guid)
        {
            ReadOnlySpan<byte> data = System.Runtime.InteropServices.MemoryMarshal.AsBytes(System.Runtime.InteropServices.MemoryMarshal.CreateReadOnlySpan(ref guid, 1));
            return System.Buffers.Binary.BinaryPrimitives.ReadUInt16LittleEndian(data[4..]) == SteamVendor && System.Buffers.Binary.BinaryPrimitives.ReadUInt16LittleEndian(data[8..]) == SteamVirtualProduct;
        }

        public SDL.GamepadType Type(IntPtr pad) => SDL.GetGamepadType(pad);

        public SDL.GamepadButtonLabel Label(IntPtr pad, SDL.GamepadButton button) => SDL.GetGamepadButtonLabel(pad, button);

        // SDL3-CS's GUIDToString faults, so the sixteen bytes are written out here in SDL's own order.
        public string Guid(IntPtr pad) => GuidText(SDL.GetJoystickGUID(SDL.GetGamepadJoystick(pad)));

        public static string GuidText(SDL.GUID guid) =>
            Convert.ToHexString(System.Runtime.InteropServices.MemoryMarshal.AsBytes(System.Runtime.InteropServices.MemoryMarshal.CreateReadOnlySpan(ref guid, 1))).ToLowerInvariant();

        public string? Path(IntPtr pad) => SDL.GetGamepadPath(pad) is { Length: > 0 } path ? path : null;

        public uint VendorProduct(IntPtr pad) => (uint)SDL.GetGamepadVendor(pad) << 16 | SDL.GetGamepadProduct(pad);

        public void SetPlayerIndex(IntPtr pad, int index) => SDL.SetGamepadPlayerIndex(pad, index);
    }
}

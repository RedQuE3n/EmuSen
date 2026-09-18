namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // One controller port as the joybus reports it - see Mars_Serial.md §3.1.
    public sealed class Controller
    {
        public bool Present;

        // A, B, Z, Start, then the four directions; L, R and the four C buttons in the byte below - see §3.1.
        public ushort Buttons;

        public sbyte StickX;

        public sbyte StickY;

        // A Controller Pak in the slot, or none - see Mars_Save.md §5.
        public ControllerPak? Pak;
    }
}

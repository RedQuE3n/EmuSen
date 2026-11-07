namespace EmuSen.WiseMan.Bindings.Systems.GameBoy
{
    // What a Game Boy image's header says about its board, every type named, an engine refusing at create what it does not build - see Mercury_Memory.md §2, EmuSen_Backend.md §17.2 L1.
    public readonly record struct GbCartridge(byte Type, string Board, int RomBanks, int RamLength, bool HasBattery, bool HasTimer, bool HasRumble, byte CgbFlag)
    {
        public const int HeaderLength = 0x0150;
        public const int RomBankSize = 0x4000;

        // The header's facts, emusen-systems' gb::cartridge; an image shorter than the header is refused as InvalidDataException.
        public static GbCartridge Read(byte[] image) => SystemCode.GbHeader(image);

        // A colour game by its $0143 flag, $80 or $C0.
        public bool ColourGame => CgbFlag is 0x80 or 0xC0;
    }
}

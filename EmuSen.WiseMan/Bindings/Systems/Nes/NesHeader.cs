namespace EmuSen.WiseMan.Bindings.Systems.Nes
{
    // In the order the state's board fields number them - see Moon_Memory.md §3.
    public enum NesMirroring
    {
        Horizontal,
        Vertical,
        SingleScreenLower,
        SingleScreenUpper,
        FourScreen,
    }

    // How much of the header survived the copiers - see Moon_Memory.md §2.2.
    public enum NesHeaderTrust
    {
        Clean,
        Unverifiable,
        Archaic,
    }

    // An iNES or NES 2.0 header read without building a board; PRG RAM is 8 KB on every board; emusen-systems' nes::header - see Moon_Memory.md §2, EmuSen_Backend.md §13.3.
    public sealed record NesHeader(int Mapper, int PrgBanks, int ChrBanks, bool HasBattery, bool HasTrainer, bool IsNes20, NesHeaderTrust Trust, NesMirroring Mirroring)
    {
        public const int Size = 16;
        public const int PrgBankSize = 0x4000;
        public const int ChrBankSize = 0x2000;
        public const int TrainerSize = 512;
        public const int PrgRamSize = 0x2000;

        public bool ChrIsRam => ChrBanks == 0;

        // An image shorter than its header says, or without the magic, is refused as InvalidDataException.
        public static NesHeader Read(byte[] image) => SystemCode.NesHeader(image);
    }
}

using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.Genesis
{
    // The Sega CD's backup RAM, inside the unit and on its cartridge, one file system, emusen-systems' genesis::bram module - see EmuSen_SaveMedia.md §7.3, EmuSen_Backend.md §13.1.
    public static class SegaCdBramFormat
    {
        public const int BlockLength = 64, MinLength = 0x2000, MaxLength = 0x8_0000;

        public static readonly ISaveMediumFormat BackupRam = new LibraryMediumFormat("mcd.backup"), RamCartridge = new LibraryMediumFormat("mcd.ramcart");
    }
}

using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.Psx
{
    // The memory card's file system as psx-spx's "Memory Card Data Format" lays it out, emusen-systems' psx::card module - see EmuSen_SaveMedia.md §6.2, Diamond_Cards.md, EmuSen_Backend.md §13.1.
    public static class PsxCardFormat
    {
        public const string Id = "psx.card";
        public const int CardLength = 128 * 1024, BlockLength = 8 * 1024, FrameLength = 128, Blocks = 15;

        public static readonly ISaveMediumFormat Instance = new LibraryMediumFormat(Id);
    }
}

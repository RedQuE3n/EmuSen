using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.Ps2
{
    // The PlayStation 2 memory card's file system, emusen-ps2-card through emusen-systems - see EmuSen_SaveMedia.md §7.4, Sapphire_Native.md §30.16.
    public static class Ps2CardFormat
    {
        public const string Id = "ps2.card";
        public const int CardLength = 8_650_752, Kilobytes = 7998;

        public static readonly ISaveMediumFormat Instance = new LibraryMediumFormat(Id);
    }
}

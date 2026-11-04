using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.N64
{
    // The Controller Pak's file system as N64brew's "Controller Pak/Filesystem" lays it out, emusen-systems' n64::pak module - see EmuSen_SaveMedia.md §7.2, EmuSen_Backend.md §13.1.
    public static class N64PakFormat
    {
        public const string Id = "n64.pak";
        public const int PakLength = 32 * 1024, PageLength = 256, Pages = 128, FirstData = 5, Notes = 16, NoteLength = 32;

        public static readonly ISaveMediumFormat Instance = new LibraryMediumFormat(Id);
    }
}

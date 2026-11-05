using System.IO;

namespace EmuSen.Mistress.Library
{
    // Where OpenEmu's game database is kept, fetched on request into home/Library and read by emusen-scrape's failover - see EmuSen_Settings_Reference.md §4.39, EmuSen_Backend.md §40.5.
    public static class OpenVgdb
    {
        public const string FileName = "openvgdb.sqlite";

        public static string DefaultPath => Path.Combine(EmuSen.Galaxia.Library.DataStore.Library, FileName);
    }
}

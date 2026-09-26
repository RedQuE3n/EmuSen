using System.IO;

namespace EmuSen.Galaxia.Library
{
    // The single answer to "where does a saved artifact go" - see EmuSen_Galaxia.md §3.
    public static class DataStore
    {
        public const string HomeDirName = "home";

        // Tests only; separate from ConfigStore's on purpose - see EmuSen_Galaxia.md §3.1.
        public static string? OverrideDirectory { get; set; }

        // One user home, reachable as /home - see `man hier` and EmuSen_Galaxia.md §3.
        public static string UsrHome => OverrideDirectory ?? Path.Combine(ConfigRoot.Directory, HomeDirName);

        public static string Logs => Path.Combine(UsrHome, "Logs");

        public static string Saves => Path.Combine(UsrHome, "Saves");

        public static string SaveStates => Path.Combine(Saves, "Save States");

        // The player's own library database, which is kept and migrated - see EmuSen_Settings_Reference.md §4.32.
        public static string Library => Path.Combine(UsrHome, "Library");

        // Pictures taken from the running game - see EmuSen_Galaxia.md §3.
        public static string Screenshots => Path.Combine(UsrHome, "Screenshots");

        // Box art the user supplies, read and never fetched - see EmuSen_Settings_Reference.md §4.33.
        public static string Artwork => Path.Combine(UsrHome, "Artwork");

        // Game media and metadata fetched from ScreenScraper, laid out as ES-DE's downloaded_media, with media.db - see EmuSen_BigPicture.md §5.6.
        public static string Media => Path.Combine(UsrHome, "Media");

        // Coprocessor firmware dumps the user supplies - see EmuSen_Firmware.md §2.
        public static string Firmware => Path.Combine(UsrHome, "Firmware");

        // Shader packs downloaded on the player's request, never shipped with EmuSen - see EmuSen_Settings_Reference.md §4.41.
        public static string Shaders => Path.Combine(UsrHome, "Shaders");

        // The user's own .cht tree, never shipped with EmuSen - see `man cheat`.
        public static string Cheats => Path.Combine(UsrHome, "Cheats");

        // A skeleton directory of the shell's tree, not the ROM library - see EmuSen_Galaxia.md §3.3.
        public static string Games => Path.Combine(UsrHome, "Games");
    }
}

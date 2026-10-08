using System.IO;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Library
{
    // The single answer to "where does a saved artifact go" - see EmuSen_Galaxia.md §3.
    public static class DataStore
    {
        public const string HomeDirName = "home";

        private static string? _overrideDirectory;

        // Tests only; separate from ConfigStore's on purpose - see EmuSen_Galaxia.md §3.1.
        public static string? OverrideDirectory
        {
            get => _overrideDirectory;
            set
            {
                lock (GalaxiaNative.Gate)
                {
                    _overrideDirectory = value;
                    GalaxiaNative.OverrideChanged(GalaxiaOverride.Data, value);
                }
            }
        }

        private static string At(GalaxiaDirectory which, string name) =>
            GalaxiaNative.Active ? GalaxiaNative.Directory(which)! : Path.Combine(Managed.UsrHome, name);

        // One user home, reachable as /home - see `man hier` and EmuSen_Galaxia.md §3.
        public static string UsrHome => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.Home)! : Managed.UsrHome;

        public static string Logs => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.Logs)! : Managed.Logs;

        public static string Saves => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.Saves)! : Managed.Saves;

        public static string SaveStates => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.SaveStates)! : Managed.SaveStates;

        // The player's own library database, which is kept and migrated - see EmuSen_Settings_Reference.md §4.32.
        public static string Library => At(GalaxiaDirectory.Library, "Library");

        // Pictures taken from the running game - see EmuSen_Galaxia.md §3.
        public static string Screenshots => At(GalaxiaDirectory.Screenshots, "Screenshots");

        // Box art the player supplies, read and never fetched - see EmuSen_Settings_Reference.md §4.33.
        public static string Artwork => At(GalaxiaDirectory.Artwork, "Artwork");

        // Game media and metadata fetched from ScreenScraper, laid out as ES-DE's downloaded_media, with media.db - see EmuSen_BigPicture.md §5.6.
        public static string Media => At(GalaxiaDirectory.Media, "Media");

        // Coprocessor firmware dumps the player supplies - see EmuSen_Firmware.md §2.
        public static string Firmware => At(GalaxiaDirectory.Firmware, "Firmware");

        // Shader packs downloaded on the player's request, never shipped with EmuSen - see EmuSen_Settings_Reference.md §4.41.
        public static string Shaders => At(GalaxiaDirectory.Shaders, "Shaders");

        // ES-DE themes downloaded on the player's request, never shipped with EmuSen - see EmuSen_Settings_Reference.md §4.53.
        public static string Themes => At(GalaxiaDirectory.Themes, "Themes");

        // The player's own .cht tree, never shipped with EmuSen - see `man cheat`.
        public static string Cheats => At(GalaxiaDirectory.Cheats, "Cheats");

        // A skeleton directory of the shell's tree, not the ROM library - see EmuSen_Galaxia.md §3.3.
        public static string Games => At(GalaxiaDirectory.Games, "Games");

        // The C# rules: the default, and what the library's are held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static string UsrHome => _overrideDirectory ?? Path.Combine(ConfigRoot.Managed.Directory, HomeDirName);

            public static string Logs => Path.Combine(UsrHome, "Logs");

            public static string Saves => Path.Combine(UsrHome, "Saves");

            public static string SaveStates => Path.Combine(Saves, "Save States");

        }
    }
}

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

        private static string At(GalaxiaDirectory which) =>
            GalaxiaNative.Directory(which)!;

        // One user home, reachable as /home - see `man hier` and EmuSen_Galaxia.md §3.
        public static string UsrHome => GalaxiaNative.Directory(GalaxiaDirectory.Home)!;

        public static string Logs => GalaxiaNative.Directory(GalaxiaDirectory.Logs)!;

        public static string Saves => GalaxiaNative.Directory(GalaxiaDirectory.Saves)!;

        public static string SaveStates => GalaxiaNative.Directory(GalaxiaDirectory.SaveStates)!;

        // The player's own library database, which is kept and migrated - see EmuSen_Settings_Reference.md §4.32.
        public static string Library => At(GalaxiaDirectory.Library);

        // Pictures taken from the running game - see EmuSen_Galaxia.md §3.
        public static string Screenshots => At(GalaxiaDirectory.Screenshots);

        // Box art the player supplies, read and never fetched - see EmuSen_Settings_Reference.md §4.33.
        public static string Artwork => At(GalaxiaDirectory.Artwork);

        // Game media and metadata fetched from ScreenScraper, laid out as ES-DE's downloaded_media, with media.db - see EmuSen_BigPicture.md §5.6.
        public static string Media => At(GalaxiaDirectory.Media);

        // Coprocessor firmware dumps the player supplies - see EmuSen_Firmware.md §2.
        public static string Firmware => At(GalaxiaDirectory.Firmware);

        // Shader packs downloaded on the player's request, never shipped with EmuSen - see EmuSen_Settings_Reference.md §4.41.
        public static string Shaders => At(GalaxiaDirectory.Shaders);

        // ES-DE themes downloaded on the player's request, never shipped with EmuSen - see EmuSen_Settings_Reference.md §4.53.
        public static string Themes => At(GalaxiaDirectory.Themes);

        // The player's own .cht tree, never shipped with EmuSen - see `man cheat`.
        public static string Cheats => At(GalaxiaDirectory.Cheats);

        // A skeleton directory of the shell's tree, not the ROM library - see EmuSen_Galaxia.md §3.3.
        public static string Games => At(GalaxiaDirectory.Games);

        // Archives opened for play, unpacked here and never beside the library's file - see EmuSen_Galaxia.md §3.7.
        public static string Unpacked => At(GalaxiaDirectory.Unpacked);

        // The player's own core libraries, approved before they load - see EmuSen_Backend.md §29.40.4.
        public static string Cores => At(GalaxiaDirectory.Cores);

        // Game packages, a folder per game with its manifest, never inside the ROM library - see EmuSen_GamePackages.md §3.1.
        public static string Packages => At(GalaxiaDirectory.Packages);

        // Disc images converted to CHD from the library, by its relative paths, never inside it - see EmuSen_GamePackages.md §4.2.
        public static string Compressed => At(GalaxiaDirectory.Compressed);
    }
}

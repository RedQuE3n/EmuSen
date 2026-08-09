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

        // Coprocessor firmware dumps the user supplies - see EmuSen_Firmware.md §2.
        public static string Firmware => Path.Combine(UsrHome, "Firmware");

        // The user's own .cht tree, never shipped with EmuSen - see `man cheat`.
        public static string Cheats => Path.Combine(UsrHome, "Cheats");

        // A skeleton directory of the shell's tree, not the ROM library - see EmuSen_Galaxia.md §3.3.
        public static string Games => Path.Combine(UsrHome, "Games");
    }
}

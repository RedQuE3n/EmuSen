namespace EmuSen.Galaxia.Models
{
    // General app preferences (log/ROM directories, selected core) - see
    // EmuSen_Config_Reference.md §3.1.
    //
    // SelectedCore is the console context the library and both cheat windows share - see EmuSen_Multicore.md §10.
    public class AppSettings
    {
        public string? LogDirectory { get; set; }
        public string? RomDirectory { get; set; }
        public string? StateDirectory { get; set; }

        // The user's own .cht tree - point this at an existing RetroArch
        // cheats folder to use it as-is. See `man cheat`.
        public string? CheatDatabaseDirectory { get; set; }
        // The no-filter choice; CoreCatalog re-exports this so a UI has one spelling.
        public const string AllConsoles = "All consoles";

        // What the value was defaulted to while nothing read it - see Upgraded() below.
        public const string LegacySelectedCoreDefault = "SNES (Venus)";

        public string SelectedCore { get; set; } = AllConsoles;

        // Off by default - forcing this on unconditionally would break any
        // real two-controller game by feeding Controller 2 the same input
        // as Controller 1 even when a genuine second pad is plugged in.
        // Exists for games that read Controller 2 instead of Controller 1
        // for classic-game-in-a-compilation reasons (Super Mario All-Stars'
        // SMB1/2/3 being the known example - see EmuSen_Games_Tested.md),
        // toggled from InputSettingsWindow - the same workaround real
        // hardware players and other emulators use, not an input redesign.
        public bool MirrorPlayer1ToPlayer2 { get; set; } = false;

        // Stick-as-d-pad and its deadzone - see EmuSen_Settings_Reference.md §4.4.
        public bool AnalogStickAsDpad { get; set; } = true;
        public double StickDeadzone { get; set; } = 0.5;

        private static readonly ConfigFile<AppSettings> File = new("appsettings.json");

        public void Save() => File.Save(this);

        public static AppSettings Load() => Upgraded(File.Load(() => new AppSettings()));

        // A stored legacy default is not a choice - see EmuSen_Multicore.md §10.3.
        private static AppSettings Upgraded(AppSettings settings)
        {
            if (settings.SelectedCore == LegacySelectedCoreDefault) settings.SelectedCore = AllConsoles;
            return settings;
        }
    }
}

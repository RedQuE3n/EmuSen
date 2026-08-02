namespace EmuSen.Galaxia.Models
{
    // General app preferences (log/ROM directories, selected core) - see
    // EmuSen_Config_Reference.md §3.1.
    //
    // SelectedCore is pure scaffolding for now - there is only one core
    // (VenusCore/SNES) and nothing reads this value to actually switch
    // cores yet. It exists so the Preferences UI has a real place to
    // persist the choice once a second core exists, rather than needing a
    // settings-file migration at that point.
    public class AppSettings
    {
        public string? LogDirectory { get; set; }
        public string? RomDirectory { get; set; }
        public string? StateDirectory { get; set; }

        // The user's own .cht tree - point this at an existing RetroArch
        // cheats folder to use it as-is. See `man cheat`.
        public string? CheatDatabaseDirectory { get; set; }
        public string SelectedCore { get; set; } = "SNES (Venus)";

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

        public static AppSettings Load() => File.Load(() => new AppSettings());
    }
}

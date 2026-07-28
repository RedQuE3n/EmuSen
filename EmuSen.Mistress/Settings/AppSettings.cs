using System;
using System.IO;
using System.Text.Json;

namespace EmuSen.Mistress.Settings
{
    // General app preferences (log/ROM directories, selected core) - same
    // shape and persistence approach as ControllerKeyMap/GamepadBindingMap
    // (Input/ControllerKeyMap.cs, GamepadBindingMap.cs): a plain
    // System.Text.Json-serialized file under
    // %AppData%/EmuSen/, best-effort load/save so a corrupt or missing
    // file falls back to defaults rather than crashing.
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
        public string SelectedCore { get; set; } = "SNES (Venus)";

        // Off by default - forcing this on unconditionally would break any
        // real two-controller game by feeding Controller 2 the same input
        // as Controller 1 even when a genuine second pad is plugged in.
        // Exists for games that read Controller 2 instead of Controller 1
        // for classic-game-in-a-compilation reasons (Super Mario All-Stars'
        // SMB1/2/3 being the known example - see
        // EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/EmuSen_Games_Tested.md), toggled from
        // InputSettingsWindow - the same workaround real hardware players
        // and other emulators use, not a general input redesign.
        public bool MirrorPlayer1ToPlayer2 { get; set; } = false;

        private static string ConfigPath => Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "EmuSen", "appsettings.json");

        public void Save()
        {
            try
            {
                string dir = Path.GetDirectoryName(ConfigPath)!;
                Directory.CreateDirectory(dir);
                string json = JsonSerializer.Serialize(this, new JsonSerializerOptions { WriteIndented = true });
                File.WriteAllText(ConfigPath, json);
            }
            catch
            {
                // Best-effort - a failed save shouldn't crash the Preferences window.
            }
        }

        public static AppSettings Load()
        {
            try
            {
                if (File.Exists(ConfigPath))
                {
                    string json = File.ReadAllText(ConfigPath);
                    var loaded = JsonSerializer.Deserialize<AppSettings>(json);
                    if (loaded is not null) return loaded;
                }
            }
            catch
            {
                // Corrupt/unreadable config - fall back to defaults rather than crash.
            }
            return new AppSettings();
        }
    }
}

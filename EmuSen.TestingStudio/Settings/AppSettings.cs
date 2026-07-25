using System;
using System.IO;
using System.Text.Json;

namespace EmuSen.Frontend.Settings
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
        public string SelectedCore { get; set; } = "SNES (Venus)";

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

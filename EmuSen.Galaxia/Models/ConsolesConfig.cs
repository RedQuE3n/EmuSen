using System.Collections.Generic;

namespace EmuSen.Galaxia.Models
{
    // /etc/EmuSen/consoles.json: each console's core settings by key, the runtime's own file, copied once from graphics.json's Consoles and never over a file that exists - see EmuSen_Backend.md §14.12 and SR12.
    public class ConsolesConfig
    {
        public Dictionary<string, Dictionary<string, string>> Consoles { get; set; } = new();

        public string? Value(string console, string key) =>
            Consoles.TryGetValue(console, out var settings) && settings.TryGetValue(key, out string? value) ? value : null;

        public void SetValue(string console, string key, string value)
        {
            if (!Consoles.TryGetValue(console, out var settings)) Consoles[console] = settings = new Dictionary<string, string>();
            settings[key] = value;
        }

        // An empty console is removed with its last value, so a reset leaves nothing behind in the file.
        public void Forget(string console, string key)
        {
            if (!Consoles.TryGetValue(console, out var settings)) return;
            settings.Remove(key);
            if (settings.Count == 0) Consoles.Remove(console);
        }

        public const string FileName = "consoles.json";

        private static readonly ConfigFile<ConsolesConfig> File = new(FileName);

        public bool Save() => File.Save(this);

        public static bool Exists => File.Exists;

        // The file, or with none yet graphics.json's map copied into a new one: the whole map, since which keys are a core's depends on the engines installed then; with nothing to copy, nothing is written.
        public static ConsolesConfig Load()
        {
            if (File.Load() is { } stored) return stored;
            var made = new ConsolesConfig();
            if (File.Exists || !GraphicsConfig.Exists) return made;
            foreach (var (console, settings) in GraphicsConfig.Load().Consoles) made.Consoles[console] = new Dictionary<string, string>(settings);
            if (made.Consoles.Count > 0) made.Save();
            return made;
        }
    }
}

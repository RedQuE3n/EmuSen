using System.IO;
using System.Linq;
using System.Text.Json;

namespace EmuSen.Galaxia.Models
{
    // The cheat list a frontend keeps for one game, in its console's folder, with the flat file of earlier builds still read - see EmuSen_Config_Reference.md §3.4b.
    public sealed class GameCheats
    {
        private readonly string? _console;

        private GameCheats(string? console, string name)
        {
            _console = console;
            Name = name;
        }

        // The ROM's file name without its extension, as the list is shown to the player.
        public string Name { get; }

        // Where Save writes: cheats/<console>/<name>.json, or the flat cheats/<name>.json for a console the catalogue does not know.
        public ConfigFile<CheatFile> File => _console is null ? Legacy : new(Path.Combine(CheatFile.CategoryDirName, _console), Name + ".json");

        private ConfigFile<CheatFile> Legacy => CheatFile.For(Name);

        public bool Exists => File.Exists || Legacy.Exists;

        // Null when the path gives no usable name; derived names are sanitized, not refused, because nobody typed them.
        public static GameCheats? For(string? console, string? romPath)
        {
            if (string.IsNullOrEmpty(romPath)) return null;
            string name = Sanitized(Path.GetFileNameWithoutExtension(romPath));
            if (!CheatFile.IsValidName(name)) return null;
            string? folder = string.IsNullOrWhiteSpace(console) ? null : Sanitized(console);
            return new GameCheats(folder is not null && CheatFile.IsValidName(folder) ? folder : null, name);
        }

        private static string Sanitized(string text) =>
            new(text.Select(c => Path.GetInvalidFileNameChars().Contains(c) ? '_' : c).ToArray());

        // The console's file when there is one, else the flat file an earlier build saved; null for neither, or unreadable.
        public CheatFile? Load() => File.Exists ? File.Load() : Legacy.Load();

        public bool Save(CheatFile list) => File.Save(list);

        // Null when nothing needed writing, else whether the write succeeded; an empty list over no saved one is nothing - see EmuSen_Config_Reference.md §3.4b.
        public bool? SaveIfChanged(CheatFile list)
        {
            CheatFile? saved = Load();
            if (saved is null && list.Cheats.Count == 0) return null;
            if (saved is not null && Same(saved, list)) return null;
            return Save(list);
        }

        private static bool Same(CheatFile a, CheatFile b) =>
            JsonSerializer.Serialize(a, ConfigJson.Options) == JsonSerializer.Serialize(b, ConfigJson.Options);
    }
}

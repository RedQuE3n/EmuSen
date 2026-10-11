using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Models
{
    // The cheat list a frontend keeps for one game, in its console's folder, with the flat file of earlier builds still read - see EmuSen_Config_Reference.md §3.4b.
    public sealed class GameCheats
    {
        private GameCheats(string category, string name)
        {
            Category = category;
            Name = name;
        }

        // The ROM's file name without its extension, as the list is shown to the player.
        public string Name { get; }

        // cheats/<console>, or the flat cheats for a console the catalogue does not know.
        internal string Category { get; }

        // Where Save writes: cheats/<console>/<name>.json, or the flat cheats/<name>.json.
        public ConfigFile<CheatFile> File => new(Category, Name + ".json");

        private ConfigFile<CheatFile> Legacy => CheatFile.For(Name);

        public bool Exists => File.Exists || Legacy.Exists;

        // Null when the path gives no usable name; derived names are sanitized, not refused, because nobody typed them.
        public static GameCheats? For(string? console, string? romPath) =>
            GalaxiaNative.Active && GalaxiaNative.Crosses(console) && GalaxiaNative.Crosses(romPath) ? ForNative(console, romPath) : Managed.For(console, romPath);

        internal static GameCheats? ForNative(string? console, string? romPath) =>
            GalaxiaNative.CheatGameList(console, romPath) is { } list ? new GameCheats(list.Category, list.Name) : null;

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

        // The C# rule: what the library's is held to while Galaxia's C# stands - see EmuSen_RustPlatform.md §3.9 and §20.
        internal static class Managed
        {
            public static GameCheats? For(string? console, string? romPath)
            {
                if (string.IsNullOrEmpty(romPath)) return null;
                string name = Sanitized(Path.GetFileNameWithoutExtension(romPath));
                if (!CheatFile.Managed.IsValidName(name)) return null;
                string? folder = string.IsNullOrWhiteSpace(console) ? null : Sanitized(console);
                bool flat = folder is null || !CheatFile.Managed.IsValidName(folder);
                return new GameCheats(flat ? CheatFile.CategoryDirName : Path.Combine(CheatFile.CategoryDirName, folder!), name);
            }

            private static string Sanitized(string text) =>
                new(text.Select(c => Path.GetInvalidFileNameChars().Contains(c) ? '_' : c).ToArray());
        }
    }
}

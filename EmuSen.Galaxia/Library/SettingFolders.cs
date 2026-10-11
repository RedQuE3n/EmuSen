using EmuSen.Galaxia.Models;

namespace EmuSen.Galaxia.Library
{
    // The folder each path setting names, or its default when blank; the one owner of that rule - see EmuSen_Config_Reference.md §7.
    public static class SettingFolders
    {
        public static string? Named(string? value) => string.IsNullOrWhiteSpace(value) ? null : value;

        public static string States(AppSettings settings) => Named(settings.StateDirectory) ?? DataStore.SaveStates;

        public static string Cheats(AppSettings settings) => Named(settings.CheatDatabaseDirectory) ?? DataStore.Cheats;

        public static string Artwork(AppSettings settings) => Named(settings.ArtworkDirectory) ?? DataStore.Artwork;

        public static string? Roms(AppSettings settings) => Named(settings.RomDirectory);
    }
}

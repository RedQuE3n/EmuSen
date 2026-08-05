using System.IO;

namespace EmuSen.Galaxia.Library
{
    // What a given ROM's save files are called, and where - see EmuSen_Galaxia.md §5.
    public static class SaveLibrary
    {
        public const string SramExtension = ".srm";
        public const string StateExtension = ".state";

        // The slot that keeps the plain <rom>.state name - see EmuSen_Galaxia.md §5.1.
        public const int DefaultStateSlot = 1;

        public static string SramPathFor(string romPath) =>
            Path.Combine(DataStore.Saves, Path.GetFileNameWithoutExtension(romPath) + SramExtension);

        // directoryOverride is a parameter, not an AppSettings read - see EmuSen_Galaxia.md §5.1.
        public static string StatePathFor(string romPath, int slot = DefaultStateSlot, string? directoryOverride = null)
        {
            string directory = string.IsNullOrWhiteSpace(directoryOverride) ? DataStore.SaveStates : directoryOverride;
            string stem = Path.GetFileNameWithoutExtension(romPath);
            string suffix = slot == DefaultStateSlot ? "" : $".slot{slot}";
            return Path.Combine(directory, stem + suffix + StateExtension);
        }
    }
}

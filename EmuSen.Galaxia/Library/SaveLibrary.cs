using System.IO;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Library
{
    // What a given ROM's save files are called, and where - see EmuSen_Galaxia.md §5.
    public static class SaveLibrary
    {
        public const string SramExtension = ".srm";
        public const string StateExtension = ".state";

        // The slot that keeps the plain <rom>.state name - see EmuSen_Galaxia.md §5.1.
        public const int DefaultStateSlot = 1;

        // One folder per console, named as the ROM library names them (NES, SNES, N64, GB, GBC) - see EmuSen_Settings_Reference.md §4.85.11.
        public static string SramPathFor(string romPath, string console) =>
            GalaxiaNative.Active ? GalaxiaNative.SavePath(GalaxiaSave.Sram, romPath, console) : Managed.SramPathFor(romPath, console);

        // Where SNES and N64 saves went before 2026-09-28, read once to copy them into their console's folder, never written.
        public static string FlatSramPathFor(string romPath) =>
            GalaxiaNative.Active ? GalaxiaNative.SavePath(GalaxiaSave.FlatSram, romPath) : Managed.FlatSramPathFor(romPath);

        // directoryOverride is a parameter, not an AppSettings read - see EmuSen_Galaxia.md §5.1.
        public static string StatePathFor(string romPath, int slot = DefaultStateSlot, string? directoryOverride = null) =>
            GalaxiaNative.Active ? GalaxiaNative.SavePath(GalaxiaSave.State, romPath, directoryOverride, slot) : Managed.StatePathFor(romPath, slot, directoryOverride);

        // Written when a game is closed and offered at its next start - see EmuSen_Galaxia.md §5.2.
        public static string ResumeStatePathFor(string romPath, string? directoryOverride = null) =>
            GalaxiaNative.Active ? GalaxiaNative.SavePath(GalaxiaSave.ResumeState, romPath, directoryOverride) : Managed.ResumeStatePathFor(romPath, directoryOverride);

        // The picture on screen when a state was written, beside it - see EmuSen_Galaxia.md §5.2.
        public static string PicturePathFor(string statePath) =>
            GalaxiaNative.Active ? GalaxiaNative.SavePath(GalaxiaSave.Picture, statePath) : Managed.PicturePathFor(statePath);

        // The C# rules: the default, and what the library's are held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static string SramPathFor(string romPath, string console) =>
                Path.Combine(DataStore.Managed.Saves, console, Path.GetFileNameWithoutExtension(romPath) + SramExtension);

            public static string FlatSramPathFor(string romPath) =>
                Path.Combine(DataStore.Managed.Saves, Path.GetFileNameWithoutExtension(romPath) + SramExtension);

            public static string StatePathFor(string romPath, int slot, string? directoryOverride)
            {
                string directory = string.IsNullOrWhiteSpace(directoryOverride) ? DataStore.Managed.SaveStates : directoryOverride;
                string stem = Path.GetFileNameWithoutExtension(romPath);
                string suffix = slot == DefaultStateSlot ? "" : $".slot{slot}";
                return Path.Combine(directory, stem + suffix + StateExtension);
            }

            public static string ResumeStatePathFor(string romPath, string? directoryOverride)
            {
                string directory = string.IsNullOrWhiteSpace(directoryOverride) ? DataStore.Managed.SaveStates : directoryOverride;
                return Path.Combine(directory, Path.GetFileNameWithoutExtension(romPath) + ".resume" + StateExtension);
            }

            public static string PicturePathFor(string statePath) => Path.ChangeExtension(statePath, ".png");
        }
    }
}

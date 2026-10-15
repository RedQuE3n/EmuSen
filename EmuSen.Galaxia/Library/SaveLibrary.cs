using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
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
            GalaxiaNative.Takes(romPath, console) ? GalaxiaNative.SavePath(GalaxiaSave.Sram, romPath, console) : Managed.SramPathFor(romPath, console);

        // The folder of a console's cards a player carries between games, beside its per-game saves - see Diamond_Cards.md §9.2.
        public const string CardsFolder = "Cards";

        // A shared or named card: <Saves>/<console>/Cards/<name><extension>, whatever game is running - see Diamond_Cards.md §9.2.
        public static string CardPathFor(string console, string name, string extension) =>
            Path.Combine(DataStore.Saves, console, CardsFolder, name + extension);

        // The names of the cards a console's folder holds with this extension, in order; none when there is no folder.
        public static IReadOnlyList<string> CardNames(string console, string extension)
        {
            string folder = Path.Combine(DataStore.Saves, console, CardsFolder);
            if (!Directory.Exists(folder)) return Array.Empty<string>();
            return Directory.EnumerateFiles(folder, "*" + extension).Select(Path.GetFileNameWithoutExtension).OfType<string>().Order(StringComparer.OrdinalIgnoreCase).ToArray();
        }

        // Where SNES and N64 saves went before 2026-09-28, read once to copy them into their console's folder, never written.
        public static string FlatSramPathFor(string romPath) =>
            GalaxiaNative.Takes(romPath) ? GalaxiaNative.SavePath(GalaxiaSave.FlatSram, romPath) : Managed.FlatSramPathFor(romPath);

        // directoryOverride is a parameter, not an AppSettings read - see EmuSen_Galaxia.md §5.1.
        public static string StatePathFor(string romPath, int slot = DefaultStateSlot, string? directoryOverride = null) =>
            GalaxiaNative.Takes(romPath, directoryOverride) ? GalaxiaNative.SavePath(GalaxiaSave.State, romPath, directoryOverride, slot) : Managed.StatePathFor(romPath, slot, directoryOverride);

        // Written when a game is closed and offered at its next start - see EmuSen_Galaxia.md §5.2.
        public static string ResumeStatePathFor(string romPath, string? directoryOverride = null) =>
            GalaxiaNative.Takes(romPath, directoryOverride) ? GalaxiaNative.SavePath(GalaxiaSave.ResumeState, romPath, directoryOverride) : Managed.ResumeStatePathFor(romPath, directoryOverride);

        // The picture on screen when a state was written, beside it - see EmuSen_Galaxia.md §5.2.
        public static string PicturePathFor(string statePath) =>
            GalaxiaNative.Takes(statePath) ? GalaxiaNative.SavePath(GalaxiaSave.Picture, statePath) : Managed.PicturePathFor(statePath);

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

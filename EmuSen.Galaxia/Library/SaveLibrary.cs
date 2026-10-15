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

        // The folders of a console's shared and named save media, beside its per-game saves: removable ones, and the console's own - see EmuSen_SaveMedia.md §3.3.
        public const string CardsFolder = "Cards", ConsoleFolder = "Console";

        public static string MediumFolder(string console, bool consoleKind) => Path.Combine(DataStore.Saves, console, consoleKind ? ConsoleFolder : CardsFolder);

        // A shared or named medium's file: <Saves>/<console>/<Cards or Console>/<name><extension>, whatever game is running.
        public static string MediumPathFor(string console, bool consoleKind, string name, string extension) => Path.Combine(MediumFolder(console, consoleKind), name + extension);

        // A shared or named card, the removable kind's file - see Diamond_Cards.md §9.2.
        public static string CardPathFor(string console, string name, string extension) => MediumPathFor(console, false, name, extension);

        // The names of a kind's files with this extension in a console's folder, in order; none when there is no folder.
        public static IReadOnlyList<string> MediumNames(string console, bool consoleKind, string extension)
        {
            string folder = MediumFolder(console, consoleKind);
            if (!Directory.Exists(folder)) return Array.Empty<string>();
            return Directory.EnumerateFiles(folder, "*" + extension).Where(f => string.Equals(Path.GetExtension(f), extension, StringComparison.OrdinalIgnoreCase))
                .Select(Path.GetFileNameWithoutExtension).OfType<string>().Order(StringComparer.OrdinalIgnoreCase).ToArray();
        }

        public static IReadOnlyList<string> CardNames(string console, string extension) => MediumNames(console, false, extension);

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

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
        // A console started with nothing in it has no game, so its states, locks and words go by this name - see Diamond_Shell.md §1.5.
        public static string ConsoleSessionName(string console) => $"{console} (no disc)";

        public const string SramExtension = ".srm";
        public const string StateExtension = ".state";

        // The slot that keeps the plain <rom>.state name - see EmuSen_Galaxia.md §5.1.
        public const int DefaultStateSlot = 1;

        // One folder per console, named as the ROM library names them (NES, SNES, N64, GB, GBC); a packaged game's in its package, by the package's name and a patched one's list key - see EmuSen_GamePackages.md §3.5, §5.5.
        public static string SramPathFor(string romPath, string console, string? packageRoot = null, string? saveKey = null)
        {
            if (string.IsNullOrWhiteSpace(packageRoot)) return GalaxiaNative.SavePath(GalaxiaSave.Sram, romPath, console);
            return GalaxiaNative.SavePath(GalaxiaSave.PackageSram, packageRoot, saveKey);
        }

        // The folders of a console's shared and named save media, beside its per-game saves: removable ones, and the console's own - see EmuSen_SaveMedia.md §3.3.
        public const string CardsFolder = "Cards", ConsoleFolder = "Console";

        public static string MediumFolder(string console, bool consoleKind) => Path.Combine(DataStore.Saves, console, consoleKind ? ConsoleFolder : CardsFolder);

        // A shared or named medium's file: <Saves>/<console>/<Cards or Console>/<name><extension>, whatever game is running.
        public static string MediumPathFor(string console, bool consoleKind, string name, string extension) => Path.Combine(MediumFolder(console, consoleKind), name + extension);

        // A hosted core's named battery files, by the core's id beside its console's media, and those it removed kept in <folder>.removed - see EmuSen_LibretroHost.md §4.5.
        public static string NamedFilesFolder(string console, bool consoleKind, string core) => Path.Combine(MediumFolder(console, consoleKind), core);

        // A game's own named battery files, beside its battery save.
        public static string GameFilesFolder(string romPath, string console) => Path.ChangeExtension(SramPathFor(romPath, console), ".files");

        public const string RemovedSuffix = ".removed";

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
            GalaxiaNative.SavePath(GalaxiaSave.FlatSram, romPath);

        // directoryOverride is a parameter, not an AppSettings read, and a package root is one too, which wins - see EmuSen_Galaxia.md §5.1.
        public static string StatePathFor(string romPath, int slot = DefaultStateSlot, string? directoryOverride = null, string? packageRoot = null, string? saveKey = null)
        {
            if (string.IsNullOrWhiteSpace(packageRoot)) return GalaxiaNative.SavePath(GalaxiaSave.State, romPath, directoryOverride, slot);
            return GalaxiaNative.SavePath(GalaxiaSave.PackageState, packageRoot, saveKey, slot);
        }

        // Written when a game is closed and offered at its next start - see EmuSen_Galaxia.md §5.2.
        public static string ResumeStatePathFor(string romPath, string? directoryOverride = null, string? packageRoot = null, string? saveKey = null)
        {
            if (string.IsNullOrWhiteSpace(packageRoot)) return GalaxiaNative.SavePath(GalaxiaSave.ResumeState, romPath, directoryOverride);
            return GalaxiaNative.SavePath(GalaxiaSave.PackageResumeState, packageRoot, saveKey);
        }

        // The picture on screen when a state was written, beside it - see EmuSen_Galaxia.md §5.2.
        public static string PicturePathFor(string statePath) =>
            GalaxiaNative.SavePath(GalaxiaSave.Picture, statePath);

        // A packaged state's record, beside it with .json, or null for a state outside a package's states folder, the twin of saves.rs' package_record_path - see EmuSen_GamePackages.md §18.1.
        public static string? PackageRecordPathFor(string statePath)
        {
            string full = Path.GetFullPath(statePath);
            if (Path.GetDirectoryName(full) is not { } folder || Path.GetFileName(folder) != Media.GamePackage.StatesFolder) return null;
            return Path.GetDirectoryName(folder) is { } root && File.Exists(Path.Combine(root, Media.GamePackage.FileName)) ? Path.ChangeExtension(full, ".json") : null;
        }
    }
}

using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Library
{
    // Copies emulator-written data out of the old Usr/Home tree - see EmuSen_Galaxia.md §3.2.
    public static class DataMigration
    {
        // Only what the emulator wrote; the ROM library is excluded - see EmuSen_Galaxia.md §3.3.
        private static readonly string[] MigratedDirectories = { "Saves", "Logs", "Firmware", "Cheats" };

        public static readonly string[] LibraryDirectories = { "Games", "Roms" };

        // Where the tree lived before it collapsed to <root>/home.
        public static string LegacyRootFor(string configRoot) =>
            GalaxiaNative.LegacyRootFor(configRoot);

        public static string LegacyRoot => GalaxiaNative.Directory(GalaxiaDirectory.LegacyRoot)!;

        // Runs once at startup; returns how many files it copied.
        public static int Run() => GalaxiaNative.Migrate(GalaxiaMigrate.Own);

        // etc/EmuSen moved under home when the shell was rooted there - see EmuSen_Galaxia.md §3.2.
        public static int RunConfig() => GalaxiaNative.Migrate(GalaxiaMigrate.OwnConfig);

        // A Mac app bundle's read-only skeleton, copied into home on every start and never over the player's files - see EmuSen_Galaxia.md §3.4.
        public static int SeedFromBundle() => GalaxiaNative.Migrate(GalaxiaMigrate.OwnSeed);

        public static int SeedFromBundle(string? seedDirectory, string home) =>
            GalaxiaNative.Migrate(GalaxiaMigrate.Seed, seedDirectory, home);

        public static int Run(string legacyRoot, string destinationRoot) =>
            GalaxiaNative.Migrate(GalaxiaMigrate.Between, legacyRoot, destinationRoot);

        // Copy, never move, and never overwrite - the ConfigFile<T>.MigrateFromLegacy rule.
        internal static int CopyTree(string source, string destination) =>
            GalaxiaNative.Migrate(GalaxiaMigrate.Tree, source, destination);

        // Old ROM folders that still hold files, so a caller can say where they are.
        public static IReadOnlyList<string> RemainingLibraryDirectories() =>
            GalaxiaNative.RemainingLibrary(null);

        public static IReadOnlyList<string> RemainingLibraryDirectories(string legacyRoot) =>
            GalaxiaNative.RemainingLibrary(legacyRoot);
    }
}

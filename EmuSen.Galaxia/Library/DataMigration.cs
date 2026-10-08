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
            GalaxiaNative.Active ? GalaxiaNative.LegacyRootFor(configRoot) : Managed.LegacyRootFor(configRoot);

        public static string LegacyRoot => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.LegacyRoot)! : Managed.LegacyRoot;

        // Runs once at startup; returns how many files it copied.
        public static int Run() => GalaxiaNative.Active ? GalaxiaNative.Migrate(GalaxiaMigrate.Own) : Managed.Run();

        // etc/EmuSen moved under home when the shell was rooted there - see EmuSen_Galaxia.md §3.2.
        public static int RunConfig() => GalaxiaNative.Active ? GalaxiaNative.Migrate(GalaxiaMigrate.OwnConfig) : Managed.RunConfig();

        // A Mac app bundle's read-only skeleton, copied into home on every start and never over the player's files - see EmuSen_Galaxia.md §3.4.
        public static int SeedFromBundle() => GalaxiaNative.Active ? GalaxiaNative.Migrate(GalaxiaMigrate.OwnSeed) : Managed.SeedFromBundle();

        public static int SeedFromBundle(string? seedDirectory, string home) =>
            GalaxiaNative.Active ? GalaxiaNative.Migrate(GalaxiaMigrate.Seed, seedDirectory, home) : Managed.SeedFromBundle(seedDirectory, home);

        public static int Run(string legacyRoot, string destinationRoot) =>
            GalaxiaNative.Active ? GalaxiaNative.Migrate(GalaxiaMigrate.Between, legacyRoot, destinationRoot) : Managed.Run(legacyRoot, destinationRoot);

        // Copy, never move, and never overwrite - the ConfigFile<T>.MigrateFromLegacy rule.
        internal static int CopyTree(string source, string destination) =>
            GalaxiaNative.Active ? GalaxiaNative.Migrate(GalaxiaMigrate.Tree, source, destination) : Managed.CopyTree(source, destination);

        // Old ROM folders that still hold files, so a caller can say where they are.
        public static IReadOnlyList<string> RemainingLibraryDirectories() =>
            GalaxiaNative.Active ? GalaxiaNative.RemainingLibrary(null) : Managed.RemainingLibraryDirectories(Managed.LegacyRoot);

        public static IReadOnlyList<string> RemainingLibraryDirectories(string legacyRoot) =>
            GalaxiaNative.Active ? GalaxiaNative.RemainingLibrary(legacyRoot) : Managed.RemainingLibraryDirectories(legacyRoot);

        // The C# rules: the default, and what the library's are held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static string LegacyRootFor(string configRoot) =>
                Path.Combine(configRoot, "EmuSen.DianaOS", "DianaOS", "Usr", "Home");

            public static string LegacyRoot => LegacyRootFor(ConfigRoot.Managed.Directory);

            public static int Run() => Run(LegacyRoot, DataStore.Managed.UsrHome) + RunConfig();

            public static int RunConfig() => CopyTree(ConfigStore.Managed.PreviousDirectory, ConfigStore.Managed.Directory);

            public static int SeedFromBundle() => SeedFromBundle(ConfigRoot.Managed.SeedDirectory, DataStore.Managed.UsrHome);

            public static int SeedFromBundle(string? seedDirectory, string home) =>
                seedDirectory is null ? 0 : CopyTree(seedDirectory, home);

            public static int Run(string legacyRoot, string destinationRoot)
            {
                if (!Directory.Exists(legacyRoot)) return 0;
                if (string.Equals(Path.GetFullPath(legacyRoot), Path.GetFullPath(destinationRoot), StringComparison.OrdinalIgnoreCase)) return 0;

                int copied = 0;
                foreach (string name in MigratedDirectories)
                {
                    copied += CopyTree(Path.Combine(legacyRoot, name), Path.Combine(destinationRoot, name));
                }
                return copied;
            }

            public static int CopyTree(string source, string destination)
            {
                if (!Directory.Exists(source)) return 0;

                int copied = 0;
                foreach (string sourceFile in Directory.EnumerateFiles(source, "*", SearchOption.AllDirectories))
                {
                    string relative = Path.GetRelativePath(source, sourceFile);
                    string target = Path.Combine(destination, relative);
                    if (File.Exists(target)) continue;

                    try
                    {
                        Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                        File.Copy(sourceFile, target);
                        copied++;
                    }
                    catch (Exception ex)
                    {
                        ConfigDiagnostics.Report($"{sourceFile}: {ex.Message} Left in place.");
                    }
                }
                return copied;
            }

            public static IReadOnlyList<string> RemainingLibraryDirectories(string legacyRoot)
            {
                var found = new List<string>();
                foreach (string name in LibraryDirectories)
                {
                    string path = Path.Combine(legacyRoot, name);
                    try
                    {
                        if (Directory.Exists(path) && Directory.EnumerateFiles(path, "*", SearchOption.AllDirectories).GetEnumerator().MoveNext())
                        {
                            found.Add(path);
                        }
                    }
                    catch (Exception ex)
                    {
                        ConfigDiagnostics.Report($"{path}: {ex.Message}");
                    }
                }
                return found;
            }
        }
    }
}

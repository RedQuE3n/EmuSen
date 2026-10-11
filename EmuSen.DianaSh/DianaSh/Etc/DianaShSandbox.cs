using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;

namespace EmuSen.DianaSh.DianaSh.Etc
{
    // A walled garden, not a security boundary - see `man hier`.
    public static class DianaShSandbox
    {
        // Root discovery lives in Galaxia, below this project - see EmuSen_Config_Reference.md §1.1.
        public const string PublishedRootDirName = ConfigRoot.PublishedRootDirName;

        // Written to a published tree's root by DianaShPublishLayout.targets.
        public const string RootMarkerFileName = ConfigRoot.RootMarkerFileName;

        // What '/' means to this shell: the sandbox home, and nothing above it - see `man hier`.
        public static string RootDirectory => DataStore.UsrHome;

        // Where the install actually lives. Outside the shell's reach on purpose - see `man hier`.
        public static string InstallDirectory => ConfigRoot.Directory;

        public static string ComputeRootFor(string baseDirectory) => ConfigRoot.ComputeFor(baseDirectory);

        // These six forward to EmuSen.Galaxia's DataStore - see EmuSen_Galaxia.md §2.
        public static string UsrHomeDirectory => DataStore.UsrHome;
        public static string LogsDirectory => DataStore.Logs;
        public static string SavesDirectory => DataStore.Saves;
        public static string SaveStatesDirectory => DataStore.SaveStates;
        public static string FirmwareDirectory => DataStore.Firmware;
        public static string CheatDatabaseDirectory => DataStore.Cheats;

        public static string TempDirectory => Path.Combine(RootDirectory, "tmp");

        // WiseMan test-run scratch, inside /tmp so the shell can reach it - see `man hier`.
        public static string ScratchDirectory => Path.Combine(TempDirectory, "WiseMan");

        // The real directories this shell's tree needs - see `man hier`.
        private static void EnsureSkeleton()
        {
            Directory.CreateDirectory(LogsDirectory);
            Directory.CreateDirectory(SavesDirectory);
            Directory.CreateDirectory(SaveStatesDirectory);
            Directory.CreateDirectory(FirmwareDirectory);
            Directory.CreateDirectory(DataStore.Cheats);
            Directory.CreateDirectory(DataStore.Games);
            Directory.CreateDirectory(ConfigStore.Directory);
            Directory.CreateDirectory(TempDirectory);
        }

        // One home for everyone; the account name no longer picks a directory - see `man hier`.
        public static string HomeDirectory(string userName) => DataStore.UsrHome;

        // Old ROM folders still holding files, for the startup notice - see EmuSen_Galaxia.md §3.3.
        public static IReadOnlyList<string> UnmigratedLibraryDirectories => DataMigration.RemainingLibraryDirectories();

        // Forces the process's cwd to the sandbox home exactly once - see `man cd` and EmuSen_Debugging_Tools_Reference_v5.md §3.18.
        private static readonly Lazy<bool> _initialized = new(() =>
        {
            EnsureSkeleton();
            DataMigration.Run();
            Environment.CurrentDirectory = DataStore.UsrHome;
            return true;
        });

        public static void EnsureInitialWorkingDirectory() => _ = _initialized.Value;

        // Resolves requestedPath (chroot-style leading '/' - see `man hier`); an already-real path inside root is honored as-is.
        public static bool TryResolve(string requestedPath, out string resolvedPath)
        {
            string root = RootDirectory;
            bool looksRooted = requestedPath.Length > 0 && (requestedPath[0] == '/' || requestedPath[0] == '\\');

            if (looksRooted)
            {
                string asIs = Path.GetFullPath(requestedPath);
                if (IsInsideRoot(asIs, root))
                {
                    resolvedPath = asIs;
                    return true;
                }
            }

            string basePath = looksRooted ? root : Environment.CurrentDirectory;
            string effectivePath = looksRooted ? requestedPath.TrimStart('/', '\\') : requestedPath;
            if (effectivePath.Length == 0) effectivePath = "."; // bare '/' means this sandbox's own root

            string candidate = Path.GetFullPath(effectivePath, basePath);

            resolvedPath = candidate;
            return IsInsideRoot(candidate, root);
        }

        // A bare name is taken from baseDirectory, a rooted one as TryResolve takes it - see `man hier`.
        public static bool TryResolveFrom(string baseDirectory, string requestedPath, out string resolvedPath)
        {
            if (requestedPath.Length > 0 && (requestedPath[0] == '/' || requestedPath[0] == '\\')) return TryResolve(requestedPath, out resolvedPath);
            resolvedPath = Path.GetFullPath(requestedPath, baseDirectory);
            return IsInsideRoot(resolvedPath, RootDirectory);
        }

        // The configured ROM library, the one place outside the root a path may name, and only for `core` - see `man hier`.
        public static string? RomLibraryDirectory =>
            SettingFolders.Roms(EmuSen.Galaxia.Models.AppSettings.Load()) is { } configured ? Path.GetFullPath(configured) : null;

        // A ROM path: an absolute one inside the library as it is, anything else as TryResolve takes it - see `man core`.
        public static bool TryResolveRom(string requestedPath, out string resolvedPath)
        {
            if (Path.IsPathFullyQualified(requestedPath) && RomLibraryDirectory is { } library && IsInsideRoot(Path.GetFullPath(requestedPath), library))
            {
                resolvedPath = Path.GetFullPath(requestedPath);
                return true;
            }
            return TryResolve(requestedPath, out resolvedPath);
        }

        private static bool IsInsideRoot(string candidate, string root) =>
            candidate.Equals(root, StringComparison.OrdinalIgnoreCase)
                || candidate.StartsWith(root + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase);
    }
}

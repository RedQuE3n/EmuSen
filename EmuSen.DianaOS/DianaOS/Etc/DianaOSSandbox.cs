using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;

namespace EmuSen.DianaOS.DianaOS.Etc
{
    // A walled garden, not a security boundary - see `man hier`.
    public static class DianaOSSandbox
    {
        // Root discovery lives in Galaxia, below this project - see EmuSen_Config_Reference.md §1.1.
        public const string PublishedRootDirName = ConfigRoot.PublishedRootDirName;

        // Written to a published tree's root by DianaOSPublishLayout.targets.
        public const string RootMarkerFileName = ConfigRoot.RootMarkerFileName;

        // What '/' means to this shell: the user's home, and nothing above it - see `man hier`.
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

        // Forces the process's cwd to the user home exactly once - see `man cd` and EmuSen_Debugging_Tools_Reference_v5.md §3.18.
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

        private static bool IsInsideRoot(string candidate, string root) =>
            candidate.Equals(root, StringComparison.OrdinalIgnoreCase)
                || candidate.StartsWith(root + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase);
    }
}

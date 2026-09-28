using System;
using System.IO;

namespace EmuSen.Galaxia
{
    // Where this program's own on-disk tree starts - see `man hier` and
    // EmuSen_Config_Reference.md §1. DianaOSSandbox delegates here rather
    // than the other way round: this project sits below it, and "which
    // directory is ours" is a path-policy question, not a shell one.
    public static class ConfigRoot
    {
        // Fallback root beside the binary, when neither marker turns up.
        public const string PublishedRootDirName = "DianaOSRoot";

        // Written to a published tree's root by DianaOSPublishLayout.targets.
        public const string RootMarkerFileName = ".dianaosroot";

        // The folder under ~/Library/Application Support that holds a Mac's whole tree - see EmuSen_Galaxia.md §3.4.
        public const string MacDataDirName = "EmuSen";

        private static readonly Lazy<string> _directory = new(() => ComputeFor(AppContext.BaseDirectory));

        private static readonly Lazy<string?> _seedDirectory = new(() => SeedDirectoryFor(AppContext.BaseDirectory, OperatingSystem.IsMacOS()));

        public static string Directory => _directory.Value;

        // The read-only skeleton an app bundle carries, copied into the tree on start; null outside a bundle - see EmuSen_Galaxia.md §3.4.
        public static string? SeedDirectory => _seedDirectory.Value;

        // baseDirectory is AppContext.BaseDirectory, never the current directory, which a launcher chooses - see `man hier`.
        public static string ComputeFor(string baseDirectory) =>
            ComputeFor(baseDirectory, OperatingSystem.IsMacOS(), Environment.GetFolderPath(Environment.SpecialFolder.UserProfile));

        // The platform and home are parameters so every branch is testable on any host - see DianaOSSandboxTests.
        public static string ComputeFor(string baseDirectory, bool macOS, string userHome)
        {
            if (macOS && BundleContentsFor(baseDirectory) is not null) return MacDataDirectoryFor(userHome);

            string dir = Path.GetFullPath(baseDirectory);
            for (int i = 0; i < 10; i++)
            {
                if (File.Exists(Path.Combine(dir, "EmuSen.sln"))) return dir;
                if (File.Exists(Path.Combine(dir, RootMarkerFileName))) return dir;
                string? parent = Path.GetDirectoryName(Path.TrimEndingDirectorySeparator(dir));
                if (parent is null || parent == dir) break;
                dir = parent;
            }
            return macOS ? MacDataDirectoryFor(userHome) : Path.GetFullPath(Path.Combine(baseDirectory, PublishedRootDirName));
        }

        public static string MacDataDirectoryFor(string userHome) =>
            Path.Combine(userHome, "Library", "Application Support", MacDataDirName);

        // The bundle's Contents folder when baseDirectory is its Contents/MacOS, else null.
        public static string? BundleContentsFor(string baseDirectory)
        {
            string dir = Path.TrimEndingDirectorySeparator(Path.GetFullPath(baseDirectory));
            string? contents = Path.GetDirectoryName(dir);
            string? bundle = contents is null ? null : Path.GetDirectoryName(contents);
            bool inBundle = bundle is not null
                && string.Equals(Path.GetFileName(dir), "MacOS", StringComparison.OrdinalIgnoreCase)
                && string.Equals(Path.GetFileName(contents), "Contents", StringComparison.OrdinalIgnoreCase)
                && bundle.EndsWith(".app", StringComparison.OrdinalIgnoreCase);
            return inBundle ? contents : null;
        }

        // Contents/Resources/home, where the bundle publish puts what a Linux publish ships in home/.
        public static string? SeedDirectoryFor(string baseDirectory, bool macOS) =>
            macOS && BundleContentsFor(baseDirectory) is { } contents
                ? Path.Combine(contents, "Resources", Library.DataStore.HomeDirName)
                : null;
    }
}

using System;
using System.IO;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia
{
    // Where this program's own on-disk tree starts; DianaShSandbox delegates here - see `man hier` and EmuSen_Config_Reference.md §1.
    public static class ConfigRoot
    {
        // Fallback root beside the binary, when neither marker turns up.
        public const string PublishedRootDirName = "DianaShRoot";

        // The names a tree published before the shell was renamed carries, still honoured - see EmuSen_Galaxia.md §3.5.
        public const string LegacyPublishedRootDirName = "DianaOSRoot";
        public const string LegacyRootMarkerFileName = ".dianaosroot";

        // Written to a published tree's root by DianaShPublishLayout.targets.
        public const string RootMarkerFileName = ".dianashroot";

        // The folder under ~/Library/Application Support that holds a Mac's whole tree - see EmuSen_Galaxia.md §3.4.
        public const string MacDataDirName = "EmuSen";

        public static string Directory => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.Root)! : Managed.Directory;

        // The folder beside the program that holds the shell's shipped command modules and their manuals - see EmuSen_DianaSh.md §12.4.
        public const string ShellModulesDirName = "dianash";

        // Read-only and outside the shell's sandbox, as the program's own files are - see EmuSen_DianaSh.md §13.5.
        public static string ShellModulesDirectory => Path.Combine(AppContext.BaseDirectory, ShellModulesDirName);

        // The read-only skeleton an app bundle carries, copied into the tree on start; null outside a bundle - see EmuSen_Galaxia.md §3.4.
        public static string? SeedDirectory => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.Seed) : Managed.SeedDirectory;

        // .NET's application-data folder whether or not it exists yet, which plain GetFolderPath answers with "" - see EmuSen_Galaxia.md §3.6.
        internal static string ApplicationData => Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData, Environment.SpecialFolderOption.DoNotVerify);

        // baseDirectory is AppContext.BaseDirectory, never the current directory, which a launcher chooses - see `man hier`.
        public static string ComputeFor(string baseDirectory) =>
            ComputeFor(baseDirectory, OperatingSystem.IsMacOS(), Environment.GetFolderPath(Environment.SpecialFolder.UserProfile));

        // The platform and home are parameters so every branch is testable on any host - see DianaShSandboxTests.
        public static string ComputeFor(string baseDirectory, bool macOS, string userHome) =>
            GalaxiaNative.Takes(baseDirectory, userHome) ? GalaxiaNative.RootFor(baseDirectory, macOS, userHome) : Managed.ComputeFor(baseDirectory, macOS, userHome);

        public static string MacDataDirectoryFor(string userHome) =>
            GalaxiaNative.Takes(userHome) ? GalaxiaNative.MacDataDirectoryFor(userHome) : Managed.MacDataDirectoryFor(userHome);

        // The bundle's Contents folder when baseDirectory is its Contents/MacOS, else null.
        public static string? BundleContentsFor(string baseDirectory) =>
            GalaxiaNative.Takes(baseDirectory) ? GalaxiaNative.BundleContentsFor(baseDirectory) : Managed.BundleContentsFor(baseDirectory);

        // Contents/Resources/home, where the bundle publish puts what a Linux publish ships in home/.
        public static string? SeedDirectoryFor(string baseDirectory, bool macOS) =>
            GalaxiaNative.Takes(baseDirectory) ? GalaxiaNative.SeedDirectoryFor(baseDirectory, macOS) : Managed.SeedDirectoryFor(baseDirectory, macOS);

        // The C# rules: the default, and what the library's are held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            private static readonly Lazy<string> _directory = new(() => ComputeFor(AppContext.BaseDirectory, OperatingSystem.IsMacOS(), Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)));

            private static readonly Lazy<string?> _seedDirectory = new(() => SeedDirectoryFor(AppContext.BaseDirectory, OperatingSystem.IsMacOS()));

            public static string Directory => _directory.Value;

            public static string? SeedDirectory => _seedDirectory.Value;

            public static string ComputeFor(string baseDirectory, bool macOS, string userHome)
            {
                if (macOS && BundleContentsFor(baseDirectory) is not null) return MacDataDirectoryFor(userHome);

                string dir = Path.GetFullPath(baseDirectory);
                for (int i = 0; i < 10; i++)
                {
                    if (File.Exists(Path.Combine(dir, "EmuSen.sln"))) return dir;
                    if (File.Exists(Path.Combine(dir, RootMarkerFileName)) || File.Exists(Path.Combine(dir, LegacyRootMarkerFileName))) return dir;
                    string? parent = Path.GetDirectoryName(Path.TrimEndingDirectorySeparator(dir));
                    if (parent is null || parent == dir) break;
                    dir = parent;
                }
                if (macOS) return MacDataDirectoryFor(userHome);
                string legacy = Path.GetFullPath(Path.Combine(baseDirectory, LegacyPublishedRootDirName));
                string fallback = Path.GetFullPath(Path.Combine(baseDirectory, PublishedRootDirName));
                return !System.IO.Directory.Exists(fallback) && System.IO.Directory.Exists(legacy) ? legacy : fallback;
            }

            public static string MacDataDirectoryFor(string userHome) =>
                Path.Combine(userHome, "Library", "Application Support", MacDataDirName);

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

            public static string? SeedDirectoryFor(string baseDirectory, bool macOS) =>
                macOS && BundleContentsFor(baseDirectory) is { } contents
                    ? Path.Combine(contents, "Resources", Library.DataStore.HomeDirName)
                    : null;
        }
    }
}

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

        public static string Directory => GalaxiaNative.Directory(GalaxiaDirectory.Root)!;

        // The folder beside the program that holds the shell's shipped command modules and their manuals - see EmuSen_DianaSh.md §12.4.
        public const string ShellModulesDirName = "dianash";

        // Read-only and outside the shell's sandbox, as the program's own files are - see EmuSen_DianaSh.md §13.5.
        public static string ShellModulesDirectory => Path.Combine(AppContext.BaseDirectory, ShellModulesDirName);

        // The read-only skeleton an app bundle carries, copied into the tree on start; null outside a bundle - see EmuSen_Galaxia.md §3.4.
        public static string? SeedDirectory => GalaxiaNative.Directory(GalaxiaDirectory.Seed);

        // .NET's application-data folder whether or not it exists yet, which plain GetFolderPath answers with "" - see EmuSen_Galaxia.md §3.6.
        internal static string ApplicationData => Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData, Environment.SpecialFolderOption.DoNotVerify);

        // baseDirectory is AppContext.BaseDirectory, never the current directory, which a launcher chooses - see `man hier`.
        public static string ComputeFor(string baseDirectory) =>
            ComputeFor(baseDirectory, OperatingSystem.IsMacOS(), Environment.GetFolderPath(Environment.SpecialFolder.UserProfile));

        // The platform and home are parameters so every branch is testable on any host - see DianaShSandboxTests.
        public static string ComputeFor(string baseDirectory, bool macOS, string userHome) =>
            GalaxiaNative.RootFor(baseDirectory, macOS, userHome);

        public static string MacDataDirectoryFor(string userHome) =>
            GalaxiaNative.MacDataDirectoryFor(userHome);

        // The bundle's Contents folder when baseDirectory is its Contents/MacOS, else null.
        public static string? BundleContentsFor(string baseDirectory) =>
            GalaxiaNative.BundleContentsFor(baseDirectory);

        // Contents/Resources/home, where the bundle publish puts what a Linux publish ships in home/.
        public static string? SeedDirectoryFor(string baseDirectory, bool macOS) =>
            GalaxiaNative.SeedDirectoryFor(baseDirectory, macOS);
    }
}

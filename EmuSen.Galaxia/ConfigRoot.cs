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

        private static readonly Lazy<string> _directory = new(() => ComputeFor(AppContext.BaseDirectory));

        public static string Directory => _directory.Value;

        // baseDirectory is AppContext.BaseDirectory in production - NOT
        // Environment.CurrentDirectory, which a launcher/shortcut gets to decide.
        // Parameterized purely so both branches are testable - see DianaOSSandboxTests.
        public static string ComputeFor(string baseDirectory)
        {
            string dir = Path.GetFullPath(baseDirectory);
            for (int i = 0; i < 10; i++)
            {
                if (File.Exists(Path.Combine(dir, "EmuSen.sln"))) return dir;
                if (File.Exists(Path.Combine(dir, RootMarkerFileName))) return dir;
                string? parent = Path.GetDirectoryName(Path.TrimEndingDirectorySeparator(dir));
                if (parent is null || parent == dir) break;
                dir = parent;
            }
            return Path.GetFullPath(Path.Combine(baseDirectory, PublishedRootDirName));
        }
    }
}

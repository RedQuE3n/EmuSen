using System;
using System.IO;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Etc
{
    // Confines every real-filesystem-touching shell command (cd, ls, mv,
    // awk's file argument, wc's file argument, redirection) to this
    // project's own directory tree. Deliberately a "walled garden," not a
    // general security boundary - nothing here defends against a hostile
    // ROM or a deliberately adversarial shell script, and it isn't meant
    // to; it exists so an ordinary `cd ../../..` or `mv foo ../../bar`
    // can't wander this shell off the project entirely, since this shell
    // was only ever meant to poke around this project's own files.
    public static class DianaOSSandbox
    {
        // The two roots this shell can have, and why the published one is a
        // subdirectory rather than the binary's own folder: see `man hier`.
        public const string PublishedRootDirName = "DianaOSRoot";

        private static readonly Lazy<string> _root = new(() => ComputeRootFor(AppContext.BaseDirectory));

        public static string RootDirectory => _root.Value;

        // baseDirectory is AppContext.BaseDirectory in production - NOT
        // Environment.CurrentDirectory, which a launcher/shortcut gets to decide.
        // Parameterized purely so both branches are testable - see DianaOSSandboxTests.
        public static string ComputeRootFor(string baseDirectory)
        {
            string dir = Path.GetFullPath(baseDirectory);
            for (int i = 0; i < 10; i++)
            {
                if (File.Exists(Path.Combine(dir, "EmuSen.sln"))) return dir;
                string? parent = Path.GetDirectoryName(Path.TrimEndingDirectorySeparator(dir));
                if (parent is null || parent == dir) break;
                dir = parent;
            }
            return Path.GetFullPath(Path.Combine(baseDirectory, PublishedRootDirName));
        }

        // The DianaOS project's own "user data" home - emulator-facing
        // output (dump/load/screenshot/recording logs, SRAM + state
        // saves) lives here now instead of under var/, see `man hier`.
        public static string UsrHomeDirectory => Path.Combine(RootDirectory, "EmuSen.DianaOS", "DianaOS", "Usr", "Home");
        public static string LogsDirectory => Path.Combine(UsrHomeDirectory, "Logs");
        public static string SavesDirectory => Path.Combine(UsrHomeDirectory, "Saves");
        public static string SaveStatesDirectory => Path.Combine(SavesDirectory, "Save States");

        // WiseMan test-run scratch space only - dev/test artifacts, not
        // emulator output, kept out of Usr/Home so it isn't mistaken for it.
        public static string SourceLogsDirectory => Path.Combine(RootDirectory, "SourceLogs");

        // The user-facing Usr/Home folders that hold no shipped content - empty in a
        // published build, already populated when running from source. See `man hier`.
        private static readonly string[] UsrHomeStubs = { "Roms", "Games", "Music", "Pictures" };

        // The real directories this shell's Unix-shaped tree needs - see `man hier`.
        private static void EnsureSkeleton()
        {
            string root = RootDirectory;
            Directory.CreateDirectory(LogsDirectory);
            Directory.CreateDirectory(SavesDirectory);
            Directory.CreateDirectory(SaveStatesDirectory);
            Directory.CreateDirectory(SourceLogsDirectory);
            foreach (string stub in UsrHomeStubs) Directory.CreateDirectory(Path.Combine(UsrHomeDirectory, stub));
            Directory.CreateDirectory(Path.Combine(root, "home", "root"));
            Directory.CreateDirectory(Path.Combine(root, "etc"));
            Directory.CreateDirectory(Path.Combine(root, "tmp"));
        }

        public static string HomeDirectory(string userName) => Path.Combine(RootDirectory, "home", userName);

        // Forces the process's cwd to root's own home dir exactly once - see `man cd` and EmuSen_Debugging_Tools_Reference_v5.md §3.18.
        private static readonly Lazy<bool> _initialized = new(() =>
        {
            EnsureSkeleton();
            Environment.CurrentDirectory = HomeDirectory("root");
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

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
        // Walks upward from wherever this assembly is actually running
        // from (AppContext.BaseDirectory - NOT Environment.CurrentDirectory,
        // which is exactly the thing a launcher/shortcut gets to decide
        // and so can't be trusted as a starting point) looking for
        // EmuSen.sln, the one file guaranteed to sit at this project's
        // own root today. Once this project is ever published as a
        // standalone binary, EmuSen.sln won't ship alongside it - the
        // walk then falls back to AppContext.BaseDirectory itself, so
        // "the folder the binary lives in" becomes the walled root,
        // which is exactly "still the parent directory" for that case.
        private static readonly Lazy<string> _root = new(ComputeRoot);

        public static string RootDirectory => _root.Value;

        private static string ComputeRoot()
        {
            string dir = Path.GetFullPath(AppContext.BaseDirectory);
            for (int i = 0; i < 10; i++)
            {
                if (File.Exists(Path.Combine(dir, "EmuSen.sln"))) return dir;
                string? parent = Path.GetDirectoryName(Path.TrimEndingDirectorySeparator(dir));
                if (parent is null || parent == dir) break;
                dir = parent;
            }
            return Path.GetFullPath(AppContext.BaseDirectory);
        }

        // The real directories this shell's Unix-shaped tree needs - see `man hier`.
        private static void EnsureSkeleton()
        {
            string root = RootDirectory;
            Directory.CreateDirectory(Path.Combine(root, "var", "log"));
            Directory.CreateDirectory(Path.Combine(root, "var", "lib"));
            Directory.CreateDirectory(Path.Combine(root, "var", "games"));
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

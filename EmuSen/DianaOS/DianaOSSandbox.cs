using System;
using System.IO;

namespace EmuSen.DianaOS
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

        // Forces the process's cwd to the sandbox root exactly once, the
        // first time any DianaOSInterpreter is created (see CreateDefault) -
        // whatever the OS/launcher happened to start the process in
        // otherwise (a GUI app's launch directory is not something this
        // project controls) isn't a directory this shell should ever be
        // sitting in by default. Deliberately NOT re-applied on every
        // call (Mistress9 rebuilds its DianaOSInterpreter on every ROM
        // (re)load) - that would silently kick a user back to the root
        // after every ROM swap even though they're still safely inside
        // the sandbox and didn't ask to be moved.
        private static readonly Lazy<bool> _initialized = new(() =>
        {
            Environment.CurrentDirectory = RootDirectory;
            return true;
        });

        public static void EnsureInitialWorkingDirectory() => _ = _initialized.Value;

        // Resolves `requestedPath` the same way any real filesystem API
        // would (relative to the process's current directory) and reports
        // whether the result stays at or under RootDirectory. Every shell
        // command that touches a real path should route through this
        // rather than calling File/Directory APIs on a raw argument.
        // Case-insensitive comparison even on Linux - simpler than
        // special-casing per platform, and erring toward treating a
        // same-path-different-case as "still inside" costs nothing here
        // (this is a walled garden against typos/accidents, not a
        // security control someone would try to defeat via case tricks).
        public static bool TryResolve(string requestedPath, out string resolvedPath)
        {
            string candidate = Path.GetFullPath(requestedPath, Environment.CurrentDirectory);
            string root = RootDirectory;

            resolvedPath = candidate;
            return candidate.Equals(root, StringComparison.OrdinalIgnoreCase)
                || candidate.StartsWith(root + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase);
        }
    }
}

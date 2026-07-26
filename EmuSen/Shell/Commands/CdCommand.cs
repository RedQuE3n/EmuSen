using System;
using System.IO;

namespace EmuSen.Shell.Commands
{
    // Unix `cd` - changes Environment.CurrentDirectory, the process-wide
    // cwd `pwd` reads and every relative path elsewhere in this shell
    // (redirection, `ls`, `mv`, `dump`/`load`, ...) resolves against.
    // Process-wide rather than a per-ShellInterpreter field deliberately:
    // this shell doesn't model real subshell process isolation anywhere
    // else either (a `$(...)` subshell shares this same interpreter's
    // notion of "the filesystem" with its parent), so there's no existing
    // boundary a shell-private cwd would actually respect.
    //
    // Walled into ShellSandbox.RootDirectory (see that file's own
    // comment) - can't cd above it no matter how many "cd .."s or an
    // absolute path outside it are used. No-arg `cd` goes to the sandbox
    // root rather than the real $HOME, since $HOME is exactly the kind
    // of "outside the project" directory this sandbox exists to keep
    // this shell out of.
    public class CdCommand : IShellCommand
    {
        public string Name => "cd";
        public string Usage => "  cd [dir]                      change directory, walled to the project root (no arg: go there)";

        public ShellResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            string dir = args.Length >= 2 ? args[1] : ShellSandbox.RootDirectory;

            if (!ShellSandbox.TryResolve(dir, out string resolved))
            {
                return ShellResult.Fail($"cd: '{dir}' is outside the project sandbox ({ShellSandbox.RootDirectory})");
            }

            if (!Directory.Exists(resolved))
            {
                return ShellResult.Fail($"cd: no such directory: {dir}");
            }

            try
            {
                Environment.CurrentDirectory = resolved;
            }
            catch (Exception ex)
            {
                return ShellResult.Fail($"cd: {ex.Message}");
            }

            return ShellResult.Ok("");
        }
    }
}

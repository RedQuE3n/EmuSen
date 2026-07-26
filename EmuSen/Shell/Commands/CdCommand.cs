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
    public class CdCommand : IShellCommand
    {
        public string Name => "cd";
        public string Usage => "  cd [dir]                      change the current working directory (no arg: go to $HOME)";

        public ShellResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            string dir = args.Length >= 2
                ? args[1]
                : Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

            if (!Directory.Exists(dir))
            {
                return ShellResult.Fail($"cd: no such directory: {dir}");
            }

            try
            {
                Environment.CurrentDirectory = dir;
            }
            catch (Exception ex)
            {
                return ShellResult.Fail($"cd: {ex.Message}");
            }

            return ShellResult.Ok("");
        }
    }
}

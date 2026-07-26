using System;
using System.IO;

namespace EmuSen.Shell.Commands
{
    // Unix `mv` - renames/moves a real file or directory. No `-i`
    // confirmation prompt (nothing here is interactive, and permissions/
    // safety prompts are explicitly out of scope for this shell - it only
    // ever runs against the local dotnet process's own filesystem access,
    // never anything privilege-sensitive), so an existing destination
    // FILE is silently overwritten, matching real `mv`'s own default
    // (non-`-i`, non-`-n`) behavior. An existing destination DIRECTORY
    // isn't overwritten, it's moved INTO - `mv foo.txt logs/` behaves
    // like real `mv`, landing at `logs/foo.txt`, not replacing `logs`
    // itself.
    public class MvCommand : IShellCommand
    {
        public string Name => "mv";
        public string Usage => "  mv <src> <dst>                move/rename a file or directory (dst may be an existing directory)";

        public ShellResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 3) return ShellResult.Fail("mv: usage: mv <src> <dst>");

            string src = args[1];
            string dst = args[2];

            if (Directory.Exists(dst)) dst = Path.Combine(dst, Path.GetFileName(src.TrimEnd('/', '\\')));

            try
            {
                if (Directory.Exists(src))
                {
                    if (File.Exists(dst) || Directory.Exists(dst))
                    {
                        return ShellResult.Fail($"mv: destination already exists: {dst}");
                    }
                    Directory.Move(src, dst);
                }
                else if (File.Exists(src))
                {
                    File.Move(src, dst, overwrite: true);
                }
                else
                {
                    return ShellResult.Fail($"mv: no such file or directory: {src}");
                }
            }
            catch (Exception ex)
            {
                return ShellResult.Fail($"mv: {ex.Message}");
            }

            return ShellResult.Ok("");
        }
    }
}

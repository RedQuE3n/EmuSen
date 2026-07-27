using System;
using System.IO;

namespace EmuSen.DianaOS.Commands.Unix
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
    // itself. Walled to DianaOSSandbox.RootDirectory - both src and dst
    // must resolve inside it (see that file's own comment).
    public class MvCommand : IDianaOSCommand
    {
        public string Name => "mv";
        public bool IsReadOnly => false;
        public string Usage => "  mv <src> <dst>                move/rename a file or directory (dst may be an existing directory)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 3) return DianaOSResult.Fail("mv: usage: mv <src> <dst>");

            if (!DianaOSSandbox.TryResolve(args[1], out string src))
            {
                return DianaOSResult.Fail($"mv: '{args[1]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }
            if (!DianaOSSandbox.TryResolve(args[2], out string dst))
            {
                return DianaOSResult.Fail($"mv: '{args[2]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            if (Directory.Exists(dst)) dst = Path.Combine(dst, Path.GetFileName(src.TrimEnd('/', '\\')));

            try
            {
                if (Directory.Exists(src))
                {
                    if (File.Exists(dst) || Directory.Exists(dst))
                    {
                        return DianaOSResult.Fail($"mv: destination already exists: {dst}");
                    }
                    Directory.Move(src, dst);
                }
                else if (File.Exists(src))
                {
                    File.Move(src, dst, overwrite: true);
                }
                else
                {
                    return DianaOSResult.Fail($"mv: no such file or directory: {src}");
                }
            }
            catch (Exception ex)
            {
                return DianaOSResult.Fail($"mv: {ex.Message}");
            }

            return DianaOSResult.Ok("");
        }
    }
}

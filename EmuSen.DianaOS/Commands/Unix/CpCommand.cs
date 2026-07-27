using System;
using System.IO;

namespace EmuSen.DianaOS.Commands.Unix
{
    // Unix `cp` - see `man cp`.
    public class CpCommand : IDianaOSCommand
    {
        public string Name => "cp";
        public bool IsReadOnly => false;
        public string Usage => "  cp [-r] <src> <dst>           copy a file (-r: copy a directory and everything in it; dst may be an existing directory)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            bool recursive = args.Length >= 2 &&
                (args[1].Equals("-r", StringComparison.OrdinalIgnoreCase) || args[1].Equals("-rf", StringComparison.OrdinalIgnoreCase));
            int pathStart = recursive ? 2 : 1;
            if (args.Length < pathStart + 2) return DianaOSResult.Fail("cp: usage: cp [-r] <src> <dst>");

            if (!DianaOSSandbox.TryResolve(args[pathStart], out string src))
            {
                return DianaOSResult.Fail($"cp: '{args[pathStart]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }
            if (!DianaOSSandbox.TryResolve(args[pathStart + 1], out string dst))
            {
                return DianaOSResult.Fail($"cp: '{args[pathStart + 1]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            if (Directory.Exists(dst)) dst = Path.Combine(dst, Path.GetFileName(src.TrimEnd('/', '\\')));

            try
            {
                if (Directory.Exists(src))
                {
                    if (!recursive)
                    {
                        return DianaOSResult.Fail($"cp: '{args[pathStart]}' is a directory (use -r to copy directories)");
                    }
                    CopyDirectory(src, dst);
                }
                else if (File.Exists(src))
                {
                    File.Copy(src, dst, overwrite: true);
                }
                else
                {
                    return DianaOSResult.Fail($"cp: no such file or directory: {args[pathStart]}");
                }
            }
            catch (Exception ex)
            {
                return DianaOSResult.Fail($"cp: {ex.Message}");
            }

            return DianaOSResult.Ok("");
        }

        private static void CopyDirectory(string src, string dst)
        {
            Directory.CreateDirectory(dst);
            foreach (string dir in Directory.GetDirectories(src, "*", SearchOption.AllDirectories))
            {
                Directory.CreateDirectory(Path.Combine(dst, Path.GetRelativePath(src, dir)));
            }
            foreach (string file in Directory.GetFiles(src, "*", SearchOption.AllDirectories))
            {
                File.Copy(file, Path.Combine(dst, Path.GetRelativePath(src, file)), overwrite: true);
            }
        }
    }
}

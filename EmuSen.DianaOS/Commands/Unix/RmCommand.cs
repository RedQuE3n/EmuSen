using System;
using System.IO;
using System.Linq;

namespace EmuSen.DianaOS.Commands.Unix
{
    // Unix `rm` - see `man rm`.
    public class RmCommand : IDianaOSCommand
    {
        public string Name => "rm";
        public bool IsReadOnly => false;
        public string Usage => "  rm [-r] <path>                delete a file (-r: delete a directory and everything in it)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaOSResult.Fail("rm: usage: rm [-r] <path>");

            bool recursive = args[1].Equals("-r", StringComparison.OrdinalIgnoreCase)
                || args[1].Equals("-rf", StringComparison.OrdinalIgnoreCase);
            int pathStart = recursive ? 2 : 1;
            if (args.Length <= pathStart) return DianaOSResult.Fail("rm: usage: rm [-r] <path>");

            string path = string.Join(' ', args.Skip(pathStart));

            if (!DianaOSSandbox.TryResolve(path, out string resolved))
            {
                return DianaOSResult.Fail($"rm: '{path}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            if (resolved.Equals(DianaOSSandbox.RootDirectory, StringComparison.OrdinalIgnoreCase))
            {
                return DianaOSResult.Fail("rm: refusing to remove the project's own root directory");
            }

            try
            {
                if (Directory.Exists(resolved))
                {
                    if (!recursive)
                    {
                        return DianaOSResult.Fail($"rm: '{path}' is a directory (use -r to remove directories)");
                    }
                    Directory.Delete(resolved, recursive: true);
                }
                else if (File.Exists(resolved))
                {
                    File.Delete(resolved);
                }
                else
                {
                    return DianaOSResult.Fail($"rm: no such file or directory: {path}");
                }
            }
            catch (Exception ex)
            {
                return DianaOSResult.Fail($"rm: {ex.Message}");
            }

            return DianaOSResult.Ok("");
        }
    }
}

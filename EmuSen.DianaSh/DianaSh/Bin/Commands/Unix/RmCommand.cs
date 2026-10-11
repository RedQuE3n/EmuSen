using System;
using System.IO;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Unix `rm` - see `man rm`.
    public class RmCommand : IDianaShCommand
    {
        public string Name => "rm";
        public bool IsReadOnly => false;
        public string Usage => "  rm [-r] <path>                SUSPENDED - delete a file (-r: delete a directory and everything in it)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (DianaShCommandSuspensions.IsSuspended(Name)) return DianaShCommandSuspensions.Refuse(Name);

            if (args.Length < 2) return DianaShResult.Fail("rm: usage: rm [-r] <path>");

            bool recursive = args[1].Equals("-r", StringComparison.OrdinalIgnoreCase)
                || args[1].Equals("-rf", StringComparison.OrdinalIgnoreCase);
            int pathStart = recursive ? 2 : 1;
            if (args.Length <= pathStart) return DianaShResult.Fail("rm: usage: rm [-r] <path>");

            string path = string.Join(' ', args.Skip(pathStart));

            if (!DianaShSandbox.TryResolve(path, out string resolved))
            {
                return DianaShResult.Fail($"rm: '{path}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
            }

            if (resolved.Equals(DianaShSandbox.RootDirectory, StringComparison.OrdinalIgnoreCase))
            {
                return DianaShResult.Fail("rm: refusing to remove the project's own root directory");
            }

            try
            {
                if (Directory.Exists(resolved))
                {
                    if (!recursive)
                    {
                        return DianaShResult.Fail($"rm: '{path}' is a directory (use -r to remove directories)");
                    }
                    Directory.Delete(resolved, recursive: true);
                }
                else if (File.Exists(resolved))
                {
                    File.Delete(resolved);
                }
                else
                {
                    return DianaShResult.Fail($"rm: no such file or directory: {path}");
                }
            }
            catch (Exception ex)
            {
                return DianaShResult.Fail($"rm: {ex.Message}");
            }

            return DianaShResult.Ok("");
        }
    }
}

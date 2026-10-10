using System;
using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Unix `touch` - see `man touch`.
    public class TouchCommand : IDianaShCommand
    {
        public string Name => "touch";
        public bool IsReadOnly => false;
        public string Usage => "  touch <path>                  create an empty file, or update an existing one's modified time";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaShResult.Fail("touch: usage: touch <path>");

            if (!DianaShSandbox.TryResolve(args[1], out string resolved))
            {
                return DianaShResult.Fail($"touch: '{args[1]}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
            }

            if (Directory.Exists(resolved))
            {
                return DianaShResult.Fail($"touch: '{args[1]}' is a directory");
            }

            try
            {
                if (File.Exists(resolved))
                {
                    File.SetLastWriteTime(resolved, DateTime.Now);
                }
                else
                {
                    File.WriteAllBytes(resolved, Array.Empty<byte>());
                }
            }
            catch (Exception ex)
            {
                return DianaShResult.Fail($"touch: {ex.Message}");
            }

            return DianaShResult.Ok("");
        }
    }
}

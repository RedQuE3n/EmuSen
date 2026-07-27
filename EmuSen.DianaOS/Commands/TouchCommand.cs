using System;
using System.IO;

namespace EmuSen.DianaOS.Commands
{
    // Unix `touch` - see `man touch`.
    public class TouchCommand : IDianaOSCommand
    {
        public string Name => "touch";
        public bool IsReadOnly => false;
        public string Usage => "  touch <path>                  create an empty file, or update an existing one's modified time";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaOSResult.Fail("touch: usage: touch <path>");

            if (!DianaOSSandbox.TryResolve(args[1], out string resolved))
            {
                return DianaOSResult.Fail($"touch: '{args[1]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            if (Directory.Exists(resolved))
            {
                return DianaOSResult.Fail($"touch: '{args[1]}' is a directory");
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
                return DianaOSResult.Fail($"touch: {ex.Message}");
            }

            return DianaOSResult.Ok("");
        }
    }
}

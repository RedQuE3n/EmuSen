using System;
using System.IO;
using System.Linq;

namespace EmuSen.DianaOS.Commands
{
    // Unix `tail` - see `man tail`.
    public class TailCommand : IDianaOSCommand
    {
        public string Name => "tail";
        public bool IsReadOnly => true;
        public string Usage => "  tail [-n N] [path]            last N lines (default 10) of stdin or a file";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            int count = 10;
            string? path = null;
            for (int i = 1; i < args.Length; i++)
            {
                if (args[i] == "-n" && i + 1 < args.Length)
                {
                    if (!int.TryParse(args[++i], out count) || count < 0)
                    {
                        return DianaOSResult.Fail("tail: -n expects a non-negative integer");
                    }
                }
                else
                {
                    path = args[i];
                }
            }

            string text;
            if (stdin != null)
            {
                text = stdin;
            }
            else if (path != null)
            {
                if (!DianaOSSandbox.TryResolve(path, out string resolved))
                {
                    return DianaOSResult.Fail($"tail: '{path}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
                }
                if (!File.Exists(resolved))
                {
                    return DianaOSResult.Fail($"tail: no such file: {path}");
                }
                text = File.ReadAllText(resolved);
            }
            else
            {
                return DianaOSResult.Fail("tail: usage: tail [-n N] [path]");
            }

            if (text.Length == 0) return "";
            string[] lines = text.Split('\n');
            return string.Join('\n', lines.Skip(Math.Max(0, lines.Length - count)));
        }
    }
}

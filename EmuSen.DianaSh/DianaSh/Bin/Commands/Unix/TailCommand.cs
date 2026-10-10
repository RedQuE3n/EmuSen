using System;
using System.IO;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Unix `tail` - see `man tail`.
    public class TailCommand : IDianaShCommand
    {
        public string Name => "tail";
        public bool IsReadOnly => true;
        public string Usage => "  tail [-n N] [path]            last N lines (default 10) of stdin or a file";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            int count = 10;
            string? path = null;
            for (int i = 1; i < args.Length; i++)
            {
                if (args[i] == "-n" && i + 1 < args.Length)
                {
                    if (!int.TryParse(args[++i], out count) || count < 0)
                    {
                        return DianaShResult.Fail("tail: -n expects a non-negative integer");
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
                if (!DianaShSandbox.TryResolve(path, out string resolved))
                {
                    return DianaShResult.Fail($"tail: '{path}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
                }
                if (!File.Exists(resolved))
                {
                    return DianaShResult.Fail($"tail: no such file: {path}");
                }
                text = File.ReadAllText(resolved);
            }
            else
            {
                return DianaShResult.Fail("tail: usage: tail [-n N] [path]");
            }

            if (text.Length == 0) return "";
            string[] lines = text.Split('\n');
            return string.Join('\n', lines.Skip(Math.Max(0, lines.Length - count)));
        }
    }
}

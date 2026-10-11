using System.IO;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Unix `cat` - see `man cat`.
    public class CatCommand : IDianaShCommand
    {
        public string Name => "cat";
        public bool IsReadOnly => true;
        public string Usage => "  cat <path>...                 print one or more files' contents";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2)
            {
                return stdin ?? DianaShResult.Fail("cat: usage: cat <path>...");
            }

            var chunks = new System.Collections.Generic.List<string>();
            foreach (string path in args.Skip(1))
            {
                if (!DianaShSandbox.TryResolve(path, out string resolved))
                {
                    return DianaShResult.Fail($"cat: '{path}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
                }
                if (!File.Exists(resolved))
                {
                    return DianaShResult.Fail($"cat: no such file: {path}");
                }
                chunks.Add(File.ReadAllText(resolved));
            }

            return string.Concat(chunks).TrimEnd('\n');
        }
    }
}

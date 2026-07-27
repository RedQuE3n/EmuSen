using System.IO;
using System.Linq;

namespace EmuSen.DianaOS.Commands.Unix
{
    // Unix `cat` - see `man cat`.
    public class CatCommand : IDianaOSCommand
    {
        public string Name => "cat";
        public bool IsReadOnly => true;
        public string Usage => "  cat <path>...                 print one or more files' contents";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2)
            {
                return stdin ?? DianaOSResult.Fail("cat: usage: cat <path>...");
            }

            var chunks = new System.Collections.Generic.List<string>();
            foreach (string path in args.Skip(1))
            {
                if (!DianaOSSandbox.TryResolve(path, out string resolved))
                {
                    return DianaOSResult.Fail($"cat: '{path}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
                }
                if (!File.Exists(resolved))
                {
                    return DianaOSResult.Fail($"cat: no such file: {path}");
                }
                chunks.Add(File.ReadAllText(resolved));
            }

            return string.Concat(chunks).TrimEnd('\n');
        }
    }
}

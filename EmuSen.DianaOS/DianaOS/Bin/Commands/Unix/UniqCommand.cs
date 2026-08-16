using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Adjacent duplicates only, as real uniq; sort first for a global dedup - see §3.17.
    public class UniqCommand : IDianaOSCommand
    {
        public string Name => "uniq";
        public bool IsReadOnly => true;
        public string Usage => "  uniq [-c]                     collapse adjacent duplicate lines (-c prefixes each with its\n" +
                                "                                repeat count) - sort first for global dedup, not just adjacent";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            bool withCount = args.Length >= 2 && args[1] == "-c";
            int textStart = withCount ? 2 : 1;

            string? text = stdin ?? (textStart < args.Length ? string.Join(' ', args, textStart, args.Length - textStart) : null);
            if (text == null) return "";

            string[] lines = text.Split('\n');
            var result = new List<string>();
            string? current = null;
            int count = 0;

            void Flush()
            {
                if (current == null) return;
                result.Add(withCount ? $"{count} {current}" : current);
            }

            foreach (string line in lines)
            {
                if (line == current)
                {
                    count++;
                }
                else
                {
                    Flush();
                    current = line;
                    count = 1;
                }
            }
            Flush();

            return string.Join('\n', result);
        }
    }
}

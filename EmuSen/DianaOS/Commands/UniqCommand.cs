using System.Collections.Generic;

namespace EmuSen.DianaOS.Commands
{
    // Unix `uniq` - collapses ADJACENT duplicate lines only, same real
    // semantic as bash's own uniq (it does not globally deduplicate) -
    // pipe through `sort` first for that: `regs | sort | uniq`. `-c`
    // prefixes each remaining line with how many consecutive times it
    // repeated, matching real uniq's own column format.
    public class UniqCommand : IDianaOSCommand
    {
        public string Name => "uniq";
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

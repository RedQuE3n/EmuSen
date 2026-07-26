using System;
using System.Linq;

namespace EmuSen.Shell.Commands
{
    // Unix `sort` - sorts input lines. Deliberately minimal: no `-u`
    // (that's `uniq`'s job - pair them, `sort | uniq`, the same way real
    // shell usage does before reaching for `sort -u` as a shortcut), no
    // field/key selection (`-k`), no locale-aware collation - ordinal
    // string comparison, or numeric with `-n`.
    public class SortCommand : IShellCommand
    {
        public string Name => "sort";
        public string Usage => "  sort [-n] [-r]                sort input lines (numeric with -n, reversed with -r) -\n" +
                                "                                reads piped stdin, or trailing literal text";

        public ShellResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            bool numeric = false, reverse = false;
            int i = 1;
            for (; i < args.Length; i++)
            {
                if (args[i] == "-n") numeric = true;
                else if (args[i] == "-r") reverse = true;
                else break;
            }

            string? text = stdin ?? (i < args.Length ? string.Join(' ', args, i, args.Length - i) : null);
            if (text == null) return "";

            string[] lines = text.Split('\n');
            IOrderedEnumerable<string> sorted = numeric
                ? lines.OrderBy(l => double.TryParse(l.Trim(), out double v) ? v : 0)
                : lines.OrderBy(l => l, StringComparer.Ordinal);

            var result = sorted.ToList();
            if (reverse) result.Reverse();
            return string.Join('\n', result);
        }
    }
}

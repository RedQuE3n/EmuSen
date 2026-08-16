using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // No -u: that is `uniq`'s job, paired as real shell usage does - see §3.17.
    public class SortCommand : IDianaOSCommand
    {
        public string Name => "sort";
        public bool IsReadOnly => true;
        public string Usage => "  sort [-n] [-r]                sort input lines (numeric with -n, reversed with -r) -\n" +
                                "                                reads piped stdin, or trailing literal text";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
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

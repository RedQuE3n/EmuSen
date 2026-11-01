using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Adjacent duplicates only, as real uniq; sort first for a global dedup; -D and -w as man uniq - see §3.17.
    public class UniqCommand : IDianaShCommand
    {
        private const string Short = "Usage: uniq [-c] [-D] [-w <chars>] - see man uniq";

        public string Name => "uniq";
        public bool IsReadOnly => true;
        public string Usage => "  uniq [-c] [-D] [-w <chars>]   collapse adjacent duplicate lines (-c prefixes each with its\n" +
                                "                                repeat count; -D prints every line of each run of two or more;\n" +
                                "                                -w compares only the first <chars> characters) - sort first";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            bool withCount = false, all = false;
            int? width = null;
            int textStart = 1;
            while (textStart < args.Length)
            {
                string a = args[textStart];
                if (a == "-c") withCount = true;
                else if (a == "-D") all = true;
                else if (a == "-w")
                {
                    if (textStart + 1 >= args.Length || !int.TryParse(args[textStart + 1], out int n) || n < 0) return DianaShResult.Fail("uniq: -w needs a number of characters\n" + Short);
                    width = n;
                    textStart++;
                }
                else if (a.Length > 2 && a.StartsWith("-w") && int.TryParse(a[2..], out int m) && m >= 0) width = m;
                else break;
                textStart++;
            }
            if (withCount && all) return DianaShResult.Fail("uniq: -c and -D together are meaningless: every line of a run is printed, not counted\n" + Short);

            string? text = stdin ?? (textStart < args.Length ? string.Join(' ', args, textStart, args.Length - textStart) : null);
            if (text == null) return "";

            string Key(string line) => width is int w ? new string(line.EnumerateRunes().Take(w).SelectMany(r => r.ToString()).ToArray()) : line;

            if (all)
            {
                bool ended = text.EndsWith('\n');
                string body = ended ? text[..^1] : text;
                string[] runLines = body.Length == 0 && ended ? new[] { "" } : body.Split('\n');
                var kept = new List<string>();
                int from = 0;
                for (int k = 1; k <= runLines.Length; k++)
                {
                    if (k == runLines.Length || Key(runLines[k]) != Key(runLines[from]))
                    {
                        if (k - from > 1) kept.AddRange(runLines[from..k]);
                        from = k;
                    }
                }
                string joined = string.Join('\n', kept);
                return ended && kept.Count > 0 ? joined + "\n" : joined;
            }

            string[] lines = text.Split('\n');
            var result = new List<string>();
            string? current = null;
            string? currentKey = null;
            int count = 0;

            void Flush()
            {
                if (current == null) return;
                result.Add(withCount ? $"{count} {current}" : current);
            }

            foreach (string line in lines)
            {
                string key = Key(line);
                if (key == currentKey)
                {
                    count++;
                }
                else
                {
                    Flush();
                    current = line;
                    currentKey = key;
                    count = 1;
                }
            }
            Flush();

            return string.Join('\n', result);
        }
    }
}

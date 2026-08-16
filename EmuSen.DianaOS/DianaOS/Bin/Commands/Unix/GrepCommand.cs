using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // A .NET regex, as `sed` chose; -q is what makes it usable as a condition - see §3.17.
    public class GrepCommand : IDianaOSCommand
    {
        public string Name => "grep";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  grep [-i] [-v] [-n] [-c] [-q] <pattern> [text...]",
            "                                filter lines matching <pattern> (regex) - -i case-insensitive,",
            "                                -v invert (non-matching), -n prefix line numbers, -c print only",
            "                                the match count, -q no output (exit code only, for if/while).",
            "                                Reads piped stdin if given, else operates on trailing literal text.",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            bool ignoreCase = false, invert = false, lineNumbers = false, countOnly = false, quiet = false;
            int i = 1;
            for (; i < args.Length; i++)
            {
                switch (args[i])
                {
                    case "-i": ignoreCase = true; continue;
                    case "-v": invert = true; continue;
                    case "-n": lineNumbers = true; continue;
                    case "-c": countOnly = true; continue;
                    case "-q": quiet = true; continue;
                }
                break;
            }

            if (i >= args.Length) return Usage;
            string pattern = args[i];
            i++;

            string? text = stdin ?? (i < args.Length ? string.Join(' ', args, i, args.Length - i) : null);
            if (text == null) return "Nothing to search - pipe a command's output in, or pass literal text: grep <pattern> <text...>";

            var regex = new Regex(pattern, ignoreCase ? RegexOptions.IgnoreCase : RegexOptions.None);
            string[] lines = text.Split('\n');
            var matches = new List<(int LineNumber, string Line)>();
            for (int lineIdx = 0; lineIdx < lines.Length; lineIdx++)
            {
                if (regex.IsMatch(lines[lineIdx]) != invert) matches.Add((lineIdx + 1, lines[lineIdx]));
            }

            // Exit 0 on at least one match, which is what lets -q drive an if or while.
            int exitCode = matches.Count > 0 ? 0 : 1;

            if (quiet) return new DianaOSResult("", exitCode);
            if (countOnly) return new DianaOSResult(matches.Count.ToString(), exitCode);
            return new DianaOSResult(string.Join('\n', matches.Select(m => lineNumbers ? $"{m.LineNumber}:{m.Line}" : m.Line)), exitCode);
        }
    }
}

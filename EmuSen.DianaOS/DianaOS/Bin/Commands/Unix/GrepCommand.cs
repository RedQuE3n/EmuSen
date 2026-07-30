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
    // Unix `grep` - filters lines by a regex pattern. The natural pipeline
    // partner to every multi-line command this shell already has (`regs`,
    // `watch log`, `sprites`, `history`...): `regs | grep PC`, `watch log 1
    // | grep -v poll`. Pattern is a .NET regex, same choice `sed` already
    // made - close enough to POSIX ERE for anything this project's own
    // tooling would realistically need.
    //
    // Deliberately a small subset of real grep: no `-E`/`-F`/`-P` mode
    // switches (pattern is always a regex), no `-A`/`-B`/`-C` context
    // lines, no multi-file support (there's one input stream - stdin, or
    // trailing literal text). `-q` is included specifically because it's
    // what makes `grep` genuinely useful as an `if`/`while` condition here
    // (`if regs | grep -q PC; then ...`), not just a text filter.
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

            // Real grep: exit 0 if at least one match, 1 otherwise -
            // that's what lets `grep -q` drive an if/while condition.
            int exitCode = matches.Count > 0 ? 0 : 1;

            if (quiet) return new DianaOSResult("", exitCode);
            if (countOnly) return new DianaOSResult(matches.Count.ToString(), exitCode);
            return new DianaOSResult(string.Join('\n', matches.Select(m => lineNumbers ? $"{m.LineNumber}:{m.Line}" : m.Line)), exitCode);
        }
    }
}

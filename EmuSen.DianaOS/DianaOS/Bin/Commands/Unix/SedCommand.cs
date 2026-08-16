using System;
using System.Collections.Generic;
using System.Text;
using System.Text.RegularExpressions;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Only s///, applied per line as real sed does - see §3.17.
    public class SedCommand : IDianaOSCommand
    {
        public string Name => "sed";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  sed s/pat/repl/[gi]           substitute (first match per line, or every match with 'g';",
            "                                'i' = case-insensitive). Standalone: sed <expr> <text...>.",
            "                                Real use is as a pipeline stage: <command> | sed s/pat/repl/",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return Usage;

            // Piped input always wins over trailing literal text, as real sed prefers stdin.
            string? text = stdin ?? (args.Length >= 3 ? string.Join(' ', args, 2, args.Length - 2) : null);
            if (text == null)
            {
                return DianaOSResult.Fail("Nothing to operate on - pipe a command's output in, or pass literal text: sed <expr> <text...>");
            }
            return Apply(args[1], text);
        }

        private static string Apply(string expression, string text)
        {
            if (expression.Length < 2 || expression[0] != 's')
            {
                throw new ArgumentException($"Unrecognized sed expression '{expression}' - only s/pattern/replacement/[gi] is supported.");
            }

            char delim = expression[1];
            string[] fields = SplitOnDelimiter(expression.Substring(2), delim);
            if (fields.Length != 3)
            {
                throw new ArgumentException($"Malformed s{delim}pattern{delim}replacement{delim}[flags] expression: '{expression}'.");
            }

            string pattern = fields[0];
            string replacement = TranslateReplacement(fields[1]);
            string flags = fields[2];
            bool global = flags.Contains('g');
            var options = flags.Contains('i') ? RegexOptions.IgnoreCase : RegexOptions.None;

            var regex = new Regex(pattern, options);
            string[] lines = text.Split('\n');
            for (int i = 0; i < lines.Length; i++)
            {
                lines[i] = global ? regex.Replace(lines[i], replacement) : regex.Replace(lines[i], replacement, 1);
            }
            return string.Join('\n', lines);
        }

        // Honours an escaped delimiter, or a pattern matching the delimiter is unparseable.
        private static string[] SplitOnDelimiter(string s, char delim)
        {
            var fields = new List<string>();
            var current = new StringBuilder();
            for (int i = 0; i < s.Length; i++)
            {
                if (s[i] == '\\' && i + 1 < s.Length && s[i + 1] == delim)
                {
                    current.Append(delim);
                    i++;
                }
                else if (s[i] == delim)
                {
                    fields.Add(current.ToString());
                    current.Clear();
                }
                else
                {
                    current.Append(s[i]);
                }
            }
            fields.Add(current.ToString());
            return fields.ToArray();
        }

        // sed's &/\1 syntax is not .NET's $&/$1, so it is translated rather than required.
        private static string TranslateReplacement(string replacement)
        {
            var result = new StringBuilder();
            for (int i = 0; i < replacement.Length; i++)
            {
                if (replacement[i] == '&')
                {
                    result.Append("$&");
                }
                else if (replacement[i] == '\\' && i + 1 < replacement.Length && char.IsDigit(replacement[i + 1]))
                {
                    result.Append('$').Append(replacement[i + 1]);
                    i++;
                }
                else if (replacement[i] == '$')
                {
                    // Escape a literal '$' so .NET does not read it as its own substitution syntax.
                    result.Append("$$");
                }
                else
                {
                    result.Append(replacement[i]);
                }
            }
            return result.ToString();
        }
    }
}

using System;
using System.Text;
using System.Text.RegularExpressions;

namespace EmuSen.Debug.Commands
{
    // A basic `sed`-style substitution filter: `s/pattern/replacement/[gi]`.
    // Real value is as a pipeline stage - `regs | sed s/PC=/pc=/` - see
    // ITextFilterCommand's own comment on why that interface exists.
    // Also callable standalone (`sed <expr> <text...>`) for scripting
    // convenience when there's nothing worth piping from.
    //
    // Deliberately minimal, not a full sed clone: only the `s///` command
    // (no line addressing, no `d`/`p`/hold-space/etc.), and the delimiter
    // can be any character (matching real sed's "the character right after
    // 's' is the delimiter for this invocation" rule - `s#/bin#/usr/bin#`
    // works the same as `s/foo/bar/`), with `\<delim>` inside pattern or
    // replacement meaning a literal delimiter character. Pattern is a
    // .NET regex (close enough to POSIX ERE/sed's own extended-regex mode
    // for anything this project's own tooling would realistically need).
    // Replacement supports `&` (whole match) and `\1`-`\9` (capture
    // groups) the way sed itself does, translated to .NET's `$&`/`$1`
    // syntax under the hood - `\&`/`\\N`-escaped literal backslash-digit
    // sequences aren't specially handled, another place this stays
    // "basic" rather than a full clone.
    //
    // Applied per line, like real sed operating on a multi-line stream -
    // without the `g` flag, only the first match on each line is replaced,
    // not just the first match in the whole piped blob. Matters here
    // because most command output this actually gets used on (`regs`,
    // `snapshot list`, `channels`...) is multi-line.
    public class SedCommand : IDebugCommand, ITextFilterCommand
    {
        public string Name => "sed";
        public string Usage => string.Join('\n', new[]
        {
            "  sed s/pat/repl/[gi]           substitute (first match per line, or every match with 'g';",
            "                                'i' = case-insensitive). Standalone: sed <expr> <text...>.",
            "                                Real use is as a pipeline stage: <command> | sed s/pat/repl/",
        });

        public string Execute(IDebugTarget target, string[] parts)
        {
            if (parts.Length < 2) return Usage;
            if (parts.Length < 3) return "Nothing to operate on - pipe a command's output in, or pass literal text: sed <expr> <text...>";
            string text = string.Join(' ', parts, 2, parts.Length - 2);
            return Apply(parts[1], text);
        }

        public string Filter(IDebugTarget target, string input, string[] parts)
        {
            if (parts.Length < 2) return input;
            return Apply(parts[1], input);
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

        // Splits "pattern<delim>replacement<delim>flags" on the delimiter,
        // honoring "\<delim>" as an escaped literal delimiter rather than a
        // field boundary - without this, a pattern that needs to match the
        // delimiter itself (e.g. `s/\//_/` to replace a literal slash)
        // would be unparseable.
        private static string[] SplitOnDelimiter(string s, char delim)
        {
            var fields = new System.Collections.Generic.List<string>();
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

        // sed's replacement syntax (`&` = whole match, `\1`-`\9` = capture
        // groups) isn't .NET Regex.Replace's own (`$&`, `$1`) - translated
        // here rather than asking callers to already know .NET's dialect.
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
                    // Escape a literal '$' so .NET doesn't mistake it for
                    // its own substitution syntax.
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

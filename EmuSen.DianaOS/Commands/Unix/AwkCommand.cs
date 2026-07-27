using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;

namespace EmuSen.DianaOS.Commands.Unix
{
    // A small AWK subset - real AWK is a whole language (arithmetic,
    // user functions, associative arrays, printf...), and this shell's
    // own header comment already draws the line at "no arithmetic
    // expansion" for itself; this command draws the same line rather
    // than trying to embed a second, more capable expression language
    // just for one command. What IS supported, because it covers the
    // overwhelming majority of real `awk` one-liners people actually
    // reach for on command output (`regs | awk '{print $1}'`,
    // `mem dump | awk -F, 'NR>1{print $2}'`):
    //   - patterns: empty (always), `/regex/`, `NR<op><int>`
    //     (==, !=, <, <=, >, >=), and `BEGIN`/`END`.
    //   - actions: `{ print expr[, expr...][; print ...] }`; a bare
    //     pattern with no `{...}` defaults to `{print $0}`, matching
    //     real awk. `{}` (an explicit, empty action) does nothing,
    //     also matching real awk.
    //   - expressions: `$0`, `$N`, `$NF`, `NR`, `NF`, `"string literal"`,
    //     bare numeric literals. No concatenation, no arithmetic, no
    //     variables beyond NR/NF.
    // Field splitting: default is runs of whitespace (leading/trailing
    // trimmed first, same as real awk's default FS); `-F sep` splits on
    // a literal separator string instead (not a regex, unlike real
    // awk's non-single-char `-F` - a documented simplification), with
    // `\t`/`\n` unescaped in the -F argument itself since real awk does
    // the same escape processing there. A trailing file path argument is
    // walled to DianaOSSandbox.RootDirectory, same as every other real-file
    // command in this shell (see that file's own comment) - piped stdin
    // is unaffected either way, since that never touches the filesystem.
    public class AwkCommand : IDianaOSCommand
    {
        public string Name => "awk";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  awk [-F sep] 'prog' [path]   run a small AWK subset over stdin or <path>, line by line",
            "                                patterns: /regex/, NR==N (also !=,<,<=,>,>=), BEGIN, END",
            "                                actions: {print $0|$N|$NF|NR|NF|\"lit\"[, ...][; print ...]}",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            string? sep = null;
            string? program = null;
            string? path = null;
            for (int i = 1; i < args.Length; i++)
            {
                if (args[i] == "-F" && i + 1 < args.Length) { sep = Unescape(args[++i]); continue; }
                if (args[i].StartsWith("-F", StringComparison.Ordinal) && args[i].Length > 2) { sep = Unescape(args[i][2..]); continue; }
                if (program is null) program = args[i];
                else path = args[i];
            }
            if (program is null) return DianaOSResult.Fail("awk: usage: awk [-F sep] 'program' [path]");

            List<AwkRule> rules;
            try { rules = ParseProgram(program); }
            catch (FormatException ex) { return DianaOSResult.Fail($"awk: {ex.Message}"); }

            if (stdin is null && path != null && !DianaOSSandbox.TryResolve(path, out path))
            {
                return DianaOSResult.Fail($"awk: '{args[^1]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            string text = stdin ?? (path != null ? File.ReadAllText(path) : "");
            string[] lines = text.Length == 0 ? Array.Empty<string>() : text.Split('\n');
            if (lines.Length > 0 && lines[^1] == "") lines = lines[..^1];

            var output = new List<string>();
            void RunAction(AwkRule rule, string line, string[] fields, int nr)
            {
                if (rule.Statements is null) { output.Add(line); return; } // bare pattern, no {...} at all -> default print $0
                foreach (List<Func<string, string[], int, string>> stmt in rule.Statements)
                {
                    output.Add(stmt.Count == 0 ? line : string.Join(' ', stmt.Select(f => f(line, fields, nr))));
                }
            }

            foreach (AwkRule rule in rules.Where(r => r.IsBegin)) RunAction(rule, "", Array.Empty<string>(), 0);

            int recordNumber = 0;
            foreach (string line in lines)
            {
                recordNumber++;
                string[] fields = SplitFields(line, sep);
                foreach (AwkRule rule in rules.Where(r => !r.IsBegin && !r.IsEnd))
                {
                    if (rule.Predicate is null || rule.Predicate(line, recordNumber)) RunAction(rule, line, fields, recordNumber);
                }
            }

            string lastLine = lines.Length > 0 ? lines[^1] : "";
            foreach (AwkRule rule in rules.Where(r => r.IsEnd)) RunAction(rule, lastLine, SplitFields(lastLine, sep), recordNumber);

            return string.Join('\n', output);
        }

        private static string[] SplitFields(string line, string? sep)
        {
            if (sep is null)
            {
                string trimmed = line.Trim();
                return trimmed.Length == 0 ? Array.Empty<string>() : Regex.Split(trimmed, "\\s+");
            }
            return line.Split(sep);
        }

        private static string Unescape(string s) => s.Replace("\\t", "\t").Replace("\\n", "\n");

        private sealed class AwkRule
        {
            public bool IsBegin;
            public bool IsEnd;
            public Func<string, int, bool>? Predicate; // null (for an ordinary rule) = matches every line
            public List<List<Func<string, string[], int, string>>>? Statements; // null = no {...} was given at all -> default print $0
        }

        // Rules are separated by ';'/newline like this shell's own
        // top-level command sequencing, but - matching real awk - a
        // rule's own closing '}' also ends it on its own, with no
        // separator required before the next rule's pattern: the common
        // `BEGIN{...} {...} END{...}` idiom (all on one line, space-
        // separated only) has to parse as three rules, not one. So this
        // scans forward looking for whichever comes first, a top-level
        // '{' (this rule has an action - the closing brace ends the
        // rule right there) or a top-level ';'/'\n' (this rule has no
        // action - defaults to `print $0`, same as a bare pattern
        // anywhere else).
        private static List<AwkRule> ParseProgram(string program)
        {
            var rules = new List<AwkRule>();
            int pos = 0;
            int len = program.Length;

            while (true)
            {
                while (pos < len && (char.IsWhiteSpace(program[pos]) || program[pos] == ';')) pos++;
                if (pos >= len) break;

                int scan = pos;
                bool inQuote = false;
                int braceIdx = -1, sepIdx = -1;
                while (scan < len)
                {
                    char c = program[scan];
                    if (inQuote)
                    {
                        if (c == '\\') { scan += 2; continue; }
                        if (c == '"') inQuote = false;
                        scan++;
                        continue;
                    }
                    if (c == '"') { inQuote = true; scan++; continue; }
                    if (c == '{') { braceIdx = scan; break; }
                    if (c == ';' || c == '\n') { sepIdx = scan; break; }
                    scan++;
                }

                string patternText;
                List<List<Func<string, string[], int, string>>>? statements = null;

                if (braceIdx >= 0)
                {
                    patternText = program[pos..braceIdx].Trim();
                    int closeIdx = FindMatchingBrace(program, braceIdx);
                    if (closeIdx < 0) throw new FormatException($"unterminated '{{' in: {program[pos..]}");
                    string actionBody = program[(braceIdx + 1)..closeIdx].Trim();
                    statements = ParseStatements(actionBody);
                    pos = closeIdx + 1;
                }
                else if (sepIdx >= 0)
                {
                    patternText = program[pos..sepIdx].Trim();
                    pos = sepIdx + 1;
                }
                else
                {
                    patternText = program[pos..].Trim();
                    pos = len;
                }

                if (patternText.Length == 0 && statements is null) continue; // stray separator between real rules

                var rule = new AwkRule { Statements = statements };
                if (patternText == "BEGIN") rule.IsBegin = true;
                else if (patternText == "END") rule.IsEnd = true;
                else if (patternText.Length > 0) rule.Predicate = ParsePattern(patternText);
                rules.Add(rule);
            }

            if (rules.Count == 0) throw new FormatException("empty program");
            return rules;
        }

        private static List<List<Func<string, string[], int, string>>> ParseStatements(string actionBody)
        {
            var statements = new List<List<Func<string, string[], int, string>>>();
            foreach (string raw in SplitTopLevel(actionBody, new[] { ';' }))
            {
                string stmt = raw.Trim();
                if (stmt.Length == 0) continue;

                if (stmt != "print" && !(stmt.StartsWith("print", StringComparison.Ordinal) && char.IsWhiteSpace(stmt[5])))
                {
                    throw new FormatException($"unsupported awk statement: '{stmt}' (only 'print' is supported)");
                }

                string argsText = stmt.Length > 5 ? stmt[5..].Trim() : "";
                List<Func<string, string[], int, string>> exprs = argsText.Length == 0
                    ? new List<Func<string, string[], int, string>>()
                    : SplitTopLevel(argsText, new[] { ',' }).Select(e => ParseExpr(e.Trim())).ToList();

                statements.Add(exprs);
            }
            return statements;
        }

        private static Func<string, int, bool> ParsePattern(string patternText)
        {
            if (patternText.Length >= 2 && patternText[0] == '/' && patternText[^1] == '/')
            {
                var re = new Regex(patternText[1..^1]);
                return (line, nr) => re.IsMatch(line);
            }

            Match m = Regex.Match(patternText, @"^NR\s*(==|!=|>=|<=|>|<)\s*(\d+)$");
            if (m.Success)
            {
                string op = m.Groups[1].Value;
                int value = int.Parse(m.Groups[2].Value);
                return (line, nr) => op switch
                {
                    "==" => nr == value,
                    "!=" => nr != value,
                    ">=" => nr >= value,
                    "<=" => nr <= value,
                    ">" => nr > value,
                    "<" => nr < value,
                    _ => false,
                };
            }

            throw new FormatException($"unsupported awk pattern: '{patternText}' (only /regex/ and NR<op><int> are supported)");
        }

        private static Func<string, string[], int, string> ParseExpr(string text)
        {
            if (text == "$0") return (line, fields, nr) => line;
            if (text == "$NF") return (line, fields, nr) => fields.Length > 0 ? fields[^1] : "";
            if (text == "NR") return (line, fields, nr) => nr.ToString();
            if (text == "NF") return (line, fields, nr) => fields.Length.ToString();

            Match fm = Regex.Match(text, @"^\$(\d+)$");
            if (fm.Success)
            {
                int n = int.Parse(fm.Groups[1].Value);
                return (line, fields, nr) => n >= 1 && n <= fields.Length ? fields[n - 1] : "";
            }

            if (text.Length >= 2 && text[0] == '"' && text[^1] == '"')
            {
                string literal = Unescape(text[1..^1]);
                return (_, _, _) => literal;
            }

            if (double.TryParse(text, out _))
            {
                return (_, _, _) => text;
            }

            throw new FormatException($"unsupported awk expression: '{text}'");
        }

        // Splits on the given separator characters, but only at brace
        // depth 0 and outside a double-quoted string - shared by rule
        // splitting (on ';'/'\n'), statement splitting (on ';'), and
        // print-argument splitting (on ',') since all three need the
        // same "don't split inside {...} or "..."" behavior.
        private static List<string> SplitTopLevel(string text, char[] seps)
        {
            var parts = new List<string>();
            var current = new StringBuilder();
            int depth = 0;
            bool inQuote = false;

            for (int i = 0; i < text.Length; i++)
            {
                char c = text[i];
                if (inQuote)
                {
                    current.Append(c);
                    if (c == '\\' && i + 1 < text.Length) { current.Append(text[++i]); continue; }
                    if (c == '"') inQuote = false;
                    continue;
                }

                if (c == '"') { inQuote = true; current.Append(c); continue; }
                if (c == '{') { depth++; current.Append(c); continue; }
                if (c == '}') { depth--; current.Append(c); continue; }

                if (depth == 0 && Array.IndexOf(seps, c) >= 0)
                {
                    parts.Add(current.ToString());
                    current.Clear();
                    continue;
                }

                current.Append(c);
            }
            parts.Add(current.ToString());
            return parts;
        }

        private static int FindMatchingBrace(string text, int openIdx)
        {
            int depth = 0;
            bool inQuote = false;
            for (int i = openIdx; i < text.Length; i++)
            {
                char c = text[i];
                if (inQuote)
                {
                    if (c == '\\') { i++; continue; }
                    if (c == '"') inQuote = false;
                    continue;
                }
                if (c == '"') { inQuote = true; continue; }
                if (c == '{') depth++;
                else if (c == '}') { depth--; if (depth == 0) return i; }
            }
            return -1;
        }
    }
}

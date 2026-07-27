using System;
using System.IO;

namespace EmuSen.DianaOS.Commands.Unix
{
    // Unix `test` (and its `[ ... ]` alias) - the main way an `if`/`while`
    // condition does anything besides check another command's own exit
    // code. Deliberately a small subset of real test(1): one value (true
    // if non-empty), a unary operator + value (-z/-n for string
    // emptiness, -f/-d for real file/directory existence - genuinely
    // useful here since redirection already reads/writes real host
    // files), or value OP value (3 tokens: string = / != , or numeric
    // -eq/-ne/-lt/-le/-gt/-ge). No `-a`/`-o`/parenthesized compound
    // expressions - combine conditions with the shell's own `&&`/`||`
    // instead (`[ -f a ] && [ -f b ]`), which is both simpler to implement
    // correctly and arguably clearer to read.
    public static class TestEvaluator
    {
        public static bool Evaluate(string[] expr)
        {
            // '!' negates whatever follows, regardless of its own arity
            // (`! -f missing`, `! a = b`, `! ""` all work) - checked before
            // the length-based dispatch below rather than as its own
            // 2-token case, so it composes with every other form instead
            // of only the single-value one.
            if (expr.Length > 0 && expr[0] == "!") return !Evaluate(expr[1..]);

            switch (expr.Length)
            {
                case 0:
                    return false;
                case 1:
                    return expr[0].Length > 0;
                case 2:
                    return expr[0] switch
                    {
                        "-z" => expr[1].Length == 0,
                        "-n" => expr[1].Length > 0,
                        "-f" => File.Exists(expr[1]),
                        "-d" => Directory.Exists(expr[1]),
                        _ => throw new ArgumentException($"Unknown unary test operator '{expr[0]}'."),
                    };
                case 3:
                    string lhs = expr[0], op = expr[1], rhs = expr[2];
                    return op switch
                    {
                        "=" or "==" => lhs == rhs,
                        "!=" => lhs != rhs,
                        "-eq" => ParseInt(lhs) == ParseInt(rhs),
                        "-ne" => ParseInt(lhs) != ParseInt(rhs),
                        "-lt" => ParseInt(lhs) < ParseInt(rhs),
                        "-le" => ParseInt(lhs) <= ParseInt(rhs),
                        "-gt" => ParseInt(lhs) > ParseInt(rhs),
                        "-ge" => ParseInt(lhs) >= ParseInt(rhs),
                        _ => throw new ArgumentException($"Unknown binary test operator '{op}'."),
                    };
                default:
                    throw new ArgumentException("test/[ only supports: VALUE, OP VALUE, or VALUE OP VALUE - combine multiple conditions with the shell's own && / || instead.");
            }
        }

        private static long ParseInt(string s)
        {
            if (!long.TryParse(s, out long v)) throw new ArgumentException($"'{s}' isn't a number (needed for a -eq/-ne/-lt/-le/-gt/-ge comparison).");
            return v;
        }
    }

    public class TestCommand : IDianaOSCommand
    {
        public string Name => "test";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  test EXPR / [ EXPR ]         VALUE (true if non-empty) | -z/-n VALUE | -f/-d PATH | !",
            "                                VALUE = / != / -eq / -ne / -lt / -le / -gt / -ge VALUE",
            "                                exit code only (0 = true) - meant for if/while, not for its own output",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            bool result = TestEvaluator.Evaluate(args[1..]);
            return result ? DianaOSResult.Ok("") : DianaOSResult.Fail("");
        }
    }

    // Same evaluator as TestCommand, just spelled `[ EXPR ]` - the
    // trailing ']' is required and stripped, matching real bash's own
    // `[` builtin (which really is just a differently-named `test`).
    public class BracketCommand : IDianaOSCommand
    {
        public string Name => "[";
        public bool IsReadOnly => true;
        public string Usage => "  [ EXPR ]                      alias for 'test EXPR' - the ']' is required";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2 || args[^1] != "]")
            {
                return DianaOSResult.Fail("Error: '[' must end with a matching ']'.");
            }
            bool result = TestEvaluator.Evaluate(args[1..^1]);
            return result ? DianaOSResult.Ok("") : DianaOSResult.Fail("");
        }
    }
}

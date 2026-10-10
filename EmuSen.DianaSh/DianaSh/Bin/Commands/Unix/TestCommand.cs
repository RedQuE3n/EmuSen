using System;
using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // A small subset; compound conditions use the shell's own && instead - see §3.17.
    public static class TestEvaluator
    {
        public static bool Evaluate(string[] expr)
        {
            // '!' is checked before the arity dispatch, so it composes with every form.
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
                        "-f" => DianaShSandbox.TryResolve(expr[1], out string file) && File.Exists(file),
                        "-d" => DianaShSandbox.TryResolve(expr[1], out string dir) && Directory.Exists(dir),
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

    public class TestCommand : IDianaShCommand
    {
        public string Name => "test";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  test EXPR / [ EXPR ]         VALUE (true if non-empty) | -z/-n VALUE | -f/-d PATH | !",
            "                                VALUE = / != / -eq / -ne / -lt / -le / -gt / -ge VALUE",
            "                                exit code only (0 = true) - meant for if/while, not for its own output",
        });

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            bool result = TestEvaluator.Evaluate(args[1..]);
            return result ? DianaShResult.Ok("") : DianaShResult.Fail("");
        }
    }

    // The same evaluator spelled `[ EXPR ]`, with the trailing ']' required.
    public class BracketCommand : IDianaShCommand
    {
        public string Name => "[";
        public bool IsReadOnly => true;
        public string Usage => "  [ EXPR ]                      alias for 'test EXPR' - the ']' is required";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2 || args[^1] != "]")
            {
                return DianaShResult.Fail("Error: '[' must end with a matching ']'.");
            }
            bool result = TestEvaluator.Evaluate(args[1..^1]);
            return result ? DianaShResult.Ok("") : DianaShResult.Fail("");
        }
    }
}

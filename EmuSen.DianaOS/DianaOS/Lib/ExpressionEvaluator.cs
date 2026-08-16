using System;
using System.Collections.Generic;
using System.Globalization;
using System.Text;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // What an expression can reach - see `man eval`.
    public interface IExpressionContext
    {
        bool TryGetSymbol(string name, out long value);

        // <space> is null for "whatever this core calls its main CPU bus".
        bool TryReadMemory(string? space, int address, int width, out long value);

        // Every symbol this context can resolve, for `eval symbols`.
        IReadOnlyList<string> SymbolNames { get; }
    }

    // A bad expression is user input, not a bug - commands report Message.
    public sealed class ExpressionException : Exception
    {
        public ExpressionException(string message) : base(message) { }
    }

    // The core-agnostic half of `eval` and conditional breakpoints - see `man eval`.
    public sealed class ExpressionEvaluator
    {
        private enum TokenKind { Number, Identifier, Operator, End }

        private readonly struct Token
        {
            public readonly TokenKind Kind;
            public readonly string Text;
            public readonly long Value;

            public Token(TokenKind kind, string text, long value = 0)
            {
                Kind = kind;
                Text = text;
                Value = value;
            }
        }

        private readonly IExpressionContext _context;
        private List<Token> _tokens = new();
        private int _at;

        public ExpressionEvaluator(IExpressionContext context) => _context = context;

        public long Evaluate(string expression)
        {
            _tokens = Tokenize(expression);
            _at = 0;
            long value = ParseBinary(0, live: true);
            if (Current.Kind != TokenKind.End) throw new ExpressionException($"Unexpected '{Current.Text}' in expression.");
            return value;
        }

        // Anything nonzero is true - see `man eval`.
        public bool EvaluateBool(string expression) => Evaluate(expression) != 0;

        // Never throws: a bad condition must not take the instruction loop down.
        public bool TryEvaluateBool(string expression, out bool result, out string error)
        {
            try
            {
                result = EvaluateBool(expression);
                error = string.Empty;
                return true;
            }
            catch (Exception ex)
            {
                result = false;
                error = ex.Message;
                return false;
            }
        }

        private Token Current => _at < _tokens.Count ? _tokens[_at] : new Token(TokenKind.End, "<end>");

        private static List<Token> Tokenize(string source)
        {
            var tokens = new List<Token>();
            int i = 0;
            while (i < source.Length)
            {
                char c = source[i];
                if (char.IsWhiteSpace(c)) { i++; continue; }

                // '%' is a binary prefix only before a binary digit; elsewhere it is modulo - see `man eval`.
                bool binaryLiteral = c == '%' && i + 1 < source.Length && (source[i + 1] == '0' || source[i + 1] == '1');
                if (c == '$' || binaryLiteral || char.IsDigit(c))
                {
                    tokens.Add(ReadNumber(source, ref i));
                    continue;
                }

                if (char.IsLetter(c) || c == '_' || c == '.')
                {
                    int start = i;
                    while (i < source.Length && (char.IsLetterOrDigit(source[i]) || source[i] == '_' || source[i] == '.')) i++;
                    tokens.Add(new Token(TokenKind.Identifier, source.Substring(start, i - start)));
                    continue;
                }

                // Two-character operators first, so '<' never shadows '<<'/'<='.
                if (i + 1 < source.Length)
                {
                    string pair = source.Substring(i, 2);
                    if (pair is "==" or "!=" or "<=" or ">=" or "&&" or "||" or "<<" or ">>")
                    {
                        tokens.Add(new Token(TokenKind.Operator, pair));
                        i += 2;
                        continue;
                    }
                }

                if ("+-*/%&|^~!<>()[]{}:".IndexOf(c) >= 0)
                {
                    tokens.Add(new Token(TokenKind.Operator, c.ToString()));
                    i++;
                    continue;
                }

                throw new ExpressionException($"Unexpected character '{c}' in expression.");
            }
            tokens.Add(new Token(TokenKind.End, "<end>"));
            return tokens;
        }

        private static Token ReadNumber(string source, ref int i)
        {
            if (source[i] == '$')
            {
                i++;
                return new Token(TokenKind.Number, "$", ReadDigits(source, ref i, 16));
            }
            if (source[i] == '%')
            {
                i++;
                return new Token(TokenKind.Number, "%", ReadDigits(source, ref i, 2));
            }
            if (source[i] == '0' && i + 1 < source.Length && (source[i + 1] == 'x' || source[i + 1] == 'X'))
            {
                i += 2;
                return new Token(TokenKind.Number, "0x", ReadDigits(source, ref i, 16));
            }
            return new Token(TokenKind.Number, "d", ReadDigits(source, ref i, 10));
        }

        private static long ReadDigits(string source, ref int i, int radix)
        {
            int start = i;
            while (i < source.Length && IsDigitIn(source[i], radix)) i++;
            if (i == start) throw new ExpressionException("Expected a number.");
            string text = source.Substring(start, i - start);
            try { return Convert.ToInt64(text, radix); }
            catch (Exception) { throw new ExpressionException($"'{text}' is not a valid base-{radix} number."); }
        }

        private static bool IsDigitIn(char c, int radix) => radix switch
        {
            2 => c == '0' || c == '1',
            10 => char.IsDigit(c),
            _ => Uri.IsHexDigit(c),
        };

        // Precedence-climbing, lowest level first - see `man eval`.
        private static readonly string[][] Levels =
        {
            new[] { "||" },
            new[] { "&&" },
            new[] { "|" },
            new[] { "^" },
            new[] { "&" },
            new[] { "==", "!=" },
            new[] { "<", ">", "<=", ">=" },
            new[] { "<<", ">>" },
            new[] { "+", "-" },
            new[] { "*", "/", "%" },
        };

        // live=false parses without evaluating, which is how short-circuiting skips a branch.
        private long ParseBinary(int level, bool live)
        {
            if (level >= Levels.Length) return ParseUnary(live);

            long left = ParseBinary(level + 1, live);
            while (Current.Kind == TokenKind.Operator && Array.IndexOf(Levels[level], Current.Text) >= 0)
            {
                string op = Current.Text;
                _at++;

                if (live && op == "&&" && left == 0) { ParseBinary(level + 1, live: false); left = 0; continue; }
                if (live && op == "||" && left != 0) { ParseBinary(level + 1, live: false); left = 1; continue; }

                long right = ParseBinary(level + 1, live);
                left = live ? Apply(op, left, right) : 0;
            }
            return left;
        }

        private static long Apply(string op, long a, long b) => op switch
        {
            "||" => (a != 0 || b != 0) ? 1 : 0,
            "&&" => (a != 0 && b != 0) ? 1 : 0,
            "|" => a | b,
            "^" => a ^ b,
            "&" => a & b,
            "==" => a == b ? 1 : 0,
            "!=" => a != b ? 1 : 0,
            "<" => a < b ? 1 : 0,
            ">" => a > b ? 1 : 0,
            "<=" => a <= b ? 1 : 0,
            ">=" => a >= b ? 1 : 0,
            "<<" => a << (int)(b & 63),
            ">>" => a >> (int)(b & 63),
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => b == 0 ? throw new ExpressionException("Division by zero.") : a / b,
            "%" => b == 0 ? throw new ExpressionException("Division by zero.") : a % b,
            _ => throw new ExpressionException($"Unknown operator '{op}'."),
        };

        private long ParseUnary(bool live)
        {
            if (Current.Kind == TokenKind.Operator)
            {
                string op = Current.Text;
                if (op == "-") { _at++; return -ParseUnary(live); }
                if (op == "!") { _at++; return ParseUnary(live) == 0 ? 1 : 0; }
                if (op == "~") { _at++; return ~ParseUnary(live); }
                if (op == "+") { _at++; return ParseUnary(live); }
            }
            return ParsePrimary(live);
        }

        private long ParsePrimary(bool live)
        {
            Token token = Current;

            if (token.Kind == TokenKind.Number) { _at++; return token.Value; }

            if (token.Kind == TokenKind.Identifier)
            {
                _at++;
                if (!_context.TryGetSymbol(token.Text, out long value))
                {
                    if (!live) return 0;
                    throw new ExpressionException($"Unknown symbol '{token.Text}'. Try `eval symbols` for the full list.");
                }
                return value;
            }

            if (token.Kind == TokenKind.Operator)
            {
                switch (token.Text)
                {
                    case "(":
                    {
                        _at++;
                        long value = ParseBinary(0, live);
                        Expect(")");
                        return value;
                    }
                    // [addr] is a byte, {addr} a little-endian word - see `man eval`.
                    case "[": return ParseMemoryRead("]", width: 1, live);
                    case "{": return ParseMemoryRead("}", width: 2, live);
                }
            }

            throw new ExpressionException($"Unexpected '{token.Text}' in expression.");
        }

        private long ParseMemoryRead(string closer, int width, bool live)
        {
            _at++;

            // An identifier followed by ':' names the space explicitly.
            string? space = null;
            if (Current.Kind == TokenKind.Identifier && _at + 1 < _tokens.Count && _tokens[_at + 1].Text == ":")
            {
                space = Current.Text;
                _at += 2;
            }

            long address = ParseBinary(0, live);
            Expect(closer);
            if (!live) return 0;

            if (!_context.TryReadMemory(space, (int)address, width, out long value))
            {
                string where = space == null ? $"0x{address:X}" : $"{space} 0x{address:X}";
                throw new ExpressionException($"Cannot read {where}.");
            }
            return value;
        }

        private void Expect(string text)
        {
            if (Current.Kind != TokenKind.Operator || Current.Text != text)
            {
                throw new ExpressionException($"Expected '{text}' but found '{Current.Text}'.");
            }
            _at++;
        }

        // Decimal, hex and binary at once - see `man eval`.
        public static string Format(long value)
        {
            var binary = new StringBuilder();
            long magnitude = value < 0 ? -value : value;
            int bits = magnitude > 0xFFFF ? 24 : magnitude > 0xFF ? 16 : 8;
            for (int bit = bits - 1; bit >= 0; bit--)
            {
                binary.Append(((magnitude >> bit) & 1) != 0 ? '1' : '0');
                if (bit % 4 == 0 && bit != 0) binary.Append(' ');
            }
            string hex = value < 0 ? $"-${magnitude.ToString("X", CultureInfo.InvariantCulture)}" : $"${value.ToString("X", CultureInfo.InvariantCulture)}";
            return $"{value.ToString(CultureInfo.InvariantCulture)}  {hex}  %{binary}";
        }
    }
}

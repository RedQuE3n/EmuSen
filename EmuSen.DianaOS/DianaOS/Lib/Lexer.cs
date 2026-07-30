using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.DianaOS.Ast;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    public enum TokenType { Word, Pipe, Semicolon, Newline, AndAnd, OrOr, Greater, DGreater, Less, EOF }

    public class Token
    {
        public TokenType Type;
        public Word? Word;
        public Token(TokenType type, Word? word = null) { Type = type; Word = word; }
    }

    public class LexResult
    {
        public List<Token> Tokens = new();

        // True when the input ends mid-quote or mid-"$(...)" - the caller
        // (DianaOSInterpreter.Submit) should hold the raw text and wait for
        // another line rather than trying to parse an incomplete token
        // stream, the same way bash's own interactive prompt shows a
        // secondary "> " prompt until a quote/substitution actually closes.
        public bool NeedsMoreInput;
    }

    // Turns raw shell text into a token stream. Understands bash's own
    // quoting rules (single quotes: fully literal; double quotes: still
    // expand $VAR/$(...) but suppress word-splitting later; unquoted:
    // expand AND word-split) well enough to tag every character correctly
    // for DianaOSInterpreter.ExpandWord to act on afterward - the lexer
    // itself never expands anything, it just records what's quoted and
    // what's a variable/substitution reference versus literal text.
    //
    // Deliberately doesn't implement: heredocs (`<<`), backtick command
    // substitution (only `$(...)`, the modern/preferred form), brace
    // expansion (`{a,b}`), tilde expansion, arithmetic expansion
    // (`$((...))` - a literal `$((` lexes as `$(` immediately followed by
    // a nested, mostly-inert `(...)`, which will not do what a bash user
    // expects), or background jobs (`&`) - a bare unpaired `&` throws a
    // clear "not supported" error rather than silently misbehaving.
    public class Lexer
    {
        private readonly string _src;
        private int _pos;

        // Tracks whether we're inside "..." at the top level of quote
        // nesting - single quotes and "$(...)" have their own fully
        // self-contained scanning (see ReadSingleQuoted/ReadCommandSub)
        // and don't need this, but "$VAR" inside "..." needs to know it's
        // still "quoted" for word-splitting purposes even though `$` also
        // triggers expansion.
        private bool _inDoubleQuote;

        private readonly List<Token> _tokens = new();
        private List<WordPart>? _currentWordParts;
        private readonly StringBuilder _literalBuf = new();
        private bool _literalBufQuoted;

        private Lexer(string src) { _src = src; }

        public static LexResult Tokenize(string src)
        {
            var lexer = new Lexer(src);
            return lexer.Run();
        }

        private char Cur => _pos < _src.Length ? _src[_pos] : '\0';
        private char Peek(int ahead = 1) => _pos + ahead < _src.Length ? _src[_pos + ahead] : '\0';
        private bool AtEnd => _pos >= _src.Length;

        private LexResult Run()
        {
            bool atWordStart = true; // true at start of input, after whitespace, or right after an operator - governs '#' comments

            while (!AtEnd)
            {
                char c = Cur;

                if (c == '#' && atWordStart && _currentWordParts == null)
                {
                    while (!AtEnd && Cur != '\n') _pos++;
                    continue;
                }

                if (c == ' ' || c == '\t')
                {
                    EndWord();
                    _pos++;
                    atWordStart = true;
                    continue;
                }

                if (c == '\n')
                {
                    EndWord();
                    _tokens.Add(new Token(TokenType.Newline));
                    _pos++;
                    atWordStart = true;
                    continue;
                }

                if (c == '\'')
                {
                    if (!ReadSingleQuoted()) return Incomplete();
                    atWordStart = false;
                    continue;
                }

                if (c == '"')
                {
                    if (!ReadDoubleQuoted()) return Incomplete();
                    atWordStart = false;
                    continue;
                }

                if (c == '\\')
                {
                    _pos++;
                    if (AtEnd) return Incomplete(); // trailing backslash - a continuation onto the next line in real bash
                    AppendLiteralChar(Cur, quoted: true);
                    _pos++;
                    atWordStart = false;
                    continue;
                }

                if (c == '$')
                {
                    if (!ReadDollar(_inDoubleQuote)) return Incomplete();
                    atWordStart = false;
                    continue;
                }

                if (!_inDoubleQuote && (c == '|' || c == ';' || c == '&' || c == '>' || c == '<'))
                {
                    EndWord();
                    EmitOperator(c);
                    atWordStart = true;
                    continue;
                }

                AppendLiteralChar(c, quoted: _inDoubleQuote);
                _pos++;
                atWordStart = false;
            }

            EndWord();
            _tokens.Add(new Token(TokenType.EOF));
            return new LexResult { Tokens = _tokens, NeedsMoreInput = false };
        }

        private LexResult Incomplete() => new LexResult { NeedsMoreInput = true };

        private void EmitOperator(char c)
        {
            switch (c)
            {
                case '|':
                    _pos++;
                    if (Cur == '|') { _pos++; _tokens.Add(new Token(TokenType.OrOr)); }
                    else _tokens.Add(new Token(TokenType.Pipe));
                    break;
                case ';':
                    _pos++;
                    _tokens.Add(new Token(TokenType.Semicolon));
                    break;
                case '&':
                    _pos++;
                    if (Cur == '&') { _pos++; _tokens.Add(new Token(TokenType.AndAnd)); }
                    else throw new NotSupportedException("Background jobs ('&') aren't supported - this shell runs everything synchronously. Did you mean '&&'?");
                    break;
                case '>':
                    _pos++;
                    if (Cur == '>') { _pos++; _tokens.Add(new Token(TokenType.DGreater)); }
                    else _tokens.Add(new Token(TokenType.Greater));
                    break;
                case '<':
                    _pos++;
                    if (Cur == '<') throw new NotSupportedException("Heredocs ('<<') aren't supported - use '<' with a real file, or a variable, instead.");
                    _tokens.Add(new Token(TokenType.Less));
                    break;
            }
        }

        private bool ReadSingleQuoted()
        {
            _pos++; // opening '
            EnsureWordExists();
            int start = _pos;
            while (Cur != '\'')
            {
                if (AtEnd) { _pos = start - 1; return false; }
                _pos++;
            }
            AppendLiteral(_src.Substring(start, _pos - start), quoted: true);
            _pos++; // closing '
            return true;
        }

        private bool ReadDoubleQuoted()
        {
            _pos++; // opening "
            EnsureWordExists();
            bool wasInDoubleQuote = _inDoubleQuote;
            _inDoubleQuote = true;

            while (Cur != '"')
            {
                if (AtEnd) { _inDoubleQuote = wasInDoubleQuote; return false; }

                if (Cur == '\\' && (Peek() == '"' || Peek() == '\\' || Peek() == '$'))
                {
                    AppendLiteralChar(Peek(), quoted: true);
                    _pos += 2;
                    continue;
                }

                if (Cur == '$')
                {
                    if (!ReadDollar(quoted: true)) { _inDoubleQuote = wasInDoubleQuote; return false; }
                    continue;
                }

                AppendLiteralChar(Cur, quoted: true);
                _pos++;
            }

            _pos++; // closing "
            _inDoubleQuote = wasInDoubleQuote;
            return true;
        }

        // Handles "$NAME", "${NAME}", "$?", and "$(...)" - `quoted` is
        // whether this reference sits inside "..." (still expanded, just
        // not later word-split).
        private bool ReadDollar(bool quoted)
        {
            int dollarPos = _pos;
            _pos++; // '$'

            if (Cur == '(')
            {
                _pos++;
                int start = _pos;
                int depth = 1;
                while (depth > 0)
                {
                    if (AtEnd) { _pos = dollarPos; return false; }
                    char c = Cur;
                    if (c == '\'')
                    {
                        _pos++;
                        while (Cur != '\'') { if (AtEnd) { _pos = dollarPos; return false; } _pos++; }
                    }
                    else if (c == '"')
                    {
                        _pos++;
                        while (Cur != '"')
                        {
                            if (AtEnd) { _pos = dollarPos; return false; }
                            if (Cur == '\\') _pos++;
                            _pos++;
                        }
                    }
                    else if (c == '(') depth++;
                    else if (c == ')') depth--;
                    if (depth > 0) _pos++;
                }
                string inner = _src.Substring(start, _pos - start);
                _pos++; // closing )
                // Flush whatever plain text was buffered so far (e.g. the
                // "Y=" in "Y=$(echo x)") BEFORE appending this part
                // directly - otherwise it would land in _currentWordParts
                // ahead of text that's actually supposed to precede it,
                // once EndWord() finally flushes the buffer afterward.
                EnsureWordExists();
                FlushLiteralBuf();
                _currentWordParts!.Add(new WordPart { Kind = PartKind.CommandSubstitution, Text = inner, Quoted = quoted });
                return true;
            }

            if (Cur == '{')
            {
                _pos++;
                int start = _pos;
                while (Cur != '}')
                {
                    if (AtEnd) { _pos = dollarPos; return false; }
                    _pos++;
                }
                string name = _src.Substring(start, _pos - start);
                _pos++; // closing }
                EnsureWordExists();
                FlushLiteralBuf();
                _currentWordParts!.Add(new WordPart { Kind = PartKind.Variable, Text = name, Quoted = quoted });
                return true;
            }

            if (Cur == '?')
            {
                _pos++;
                EnsureWordExists();
                FlushLiteralBuf();
                _currentWordParts!.Add(new WordPart { Kind = PartKind.Variable, Text = "?", Quoted = quoted });
                return true;
            }

            if (char.IsLetter(Cur) || Cur == '_')
            {
                int start = _pos;
                while (char.IsLetterOrDigit(Cur) || Cur == '_') _pos++;
                string name = _src.Substring(start, _pos - start);
                EnsureWordExists();
                FlushLiteralBuf();
                _currentWordParts!.Add(new WordPart { Kind = PartKind.Variable, Text = name, Quoted = quoted });
                return true;
            }

            // Bare '$' not followed by a valid name/'('/'{'/'?' - bash
            // treats it as a literal dollar sign.
            AppendLiteralChar('$', quoted);
            return true;
        }

        private void EnsureWordExists()
        {
            _currentWordParts ??= new List<WordPart>();
        }

        private void AppendLiteralChar(char c, bool quoted)
        {
            EnsureWordExists();
            if (_literalBuf.Length > 0 && _literalBufQuoted != quoted) FlushLiteralBuf();
            _literalBufQuoted = quoted;
            _literalBuf.Append(c);
        }

        private void AppendLiteral(string text, bool quoted)
        {
            EnsureWordExists();
            if (_literalBuf.Length > 0 && _literalBufQuoted != quoted) FlushLiteralBuf();
            _literalBufQuoted = quoted;
            _literalBuf.Append(text);
            // An empty single-quoted string ('') must still register as
            // real (if empty) word content - handled by EnsureWordExists
            // above already having created _currentWordParts, so the word
            // exists even though nothing gets appended to _literalBuf here.
        }

        private void FlushLiteralBuf()
        {
            if (_literalBuf.Length == 0) return;
            _currentWordParts!.Add(new WordPart { Kind = PartKind.Literal, Text = _literalBuf.ToString(), Quoted = _literalBufQuoted });
            _literalBuf.Clear();
        }

        private void EndWord()
        {
            FlushLiteralBuf();
            if (_currentWordParts == null) return;
            _tokens.Add(new Token(TokenType.Word, new Word { Parts = _currentWordParts }));
            _currentWordParts = null;
        }
    }
}

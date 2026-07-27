using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;
using EmuSen.DianaOS.Ast;

namespace EmuSen.DianaOS
{
    // Thrown for a genuine syntax error (unbalanced 'if'/'fi', a
    // redirection with nothing after it, etc.) - distinct from
    // LexResult.NeedsMoreInput, which means "not wrong, just not finished
    // yet" and is handled entirely inside DianaOSInterpreter.Submit before
    // the parser ever runs.
    public class ShellSyntaxException : Exception
    {
        public ShellSyntaxException(string message) : base(message) { }
    }

    // Thrown instead of ShellSyntaxException specifically when the parser
    // ran out of tokens (hit EOF) at a point where it needed more, rather
    // than finding something actually wrong - "if true; then echo hi" with
    // no 'fi' yet isn't broken, it's just not finished. DianaOSInterpreter
    // catches this the same way it handles LexResult.NeedsMoreInput: hold
    // the raw text and wait for another line.
    public class ShellIncompleteException : Exception
    {
        public ShellIncompleteException(string message) : base(message) { }
    }

    // Hand-written recursive-descent parser, matching this codebase's own
    // style everywhere else a dispatch table or interpreter loop is
    // hand-built (Cpu.cs, Spc700.cs) rather than reaching for a parser-
    // generator dependency for what's still a deliberately small grammar.
    //
    // Grammar (informal, EBNF-ish):
    //   Script       := StatementList EOF
    //   StatementList:= sep* (Statement (sep+ Statement)*)? sep*      -- sep = ';' | Newline
    //   Statement    := IfStmt | ForStmt | WhileStmt | 'break' | 'continue' | AndOrList
    //   AndOrList    := Pipeline (('&&'|'||') Pipeline)*
    //   Pipeline     := ['!'] SimpleCommand ('|' SimpleCommand)*
    //   SimpleCommand:= (Assignment | Word | Redirection)+            -- at least one of these
    //   IfStmt       := 'if' StatementList 'then' StatementList
    //                    ('elif' StatementList 'then' StatementList)*
    //                    ('else' StatementList)? 'fi'
    //   ForStmt      := 'for' NAME 'in' Word* sep 'do' StatementList 'done'
    //   WhileStmt    := ('while'|'until') StatementList 'do' StatementList 'done'
    public class Parser
    {
        private readonly List<Token> _tokens;
        private int _pos;

        private static readonly Regex IdentifierPattern = new("^[A-Za-z_][A-Za-z0-9_]*$", RegexOptions.Compiled);
        private static readonly HashSet<string> Keywords = new(StringComparer.Ordinal)
        {
            "if", "then", "elif", "else", "fi", "for", "in", "do", "done", "while", "until", "break", "continue",
        };

        private Parser(List<Token> tokens) { _tokens = tokens; }

        public static StatementList Parse(List<Token> tokens)
        {
            var parser = new Parser(tokens);
            var list = parser.ParseStatementListUntil();
            parser.Expect(TokenType.EOF);
            return list;
        }

        private Token Cur => _tokens[_pos];
        private bool AtEnd => Cur.Type == TokenType.EOF;

        private void Advance() { if (!AtEnd) _pos++; }

        private void Expect(TokenType type)
        {
            if (Cur.Type != type) Fail($"Expected {type} but found {Describe(Cur)}.");
            Advance();
        }

        // Routes a parse failure to ShellIncompleteException when we ran
        // out of tokens (the user just hasn't finished typing yet) or
        // ShellSyntaxException otherwise (a token IS present, it's just
        // wrong) - see ShellIncompleteException's own comment.
        private void Fail(string message)
        {
            if (AtEnd) throw new ShellIncompleteException(message);
            throw new ShellSyntaxException(message);
        }

        private static string Describe(Token t) => t.Type == TokenType.Word ? $"'{WordLiteralText(t.Word!)}'" : t.Type.ToString();

        // Only meaningful for a word made of one unquoted literal part -
        // used purely for error messages and keyword/identifier checks,
        // never for real expansion (that's DianaOSInterpreter.ExpandWord's
        // job, at run time, with variables/substitutions resolved).
        private static string? WordLiteralText(Word w)
        {
            if (w.Parts.Count != 1 || w.Parts[0].Kind != PartKind.Literal || w.Parts[0].Quoted) return null;
            return w.Parts[0].Text;
        }

        private bool IsKeyword(string kw) => Cur.Type == TokenType.Word && WordLiteralText(Cur.Word!) == kw;

        private void ExpectKeyword(string kw)
        {
            if (!IsKeyword(kw)) Fail($"Expected '{kw}' but found {Describe(Cur)}.");
            Advance();
        }

        private bool IsSeparator() => Cur.Type == TokenType.Semicolon || Cur.Type == TokenType.Newline;

        private void SkipSeparators() { while (IsSeparator()) Advance(); }

        // Parses statements until EOF or a Word token matching one of
        // `terminators` (a keyword like 'then'/'do'/'fi'/'done'/'elif'/
        // 'else') - the terminator itself is left unconsumed, for the
        // caller (IfStmt/ForStmt/WhileStmt) to Expect() explicitly, the
        // same way real bash requires the exact right closing keyword.
        private StatementList ParseStatementListUntil(params string[] terminators)
        {
            var list = new StatementList();
            SkipSeparators();
            while (!AtEnd && !MatchesAny(terminators))
            {
                list.Statements.Add(ParseStatement());
                if (!AtEnd && !IsSeparator() && !MatchesAny(terminators))
                {
                    throw new ShellSyntaxException($"Expected ';', a newline, or one of [{string.Join(", ", terminators)}] but found {Describe(Cur)}.");
                }
                SkipSeparators();
            }

            // Reached EOF without ever finding a required terminator - the
            // block (if/for/while body or condition) isn't closed yet.
            // terminators.Length == 0 only happens for the top-level
            // script list, where hitting EOF is just "done", not this.
            if (terminators.Length > 0 && !MatchesAny(terminators))
            {
                throw new ShellIncompleteException($"Expected one of [{string.Join(", ", terminators)}] to close this block.");
            }
            return list;
        }

        private bool MatchesAny(string[] terminators)
        {
            foreach (string t in terminators) if (IsKeyword(t)) return true;
            return false;
        }

        private Node ParseStatement()
        {
            if (IsKeyword("if")) return ParseIf();
            if (IsKeyword("for")) return ParseFor();
            if (IsKeyword("while") || IsKeyword("until")) return ParseWhile();
            if (IsKeyword("break")) { Advance(); return new BreakNode(); }
            if (IsKeyword("continue")) { Advance(); return new ContinueNode(); }
            return ParseAndOrList();
        }

        private IfNode ParseIf()
        {
            var node = new IfNode();
            Advance(); // 'if'
            var cond = ParseStatementListUntil("then");
            ExpectKeyword("then");
            var body = ParseStatementListUntil("elif", "else", "fi");
            node.Branches.Add((cond, body));

            while (IsKeyword("elif"))
            {
                Advance();
                var elifCond = ParseStatementListUntil("then");
                ExpectKeyword("then");
                var elifBody = ParseStatementListUntil("elif", "else", "fi");
                node.Branches.Add((elifCond, elifBody));
            }

            if (IsKeyword("else"))
            {
                Advance();
                node.ElseBody = ParseStatementListUntil("fi");
            }

            ExpectKeyword("fi");
            return node;
        }

        private ForNode ParseFor()
        {
            Advance(); // 'for'
            if (Cur.Type != TokenType.Word) Fail($"Expected a loop variable name after 'for' but found {Describe(Cur)}.");
            string? name = WordLiteralText(Cur.Word!);
            if (name == null || !IdentifierPattern.IsMatch(name)) throw new ShellSyntaxException("'for' loop variable must be a plain, unquoted name.");
            Advance();

            ExpectKeyword("in");
            var node = new ForNode { VarName = name };
            while (Cur.Type == TokenType.Word) { node.Items.Add(Cur.Word!); Advance(); }

            if (!IsSeparator()) Fail($"Expected ';' or a newline after the 'for ... in' list but found {Describe(Cur)}.");
            SkipSeparators();
            ExpectKeyword("do");
            node.Body = ParseStatementListUntil("done");
            ExpectKeyword("done");
            return node;
        }

        private WhileNode ParseWhile()
        {
            bool until = IsKeyword("until");
            Advance(); // 'while'/'until'
            var node = new WhileNode { Until = until };
            node.Condition = ParseStatementListUntil("do");
            ExpectKeyword("do");
            node.Body = ParseStatementListUntil("done");
            ExpectKeyword("done");
            return node;
        }

        private AndOrList ParseAndOrList()
        {
            var node = new AndOrList { First = ParsePipeline() };
            while (Cur.Type == TokenType.AndAnd || Cur.Type == TokenType.OrOr)
            {
                var kind = Cur.Type == TokenType.AndAnd ? AndOrKind.And : AndOrKind.Or;
                Advance();
                node.Rest.Add((kind, ParsePipeline()));
            }
            return node;
        }

        private Pipeline ParsePipeline()
        {
            var pipeline = new Pipeline();
            if (Cur.Type == TokenType.Word && WordLiteralText(Cur.Word!) == "!")
            {
                pipeline.Negate = true;
                Advance();
            }

            pipeline.Stages.Add(ParseSimpleCommand());
            while (Cur.Type == TokenType.Pipe)
            {
                Advance();
                pipeline.Stages.Add(ParseSimpleCommand());
            }
            return pipeline;
        }

        private SimpleCommand ParseSimpleCommand()
        {
            var cmd = new SimpleCommand();
            bool stillAcceptingAssignments = true;

            while (true)
            {
                if (Cur.Type == TokenType.Greater || Cur.Type == TokenType.DGreater || Cur.Type == TokenType.Less)
                {
                    var kind = Cur.Type switch
                    {
                        TokenType.Greater => RedirectKind.Truncate,
                        TokenType.DGreater => RedirectKind.Append,
                        _ => RedirectKind.Input,
                    };
                    Advance();
                    if (Cur.Type != TokenType.Word) Fail("Expected a filename after a redirection operator.");
                    cmd.Redirections.Add(new Redirection { Kind = kind, Target = Cur.Word! });
                    Advance();
                    stillAcceptingAssignments = false;
                    continue;
                }

                if (Cur.Type == TokenType.Word)
                {
                    if (stillAcceptingAssignments && TryParseAssignment(Cur.Word!, out Assignment assignment))
                    {
                        cmd.Assignments.Add(assignment);
                        Advance();
                        continue;
                    }
                    stillAcceptingAssignments = false;
                    cmd.Words.Add(Cur.Word!);
                    Advance();
                    continue;
                }

                break;
            }

            if (cmd.Assignments.Count == 0 && cmd.Words.Count == 0 && cmd.Redirections.Count == 0)
            {
                Fail($"Expected a command but found {Describe(Cur)}.");
            }
            return cmd;
        }

        // NAME=value is only an assignment in *command position* and only
        // when unquoted and NAME is a valid identifier - "FOO=bar" (quoted
        // or with a non-identifier prefix) is just a literal argument.
        private static bool TryParseAssignment(Word w, out Assignment assignment)
        {
            assignment = null!;
            if (w.Parts.Count == 0) return false;
            WordPart first = w.Parts[0];
            if (first.Kind != PartKind.Literal || first.Quoted) return false;

            int eq = first.Text.IndexOf('=');
            if (eq <= 0) return false;

            string name = first.Text.Substring(0, eq);
            if (!IdentifierPattern.IsMatch(name)) return false;

            var valueParts = new List<WordPart>();
            string restOfFirst = first.Text.Substring(eq + 1);
            if (restOfFirst.Length > 0) valueParts.Add(new WordPart { Kind = PartKind.Literal, Text = restOfFirst, Quoted = false });
            for (int i = 1; i < w.Parts.Count; i++) valueParts.Add(w.Parts[i]);

            assignment = new Assignment { Name = name, Value = new Word { Parts = valueParts } };
            return true;
        }
    }
}

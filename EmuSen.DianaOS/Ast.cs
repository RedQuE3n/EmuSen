using System.Collections.Generic;

namespace EmuSen.DianaOS.Ast
{
    // One segment of a lexed word - a bash "word" is rarely one uniform
    // string before expansion (`pre$VARpost"lit $OTHER"end` is a single
    // word made of five differently-sourced pieces), so each piece is
    // tracked separately, tagged with where it came from - that tagging is
    // exactly what lets the interpreter apply bash's real quoting rules
    // (single quotes: literal, no expansion, never word-split; double
    // quotes: expanded, never word-split; unquoted: expanded, then
    // word-split on whitespace) instead of a single all-or-nothing flag
    // per word. See DianaOSInterpreter.ExpandWord for where this matters.
    public enum PartKind { Literal, Variable, CommandSubstitution }

    public class WordPart
    {
        public PartKind Kind;

        // Literal: the literal text itself (already unescaped).
        // Variable: the variable name (without the leading '$').
        // CommandSubstitution: the raw, unparsed source text between
        // "$(" and its matching ")" - parsed/executed recursively at
        // expansion time, not at lex time, so a substitution's own
        // contents (which might themselves contain quotes, pipes,
        // nested "$(...)") don't have to be understood until the outer
        // word actually needs its value.
        public string Text = "";

        // True if this part came from inside '...' or "..." - single
        // quotes always produce a Literal part; double quotes can still
        // produce Variable/CommandSubstitution parts (they're expanded
        // inside double quotes, just not further word-split).
        public bool Quoted;
    }

    public class Word
    {
        public List<WordPart> Parts = new();
    }

    // --- Redirection ---

    public enum RedirectKind { Truncate, Append, Input } // > , >> , <

    public class Redirection
    {
        public RedirectKind Kind;
        public Word Target = null!;
    }

    // --- Commands ---

    // A leading NAME=value assignment word, recognized only in command
    // position (bash: `FOO=bar` alone is a persistent assignment; `FOO=bar
    // echo $FOO` is the same simplification this project makes - see
    // DianaOSInterpreter's own comment on why per-command-ephemeral scoping
    // isn't implemented).
    public class Assignment
    {
        public string Name = "";
        public Word Value = null!;
    }

    public abstract class Node { }

    // One pipeline stage: zero or more leading VAR=value assignments,
    // then optionally a command name + argument words. "zero commands,
    // only assignments" is itself valid (a bare `FOO=bar` statement).
    public class SimpleCommand : Node
    {
        public List<Assignment> Assignments = new();
        public List<Word> Words = new(); // Words[0] is the command name, if present
        public List<Redirection> Redirections = new();
    }

    // command1 | command2 | command3 - redirection is scoped to the whole
    // pipeline (attached here, not per-stage) - see DianaOSInterpreter's own
    // comment on why per-stage redirection isn't implemented.
    public class Pipeline : Node
    {
        public List<SimpleCommand> Stages = new();
        public bool Negate; // leading '!' - bash's logical-NOT of the pipeline's exit status
    }

    public enum AndOrKind { And, Or }

    // pipeline1 && pipeline2 || pipeline3 ... - left-associative, evaluated
    // left to right with the usual short-circuit rules.
    public class AndOrList : Node
    {
        public Pipeline First = null!;
        public List<(AndOrKind Kind, Pipeline Pipeline)> Rest = new();
    }

    // A sequence of statements separated by ';' or newlines - the "LIST"
    // production bash's own grammar uses everywhere a compound command's
    // body is expected (if/then bodies, loop bodies, top-level scripts).
    // Each entry is either an AndOrList (an ordinary pipeline chain) or a
    // compound command (IfNode/ForNode/WhileNode/BreakNode/ContinueNode) -
    // a compound command can appear anywhere a simple command can, same as
    // real bash (an `if` can contain a `for`, a `while`'s body can contain
    // another `if`, etc.).
    public class StatementList : Node
    {
        public List<Node> Statements = new();
    }

    public class IfNode : Node
    {
        // (condition, thenBody) pairs - the first is the `if`, the rest
        // (if any) are `elif`s, evaluated in order until one condition
        // succeeds.
        public List<(StatementList Condition, StatementList Body)> Branches = new();
        public StatementList? ElseBody;
    }

    public class ForNode : Node
    {
        public string VarName = "";
        public List<Word> Items = new(); // the "in word1 word2 ..." list
        public StatementList Body = null!;
    }

    public class WhileNode : Node
    {
        public StatementList Condition = null!;
        public StatementList Body = null!;
        public bool Until; // `until` = `while` with the condition's success/failure inverted
    }

    public class BreakNode : Node { }
    public class ContinueNode : Node { }
}

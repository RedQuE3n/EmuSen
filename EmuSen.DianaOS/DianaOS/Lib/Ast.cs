using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.Ast
{
    // A word is many differently-sourced pieces, each tagged - see EmuSen_Debugging_Tools_Reference_v5.md §3.17b.
    public enum PartKind { Literal, Variable, CommandSubstitution }

    public class WordPart
    {
        public PartKind Kind;

        // CommandSubstitution holds raw source, parsed at expansion time - see §3.17b.
        public string Text = "";

        // Single quotes give a Literal; double quotes still expand, just never split.
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

    // Recognised only in command position - see §3.17b.
    public class Assignment
    {
        public string Name = "";
        public Word Value = null!;
    }

    public abstract class Node { }

    // Assignments with no command is itself a valid statement.
    public class SimpleCommand : Node
    {
        public List<Assignment> Assignments = new();
        public List<Word> Words = new(); // Words[0] is the command name, if present
        public List<Redirection> Redirections = new();
    }

    // Redirection is scoped to the whole pipeline, not per stage - see §3.17b.
    public class Pipeline : Node
    {
        public List<SimpleCommand> Stages = new();
        public bool Negate; // leading '!' - bash's logical-NOT of the pipeline's exit status
    }

    public enum AndOrKind { And, Or }

    // Left-associative, with the usual short-circuit rules.
    public class AndOrList : Node
    {
        public Pipeline First = null!;
        public List<(AndOrKind Kind, Pipeline Pipeline)> Rest = new();
    }

    // A compound command may appear anywhere a simple one can, as in bash.
    public class StatementList : Node
    {
        public List<Node> Statements = new();
    }

    public class IfNode : Node
    {
        // The first pair is the `if`; the rest are `elif`s, in order.
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

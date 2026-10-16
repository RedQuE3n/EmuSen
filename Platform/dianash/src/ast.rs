//! The tree a line parses to: the C# `Ast`. See EmuSen_Debugging_Tools_Reference_v5.md §3.17b.

/// Where a piece of a word comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartKind {
    Literal,
    Variable,
    /// Raw source, parsed when it is expanded.
    CommandSubstitution,
}

/// One differently-sourced piece of a word. `quoted` means expanded but never split.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordPart {
    pub kind: PartKind,
    pub text: String,
    pub quoted: bool,
}

/// A word with no parts is `''`: one empty argument.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Word {
    pub parts: Vec<WordPart>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedirectKind {
    /// `>`
    Truncate,
    /// `>>`
    Append,
    /// `<`
    Input,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Redirection {
    pub kind: RedirectKind,
    pub target: Word,
}

/// `NAME=value`, recognised only in command position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment {
    pub name: String,
    pub value: Word,
}

/// Assignments with no command is itself a valid statement; `words[0]` is the command's name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SimpleCommand {
    pub assignments: Vec<Assignment>,
    pub words: Vec<Word>,
    pub redirections: Vec<Redirection>,
}

/// Stages joined by `|`; `negate` is a leading `!`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pipeline {
    pub stages: Vec<SimpleCommand>,
    pub negate: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AndOr {
    And,
    Or,
}

/// Left-associative, with the usual short-circuit rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AndOrList {
    pub first: Pipeline,
    pub rest: Vec<(AndOr, Pipeline)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatementList {
    pub statements: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    AndOr(AndOrList),
    /// The first pair is the `if`; the rest are `elif`s, in order.
    If { branches: Vec<(StatementList, StatementList)>, else_body: Option<StatementList> },
    For { var: String, items: Vec<Word>, body: StatementList },
    /// `until` is `while` with the condition's success and failure inverted.
    While { condition: StatementList, body: StatementList, until: bool },
    Break,
    Continue,
}

//! Words and operators, with quoting and references tagged but nothing expanded: the C# `Lexer`.
//! See EmuSen_Debugging_Tools_Reference_v5.md §3.17b.

use crate::ast::{PartKind, Word, WordPart};
use crate::dotnet;

/// Named as the C# `TokenType` is, since a syntax error prints the name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    Word,
    Pipe,
    Semicolon,
    Newline,
    AndAnd,
    OrOr,
    Greater,
    DGreater,
    Less,
    EOF,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenType,
    pub word: Option<Word>,
}

/// Tokens, or "not wrong, just not finished": a quote, `$(`, `${` or a trailing backslash still open.
#[derive(Debug, PartialEq, Eq)]
pub enum Lexed {
    Tokens(Vec<Token>),
    NeedsMoreInput,
}

/// A line the shell refuses outright (`&` alone, `<<`), with its words.
pub type LexError = String;

pub fn tokenize(src: &str) -> Result<Lexed, LexError> {
    Lexer { src: src.chars().collect(), pos: 0, in_double_quote: false, tokens: Vec::new(), current: None, literal: String::new(), literal_quoted: false }.run()
}

struct Lexer {
    src: Vec<char>,
    pos: usize,
    /// `$VAR` inside "..." is still quoted for word splitting though `$` expands.
    in_double_quote: bool,
    tokens: Vec<Token>,
    current: Option<Vec<WordPart>>,
    literal: String,
    literal_quoted: bool,
}

impl Lexer {
    fn cur(&self) -> char {
        self.src.get(self.pos).copied().unwrap_or('\0')
    }

    fn peek(&self) -> char {
        self.src.get(self.pos + 1).copied().unwrap_or('\0')
    }

    fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn run(mut self) -> Result<Lexed, LexError> {
        // True at the start, after whitespace or right after an operator: where a '#' begins a comment.
        let mut at_word_start = true;
        while !self.at_end() {
            let c = self.cur();
            if c == '#' && at_word_start && self.current.is_none() {
                while !self.at_end() && self.cur() != '\n' {
                    self.pos += 1;
                }
                continue;
            }
            if c == ' ' || c == '\t' {
                self.end_word();
                self.pos += 1;
                at_word_start = true;
                continue;
            }
            if c == '\n' {
                self.end_word();
                self.tokens.push(Token { kind: TokenType::Newline, word: None });
                self.pos += 1;
                at_word_start = true;
                continue;
            }
            if c == '\'' {
                if !self.read_single_quoted() {
                    return Ok(Lexed::NeedsMoreInput);
                }
                at_word_start = false;
                continue;
            }
            if c == '"' {
                if !self.read_double_quoted() {
                    return Ok(Lexed::NeedsMoreInput);
                }
                at_word_start = false;
                continue;
            }
            if c == '\\' {
                self.pos += 1;
                if self.at_end() {
                    return Ok(Lexed::NeedsMoreInput);
                }
                let escaped = self.cur();
                self.append_char(escaped, true);
                self.pos += 1;
                at_word_start = false;
                continue;
            }
            if c == '$' {
                let quoted = self.in_double_quote;
                if !self.read_dollar(quoted) {
                    return Ok(Lexed::NeedsMoreInput);
                }
                at_word_start = false;
                continue;
            }
            if !self.in_double_quote && matches!(c, '|' | ';' | '&' | '>' | '<') {
                self.end_word();
                self.operator(c)?;
                at_word_start = true;
                continue;
            }
            let quoted = self.in_double_quote;
            self.append_char(c, quoted);
            self.pos += 1;
            at_word_start = false;
        }
        self.end_word();
        self.tokens.push(Token { kind: TokenType::EOF, word: None });
        Ok(Lexed::Tokens(self.tokens))
    }

    fn push(&mut self, kind: TokenType) {
        self.tokens.push(Token { kind, word: None });
    }

    fn operator(&mut self, c: char) -> Result<(), LexError> {
        self.pos += 1;
        match c {
            '|' if self.cur() == '|' => {
                self.pos += 1;
                self.push(TokenType::OrOr);
            }
            '|' => self.push(TokenType::Pipe),
            ';' => self.push(TokenType::Semicolon),
            '&' if self.cur() == '&' => {
                self.pos += 1;
                self.push(TokenType::AndAnd);
            }
            '&' => return Err("Background jobs ('&') aren't supported - this shell runs everything synchronously. Did you mean '&&'?".to_string()),
            '>' if self.cur() == '>' => {
                self.pos += 1;
                self.push(TokenType::DGreater);
            }
            '>' => self.push(TokenType::Greater),
            '<' if self.cur() == '<' => {
                return Err("Heredocs ('<<') aren't supported - use '<' with a real file, or a variable, instead.".to_string());
            }
            _ => self.push(TokenType::Less),
        }
        Ok(())
    }

    fn read_single_quoted(&mut self) -> bool {
        self.pos += 1;
        self.ensure_word();
        let start = self.pos;
        while self.cur() != '\'' {
            if self.at_end() {
                return false;
            }
            self.pos += 1;
        }
        let text: String = self.src[start..self.pos].iter().collect();
        self.append_text(&text, true);
        self.pos += 1;
        true
    }

    fn read_double_quoted(&mut self) -> bool {
        self.pos += 1;
        self.ensure_word();
        let was = self.in_double_quote;
        self.in_double_quote = true;
        while self.cur() != '"' {
            if self.at_end() {
                self.in_double_quote = was;
                return false;
            }
            if self.cur() == '\\' && matches!(self.peek(), '"' | '\\' | '$') {
                let escaped = self.peek();
                self.append_char(escaped, true);
                self.pos += 2;
                continue;
            }
            if self.cur() == '$' {
                if !self.read_dollar(true) {
                    self.in_double_quote = was;
                    return false;
                }
                continue;
            }
            let c = self.cur();
            self.append_char(c, true);
            self.pos += 1;
        }
        self.pos += 1;
        self.in_double_quote = was;
        true
    }

    /// `$NAME`, `${NAME}`, `$?` and `$(...)`; a `$` with none of them after it is a literal dollar, as in bash.
    fn read_dollar(&mut self, quoted: bool) -> bool {
        self.pos += 1;
        if self.cur() == '(' {
            self.pos += 1;
            let start = self.pos;
            let mut depth = 1;
            while depth > 0 {
                if self.at_end() {
                    return false;
                }
                match self.cur() {
                    '\'' => {
                        self.pos += 1;
                        while self.cur() != '\'' {
                            if self.at_end() {
                                return false;
                            }
                            self.pos += 1;
                        }
                    }
                    '"' => {
                        self.pos += 1;
                        while self.cur() != '"' {
                            if self.at_end() {
                                return false;
                            }
                            if self.cur() == '\\' {
                                self.pos += 1;
                            }
                            self.pos += 1;
                        }
                    }
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                if depth > 0 {
                    self.pos += 1;
                }
            }
            let inner: String = self.src[start..self.pos].iter().collect();
            self.pos += 1;
            self.part(PartKind::CommandSubstitution, inner, quoted);
            return true;
        }
        if self.cur() == '{' {
            self.pos += 1;
            let start = self.pos;
            while self.cur() != '}' {
                if self.at_end() {
                    return false;
                }
                self.pos += 1;
            }
            let name: String = self.src[start..self.pos].iter().collect();
            self.pos += 1;
            self.part(PartKind::Variable, name, quoted);
            return true;
        }
        if self.cur() == '?' {
            self.pos += 1;
            self.part(PartKind::Variable, "?".to_string(), quoted);
            return true;
        }
        if dotnet::is_letter(self.cur()) || self.cur() == '_' {
            let start = self.pos;
            while dotnet::is_letter_or_digit(self.cur()) || self.cur() == '_' {
                self.pos += 1;
            }
            let name: String = self.src[start..self.pos].iter().collect();
            self.part(PartKind::Variable, name, quoted);
            return true;
        }
        self.append_char('$', quoted);
        true
    }

    /// A reference, after the plain text buffered before it, or that text would land after it.
    fn part(&mut self, kind: PartKind, text: String, quoted: bool) {
        self.ensure_word();
        self.flush_literal();
        self.current.as_mut().expect("ensured").push(WordPart { kind, text, quoted });
    }

    fn ensure_word(&mut self) {
        self.current.get_or_insert_with(Vec::new);
    }

    fn append_char(&mut self, c: char, quoted: bool) {
        self.ensure_word();
        if !self.literal.is_empty() && self.literal_quoted != quoted {
            self.flush_literal();
        }
        self.literal_quoted = quoted;
        self.literal.push(c);
    }

    /// An empty `''` still makes the word exist, which is what makes it one argument.
    fn append_text(&mut self, text: &str, quoted: bool) {
        self.ensure_word();
        if !self.literal.is_empty() && self.literal_quoted != quoted {
            self.flush_literal();
        }
        self.literal_quoted = quoted;
        self.literal.push_str(text);
    }

    fn flush_literal(&mut self) {
        if self.literal.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.literal);
        self.current.as_mut().expect("a literal belongs to a word").push(WordPart { kind: PartKind::Literal, text, quoted: self.literal_quoted });
    }

    fn end_word(&mut self) {
        self.flush_literal();
        if let Some(parts) = self.current.take() {
            self.tokens.push(Token { kind: TokenType::Word, word: Some(Word { parts }) });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(src: &str) -> Vec<Vec<(PartKind, String, bool)>> {
        let Ok(Lexed::Tokens(tokens)) = tokenize(src) else { panic!("{src}") };
        tokens.into_iter().filter_map(|t| t.word).map(|w| w.parts.into_iter().map(|p| (p.kind, p.text, p.quoted)).collect()).collect()
    }

    #[test]
    fn quoting_and_references_are_tagged_not_expanded() {
        use PartKind::*;
        assert_eq!(words("echo 'a b' \"x $Y\" $(ls -l) ${Z}w"), vec![
            vec![(Literal, "echo".into(), false)],
            vec![(Literal, "a b".into(), true)],
            vec![(Literal, "x ".into(), true), (Variable, "Y".into(), true)],
            vec![(CommandSubstitution, "ls -l".into(), false)],
            vec![(Variable, "Z".into(), false), (Literal, "w".into(), false)],
        ]);
        assert_eq!(words("''"), vec![Vec::<(PartKind, String, bool)>::new()]);
        assert_eq!(words("a$ b$?"), vec![vec![(Literal, "a$".into(), false)], vec![(Literal, "b".into(), false), (Variable, "?".into(), false)]]);
    }

    #[test]
    fn an_open_quote_asks_for_more_and_a_lone_ampersand_is_refused() {
        assert_eq!(tokenize("echo 'a"), Ok(Lexed::NeedsMoreInput));
        assert_eq!(tokenize("echo $(ls"), Ok(Lexed::NeedsMoreInput));
        assert_eq!(tokenize("echo \\"), Ok(Lexed::NeedsMoreInput));
        assert!(tokenize("sleep &").unwrap_err().starts_with("Background jobs"));
        assert!(tokenize("cat <<EOF").unwrap_err().starts_with("Heredocs"));
    }

    #[test]
    fn a_hash_is_a_comment_only_where_a_word_could_start() {
        assert_eq!(words("echo a#b # c"), vec![vec![(PartKind::Literal, "echo".into(), false)], vec![(PartKind::Literal, "a#b".into(), false)]]);
    }
}

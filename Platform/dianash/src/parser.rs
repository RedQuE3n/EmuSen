//! Hand-written recursive descent over the lexer's tokens: the C# `Parser`. The grammar is in
//! EmuSen_Debugging_Tools_Reference_v5.md §3.17b.

use crate::ast::*;
use crate::lexer::{Token, TokenType};

/// Out of tokens is "not finished yet"; a wrong token present is a syntax error.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Incomplete(String),
    Syntax(String),
}

pub fn parse(tokens: &[Token]) -> Result<StatementList, ParseError> {
    let mut parser = Parser { tokens, pos: 0 };
    let list = parser.statement_list_until(&[])?;
    parser.expect(TokenType::EOF)?;
    Ok(list)
}

/// `^[A-Za-z_][A-Za-z0-9_]*$`.
pub fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_') && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A word's text when it is one unquoted literal: for messages and keywords only, never for expansion.
pub fn literal_text(word: &Word) -> Option<&str> {
    match word.parts.as_slice() {
        [part] if part.kind == PartKind::Literal && !part.quoted => Some(&part.text),
        _ => None,
    }
}

fn describe(token: &Token) -> String {
    match &token.word {
        Some(word) if token.kind == TokenType::Word => format!("'{}'", literal_text(word).unwrap_or("")),
        _ => format!("{:?}", token.kind),
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl Parser<'_> {
    fn cur(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn at_end(&self) -> bool {
        self.cur().kind == TokenType::EOF
    }

    fn advance(&mut self) {
        if !self.at_end() {
            self.pos += 1;
        }
    }

    fn fail<T>(&self, message: String) -> Result<T, ParseError> {
        Err(if self.at_end() { ParseError::Incomplete(message) } else { ParseError::Syntax(message) })
    }

    fn expect(&mut self, kind: TokenType) -> Result<(), ParseError> {
        if self.cur().kind != kind {
            return self.fail(format!("Expected {kind:?} but found {}.", describe(self.cur())));
        }
        self.advance();
        Ok(())
    }

    fn is_keyword(&self, keyword: &str) -> bool {
        self.cur().kind == TokenType::Word && self.cur().word.as_ref().and_then(literal_text) == Some(keyword)
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), ParseError> {
        if !self.is_keyword(keyword) {
            return self.fail(format!("Expected '{keyword}' but found {}.", describe(self.cur())));
        }
        self.advance();
        Ok(())
    }

    fn is_separator(&self) -> bool {
        matches!(self.cur().kind, TokenType::Semicolon | TokenType::Newline)
    }

    fn skip_separators(&mut self) {
        while self.is_separator() {
            self.advance();
        }
    }

    fn matches_any(&self, terminators: &[&str]) -> bool {
        terminators.iter().any(|t| self.is_keyword(t))
    }

    /// The terminator is left for the caller to expect.
    fn statement_list_until(&mut self, terminators: &[&str]) -> Result<StatementList, ParseError> {
        let mut list = StatementList::default();
        self.skip_separators();
        while !self.at_end() && !self.matches_any(terminators) {
            list.statements.push(self.statement()?);
            if !self.at_end() && !self.is_separator() && !self.matches_any(terminators) {
                return Err(ParseError::Syntax(format!(
                    "Expected ';', a newline, or one of [{}] but found {}.",
                    terminators.join(", "),
                    describe(self.cur())
                )));
            }
            self.skip_separators();
        }
        if !terminators.is_empty() && !self.matches_any(terminators) {
            return Err(ParseError::Incomplete(format!("Expected one of [{}] to close this block.", terminators.join(", "))));
        }
        Ok(list)
    }

    fn statement(&mut self) -> Result<Node, ParseError> {
        if self.is_keyword("if") {
            return self.if_statement();
        }
        if self.is_keyword("for") {
            return self.for_statement();
        }
        if self.is_keyword("while") || self.is_keyword("until") {
            return self.while_statement();
        }
        if self.is_keyword("break") {
            self.advance();
            return Ok(Node::Break);
        }
        if self.is_keyword("continue") {
            self.advance();
            return Ok(Node::Continue);
        }
        Ok(Node::AndOr(self.and_or_list()?))
    }

    fn if_statement(&mut self) -> Result<Node, ParseError> {
        let mut branches = Vec::new();
        self.advance();
        let condition = self.statement_list_until(&["then"])?;
        self.expect_keyword("then")?;
        let body = self.statement_list_until(&["elif", "else", "fi"])?;
        branches.push((condition, body));
        while self.is_keyword("elif") {
            self.advance();
            let condition = self.statement_list_until(&["then"])?;
            self.expect_keyword("then")?;
            let body = self.statement_list_until(&["elif", "else", "fi"])?;
            branches.push((condition, body));
        }
        let mut else_body = None;
        if self.is_keyword("else") {
            self.advance();
            else_body = Some(self.statement_list_until(&["fi"])?);
        }
        self.expect_keyword("fi")?;
        Ok(Node::If { branches, else_body })
    }

    fn for_statement(&mut self) -> Result<Node, ParseError> {
        self.advance();
        if self.cur().kind != TokenType::Word {
            return self.fail(format!("Expected a loop variable name after 'for' but found {}.", describe(self.cur())));
        }
        let var = match self.cur().word.as_ref().and_then(literal_text) {
            Some(name) if is_identifier(name) => name.to_string(),
            _ => return Err(ParseError::Syntax("'for' loop variable must be a plain, unquoted name.".to_string())),
        };
        self.advance();
        self.expect_keyword("in")?;
        let mut items = Vec::new();
        while self.cur().kind == TokenType::Word {
            items.push(self.cur().word.clone().expect("a word token"));
            self.advance();
        }
        if !self.is_separator() {
            return self.fail(format!("Expected ';' or a newline after the 'for ... in' list but found {}.", describe(self.cur())));
        }
        self.skip_separators();
        self.expect_keyword("do")?;
        let body = self.statement_list_until(&["done"])?;
        self.expect_keyword("done")?;
        Ok(Node::For { var, items, body })
    }

    fn while_statement(&mut self) -> Result<Node, ParseError> {
        let until = self.is_keyword("until");
        self.advance();
        let condition = self.statement_list_until(&["do"])?;
        self.expect_keyword("do")?;
        let body = self.statement_list_until(&["done"])?;
        self.expect_keyword("done")?;
        Ok(Node::While { condition, body, until })
    }

    fn and_or_list(&mut self) -> Result<AndOrList, ParseError> {
        let first = self.pipeline()?;
        let mut rest = Vec::new();
        while matches!(self.cur().kind, TokenType::AndAnd | TokenType::OrOr) {
            let kind = if self.cur().kind == TokenType::AndAnd { AndOr::And } else { AndOr::Or };
            self.advance();
            rest.push((kind, self.pipeline()?));
        }
        Ok(AndOrList { first, rest })
    }

    fn pipeline(&mut self) -> Result<Pipeline, ParseError> {
        let mut pipeline = Pipeline::default();
        if self.cur().kind == TokenType::Word && self.cur().word.as_ref().and_then(literal_text) == Some("!") {
            pipeline.negate = true;
            self.advance();
        }
        pipeline.stages.push(self.simple_command()?);
        while self.cur().kind == TokenType::Pipe {
            self.advance();
            pipeline.stages.push(self.simple_command()?);
        }
        Ok(pipeline)
    }

    fn simple_command(&mut self) -> Result<SimpleCommand, ParseError> {
        let mut command = SimpleCommand::default();
        let mut accepting_assignments = true;
        loop {
            let kind = self.cur().kind;
            if matches!(kind, TokenType::Greater | TokenType::DGreater | TokenType::Less) {
                let kind = match kind {
                    TokenType::Greater => RedirectKind::Truncate,
                    TokenType::DGreater => RedirectKind::Append,
                    _ => RedirectKind::Input,
                };
                self.advance();
                if self.cur().kind != TokenType::Word {
                    return self.fail("Expected a filename after a redirection operator.".to_string());
                }
                command.redirections.push(Redirection { kind, target: self.cur().word.clone().expect("a word token") });
                self.advance();
                accepting_assignments = false;
                continue;
            }
            if kind == TokenType::Word {
                let word = self.cur().word.clone().expect("a word token");
                if accepting_assignments {
                    if let Some(assignment) = assignment(&word) {
                        command.assignments.push(assignment);
                        self.advance();
                        continue;
                    }
                }
                accepting_assignments = false;
                command.words.push(word);
                self.advance();
                continue;
            }
            break;
        }
        if command.assignments.is_empty() && command.words.is_empty() && command.redirections.is_empty() {
            return self.fail(format!("Expected a command but found {}.", describe(self.cur())));
        }
        Ok(command)
    }
}

/// `NAME=value` in command position: an unquoted literal first part, a valid name before the first `=`.
fn assignment(word: &Word) -> Option<Assignment> {
    let first = word.parts.first()?;
    if first.kind != PartKind::Literal || first.quoted {
        return None;
    }
    let eq = first.text.find('=')?;
    if eq == 0 {
        return None;
    }
    let name = &first.text[..eq];
    if !is_identifier(name) {
        return None;
    }
    let mut parts = Vec::new();
    let rest = &first.text[eq + 1..];
    if !rest.is_empty() {
        parts.push(WordPart { kind: PartKind::Literal, text: rest.to_string(), quoted: false });
    }
    parts.extend(word.parts[1..].iter().cloned());
    Some(Assignment { name: name.to_string(), value: Word { parts } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{tokenize, Lexed};

    fn parsed(src: &str) -> Result<StatementList, ParseError> {
        let Ok(Lexed::Tokens(tokens)) = tokenize(src) else { panic!("{src}") };
        parse(&tokens)
    }

    #[test]
    fn errors_name_the_token_as_the_csharp_does() {
        assert_eq!(parsed("| x"), Err(ParseError::Syntax("Expected a command but found Pipe.".into())));
        assert_eq!(parsed("&& echo x"), Err(ParseError::Syntax("Expected a command but found AndAnd.".into())));
        assert_eq!(parsed("echo x |"), Err(ParseError::Incomplete("Expected a command but found EOF.".into())));
        assert_eq!(parsed("if true; then echo"), Err(ParseError::Incomplete("Expected one of [elif, else, fi] to close this block.".into())));
        assert_eq!(parsed("for 'x' in a; do echo; done"), Err(ParseError::Syntax("'for' loop variable must be a plain, unquoted name.".into())));
    }

    #[test]
    fn an_assignment_is_only_a_valid_name_in_command_position() {
        let list = parsed("A=1 B=$X echo C=2").unwrap();
        let Node::AndOr(and_or) = &list.statements[0] else { panic!() };
        let command = &and_or.first.stages[0];
        assert_eq!(command.assignments.len(), 2);
        assert_eq!(command.words.len(), 2);
        assert_eq!(command.assignments[1].value.parts[0].kind, PartKind::Variable);
    }
}

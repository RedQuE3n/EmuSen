//! The expression language behind `eval`, breakpoint conditions and logpoints, the C# `ExpressionEvaluator` rule for
//! rule and word for word (`man eval`). The language knows no console: what a name or a memory read means is the
//! [`Context`]'s.

use emusen_dotnet::text::{is_digit, is_letter, is_letter_or_digit, is_white_space};

/// What an expression can reach, `IExpressionContext`.
pub trait Context {
    /// A symbol's value, or none for a name the context does not know.
    fn symbol(&mut self, name: &str) -> Option<i64>;

    /// `width` bytes little-endian at `address`; `space` none for the context's own bus. None where it cannot be read.
    fn read_memory(&mut self, space: Option<&str>, address: i32, width: i32) -> Option<i64>;

    /// Every symbol the context resolves, for `eval symbols`.
    fn symbol_names(&mut self) -> Vec<String>;
}

/// Why an expression has no value: the player's input refused (`ExpressionException`), or the arithmetic .NET refuses
/// (`OverflowException`), which a caller that catches every exception reports by the same words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Expression(String),
    Overflow,
}

impl Error {
    /// The exception's `Message`.
    pub fn message(&self) -> &str {
        match self {
            Error::Expression(words) => words,
            Error::Overflow => "Arithmetic operation resulted in an overflow.",
        }
    }
}

fn refused<T>(words: String) -> Result<T, Error> {
    Err(Error::Expression(words))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Number,
    Identifier,
    Operator,
    End,
}

/// A token, its text a range of the source except a number's, whose text is its prefix as the C# keeps it.
#[derive(Clone, Copy, Debug)]
struct Token {
    kind: Kind,
    start: usize,
    end: usize,
    value: i64,
    prefix: &'static str,
}

impl Token {
    fn text<'s>(&self, source: &'s str) -> &'s str {
        match self.kind {
            Kind::Number => self.prefix,
            Kind::End => "<end>",
            _ => &source[self.start..self.end],
        }
    }
}

const END: Token = Token { kind: Kind::End, start: 0, end: 0, value: 0, prefix: "" };

/// Precedence climbing, lowest level first.
const LEVELS: [&[&str]; 10] = [&["||"], &["&&"], &["|"], &["^"], &["&"], &["==", "!="], &["<", ">", "<=", ">="], &["<<", ">>"], &["+", "-"], &["*", "/", "%"]];

/// The evaluator over one context, its token list kept between calls.
pub struct Evaluator<'c> {
    context: &'c mut dyn Context,
    source: String,
    tokens: Vec<Token>,
    at: usize,
    checking: bool,
}

impl<'c> Evaluator<'c> {
    pub fn new(context: &'c mut dyn Context) -> Evaluator<'c> {
        Evaluator { context, source: String::new(), tokens: Vec::new(), at: 0, checking: false }
    }

    fn load(&mut self, expression: &str) -> Result<(), Error> {
        self.source.clear();
        self.source.push_str(expression);
        tokenize(expression, &mut self.tokens)?;
        self.at = 0;
        Ok(())
    }

    /// `Evaluate`.
    pub fn evaluate(&mut self, expression: &str) -> Result<i64, Error> {
        self.load(expression)?;
        let value = self.binary(0, true)?;
        let current = self.current();
        if current.kind != Kind::End {
            return refused(format!("Unexpected '{}' in expression.", current.text(&self.source)));
        }
        Ok(value)
    }

    /// `Check`: parsed without evaluating or reading memory, a syntax error or an unknown symbol refused.
    pub fn check(&mut self, expression: &str) -> Result<(), Error> {
        self.load(expression)?;
        self.checking = true;
        let result = self.binary(0, false).and_then(|_| {
            let current = self.current();
            if current.kind != Kind::End { refused(format!("Unexpected '{}' in expression.", current.text(&self.source))) } else { Ok(()) }
        });
        self.checking = false;
        result
    }

    /// `EvaluateBool`: anything nonzero is true.
    pub fn evaluate_bool(&mut self, expression: &str) -> Result<bool, Error> {
        self.evaluate(expression).map(|v| v != 0)
    }

    fn current(&self) -> Token {
        self.tokens.get(self.at).copied().unwrap_or(END)
    }

    fn current_is(&self, text: &str) -> bool {
        let t = self.current();
        t.kind == Kind::Operator && t.text(&self.source) == text
    }

    fn binary(&mut self, level: usize, live: bool) -> Result<i64, Error> {
        if level >= LEVELS.len() {
            return self.unary(live);
        }
        let mut left = self.binary(level + 1, live)?;
        loop {
            let t = self.current();
            if t.kind != Kind::Operator {
                break;
            }
            let op = &self.source[t.start..t.end];
            let Some(&op) = LEVELS[level].iter().find(|&&o| o == op) else { break };
            self.at += 1;
            if live && op == "&&" && left == 0 {
                self.binary(level + 1, false)?;
                left = 0;
                continue;
            }
            if live && op == "||" && left != 0 {
                self.binary(level + 1, false)?;
                left = 1;
                continue;
            }
            let right = self.binary(level + 1, live)?;
            left = if live { apply(op, left, right)? } else { 0 };
        }
        Ok(left)
    }

    fn unary(&mut self, live: bool) -> Result<i64, Error> {
        let t = self.current();
        if t.kind == Kind::Operator {
            match &self.source[t.start..t.end] {
                "-" => {
                    self.at += 1;
                    return self.unary(live).map(i64::wrapping_neg);
                }
                "!" => {
                    self.at += 1;
                    return self.unary(live).map(|v| (v == 0) as i64);
                }
                "~" => {
                    self.at += 1;
                    return self.unary(live).map(|v| !v);
                }
                "+" => {
                    self.at += 1;
                    return self.unary(live);
                }
                _ => {}
            }
        }
        self.primary(live)
    }

    fn primary(&mut self, live: bool) -> Result<i64, Error> {
        let t = self.current();
        match t.kind {
            Kind::Number => {
                self.at += 1;
                Ok(t.value)
            }
            Kind::Identifier => {
                self.at += 1;
                let name = &self.source[t.start..t.end];
                match self.context.symbol(name) {
                    Some(v) => Ok(v),
                    None if !live && !self.checking => Ok(0),
                    None => refused(format!("Unknown symbol '{name}'. Try `eval symbols` for the full list.")),
                }
            }
            Kind::Operator => match t.text(&self.source) {
                "(" => {
                    self.at += 1;
                    let v = self.binary(0, live)?;
                    self.expect(")")?;
                    Ok(v)
                }
                "[" => self.memory("]", 1, live),
                "{" => self.memory("}", 2, live),
                other => refused(format!("Unexpected '{other}' in expression.")),
            },
            Kind::End => refused("Unexpected '<end>' in expression.".to_string()),
        }
    }

    fn memory(&mut self, closer: &str, width: i32, live: bool) -> Result<i64, Error> {
        self.at += 1;
        // An identifier followed by ':' names the space.
        let mut space: Option<(usize, usize)> = None;
        let t = self.current();
        if t.kind == Kind::Identifier && self.tokens.get(self.at + 1).is_some_and(|n| n.text(&self.source) == ":") {
            space = Some((t.start, t.end));
            self.at += 2;
        }
        let address = self.binary(0, live)?;
        self.expect(closer)?;
        if !live {
            return Ok(0);
        }
        let space_name = space.map(|(s, e)| &self.source[s..e]);
        match self.context.read_memory(space_name, address as i32, width) {
            Some(v) => Ok(v),
            None => {
                let where_ = match space_name {
                    None => format!("0x{address:X}"),
                    Some(s) => format!("{s} 0x{address:X}"),
                };
                refused(format!("Cannot read {where_}."))
            }
        }
    }

    fn expect(&mut self, text: &str) -> Result<(), Error> {
        if !self.current_is(text) {
            return refused(format!("Expected '{text}' but found '{}'.", self.current().text(&self.source)));
        }
        self.at += 1;
        Ok(())
    }
}

fn apply(op: &str, a: i64, b: i64) -> Result<i64, Error> {
    Ok(match op {
        "||" => (a != 0 || b != 0) as i64,
        "&&" => (a != 0 && b != 0) as i64,
        "|" => a | b,
        "^" => a ^ b,
        "&" => a & b,
        "==" => (a == b) as i64,
        "!=" => (a != b) as i64,
        "<" => (a < b) as i64,
        ">" => (a > b) as i64,
        "<=" => (a <= b) as i64,
        ">=" => (a >= b) as i64,
        "<<" => a.wrapping_shl((b & 63) as u32),
        ">>" => a.wrapping_shr((b & 63) as u32),
        "+" => a.wrapping_add(b),
        "-" => a.wrapping_sub(b),
        "*" => a.wrapping_mul(b),
        "/" | "%" if b == 0 => return refused("Division by zero.".to_string()),
        // .NET's division of the least long by -1 throws.
        "/" | "%" if a == i64::MIN && b == -1 => return Err(Error::Overflow),
        "/" => a / b,
        "%" => a % b,
        _ => return refused(format!("Unknown operator '{op}'.")),
    })
}

fn tokenize(source: &str, tokens: &mut Vec<Token>) -> Result<(), Error> {
    tokens.clear();
    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let at = |i: usize| chars.get(i).map(|&(b, _)| b).unwrap_or(source.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i].1;
        if is_white_space(c) {
            i += 1;
            continue;
        }
        // '%' is a binary prefix only before a binary digit; elsewhere it is modulo.
        let binary = c == '%' && chars.get(i + 1).is_some_and(|&(_, n)| n == '0' || n == '1');
        if c == '$' || binary || is_digit(c) {
            tokens.push(number(&chars, &mut i, source)?);
            continue;
        }
        if is_letter(c) || c == '_' || c == '.' {
            let start = i;
            while i < chars.len() && (is_letter_or_digit(chars[i].1) || chars[i].1 == '_' || chars[i].1 == '.') {
                i += 1;
            }
            tokens.push(Token { kind: Kind::Identifier, start: at(start), end: at(i), value: 0, prefix: "" });
            continue;
        }
        if let Some(&(_, n)) = chars.get(i + 1) {
            let pair = [c, n];
            if matches!(pair, ['=', '='] | ['!', '='] | ['<', '='] | ['>', '='] | ['&', '&'] | ['|', '|'] | ['<', '<'] | ['>', '>']) {
                tokens.push(Token { kind: Kind::Operator, start: at(i), end: at(i + 2), value: 0, prefix: "" });
                i += 2;
                continue;
            }
        }
        if "+-*/%&|^~!<>()[]{}:".contains(c) {
            tokens.push(Token { kind: Kind::Operator, start: at(i), end: at(i + 1), value: 0, prefix: "" });
            i += 1;
            continue;
        }
        return refused(format!("Unexpected character '{}' in expression.", utf16_char(c)));
    }
    tokens.push(END);
    Ok(())
}

/// A character as the C# formats a `char`: one outside the Basic Multilingual Plane is a surrogate pair, and the
/// tokenizer meets its first half alone.
fn utf16_char(c: char) -> String {
    let mut units = [0u16; 2];
    let encoded = c.encode_utf16(&mut units);
    String::from_utf16_lossy(&encoded[..1])
}

fn number(chars: &[(usize, char)], i: &mut usize, source: &str) -> Result<Token, Error> {
    let start_byte = chars[*i].0;
    let (prefix, radix) = match chars[*i].1 {
        '$' => {
            *i += 1;
            ("$", 16)
        }
        '%' => {
            *i += 1;
            ("%", 2)
        }
        '0' if chars.get(*i + 1).is_some_and(|&(_, x)| x == 'x' || x == 'X') => {
            *i += 2;
            ("0x", 16)
        }
        _ => ("d", 10),
    };
    let first = *i;
    while *i < chars.len() && digit_in(chars[*i].1, radix) {
        *i += 1;
    }
    if *i == first {
        return refused("Expected a number.".to_string());
    }
    let text: String = chars[first..*i].iter().map(|&(_, c)| c).collect();
    let value = convert(&text, radix).ok_or_else(|| Error::Expression(format!("'{text}' is not a valid base-{radix} number.")))?;
    let end_byte = chars.get(*i).map(|&(b, _)| b).unwrap_or(source.len());
    Ok(Token { kind: Kind::Number, start: start_byte, end: end_byte, value, prefix })
}

fn digit_in(c: char, radix: u32) -> bool {
    match radix {
        2 => c == '0' || c == '1',
        10 => is_digit(c),
        _ => c.is_ascii_hexdigit(),
    }
}

/// `Convert.ToInt64(text, radix)`: base 10 signed and checked; bases 2 and 16 the 64 bits as given, two's complement.
fn convert(text: &str, radix: u32) -> Option<i64> {
    if radix == 10 {
        if !text.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        return text.parse::<i64>().ok();
    }
    let mut value: u64 = 0;
    for c in text.chars() {
        let d = c.to_digit(radix)? as u64;
        value = value.checked_mul(radix as u64)?.checked_add(d)?;
    }
    Some(value as i64)
}

/// `ExpressionEvaluator.Format`: decimal, hex and binary at once.
pub fn format(value: i64) -> String {
    let magnitude = if value < 0 { value.wrapping_neg() } else { value };
    let bits = if magnitude > 0xFFFF { 24 } else if magnitude > 0xFF { 16 } else { 8 };
    let mut binary = String::new();
    for bit in (0..bits).rev() {
        binary.push(if (magnitude >> bit) & 1 != 0 { '1' } else { '0' });
        if bit % 4 == 0 && bit != 0 {
            binary.push(' ');
        }
    }
    let hex = if value < 0 { format!("-${magnitude:X}") } else { format!("${value:X}") };
    format!("{value}  {hex}  %{binary}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct Fake {
        symbols: HashMap<String, i64>,
        memory: HashMap<(Option<String>, i32), u8>,
        asked: Vec<String>,
    }

    impl Context for Fake {
        fn symbol(&mut self, name: &str) -> Option<i64> {
            self.asked.push(name.to_string());
            self.symbols.get(name).copied()
        }

        fn read_memory(&mut self, space: Option<&str>, address: i32, width: i32) -> Option<i64> {
            let mut v = 0i64;
            for i in 0..width {
                v |= (*self.memory.get(&(space.map(str::to_string), address + i))? as i64) << (8 * i);
            }
            Some(v)
        }

        fn symbol_names(&mut self) -> Vec<String> {
            self.symbols.keys().cloned().collect()
        }
    }

    fn eval(e: &str) -> Result<i64, Error> {
        let mut f = Fake::default();
        f.symbols.insert("A".into(), 0x42);
        f.memory.insert((None, 0x10), 0x34);
        f.memory.insert((None, 0x11), 0x12);
        Evaluator::new(&mut f).evaluate(e)
    }

    #[test]
    fn the_languages_arithmetic_is_the_csharps() {
        assert_eq!(eval("2 * 3 + 4"), Ok(10));
        assert_eq!(eval("%1010 + $F + 0x10"), Ok(41));
        assert_eq!(eval("7 % 4"), Ok(3));
        assert_eq!(eval("{$10} + [$10]"), Ok(0x1234 + 0x34));
        assert_eq!(eval("A == $42 && 1"), Ok(1));
        assert_eq!(eval("1 2"), Err(Error::Expression("Unexpected 'd' in expression.".into())));
        assert_eq!(eval("$8000000000000000"), Ok(i64::MIN));
        assert_eq!(eval("9223372036854775808"), Err(Error::Expression("'9223372036854775808' is not a valid base-10 number.".into())));
        assert_eq!(eval("$8000000000000000 / -1"), Err(Error::Overflow));
        assert_eq!(eval("B"), Err(Error::Expression("Unknown symbol 'B'. Try `eval symbols` for the full list.".into())));
        assert_eq!(eval("[3]"), Err(Error::Expression("Cannot read 0x3.".into())));
        assert_eq!(eval("1 #"), Err(Error::Expression("Unexpected character '#' in expression.".into())));
    }

    #[test]
    fn a_skipped_branch_still_names_its_symbols_but_reads_nothing() {
        let mut f = Fake::default();
        let v = Evaluator::new(&mut f).evaluate("0 && (B + [7])");
        assert_eq!(v, Ok(0));
        assert_eq!(f.asked, ["B"]);
    }

    #[test]
    fn the_format_is_three_bases() {
        assert_eq!(format(255), "255  $FF  %1111 1111");
        assert_eq!(format(-1), "-1  -$1  %0000 0001");
        assert_eq!(format(0x12345), "74565  $12345  %0000 0001 0010 0011 0100 0101");
        assert_eq!(format(i64::MIN), "-9223372036854775808  -$8000000000000000  %0000 0000");
    }
}

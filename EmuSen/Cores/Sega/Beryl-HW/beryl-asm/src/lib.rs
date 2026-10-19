//! The front half every Beryl assembler shares (Beryl-HW/README.md, "The assemblers"): expressions, labels and their
//! scopes, constants, conditionals and includes, and the splitting of a line into its label, word and operands. Each
//! processor's crate keeps its operands, its encoding, its data directives and its layout, behind its `asm` feature.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsmError {
    /// One-based line of the source; 0 for the assembler as a whole.
    pub line: usize,
    pub message: String,
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for AsmError {}

pub fn err<T>(line: usize, message: impl Into<String>) -> Result<T, AsmError> {
    Err(AsmError { line, message: message.into() })
}

/// An assembled program: its bytes from `origin`, gaps between `org`s filled, and every label's and constant's value.
#[derive(Clone, Debug, Default)]
pub struct Program {
    pub origin: u32,
    pub bytes: Vec<u8>,
    pub symbols: BTreeMap<String, i64>,
}

// ---------------------------------------------------------------------------------------------------------------------
// Expressions

#[derive(Clone, Debug)]
pub enum Expr {
    Num(i64),
    Sym(String),
    Pc,
    Neg(Box<Expr>),
    Not(Box<Expr>),
    LNot(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
}

struct Lexer<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Lexer<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && (self.s[self.i] == b' ' || self.s[self.i] == b'\t') {
            self.i += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.ws();
        self.s.get(self.i).copied()
    }

    fn eat(&mut self, t: &str) -> bool {
        self.ws();
        if self.s[self.i..].starts_with(t.as_bytes()) {
            self.i += t.len();
            true
        } else {
            false
        }
    }
}

pub fn is_sym_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'.' || c == b'@'
}

pub fn is_sym(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'@'
}

/// An expression in C's order of operators: decimal, `$` and `0x` hexadecimal, `%` binary, character constants of up
/// to four bytes, and `*` or a `$` that starts no number for the current address.
pub fn parse_expr(text: &str, scope: &str) -> Result<Expr, String> {
    let mut l = Lexer { s: text.as_bytes(), i: 0 };
    let e = expr_cmp(&mut l, scope)?;
    l.ws();
    if l.i != l.s.len() {
        return Err(format!("unexpected `{}` in expression `{text}`", &text[l.i..]));
    }
    Ok(e)
}

fn expr_cmp(l: &mut Lexer, sc: &str) -> Result<Expr, String> {
    let e = expr_or(l, sc)?;
    for (t, op) in [("==", 'e'), ("!=", 'n'), ("<>", 'n'), ("<=", 'l'), (">=", 'g'), ("=", 'e'), ("<", 'L'), (">", 'G')] {
        let save = l.i;
        if l.eat(t) {
            if (t == "<" || t == ">") && l.s.get(l.i) == Some(&t.as_bytes()[0]) {
                l.i = save;
                continue;
            }
            return Ok(Expr::Bin(op, Box::new(e), Box::new(expr_or(l, sc)?)));
        }
    }
    Ok(e)
}

fn expr_or(l: &mut Lexer, sc: &str) -> Result<Expr, String> {
    let mut e = expr_xor(l, sc)?;
    while l.peek() == Some(b'|') {
        l.i += 1;
        e = Expr::Bin('|', Box::new(e), Box::new(expr_xor(l, sc)?));
    }
    Ok(e)
}

fn expr_xor(l: &mut Lexer, sc: &str) -> Result<Expr, String> {
    let mut e = expr_and(l, sc)?;
    while l.peek() == Some(b'^') {
        l.i += 1;
        e = Expr::Bin('^', Box::new(e), Box::new(expr_and(l, sc)?));
    }
    Ok(e)
}

fn expr_and(l: &mut Lexer, sc: &str) -> Result<Expr, String> {
    let mut e = expr_shift(l, sc)?;
    while l.peek() == Some(b'&') {
        l.i += 1;
        e = Expr::Bin('&', Box::new(e), Box::new(expr_shift(l, sc)?));
    }
    Ok(e)
}

fn expr_shift(l: &mut Lexer, sc: &str) -> Result<Expr, String> {
    let mut e = expr_add(l, sc)?;
    loop {
        if l.eat("<<") {
            e = Expr::Bin('<', Box::new(e), Box::new(expr_add(l, sc)?));
        } else if l.eat(">>") {
            e = Expr::Bin('>', Box::new(e), Box::new(expr_add(l, sc)?));
        } else {
            return Ok(e);
        }
    }
}

fn expr_add(l: &mut Lexer, sc: &str) -> Result<Expr, String> {
    let mut e = expr_mul(l, sc)?;
    loop {
        match l.peek() {
            Some(c @ (b'+' | b'-')) => {
                l.i += 1;
                e = Expr::Bin(c as char, Box::new(e), Box::new(expr_mul(l, sc)?));
            }
            _ => return Ok(e),
        }
    }
}

fn expr_mul(l: &mut Lexer, sc: &str) -> Result<Expr, String> {
    let mut e = expr_unary(l, sc)?;
    loop {
        match l.peek() {
            Some(c @ (b'*' | b'/' | b'%')) => {
                l.i += 1;
                e = Expr::Bin(c as char, Box::new(e), Box::new(expr_unary(l, sc)?));
            }
            _ => return Ok(e),
        }
    }
}

fn expr_unary(l: &mut Lexer, sc: &str) -> Result<Expr, String> {
    match l.peek() {
        Some(b'-') => {
            l.i += 1;
            Ok(Expr::Neg(Box::new(expr_unary(l, sc)?)))
        }
        Some(b'+') => {
            l.i += 1;
            expr_unary(l, sc)
        }
        Some(b'~') => {
            l.i += 1;
            Ok(Expr::Not(Box::new(expr_unary(l, sc)?)))
        }
        Some(b'!') => {
            l.i += 1;
            Ok(Expr::LNot(Box::new(expr_unary(l, sc)?)))
        }
        _ => expr_primary(l, sc),
    }
}

fn expr_primary(l: &mut Lexer, sc: &str) -> Result<Expr, String> {
    let c = l.peek().ok_or("expression ends early")?;
    let start = l.i;
    let digits = |l: &mut Lexer, radix: u32| -> Result<i64, String> {
        let from = l.i;
        while l.i < l.s.len() && ((l.s[l.i] as char).is_digit(radix) || l.s[l.i] == b'_') {
            l.i += 1;
        }
        let t: String = std::str::from_utf8(&l.s[from..l.i]).unwrap().replace('_', "");
        i64::from_str_radix(&t, radix).map_err(|_| format!("bad number `{}`", std::str::from_utf8(&l.s[start..l.i]).unwrap()))
    };
    match c {
        b'(' => {
            l.i += 1;
            let e = expr_or(l, sc)?;
            if !l.eat(")") {
                return Err("missing `)`".into());
            }
            Ok(e)
        }
        b'$' if !l.s.get(l.i + 1).is_some_and(|x| x.is_ascii_hexdigit()) => {
            l.i += 1;
            Ok(Expr::Pc)
        }
        b'$' => {
            l.i += 1;
            Ok(Expr::Num(digits(l, 16)?))
        }
        b'%' => {
            l.i += 1;
            Ok(Expr::Num(digits(l, 2)?))
        }
        b'*' => {
            l.i += 1;
            Ok(Expr::Pc)
        }
        b'\'' | b'"' => {
            l.i += 1;
            let mut v: i64 = 0;
            let mut n = 0;
            while l.i < l.s.len() && l.s[l.i] != c {
                v = v << 8 | l.s[l.i] as i64;
                l.i += 1;
                n += 1;
            }
            if l.i >= l.s.len() || n == 0 || n > 4 {
                return Err("bad character constant".into());
            }
            l.i += 1;
            Ok(Expr::Num(v))
        }
        b'0' if l.s.get(l.i + 1).is_some_and(|x| *x == b'x' || *x == b'X') => {
            l.i += 2;
            Ok(Expr::Num(digits(l, 16)?))
        }
        b'0'..=b'9' => Ok(Expr::Num(digits(l, 10)?)),
        c if is_sym_start(c) => {
            while l.i < l.s.len() && is_sym(l.s[l.i]) {
                l.i += 1;
            }
            let name = std::str::from_utf8(&l.s[start..l.i]).unwrap();
            Ok(Expr::Sym(scoped(name, sc)))
        }
        _ => Err(format!("unexpected `{}`", c as char)),
    }
}

/// A local label (`.name` or `@name`) belongs to the global label before it.
pub fn scoped(name: &str, scope: &str) -> String {
    if name.starts_with('.') || name.starts_with('@') { format!("{scope}{name}") } else { name.to_string() }
}

pub struct Ctx<'a> {
    pub symbols: &'a HashMap<String, i64>,
    pub pc: i64,
}

/// The expression's value, or none while a symbol in it is unknown or a division is by zero.
pub fn eval(e: &Expr, c: &Ctx) -> Option<i64> {
    Some(match e {
        Expr::Num(n) => *n,
        Expr::Sym(s) => *c.symbols.get(s)?,
        Expr::Pc => c.pc,
        Expr::Neg(a) => eval(a, c)?.wrapping_neg(),
        Expr::Not(a) => !eval(a, c)?,
        Expr::LNot(a) => (eval(a, c)? == 0) as i64,
        Expr::Bin(op, a, b) => {
            let (a, b) = (eval(a, c)?, eval(b, c)?);
            match op {
                '+' => a.wrapping_add(b),
                '-' => a.wrapping_sub(b),
                '*' => a.wrapping_mul(b),
                '/' => a.checked_div(b)?,
                '%' => a.checked_rem(b)?,
                '&' => a & b,
                '|' => a | b,
                '^' => a ^ b,
                '<' => a.wrapping_shl(b as u32),
                '>' => a.wrapping_shr(b as u32),
                'e' => (a == b) as i64,
                'n' => (a != b) as i64,
                'l' => (a <= b) as i64,
                'g' => (a >= b) as i64,
                'L' => (a < b) as i64,
                _ => (a > b) as i64,
            }
        }
    })
}

pub fn fits_signed(v: i64, bits: u32) -> bool {
    let lim = 1i64 << (bits - 1);
    (-lim..lim).contains(&v)
}

/// A value that fits `bits` read as signed or as unsigned.
pub fn fits_either(v: i64, bits: u32) -> bool {
    let lim = 1i64 << (bits - 1);
    (-lim..(lim << 1)).contains(&v)
}

// ---------------------------------------------------------------------------------------------------------------------
// Lines

/// The operands, split at the commas outside parentheses and quotes, each trimmed; a `'` after a letter or digit is
/// part of a name (the Z80's `AF'`), not a quote.
pub fn split_top(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut depth, mut quote, mut cur) = (0i32, None::<char>, String::new());
    let mut prev = ' ';
    for c in s.chars() {
        let after_word = std::mem::replace(&mut prev, c).is_ascii_alphanumeric();
        match (quote, c) {
            (Some(q), c) if c == q => {
                quote = None;
                cur.push(c);
            }
            (Some(_), c) => cur.push(c),
            (None, '\'') if after_word => cur.push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                cur.push(c);
            }
            (None, '(') => {
                depth += 1;
                cur.push(c);
            }
            (None, ')') => {
                depth -= 1;
                cur.push(c);
            }
            (None, ',') if depth == 0 => out.push(std::mem::take(&mut cur)),
            (None, c) => cur.push(c),
        }
    }
    if !cur.trim().is_empty() || !out.is_empty() {
        out.push(cur);
    }
    out.into_iter().map(|s| s.trim().to_string()).collect()
}

/// The `(` that opens an operand's final parenthesised group.
pub fn top_open(t: &str) -> Option<usize> {
    let b = t.as_bytes();
    let mut depth = 0;
    for i in (0..b.len()).rev() {
        match b[i] {
            b')' => depth += 1,
            b'(' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// The line without its comment: from a `;` outside quotes, or the whole line when it starts with `*`.
pub fn strip_comment(s: &str) -> &str {
    if s.trim_start().starts_with('*') {
        return "";
    }
    let mut quote = None;
    let mut prev = ' ';
    for (i, c) in s.char_indices() {
        let after_word = std::mem::replace(&mut prev, c).is_ascii_alphanumeric();
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (None, '\'') if after_word => {}
            (None, '\'' | '"') => quote = Some(c),
            (None, ';') => return &s[..i],
            _ => {}
        }
    }
    s
}

/// A label starts in the first column, or ends with `:`.
pub fn split_label(s: &str) -> (Option<&str>, &str) {
    let first = s.chars().next().is_some_and(|c| !c.is_whitespace());
    let t = s.trim_start();
    let end = t.find(|c: char| c.is_whitespace()).unwrap_or(t.len());
    let tok = &t[..end];
    if let Some(name) = tok.strip_suffix(':') {
        return (Some(name), &t[end..]);
    }
    if first {
        return (Some(tok), &t[end..]);
    }
    (None, t)
}

pub fn split_word(s: &str) -> (&str, &str) {
    let t = s.trim_start();
    let end = t.find(|c: char| c.is_whitespace()).unwrap_or(t.len());
    (&t[..end], t[end..].trim())
}

// ---------------------------------------------------------------------------------------------------------------------
// Statements

/// A statement: a label, a constant, or a line the processor's assembler read.
#[derive(Clone, Debug)]
pub enum Kind<K> {
    Label(String),
    Equ(String, Expr),
    Line(K),
}

#[derive(Clone, Debug)]
pub struct Stmt<K> {
    pub line: usize,
    pub kind: Kind<K>,
}

/// A line's word and operands, handed to the processor's assembler: the word lower-cased, the label scope it is in.
pub struct Word<'a> {
    pub word: &'a str,
    pub args: &'a str,
    pub scope: &'a str,
    pub line: usize,
}

impl Word<'_> {
    pub fn expr(&self, s: &str) -> Result<Expr, AsmError> {
        parse_expr(s, self.scope).map_err(|m| AsmError { line: self.line, message: m })
    }
}

/// The source's statements, conditionals resolved with the constants known in order and those given; `if`, `else`,
/// `endif` (`endc`), `include` and the words in `constants` (`equ`, `=`, and the 68000's `set`, an instruction on the
/// Z80) are read here, every other word by `word`.
pub fn parse<K>(source: &str, defines: &HashMap<String, i64>, include: &dyn Fn(&str) -> Option<String>, out: &mut Vec<Stmt<K>>, depth: usize, constants: &[&str], word: &dyn Fn(&Word) -> Result<K, AsmError>) -> Result<(), AsmError> {
    if depth > 16 {
        return err(0, "includes nested too deeply");
    }
    let mut scope = String::new();
    let mut consts = defines.clone();
    // Each open `if`: whether its lines are taken, and whether a branch was taken.
    let mut conds: Vec<(bool, bool)> = Vec::new();
    for (n, raw) in source.lines().enumerate() {
        let line = n + 1;
        let text = strip_comment(raw);
        if text.trim().is_empty() {
            continue;
        }
        let (label, rest) = split_label(text);
        let (w, args) = split_word(rest);
        let w = w.to_ascii_lowercase();
        let active = conds.iter().all(|c| c.0);
        match w.as_str() {
            "if" => {
                let v = if active {
                    let e = parse_expr(args, &scope).map_err(|m| AsmError { line, message: m })?;
                    eval(&e, &Ctx { symbols: &consts, pc: 0 }).ok_or(AsmError { line, message: "a condition must be a known constant".into() })? != 0
                } else {
                    false
                };
                conds.push((v, v));
                continue;
            }
            "else" => {
                let c = conds.last_mut().ok_or(AsmError { line, message: "`else` without `if`".into() })?;
                c.0 = !c.1;
                c.1 = true;
                continue;
            }
            "endif" | "endc" => {
                conds.pop().ok_or(AsmError { line, message: "`endif` without `if`".into() })?;
                continue;
            }
            _ => {}
        }
        if !active {
            continue;
        }
        if constants.contains(&w.as_str()) {
            let name = label.ok_or(AsmError { line, message: "a constant needs a name".into() })?;
            let e = parse_expr(args, &scope).map_err(|m| AsmError { line, message: m })?;
            let name = scoped(name, &scope);
            if let Some(v) = eval(&e, &Ctx { symbols: &consts, pc: 0 }) {
                consts.insert(name.clone(), v);
            }
            out.push(Stmt { line, kind: Kind::Equ(name, e) });
            continue;
        }
        if let Some(l) = label {
            if !l.starts_with('.') && !l.starts_with('@') {
                scope = l.to_string();
            }
            out.push(Stmt { line, kind: Kind::Label(scoped(l, &scope)) });
        }
        if w.is_empty() {
            continue;
        }
        if w == "include" {
            let name = args.trim().trim_matches('"');
            let text = include(name).ok_or(AsmError { line, message: format!("cannot include `{name}`") })?;
            let mut inner = Vec::new();
            parse(&text, &consts, include, &mut inner, depth + 1, constants, word).map_err(|x| AsmError { line, message: format!("in {name}: {x}") })?;
            for s in &inner {
                if let Kind::Equ(n, x) = &s.kind {
                    if let Some(v) = eval(x, &Ctx { symbols: &consts, pc: 0 }) {
                        consts.insert(n.clone(), v);
                    }
                }
            }
            out.extend(inner.into_iter().map(|s| Stmt { line, kind: s.kind }));
            continue;
        }
        let kind = word(&Word { word: &w, args, scope: &scope, line })?;
        out.push(Stmt { line, kind: Kind::Line(kind) });
    }
    if !conds.is_empty() {
        return err(0, "an `if` is not closed");
    }
    Ok(())
}

/// Places bytes into the program's image from their addresses, refusing any before the origin.
pub fn image(origin: u32, bytes: BTreeMap<i64, u8>, symbols: HashMap<String, i64>) -> Result<Program, AsmError> {
    let end = bytes.keys().next_back().map_or(origin as i64, |k| k + 1);
    let mut out = vec![0u8; (end - origin as i64).max(0) as usize];
    for (k, v) in bytes {
        if k < origin as i64 {
            return err(0, "bytes placed before the origin");
        }
        out[(k - origin as i64) as usize] = v;
    }
    Ok(Program { origin, bytes: out, symbols: symbols.into_iter().collect() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(text: &str, pc: i64) -> Option<i64> {
        let symbols = HashMap::from([("start".to_string(), 0x200), ("start.loop".to_string(), 0x208)]);
        eval(&parse_expr(text, "start").ok()?, &Ctx { symbols: &symbols, pc })
    }

    #[test]
    fn expressions_follow_c_s_order() {
        assert_eq!(value("1+2*3", 0), Some(7));
        assert_eq!(value("(1+2)*3", 0), Some(9));
        assert_eq!(value("$FF & %1010 | 1 << 4", 0), Some(0x1A));
        assert_eq!(value("0x10 - 'A'", 0), Some(16 - 65));
        assert_eq!(value("start+2", 0), Some(0x202));
        assert_eq!(value(".loop-start", 0), Some(8), "a local label belongs to its scope");
        assert_eq!((value("*+2", 0x100), value("$+2", 0x100), value("$2", 0x100)), (Some(0x102), Some(0x102), Some(2)));
        assert_eq!(value("3 = 3", 0), Some(1));
        assert_eq!(value("1/0", 0), None);
        assert_eq!(value("unknown", 0), None);
        assert!(parse_expr("1 +", "").is_err());
    }

    #[test]
    fn lines_split_into_label_word_and_operands() {
        assert_eq!(split_label("start: ld a,1"), (Some("start"), " ld a,1"));
        assert_eq!(split_label("  ld a,1"), (None, "ld a,1"));
        assert_eq!(split_word("ld a,(ix+2)"), ("ld", "a,(ix+2)"));
        assert_eq!(split_top("a, (1,2), 'x,y'"), ["a", "(1,2)", "'x,y'"]);
        assert_eq!(strip_comment(" db ';' ; a comment"), " db ';' ");
        assert_eq!(strip_comment(" ex af,af' ; swap"), " ex af,af' ");
        assert_eq!(split_top("af,af'"), ["af", "af'"]);
        assert_eq!(top_open("2*(3+4)(a0)"), Some(7));
    }

    #[test]
    fn conditionals_includes_and_constants_are_read_here() {
        let src = "N equ 2\n if N=2\nx: yes\n else\n no\n endif\n include \"more\"\n";
        let mut out = Vec::new();
        let words = |w: &Word| -> Result<String, AsmError> { Ok(format!("{}@{}", w.word, w.line)) };
        parse(src, &HashMap::new(), &|n| (n == "more").then(|| " also\n".to_string()), &mut out, 0, &["equ"], &words).unwrap();
        let kinds: Vec<String> = out.iter().map(|s| match &s.kind {
            Kind::Label(l) => format!("label {l}"),
            Kind::Equ(n, _) => format!("equ {n}"),
            Kind::Line(k) => k.clone(),
        }).collect();
        assert_eq!(kinds, ["equ N", "label x", "yes@3", "also@1"]);
        assert_eq!(out[3].line, 7, "an included line carries the include's");
        assert_eq!(parse(" if 1\n", &HashMap::new(), &|_| None, &mut Vec::<Stmt<String>>::new(), 0, &["equ"], &words).unwrap_err().line, 0);
    }
}

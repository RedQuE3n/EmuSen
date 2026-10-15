//! The 68000 assembler, a development tool for test programs and the open firmware: Motorola's syntax as the
//! Programmer's Reference Manual writes it and as `disasm` prints it, with labels, constants, conditionals and data
//! (Beryl_M68k.md §8). It encodes what the manual's tables name and nothing synthetic beyond the usual choices of a
//! form (ADD #n becomes ADDI, MOVE to An becomes MOVEA, an unsized branch the short form when its target is near).

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

fn err<T>(line: usize, message: impl Into<String>) -> Result<T, AsmError> {
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
enum Expr {
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

fn is_sym_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'.' || c == b'@'
}

fn is_sym(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'@'
}

fn parse_expr(text: &str, scope: &str) -> Result<Expr, String> {
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
fn scoped(name: &str, scope: &str) -> String {
    if name.starts_with('.') || name.starts_with('@') { format!("{scope}{name}") } else { name.to_string() }
}

struct Ctx<'a> {
    symbols: &'a HashMap<String, i64>,
    pc: i64,
}

fn eval(e: &Expr, c: &Ctx) -> Option<i64> {
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

// ---------------------------------------------------------------------------------------------------------------------
// Operands

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sz {
    B,
    W,
    L,
}

impl Sz {
    fn bits(self) -> u16 {
        match self {
            Sz::B => 0,
            Sz::W => 1,
            Sz::L => 2,
        }
    }
}

#[derive(Clone, Debug)]
struct Index {
    reg: u16,
    addr: bool,
    long: bool,
}

#[derive(Clone, Debug)]
enum Op {
    D(u16),
    A(u16),
    Ind(u16),
    Post(u16),
    Pre(u16),
    Disp(Expr, u16),
    Idx(Expr, u16, Index),
    AbsW(Expr),
    AbsL(Expr),
    /// An absolute address with no size: word or long, chosen once (§8.2).
    Abs(Expr),
    PcDisp(Expr),
    PcIdx(Expr, Index),
    Imm(Expr),
    Sr,
    Ccr,
    Usp,
    List(u16),
}

fn reg(t: &str) -> Option<(bool, u16)> {
    let u = t.trim().to_ascii_uppercase();
    if u == "SP" {
        return Some((true, 7));
    }
    let b = u.as_bytes();
    if b.len() == 2 && (b[0] == b'D' || b[0] == b'A') && (b'0'..=b'7').contains(&b[1]) {
        return Some((b[0] == b'A', (b[1] - b'0') as u16));
    }
    None
}

/// `D0-D3/A0/A6-A7`: bit 0 D0 to bit 15 A7.
fn reglist(t: &str) -> Option<u16> {
    let mut mask = 0u16;
    for part in t.split('/') {
        let (a, b) = match part.split_once('-') {
            Some((a, b)) => (reg(a)?, reg(b)?),
            None => (reg(part)?, reg(part)?),
        };
        let (from, to) = (a.0 as u16 * 8 + a.1, b.0 as u16 * 8 + b.1);
        if from > to {
            return None;
        }
        for r in from..=to {
            mask |= 1 << r;
        }
    }
    Some(mask)
}

fn index(t: &str) -> Option<Index> {
    let u = t.trim().to_ascii_uppercase();
    let (r, long) = if let Some(r) = u.strip_suffix(".L") {
        (r, true)
    } else if let Some(r) = u.strip_suffix(".W") {
        (r, false)
    } else {
        (u.as_str(), false)
    };
    let (addr, reg) = reg(r)?;
    Some(Index { reg, addr, long })
}

fn split_top(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut depth, mut quote, mut cur) = (0i32, None::<char>, String::new());
    for c in s.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => {
                quote = None;
                cur.push(c);
            }
            (Some(_), c) => cur.push(c),
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

fn parse_operand(t: &str, sc: &str) -> Result<Op, String> {
    let u = t.to_ascii_uppercase();
    match u.as_str() {
        "SR" => return Ok(Op::Sr),
        "CCR" => return Ok(Op::Ccr),
        "USP" => return Ok(Op::Usp),
        _ => {}
    }
    if let Some(i) = t.strip_prefix('#') {
        return Ok(Op::Imm(parse_expr(i, sc)?));
    }
    if let Some((a, r)) = reg(t) {
        return Ok(if a { Op::A(r) } else { Op::D(r) });
    }
    if (t.contains('/') || t.contains('-')) && !t.contains('(') {
        if let Some(m) = reglist(t) {
            return Ok(Op::List(m));
        }
    }
    if let Some(inner) = u.strip_prefix("-(").and_then(|s| s.strip_suffix(')')) {
        if let Some((true, r)) = reg(inner) {
            return Ok(Op::Pre(r));
        }
    }
    if let Some(inner) = u.strip_prefix('(').and_then(|s| s.strip_suffix(")+")) {
        if let Some((true, r)) = reg(inner) {
            return Ok(Op::Post(r));
        }
    }
    // (expr).W / (expr).L / expr.W / expr.L
    for (suf, long) in [(".W", false), (".L", true)] {
        if u.len() > 2 && u.ends_with(suf) && !u.contains("(PC") && !u.contains("(A") && !u.contains("(SP") {
            let e = parse_expr(&t[..t.len() - 2], sc)?;
            return Ok(if long { Op::AbsL(e) } else { Op::AbsW(e) });
        }
    }
    // d(An), d(An,Xn), (d,An), (d,An,Xn), (An), (An,Xn) and the PC forms.
    if u.ends_with(')') {
        let open = top_open(t).ok_or_else(|| format!("bad operand `{t}`"))?;
        let outer = t[..open].trim();
        let inside = split_top(&t[open + 1..t.len() - 1]);
        let mut parts: Vec<String> = inside;
        let mut disp: Option<String> = if outer.is_empty() { None } else { Some(outer.to_string()) };
        // (d,An...) puts the displacement first inside.
        if disp.is_none() && parts.len() >= 2 && reg(&parts[0]).is_none() && !parts[0].eq_ignore_ascii_case("PC") {
            disp = Some(parts.remove(0));
        }
        let base = parts.first().ok_or_else(|| format!("bad operand `{t}`"))?.to_ascii_uppercase();
        let idx = match parts.get(1) {
            Some(x) => Some(index(x).ok_or_else(|| format!("bad index `{x}`"))?),
            None => None,
        };
        if parts.len() > 2 {
            return Err(format!("bad operand `{t}`"));
        }
        let d = match &disp {
            Some(s) => parse_expr(s, sc)?,
            None => Expr::Num(0),
        };
        if base == "PC" {
            return Ok(match idx {
                Some(x) => Op::PcIdx(d, x),
                None if disp.is_none() => return Err(format!("`{t}` needs its target")),
                None => Op::PcDisp(d),
            });
        }
        let (a, r) = reg(&base).ok_or_else(|| format!("bad base register in `{t}`"))?;
        if !a {
            return Err(format!("`{t}` needs an address register"));
        }
        return Ok(match (idx, disp) {
            (Some(x), _) => Op::Idx(d, r, x),
            (None, None) => Op::Ind(r),
            (None, Some(_)) => Op::Disp(d, r),
        });
    }
    Ok(Op::Abs(parse_expr(t, sc)?))
}

/// The `(` that opens an operand's final parenthesised group.
fn top_open(t: &str) -> Option<usize> {
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

// ---------------------------------------------------------------------------------------------------------------------
// Statements

#[derive(Clone, Debug)]
enum Data {
    Expr(Expr),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug)]
enum Kind {
    Label(String),
    Equ(String, Expr),
    Org(Expr),
    Even,
    Align(Expr),
    Dc(Sz, Vec<Data>),
    Ds(Sz, Expr, Expr),
    Inst { name: String, size: Option<char>, ops: Vec<Op> },
}

#[derive(Clone, Debug)]
struct Stmt {
    line: usize,
    kind: Kind,
}

/// The source's statements, conditionals resolved with the constants known in order and those given.
fn parse(source: &str, defines: &HashMap<String, i64>, include: &dyn Fn(&str) -> Option<String>, out: &mut Vec<Stmt>, depth: usize) -> Result<(), AsmError> {
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
        let (word, args) = split_word(rest);
        let w = word.to_ascii_lowercase();
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
        if matches!(w.as_str(), "equ" | "=" | "set") {
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
        if word.is_empty() {
            continue;
        }
        let e = |s: &str| parse_expr(s, &scope).map_err(|m| AsmError { line, message: m });
        let (base, size) = match w.rsplit_once('.') {
            Some((b, s)) if !b.is_empty() && s.len() == 1 => (b.to_string(), s.chars().next()),
            _ => (w.clone(), None),
        };
        let data_size = |s: Option<char>| match s {
            Some('b') => Ok(Sz::B),
            Some('w') | None => Ok(Sz::W),
            Some('l') => Ok(Sz::L),
            _ => err(line, "bad size"),
        };
        let kind = match base.as_str() {
            "org" => Kind::Org(e(args)?),
            "even" => Kind::Even,
            "align" => Kind::Align(e(args)?),
            "include" => {
                let name = args.trim().trim_matches('"');
                let text = include(name).ok_or(AsmError { line, message: format!("cannot include `{name}`") })?;
                let mut inner = Vec::new();
                parse(&text, &consts, include, &mut inner, depth + 1).map_err(|x| AsmError { line, message: format!("in {name}: {x}") })?;
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
            "dc" => {
                let sz = data_size(size)?;
                let mut items = Vec::new();
                for a in split_top(args) {
                    let q = a.as_bytes();
                    if q.len() >= 2 && (q[0] == b'"' || (q[0] == b'\'' && sz == Sz::B)) && q[q.len() - 1] == q[0] {
                        if sz != Sz::B {
                            return err(line, "a string is a list of bytes: dc.b");
                        }
                        items.push(Data::Bytes(q[1..q.len() - 1].to_vec()));
                    } else {
                        items.push(Data::Expr(e(&a)?));
                    }
                }
                Kind::Dc(sz, items)
            }
            "ds" | "dcb" => {
                let sz = data_size(size)?;
                let a = split_top(args);
                let fill = if a.len() > 1 { e(&a[1])? } else { Expr::Num(0) };
                Kind::Ds(sz, e(a.first().map(String::as_str).unwrap_or(""))?, fill)
            }
            _ => {
                let ops = if args.trim().is_empty() { Vec::new() } else { split_top(args).iter().map(|o| parse_operand(o, &scope).map_err(|m| AsmError { line, message: m })).collect::<Result<_, _>>()? };
                Kind::Inst { name: base.to_ascii_uppercase(), size, ops }
            }
        };
        out.push(Stmt { line, kind });
    }
    if !conds.is_empty() {
        return err(0, "an `if` is not closed");
    }
    Ok(())
}

fn strip_comment(s: &str) -> &str {
    if s.trim_start().starts_with('*') {
        return "";
    }
    let mut quote = None;
    for (i, c) in s.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (None, '\'' | '"') => quote = Some(c),
            (None, ';') => return &s[..i],
            _ => {}
        }
    }
    s
}

/// A label starts in the first column, or ends with `:`.
fn split_label(s: &str) -> (Option<&str>, &str) {
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

fn split_word(s: &str) -> (&str, &str) {
    let t = s.trim_start();
    let end = t.find(|c: char| c.is_whitespace()).unwrap_or(t.len());
    (&t[..end], t[end..].trim())
}

// ---------------------------------------------------------------------------------------------------------------------
// Encoding

const CONDS: [&str; 16] = ["T", "F", "HI", "LS", "CC", "CS", "NE", "EQ", "VC", "VS", "PL", "MI", "GE", "LT", "GT", "LE"];

fn cond(s: &str) -> Option<u16> {
    match s {
        "HS" => Some(4),
        "LO" => Some(5),
        "RA" => Some(1),
        _ => CONDS.iter().position(|c| *c == s).map(|i| i as u16),
    }
}

/// What encoding needs beyond the statement: the symbols, and whether values must be final.
struct Enc<'a> {
    ctx: Ctx<'a>,
    fin: bool,
    line: usize,
    /// For each unsized absolute operand, in order: whether it takes the word form.
    abs_short: &'a mut Vec<bool>,
    abs_next: usize,
    /// An unsized branch's form: true for the short one.
    branch_short: bool,
}

impl Enc<'_> {
    fn val(&self, e: &Expr) -> Result<i64, AsmError> {
        match eval(e, &self.ctx) {
            Some(v) => Ok(v),
            None if self.fin => err(self.line, format!("undefined symbol in `{e:?}`").replace("Sym(\"", "").replace("\")", "")),
            None => Ok(0),
        }
    }

    fn check(&self, ok: bool, what: &str) -> Result<(), AsmError> {
        if ok || !self.fin { Ok(()) } else { err(self.line, what.to_string()) }
    }

    fn abs_choice(&mut self, e: &Expr) -> bool {
        if self.abs_next < self.abs_short.len() {
            let c = self.abs_short[self.abs_next];
            self.abs_next += 1;
            return c;
        }
        let short = eval(e, &self.ctx).is_some_and(fits_short_abs);
        self.abs_short.push(short);
        self.abs_next += 1;
        short
    }
}

/// An address the word form reaches: its sign extension, over the 68000's 24 address lines.
fn fits_short_abs(v: i64) -> bool {
    let a = v as u32 & 0xFF_FFFF;
    a < 0x8000 || a >= 0xFF_8000
}

/// The distance from `from` to `to` over the 68000's 24 address lines.
fn rel(to: i64, from: i64) -> i64 {
    (to - from) << 40 >> 40
}

fn fits_signed(v: i64, bits: u32) -> bool {
    let lim = 1i64 << (bits - 1);
    (-lim..lim).contains(&v)
}

fn fits_either(v: i64, bits: u32) -> bool {
    let lim = 1i64 << (bits - 1);
    (-lim..(lim << 1)).contains(&v)
}

/// An operand's mode and register, and its extension words after the opcode, the first at `ext_pc`.
fn ea(en: &mut Enc, op: &Op, sz: Sz, ext_pc: i64) -> Result<(u16, Vec<u16>), AsmError> {
    let brief = |en: &Enc, d: i64, x: &Index, base: u16| -> Result<(u16, Vec<u16>), AsmError> {
        en.check(fits_signed(d, 8), "index displacement out of range")?;
        let w = (x.addr as u16) << 15 | x.reg << 12 | (x.long as u16) << 11 | (d as u16 & 0xFF);
        Ok((base, vec![w]))
    };
    Ok(match op {
        Op::D(r) => (*r, vec![]),
        Op::A(r) => (8 | r, vec![]),
        Op::Ind(r) => (16 | r, vec![]),
        Op::Post(r) => (24 | r, vec![]),
        Op::Pre(r) => (32 | r, vec![]),
        Op::Disp(e, r) => {
            let d = en.val(e)?;
            en.check(fits_signed(d, 16), "displacement out of range")?;
            (40 | r, vec![d as u16])
        }
        Op::Idx(e, r, x) => {
            let d = en.val(e)?;
            brief(en, d, x, 48 | r)?
        }
        Op::AbsW(e) => {
            let v = en.val(e)?;
            en.check(fits_short_abs(v) || (0..=0xFFFF).contains(&v), "address out of the word form's reach")?;
            (56, vec![v as u16])
        }
        Op::AbsL(e) => {
            let v = en.val(e)?;
            (57, vec![(v >> 16) as u16, v as u16])
        }
        Op::Abs(e) => {
            if en.abs_choice(e) {
                let v = en.val(e)?;
                en.check(fits_short_abs(v), "address out of the word form's reach")?;
                (56, vec![v as u16])
            } else {
                let v = en.val(e)?;
                (57, vec![(v >> 16) as u16, v as u16])
            }
        }
        Op::PcDisp(e) => {
            let d = rel(en.val(e)?, ext_pc);
            en.check(fits_signed(d, 16), "PC-relative target out of range")?;
            (58, vec![d as u16])
        }
        Op::PcIdx(e, x) => {
            let d = rel(en.val(e)?, ext_pc);
            brief(en, d, x, 59)?
        }
        Op::Imm(e) => {
            let v = en.val(e)?;
            match sz {
                Sz::B => {
                    en.check(fits_either(v, 8), "immediate out of range for a byte")?;
                    (60, vec![v as u16 & 0xFF])
                }
                Sz::W => {
                    en.check(fits_either(v, 16), "immediate out of range for a word")?;
                    (60, vec![v as u16])
                }
                Sz::L => (60, vec![(v >> 16) as u16, v as u16]),
            }
        }
        _ => return err(en.line, "this operand is not an effective address"),
    })
}

#[derive(Clone, Copy)]
enum Cat {
    All,
    Data,
    DataAlt,
    MemAlt,
    Control,
    ControlAlt,
    Alt,
}

fn allowed(cat: Cat, op: &Op) -> bool {
    let (m, r) = match op {
        Op::D(_) => (0, 0),
        Op::A(_) => (1, 0),
        Op::Ind(_) => (2, 0),
        Op::Post(_) => (3, 0),
        Op::Pre(_) => (4, 0),
        Op::Disp(..) => (5, 0),
        Op::Idx(..) => (6, 0),
        Op::AbsW(_) | Op::Abs(_) => (7, 0),
        Op::AbsL(_) => (7, 1),
        Op::PcDisp(_) => (7, 2),
        Op::PcIdx(..) => (7, 3),
        Op::Imm(_) => (7, 4),
        _ => return false,
    };
    let pc_imm = m == 7 && r >= 2;
    match cat {
        Cat::All => true,
        Cat::Data => m != 1,
        Cat::DataAlt => m != 1 && !pc_imm,
        Cat::MemAlt => m >= 2 && !pc_imm,
        Cat::Control => matches!(m, 2 | 5 | 6 | 7) && !(m == 7 && r == 4),
        Cat::ControlAlt => matches!(m, 2 | 5 | 6 | 7) && !pc_imm,
        Cat::Alt => !pc_imm,
    }
}

/// The words of one instruction at `pc`.
fn encode(en: &mut Enc, name: &str, size: Option<char>, ops: &[Op], pc: i64) -> Result<Vec<u16>, AsmError> {
    let line = en.line;
    let bad = || err(line, format!("`{name}` does not take these operands"));
    let sz = |default: Sz| -> Result<Sz, AsmError> {
        match size {
            None => Ok(default),
            Some('b') => Ok(Sz::B),
            Some('w') => Ok(Sz::W),
            Some('l') => Ok(Sz::L),
            Some(c) => err(line, format!("bad size `.{c}`")),
        }
    };
    let need = |n: usize| if ops.len() == n { Ok(()) } else { err(line, format!("`{name}` takes {n} operand(s)")) };
    // Opcode, then the operands' extensions in order.
    let mut out: Vec<u16> = vec![0];
    macro_rules! ea_at {
        ($op:expr, $s:expr) => {{
            let at = pc + 2 * out.len() as i64;
            let (b, w) = ea(en, $op, $s, at)?;
            out.extend(w);
            b
        }};
    }
    let n = name;
    // Branches.
    if is_branch(n) {
        need(1)?;
        let c = if n == "BRA" { 0 } else if n == "BSR" { 1 } else { cond(&n[1..]).unwrap() };
        let (Op::Abs(t) | Op::AbsW(t) | Op::AbsL(t)) = &ops[0] else { return bad() };
        let d = rel(en.val(t)?, pc + 2);
        let short = match size {
            Some('s' | 'b') => true,
            Some('w') => false,
            None => en.branch_short,
            _ => return err(line, "a 68000 branch is .S or .W"),
        };
        if short {
            en.check(fits_signed(d, 8) && d != 0, "short branch out of range")?;
            out[0] = 0x6000 | c << 8 | (d as u16 & 0xFF);
        } else {
            en.check(fits_signed(d, 16), "branch out of range")?;
            out[0] = 0x6000 | c << 8;
            out.push(d as u16);
        }
        return Ok(out);
    }
    if n.starts_with("DB") && (n == "DBRA" || cond(&n[2..]).is_some()) {
        need(2)?;
        let c = if n == "DBRA" { 1 } else { cond(&n[2..]).unwrap() };
        let Op::D(r) = ops[0] else { return bad() };
        let (Op::Abs(t) | Op::AbsW(t) | Op::AbsL(t)) = &ops[1] else { return bad() };
        let d = rel(en.val(t)?, pc + 2);
        en.check(fits_signed(d, 16), "branch out of range")?;
        out[0] = 0x50C8 | c << 8 | r;
        out.push(d as u16);
        return Ok(out);
    }
    if n.len() >= 2 && n.starts_with('S') && cond(&n[1..]).is_some() && n != "SUB" && n != "SBCD" && n != "SWAP" && n != "STOP" && n != "SUBA" && n != "SUBX" && n != "SUBI" && n != "SUBQ" {
        need(1)?;
        if !allowed(Cat::DataAlt, &ops[0]) {
            return bad();
        }
        let c = cond(&n[1..]).unwrap();
        let e = ea_at!(&ops[0], Sz::B);
        out[0] = 0x50C0 | c << 8 | e;
        return Ok(out);
    }
    match n {
        "NOP" | "RESET" | "RTE" | "RTS" | "TRAPV" | "RTR" | "ILLEGAL" => {
            need(0)?;
            out[0] = match n {
                "NOP" => 0x4E71,
                "RESET" => 0x4E70,
                "RTE" => 0x4E73,
                "RTS" => 0x4E75,
                "TRAPV" => 0x4E76,
                "RTR" => 0x4E77,
                _ => 0x4AFC,
            };
        }
        "STOP" => {
            need(1)?;
            let Op::Imm(e) = &ops[0] else { return bad() };
            let v = en.val(e)?;
            out[0] = 0x4E72;
            out.push(v as u16);
        }
        "TRAP" => {
            need(1)?;
            let Op::Imm(e) = &ops[0] else { return bad() };
            let v = en.val(e)?;
            en.check((0..16).contains(&v), "TRAP's vector is 0 to 15")?;
            out[0] = 0x4E40 | (v as u16 & 15);
        }
        "LINK" => {
            need(2)?;
            let (Op::A(r), Op::Imm(e)) = (&ops[0], &ops[1]) else { return bad() };
            let v = en.val(e)?;
            en.check(fits_signed(v, 16), "LINK's displacement out of range")?;
            out[0] = 0x4E50 | r;
            out.push(v as u16);
        }
        "UNLK" => {
            need(1)?;
            let Op::A(r) = ops[0] else { return bad() };
            out[0] = 0x4E58 | r;
        }
        "SWAP" => {
            need(1)?;
            let Op::D(r) = ops[0] else { return bad() };
            out[0] = 0x4840 | r;
        }
        "EXT" => {
            need(1)?;
            let Op::D(r) = ops[0] else { return bad() };
            out[0] = if sz(Sz::W)? == Sz::L { 0x48C0 } else { 0x4880 } | r;
        }
        "JMP" | "JSR" | "PEA" => {
            need(1)?;
            if !allowed(Cat::Control, &ops[0]) {
                return bad();
            }
            let e = ea_at!(&ops[0], Sz::L);
            out[0] = match n {
                "JMP" => 0x4EC0,
                "JSR" => 0x4E80,
                _ => 0x4840,
            } | e;
        }
        "LEA" => {
            need(2)?;
            let Op::A(r) = ops[1] else { return bad() };
            if !allowed(Cat::Control, &ops[0]) {
                return bad();
            }
            let e = ea_at!(&ops[0], Sz::L);
            out[0] = 0x41C0 | r << 9 | e;
        }
        "CHK" => {
            need(2)?;
            let Op::D(r) = ops[1] else { return bad() };
            if !allowed(Cat::Data, &ops[0]) {
                return bad();
            }
            let e = ea_at!(&ops[0], Sz::W);
            out[0] = 0x4180 | r << 9 | e;
        }
        "NBCD" | "TAS" => {
            need(1)?;
            if !allowed(Cat::DataAlt, &ops[0]) {
                return bad();
            }
            let e = ea_at!(&ops[0], Sz::B);
            out[0] = if n == "NBCD" { 0x4800 } else { 0x4AC0 } | e;
        }
        "NEGX" | "CLR" | "NEG" | "NOT" | "TST" => {
            need(1)?;
            let s = sz(Sz::W)?;
            if !allowed(Cat::DataAlt, &ops[0]) {
                return bad();
            }
            let e = ea_at!(&ops[0], s);
            let base = match n {
                "NEGX" => 0x4000,
                "CLR" => 0x4200,
                "NEG" => 0x4400,
                "NOT" => 0x4600,
                _ => 0x4A00,
            };
            out[0] = base | s.bits() << 6 | e;
        }
        "MOVEQ" => {
            need(2)?;
            let (Op::Imm(e), Op::D(r)) = (&ops[0], &ops[1]) else { return bad() };
            let v = en.val(e)?;
            en.check(fits_either(v, 8), "MOVEQ's value out of range")?;
            out[0] = 0x7000 | r << 9 | (v as u16 & 0xFF);
        }
        "ADDQ" | "SUBQ" => {
            need(2)?;
            let s = sz(Sz::W)?;
            let Op::Imm(e) = &ops[0] else { return bad() };
            let v = en.val(e)?;
            en.check((1..=8).contains(&v), "quick data is 1 to 8")?;
            if !allowed(Cat::Alt, &ops[1]) || (s == Sz::B && matches!(ops[1], Op::A(_))) {
                return bad();
            }
            let d = ea_at!(&ops[1], s);
            out[0] = if n == "ADDQ" { 0x5000 } else { 0x5100 } | (v as u16 & 7) << 9 | s.bits() << 6 | d;
        }
        "MOVEM" => {
            need(2)?;
            let s = sz(Sz::W)?;
            if s == Sz::B {
                return bad();
            }
            let list = |o: &Op| -> Option<u16> {
                match o {
                    Op::List(m) => Some(*m),
                    Op::D(r) => Some(1 << r),
                    Op::A(r) => Some(1 << (r + 8)),
                    _ => None,
                }
            };
            let raw = |en: &Enc, o: &Op| -> Result<Option<u16>, AsmError> {
                if let Op::Imm(e) = o { Ok(Some(en.val(e)? as u16)) } else { Ok(None) }
            };
            // Registers to memory when the first operand is the list.
            let to_mem = list(&ops[0]).is_some() && !matches!(ops[0], Op::D(_) | Op::A(_)) || matches!(ops[0], Op::Imm(_)) || (matches!(ops[0], Op::D(_) | Op::A(_)) && list(&ops[1]).is_none());
            if to_mem {
                let dst = &ops[1];
                if !(allowed(Cat::ControlAlt, dst) || matches!(dst, Op::Pre(_))) {
                    return bad();
                }
                let m = match raw(en, &ops[0])? {
                    Some(m) => m,
                    None => {
                        let m = list(&ops[0]).ok_or(AsmError { line, message: "bad register list".into() })?;
                        if matches!(dst, Op::Pre(_)) { m.reverse_bits() } else { m }
                    }
                };
                out.push(m);
                let e = ea_at!(dst, s);
                out[0] = 0x4880 | ((s == Sz::L) as u16) << 6 | e;
            } else {
                let src = &ops[0];
                if !(allowed(Cat::Control, src) || matches!(src, Op::Post(_))) {
                    return bad();
                }
                let m = match raw(en, &ops[1])? {
                    Some(m) => m,
                    None => list(&ops[1]).ok_or(AsmError { line, message: "bad register list".into() })?,
                };
                out.push(m);
                let e = ea_at!(src, s);
                out[0] = 0x4C80 | ((s == Sz::L) as u16) << 6 | e;
            }
        }
        "MOVEP" => {
            need(2)?;
            let s = sz(Sz::W)?;
            let l = (s == Sz::L) as u16;
            let (to_mem, d, mem) = match (&ops[0], &ops[1]) {
                (Op::D(d), m) => (true, *d, m),
                (m, Op::D(d)) => (false, *d, m),
                _ => return bad(),
            };
            let (disp, a) = match mem {
                Op::Disp(e, a) => (en.val(e)?, *a),
                Op::Ind(a) => (0, *a),
                _ => return bad(),
            };
            en.check(fits_signed(disp, 16), "displacement out of range")?;
            out[0] = if to_mem { 0x0188 } else { 0x0108 } | d << 9 | l << 6 | a;
            out.push(disp as u16);
        }
        "EXG" => {
            need(2)?;
            out[0] = match (&ops[0], &ops[1]) {
                (Op::D(x), Op::D(y)) => 0xC140 | x << 9 | y,
                (Op::A(x), Op::A(y)) => 0xC148 | x << 9 | y,
                (Op::D(x), Op::A(y)) | (Op::A(y), Op::D(x)) => 0xC188 | x << 9 | y,
                _ => return bad(),
            };
        }
        "ABCD" | "SBCD" | "ADDX" | "SUBX" => {
            need(2)?;
            let base = match n {
                "ABCD" => 0xC100,
                "SBCD" => 0x8100,
                "ADDX" => 0xD100,
                _ => 0x9100,
            };
            let s = if n.ends_with('X') { sz(Sz::W)?.bits() << 6 } else { 0 };
            out[0] = base | s | match (&ops[0], &ops[1]) {
                (Op::D(y), Op::D(x)) => x << 9 | y,
                (Op::Pre(y), Op::Pre(x)) => x << 9 | 8 | y,
                _ => return bad(),
            };
        }
        "CMPM" => {
            need(2)?;
            let (Op::Post(y), Op::Post(x)) = (&ops[0], &ops[1]) else { return bad() };
            out[0] = 0xB108 | x << 9 | sz(Sz::W)?.bits() << 6 | y;
        }
        "MULU" | "MULS" | "DIVU" | "DIVS" => {
            need(2)?;
            if size.is_some_and(|c| c != 'w') {
                return err(line, "the 68000 multiplies and divides words");
            }
            let Op::D(r) = ops[1] else { return bad() };
            if !allowed(Cat::Data, &ops[0]) {
                return bad();
            }
            let e = ea_at!(&ops[0], Sz::W);
            out[0] = match n {
                "MULU" => 0xC0C0,
                "MULS" => 0xC1C0,
                "DIVU" => 0x80C0,
                _ => 0x81C0,
            } | r << 9 | e;
        }
        "BTST" | "BCHG" | "BCLR" | "BSET" => {
            need(2)?;
            let t: u16 = match n {
                "BTST" => 0,
                "BCHG" => 1,
                "BCLR" => 2,
                _ => 3,
            };
            let cat = if t == 0 { Cat::Data } else { Cat::DataAlt };
            match &ops[0] {
                Op::D(r) => {
                    if !allowed(cat, &ops[1]) {
                        return bad();
                    }
                    let e = ea_at!(&ops[1], Sz::B);
                    out[0] = 0x0100 | r << 9 | t << 6 | e;
                }
                Op::Imm(x) => {
                    if !allowed(cat, &ops[1]) || matches!(ops[1], Op::Imm(_)) {
                        return bad();
                    }
                    let v = en.val(x)?;
                    en.check((0..=255).contains(&v), "bit number out of range")?;
                    out.push(v as u16);
                    let e = ea_at!(&ops[1], Sz::B);
                    out[0] = 0x0800 | t << 6 | e;
                }
                _ => return bad(),
            }
        }
        "ASL" | "ASR" | "LSL" | "LSR" | "ROXL" | "ROXR" | "ROL" | "ROR" => {
            let (ty, left): (u16, bool) = match n {
                "ASL" => (0, true),
                "ASR" => (0, false),
                "LSL" => (1, true),
                "LSR" => (1, false),
                "ROXL" => (2, true),
                "ROXR" => (2, false),
                "ROL" => (3, true),
                _ => (3, false),
            };
            let dir = (left as u16) << 8;
            match ops {
                [Op::D(r)] => {
                    let s = sz(Sz::W)?;
                    out[0] = 0xE000 | 1 << 9 | dir | s.bits() << 6 | ty << 3 | r;
                }
                [one] => {
                    if size.is_some_and(|c| c != 'w') || !allowed(Cat::MemAlt, one) {
                        return bad();
                    }
                    let e = ea_at!(one, Sz::W);
                    out[0] = 0xE0C0 | ty << 9 | dir | e;
                }
                [Op::Imm(c), Op::D(r)] => {
                    let s = sz(Sz::W)?;
                    let v = en.val(c)?;
                    en.check((1..=8).contains(&v), "shift count is 1 to 8")?;
                    out[0] = 0xE000 | (v as u16 & 7) << 9 | dir | s.bits() << 6 | ty << 3 | r;
                }
                [Op::D(c), Op::D(r)] => {
                    let s = sz(Sz::W)?;
                    out[0] = 0xE020 | c << 9 | dir | s.bits() << 6 | ty << 3 | r;
                }
                _ => return bad(),
            }
        }
        "MOVE" | "MOVEA" => {
            need(2)?;
            match (&ops[0], &ops[1]) {
                (Op::Sr, d) => {
                    if !allowed(Cat::DataAlt, d) {
                        return bad();
                    }
                    let e = ea_at!(d, Sz::W);
                    out[0] = 0x40C0 | e;
                }
                (s, Op::Ccr) => {
                    if !allowed(Cat::Data, s) {
                        return bad();
                    }
                    let e = ea_at!(s, Sz::W);
                    out[0] = 0x44C0 | e;
                }
                (s, Op::Sr) => {
                    if !allowed(Cat::Data, s) {
                        return bad();
                    }
                    let e = ea_at!(s, Sz::W);
                    out[0] = 0x46C0 | e;
                }
                (Op::A(r), Op::Usp) => out[0] = 0x4E60 | r,
                (Op::Usp, Op::A(r)) => out[0] = 0x4E68 | r,
                (s, d) => {
                    let z = sz(Sz::W)?;
                    let code = match z {
                        Sz::B => 1,
                        Sz::L => 2,
                        Sz::W => 3,
                    };
                    if z == Sz::B && matches!(s, Op::A(_)) {
                        return bad();
                    }
                    if !allowed(Cat::All, s) {
                        return bad();
                    }
                    let se = ea_at!(s, z);
                    if let Op::A(r) = d {
                        if z == Sz::B {
                            return bad();
                        }
                        out[0] = code << 12 | r << 9 | 1 << 6 | se;
                    } else {
                        if n == "MOVEA" || !allowed(Cat::DataAlt, d) {
                            return bad();
                        }
                        let de = ea_at!(d, z);
                        out[0] = code << 12 | (de & 7) << 9 | (de >> 3) << 6 | se;
                    }
                }
            }
        }
        "ORI" | "ANDI" | "EORI" | "SUBI" | "ADDI" | "CMPI" => {
            need(2)?;
            let t: u16 = match n {
                "ORI" => 0,
                "ANDI" => 1,
                "SUBI" => 2,
                "ADDI" => 3,
                "EORI" => 5,
                _ => 6,
            };
            let Op::Imm(x) = &ops[0] else { return bad() };
            match &ops[1] {
                Op::Ccr | Op::Sr if matches!(t, 0 | 1 | 5) => {
                    let v = en.val(x)?;
                    let sr = matches!(ops[1], Op::Sr);
                    en.check(if sr { fits_either(v, 16) } else { fits_either(v, 8) }, "immediate out of range")?;
                    out[0] = t << 9 | if sr { 0x7C } else { 0x3C };
                    out.push(if sr { v as u16 } else { v as u16 & 0xFF });
                }
                d => {
                    let s = sz(Sz::W)?;
                    if !allowed(Cat::DataAlt, d) {
                        return bad();
                    }
                    ea_at!(&ops[0], s);
                    let e = ea_at!(d, s);
                    out[0] = t << 9 | s.bits() << 6 | e;
                }
            }
        }
        "ADD" | "SUB" | "AND" | "OR" | "EOR" | "CMP" | "ADDA" | "SUBA" | "CMPA" => {
            need(2)?;
            let s = sz(Sz::W)?;
            let arith = matches!(n, "ADD" | "SUB" | "CMP" | "ADDA" | "SUBA" | "CMPA");
            // The address forms.
            if let Op::A(r) = ops[1] {
                if !arith || s == Sz::B {
                    return bad();
                }
                let base = match &n[..3] {
                    "ADD" => 0xD0C0,
                    "SUB" => 0x90C0,
                    _ => 0xB0C0,
                };
                let e = ea_at!(&ops[0], s);
                out[0] = base | r << 9 | ((s == Sz::L) as u16) << 8 | e;
                return Ok(out);
            }
            if n.ends_with('A') {
                return bad();
            }
            if matches!(ops[0], Op::Imm(_)) && !matches!(ops[1], Op::D(_)) || n == "EOR" && matches!(ops[0], Op::Imm(_)) {
                let i = match n {
                    "ADD" => "ADDI",
                    "SUB" => "SUBI",
                    "AND" => "ANDI",
                    "OR" => "ORI",
                    "EOR" => "EORI",
                    _ => "CMPI",
                };
                return encode(en, i, size, ops, pc);
            }
            if n == "CMP" {
                if let (Op::Post(_), Op::Post(_)) = (&ops[0], &ops[1]) {
                    return encode(en, "CMPM", size, ops, pc);
                }
            }
            let base: u16 = match n {
                "ADD" => 0xD000,
                "SUB" => 0x9000,
                "AND" => 0xC000,
                "OR" => 0x8000,
                "CMP" => 0xB000,
                _ => 0xB100,
            };
            match (&ops[0], &ops[1]) {
                (src, Op::D(r)) if n != "EOR" => {
                    let cat = if arith { Cat::All } else { Cat::Data };
                    if !allowed(cat, src) || (s == Sz::B && matches!(src, Op::A(_))) {
                        return bad();
                    }
                    let e = ea_at!(src, s);
                    out[0] = base | r << 9 | s.bits() << 6 | e;
                }
                (Op::D(r), dst) => {
                    if n == "CMP" {
                        return bad();
                    }
                    let cat = if n == "EOR" { Cat::DataAlt } else { Cat::MemAlt };
                    if !allowed(cat, dst) {
                        return bad();
                    }
                    let e = ea_at!(dst, s);
                    out[0] = base | r << 9 | (4 | s.bits()) << 6 | e;
                }
                _ => return bad(),
            }
        }
        _ => return err(line, format!("unknown instruction `{name}`")),
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------------------------------
// Layout

struct Layout {
    symbols: HashMap<String, i64>,
    /// Per statement: its unsized absolute operands' forms, and whether an unsized branch is short.
    abs: Vec<Vec<bool>>,
    short: Vec<bool>,
}

fn data_bytes(sz: Sz) -> i64 {
    match sz {
        Sz::B => 1,
        Sz::W => 2,
        Sz::L => 4,
    }
}

/// One walk over the statements; with `emit`, the bytes.
fn walk(stmts: &[Stmt], origin: u32, lay: &mut Layout, emit: Option<&mut BTreeMap<i64, u8>>, first: bool) -> Result<bool, AsmError> {
    let mut pc = origin as i64;
    let mut changed = false;
    let fin = emit.is_some();
    let mut out = emit;
    for (i, s) in stmts.iter().enumerate() {
        let line = s.line;
        let put = |out: &mut Option<&mut BTreeMap<i64, u8>>, at: i64, b: &[u8]| {
            if let Some(o) = out.as_mut() {
                for (k, v) in b.iter().enumerate() {
                    o.insert(at + k as i64, *v);
                }
            }
        };
        match &s.kind {
            Kind::Label(n) => {
                if lay.symbols.insert(n.clone(), pc) != Some(pc) {
                    changed = true;
                }
            }
            Kind::Equ(n, e) => {
                if let Some(v) = eval(e, &Ctx { symbols: &lay.symbols, pc }) {
                    if lay.symbols.insert(n.clone(), v) != Some(v) {
                        changed = true;
                    }
                } else if fin {
                    return err(line, format!("constant `{n}` cannot be evaluated"));
                }
            }
            Kind::Org(e) => {
                let v = eval(e, &Ctx { symbols: &lay.symbols, pc }).ok_or(AsmError { line, message: "`org` needs a known value".into() })?;
                if v < pc {
                    return err(line, "`org` moves backward");
                }
                pc = v;
            }
            Kind::Even => pc += pc & 1,
            Kind::Align(e) => {
                let a = eval(e, &Ctx { symbols: &lay.symbols, pc }).ok_or(AsmError { line, message: "`align` needs a known value".into() })?;
                if a <= 0 {
                    return err(line, "bad alignment");
                }
                pc = (pc + a - 1) / a * a;
            }
            Kind::Dc(sz, items) => {
                let n = data_bytes(*sz);
                if n > 1 && pc & 1 != 0 {
                    return err(line, "a word or long at an odd address");
                }
                for it in items {
                    match it {
                        Data::Bytes(b) => {
                            put(&mut out, pc, b);
                            pc += b.len() as i64;
                        }
                        Data::Expr(e) => {
                            let v = match eval(e, &Ctx { symbols: &lay.symbols, pc }) {
                                Some(v) => v,
                                None if fin => return err(line, "undefined symbol in data"),
                                None => 0,
                            };
                            if fin && n < 4 && !fits_either(v, 8 * n as u32) {
                                return err(line, "data out of range for its size");
                            }
                            let b = (v as u32).to_be_bytes();
                            put(&mut out, pc, &b[4 - n as usize..]);
                            pc += n;
                        }
                    }
                }
            }
            Kind::Ds(sz, count, fill) => {
                let n = data_bytes(*sz);
                if n > 1 && pc & 1 != 0 {
                    return err(line, "a word or long at an odd address");
                }
                let c = eval(count, &Ctx { symbols: &lay.symbols, pc }).ok_or(AsmError { line, message: "a count needs a known value".into() })?;
                let f = eval(fill, &Ctx { symbols: &lay.symbols, pc }).unwrap_or(0);
                let b = (f as u32).to_be_bytes();
                for _ in 0..c.max(0) {
                    put(&mut out, pc, &b[4 - n as usize..]);
                    pc += n;
                }
            }
            Kind::Inst { name, size, ops } => {
                if pc & 1 != 0 {
                    return err(line, "an instruction at an odd address");
                }
                let mut abs = std::mem::take(&mut lay.abs[i]);
                let words = {
                    let mut en = Enc { ctx: Ctx { symbols: &lay.symbols, pc }, fin, line, abs_short: &mut abs, abs_next: 0, branch_short: lay.short[i] };
                    encode(&mut en, name, *size, ops, pc)?
                };
                lay.abs[i] = abs;
                // Shorten an unsized branch whose target is near, never lengthening one (§8.2).
                if !first && !fin && size.is_none() && !lay.short[i] && is_branch(name) {
                    if let [Op::Abs(t)] = ops.as_slice() {
                        if let Some(v) = eval(t, &Ctx { symbols: &lay.symbols, pc }) {
                            let d = rel(v, pc + 2);
                            if fits_signed(d, 8) && d != 0 {
                                lay.short[i] = true;
                                changed = true;
                            }
                        }
                    }
                }
                let b: Vec<u8> = words.iter().flat_map(|w| w.to_be_bytes()).collect();
                put(&mut out, pc, &b);
                pc += b.len() as i64;
            }
        }
    }
    Ok(changed)
}

fn is_branch(n: &str) -> bool {
    n == "BSR" || n.len() >= 2 && n.starts_with('B') && cond(&n[1..]).is_some() && !matches!(n, "BCHG" | "BCLR" | "BSET" | "BTST")
}

/// A program placed from `origin`, with `defines` known to its conditionals and `include` reading its includes.
pub fn assemble_with(source: &str, origin: u32, defines: &[(&str, i64)], include: &dyn Fn(&str) -> Option<String>) -> Result<Program, AsmError> {
    let defs: HashMap<String, i64> = defines.iter().map(|(k, v)| (k.to_string(), *v)).collect();
    let mut stmts = Vec::new();
    parse(source, &defs, include, &mut stmts, 0)?;
    let mut lay = Layout { symbols: defs, abs: vec![Vec::new(); stmts.len()], short: vec![false; stmts.len()] };
    walk(&stmts, origin, &mut lay, None, true)?;
    let mut rounds = 0;
    while walk(&stmts, origin, &mut lay, None, false)? {
        rounds += 1;
        if rounds > 64 {
            return err(0, "the layout does not settle");
        }
    }
    let mut bytes = BTreeMap::new();
    walk(&stmts, origin, &mut lay, Some(&mut bytes), false)?;
    let end = bytes.keys().next_back().map_or(origin as i64, |k| k + 1);
    let mut image = vec![0u8; (end - origin as i64).max(0) as usize];
    for (k, v) in bytes {
        if k < origin as i64 {
            return err(0, "bytes placed before the origin");
        }
        image[(k - origin as i64) as usize] = v;
    }
    Ok(Program { origin, bytes: image, symbols: lay.symbols.into_iter().collect() })
}

/// A program with no includes or defines.
pub fn assemble(source: &str, origin: u32) -> Result<Program, AsmError> {
    assemble_with(source, origin, &[], &|_| None)
}

/// One instruction, as the disassembler writes it, at `pc`: its words.
pub fn assemble_line(text: &str, pc: u32) -> Result<Vec<u16>, AsmError> {
    let p = assemble(&format!(" {text}"), pc)?;
    Ok(p.bytes.chunks(2).map(|c| u16::from_be_bytes([c[0], *c.get(1).unwrap_or(&0)])).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disasm::disassemble;

    fn words(text: &str, pc: u32) -> Vec<u16> {
        assemble_line(text, pc).unwrap_or_else(|e| panic!("{text}: {e}"))
    }

    #[test]
    fn instructions_assemble_as_the_manual_encodes_them() {
        assert_eq!(words("NOP", 0), [0x4E71]);
        assert_eq!(words("MOVE.L (A0)+,(A1)+", 0), [0x22D8]);
        assert_eq!(words("MOVE.W #$1234,-$10(A5)", 0), [0x3B7C, 0x1234, 0xFFF0]);
        assert_eq!(words("LEA $00FF0000.L,A0", 0), [0x41F9, 0x00FF, 0x0000]);
        assert_eq!(words("JSR $00000500.L", 0x400), [0x4EB9, 0x0000, 0x0500]);
        assert_eq!(words("BSR.W $000412", 0x400), [0x6100, 0x0010]);
        assert_eq!(words("DBRA D7,$000410", 0x424), [0x51CF, 0xFFEA]);
        assert_eq!(words("MOVEM.L D0-D1/A0-A1,-(A7)", 0), [0x48E7, 0xC0C0]);
        assert_eq!(words("MOVEM.L (SP)+,D0-D1/A0-A1", 0), [0x4CDF, 0x0303]);
        assert_eq!(words("MOVE #$2700,SR", 0), [0x46FC, 0x2700]);
        assert_eq!(words("ANDI #$F8FF,SR", 0), [0x027C, 0xF8FF]);
        assert_eq!(words("BTST #3,$00A10003.L", 0), [0x0839, 0x0003, 0x00A1, 0x0003]);
        assert_eq!(words("LSL.W #3,D2", 0), [0xE74A]);
        assert_eq!(words("ADD.W #1,D0", 0), [0xD07C, 0x0001]);
        assert_eq!(words("ADD.W #1,(A0)", 0), [0x0650, 0x0001]);
        assert_eq!(words("MOVE.L D0,A1", 0), [0x2240]);
        assert_eq!(words("CMP.B (A0)+,(A1)+", 0), [0xB308]);
        assert_eq!(words("MOVE.W $FFFD00,D0", 0), [0x3038, 0xFD00]);
        assert_eq!(words("MOVE.W $A12000,D0", 0), [0x3039, 0x00A1, 0x2000]);
    }

    #[test]
    fn labels_constants_data_and_branch_sizes() {
        let src = "
VDP     equ $C00004
start:  move.w  #$8F02,VDP
        moveq   #3,d0
.loop   dbra    d0,.loop
        bra     start
        bne     far
        lea     text(pc),a0
        rts
text    dc.b    \"AB\",0
        even
        dc.w    start,text-start
        ds.b    300
far:    nop
";
        let p = assemble(src, 0x200).unwrap();
        let w: Vec<u16> = p.bytes.chunks(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
        assert_eq!(&w[..4], &[0x33FC, 0x8F02, 0x00C0, 0x0004]);
        assert_eq!(w[4], 0x7003);
        assert_eq!(&w[5..7], &[0x51C8, 0xFFFE]);
        assert_eq!(w[7], 0x60F0, "a near backward branch is short");
        assert_eq!(&w[8..10], &[0x6600, 0x013C], "a far one is a word");
        assert_eq!(p.symbols["start.loop"], 0x20A);
        assert_eq!(p.symbols["far"], 0x34E);
    }

    #[test]
    fn conditionals_take_the_defined_branch() {
        let src = "  if REGION=1\n  dc.b 'U'\n  else\n  dc.b 'E'\n  endif\n";
        assert_eq!(assemble_with(src, 0, &[("REGION", 1)], &|_| None).unwrap().bytes, b"U");
        assert_eq!(assemble_with(src, 0, &[("REGION", 2)], &|_| None).unwrap().bytes, b"E");
    }

    #[test]
    fn errors_name_their_line() {
        assert_eq!(assemble(" nop\n bogus d0\n", 0).unwrap_err().line, 2);
        assert!(assemble(" bra.s *+2\n", 0).is_err(), "a short branch to the next word is the word form's marker");
        assert!(assemble(" move.w d0,undefined\n", 0).is_err());
        assert!(assemble(" moveq #300,d0\n", 0).is_err());
        let p = assemble("a: bra .w\n.w: nop\n", 0).unwrap();
        assert_eq!(p.bytes, [0x60, 0x00, 0x00, 0x02, 0x4E, 0x71], "a local label named like a size");
    }

    /// Extension words with the fields the 68000 ignores clear, so that every word is the disassembler's to print.
    const PLAIN: [u16; 5] = [0x0012, 0x0034, 0x0056, 0x0078, 0x009A];
    /// Address-register and long indexes, negative displacements and wide immediates: compared by text.
    const WIDE: [[u16; 5]; 2] = [[0x9812, 0xFF84, 0x8001, 0x7FFE, 0xC0DE], [0xF0FE, 0x0080, 0x00FF, 0x8000, 0x0000]];

    fn dis(op: u16, ext: &[u16; 5], at: u32) -> emusen_native::core::Instruction {
        let w = [op, ext[0], ext[1], ext[2], ext[3], ext[4]];
        disassemble(&move |a: u32| w.get((a.wrapping_sub(at) / 2) as usize).copied().unwrap_or(0), at)
    }

    /// Every opcode the 68000 decodes, disassembled, assembled again: the same words with plain extensions, and the
    /// same text with the others (Beryl_M68k.md §8.3).
    #[test]
    fn every_instruction_form_assembles_back_to_its_words() {
        let at = 0x1000;
        let (mut checked, mut wrong) = (0, Vec::new());
        for op in 0..=0xFFFFu16 {
            let i = dis(op, &PLAIN, at);
            if i.mnemonic == "DC.W" {
                continue;
            }
            checked += 1;
            let text = format!("{} {}", i.mnemonic, i.operands);
            let want: Vec<u16> = i.bytes.chunks(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
            match assemble_line(text.trim(), at) {
                Ok(got) if got == want => {}
                Ok(got) => wrong.push(format!("{op:04X} {text}: {got:04X?}, want {want:04X?}")),
                Err(e) => wrong.push(format!("{op:04X} {text}: {e}")),
            }
            for ext in &WIDE {
                let i = dis(op, ext, at);
                let text = format!("{} {}", i.mnemonic, i.operands);
                match assemble_line(text.trim(), at) {
                    Ok(got) => {
                        let again = disassemble(&|a: u32| got.get((a.wrapping_sub(at) / 2) as usize).copied().unwrap_or(0), at);
                        let back = format!("{} {}", again.mnemonic, again.operands);
                        if back != text {
                            wrong.push(format!("{op:04X} {text}: reads back as {back}"));
                        }
                    }
                    Err(e) => wrong.push(format!("{op:04X} {text}: {e}")),
                }
            }
        }
        eprintln!("{checked} opcodes, {} differ", wrong.len());
        for w in wrong.iter().take(40) {
            eprintln!("  {w}");
        }
        assert!(checked > 45_000);
        assert!(wrong.is_empty());
    }
}

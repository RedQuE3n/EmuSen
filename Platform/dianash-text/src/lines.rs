//! The commands that work on lines of text: sort, uniq, grep, sed and the AWK subset.

use crate::pattern;
use crate::{Answer, Run};
use dianash_module::Call;
use emusen_dotnet::file::read_all_text;
use regex::Regex;
use std::cmp::Ordering;

/// Standard input, else the words after the options joined by spaces, else none.
fn input(stdin: Option<String>, args: &[String], from: usize) -> Option<String> {
    stdin.or_else(|| (from < args.len()).then(|| args[from..].join(" ")))
}

pub fn sort(args: &[String], stdin: Option<String>) -> Answer {
    let (mut numeric, mut reverse) = (false, false);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => numeric = true,
            "-r" => reverse = true,
            _ => break,
        }
        i += 1;
    }
    let Some(text) = input(stdin, args, i) else { return Answer::ok("") };
    let mut lines: Vec<&str> = text.split('\n').collect();
    if numeric {
        // OrderBy is stable, and Comparer<double> puts NaN first and holds -0 equal to 0.
        let key = |line: &str| line.trim().parse::<f64>().unwrap_or(0.0);
        lines.sort_by(|a, b| {
            let (a, b) = (key(a), key(b));
            match (a.is_nan(), b.is_nan()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
            }
        });
    } else {
        lines.sort_by(|a, b| a.cmp(b));
    }
    if reverse {
        lines.reverse();
    }
    Answer::ok(lines.join("\n"))
}

const UNIQ_USAGE: &str = "Usage: uniq [-c] [-D] [-w <chars>] - see man uniq";

/// `uniq`: adjacent lines with the same key collapsed to the first, or with `-D` every line of a run of two or more
/// printed; the key is the line, or its first `-w` characters. See `man uniq`.
pub fn uniq(args: &[String], stdin: Option<String>) -> Answer {
    let (mut counted, mut all, mut width) = (false, false, None::<usize>);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => counted = true,
            "-D" => all = true,
            "-w" => match args.get(i + 1).and_then(|n| n.parse().ok()) {
                Some(n) => {
                    width = Some(n);
                    i += 1;
                }
                None => return Answer::fail(format!("uniq: -w needs a number of characters\n{UNIQ_USAGE}")),
            },
            w if w.len() > 2 && w.starts_with("-w") && w[2..].parse::<usize>().is_ok() => width = w[2..].parse().ok(),
            _ => break,
        }
        i += 1;
    }
    if counted && all {
        return Answer::fail(format!("uniq: -c and -D together are meaningless: every line of a run is printed, not counted\n{UNIQ_USAGE}"));
    }
    let Some(text) = input(stdin, args, i) else { return Answer::ok("") };
    let key = |line: &'_ str| -> String { width.map_or_else(|| line.to_string(), |n| line.chars().take(n).collect()) };
    if all {
        let ended = text.ends_with('\n');
        let body = if ended { &text[..text.len() - 1] } else { &text[..] };
        let lines: Vec<&str> = if body.is_empty() && ended { vec![""] } else { body.split('\n').collect() };
        let mut out: Vec<&str> = Vec::new();
        let mut from = 0;
        for k in 1..=lines.len() {
            if k == lines.len() || key(lines[k]) != key(lines[from]) {
                if k - from > 1 {
                    out.extend(&lines[from..k]);
                }
                from = k;
            }
        }
        let mut joined = out.join("\n");
        if ended && !out.is_empty() {
            joined.push('\n');
        }
        return Answer::ok(joined);
    }
    let mut out: Vec<String> = Vec::new();
    let mut current: Option<(&str, String)> = None;
    let mut count = 0;
    let flush = |current: &Option<(&str, String)>, count: usize, out: &mut Vec<String>| {
        if let Some((line, _)) = current {
            out.push(if counted { format!("{count} {line}") } else { line.to_string() });
        }
    };
    for line in text.split('\n') {
        let k = key(line);
        if current.as_ref().is_some_and(|(_, c)| *c == k) {
            count += 1;
        } else {
            flush(&current, count, &mut out);
            current = Some((line, k));
            count = 1;
        }
    }
    flush(&current, count, &mut out);
    Answer::ok(out.join("\n"))
}

const GREP_USAGE: &str = concat!(
    "  grep [-i] [-v] [-n] [-c] [-q] <pattern> [text...]\n",
    "                                filter lines matching <pattern> (regex) - -i case-insensitive,\n",
    "                                -v invert (non-matching), -n prefix line numbers, -c print only\n",
    "                                the match count, -q no output (exit code only, for if/while).\n",
    "                                Reads piped stdin if given, else operates on trailing literal text."
);

pub fn grep(call: &Call, args: &[String], stdin: Option<String>) -> Run {
    let (mut ignore_case, mut invert, mut numbered, mut count_only, mut quiet) = (false, false, false, false, false);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-i" => ignore_case = true,
            "-v" => invert = true,
            "-n" => numbered = true,
            "-c" => count_only = true,
            "-q" => quiet = true,
            _ => break,
        }
        i += 1;
    }
    if i >= args.len() {
        return Ok(Answer::ok(GREP_USAGE));
    }
    let regex = &args[i];
    let Some(text) = input(stdin, args, i + 1) else {
        return Ok(Answer::ok("Nothing to search - pipe a command's output in, or pass literal text: grep <pattern> <text...>"));
    };
    let regex = pattern::cached(call, regex, ignore_case)?;
    let matches: Vec<(usize, &str)> = text.split('\n').enumerate().take_while(|_| !call.cancelled()).filter(|(_, line)| regex.is_match(line) != invert).map(|(n, line)| (n + 1, line)).collect();
    // Exit 0 on at least one match, which is what lets -q drive an if or while.
    let status = if matches.is_empty() { 1 } else { 0 };
    if quiet {
        return Ok(Answer::with("", status));
    }
    if count_only {
        return Ok(Answer::with(matches.len().to_string(), status));
    }
    let lines: Vec<String> = matches.iter().map(|(n, line)| if numbered { format!("{n}:{line}") } else { line.to_string() }).collect();
    Ok(Answer::with(lines.join("\n"), status))
}

const SED_USAGE: &str = concat!(
    "  sed s/pat/repl/[gi]           substitute (first match per line, or every match with 'g';\n",
    "                                'i' = case-insensitive). Standalone: sed <expr> <text...>.\n",
    "                                Real use is as a pipeline stage: <command> | sed s/pat/repl/"
);

pub fn sed(call: &Call, args: &[String], stdin: Option<String>) -> Run {
    if args.len() < 2 {
        return Ok(Answer::ok(SED_USAGE));
    }
    // Piped input always wins over trailing literal text, as real sed prefers stdin.
    let Some(text) = input(stdin, args, 2) else {
        return Ok(Answer::fail("Nothing to operate on - pipe a command's output in, or pass literal text: sed <expr> <text...>"));
    };
    let expression = &args[1];
    let chars: Vec<char> = expression.chars().collect();
    if chars.len() < 2 || chars[0] != 's' {
        return Err(format!("Unrecognized sed expression '{expression}' - only s/pattern/replacement/[gi] is supported."));
    }
    let delimiter = chars[1];
    let fields = split_on(&chars[2..], delimiter);
    if fields.len() != 3 {
        return Err(format!("Malformed s{delimiter}pattern{delimiter}replacement{delimiter}[flags] expression: '{expression}'."));
    }
    let global = fields[2].contains('g');
    let regex = pattern::cached(call, &fields[0], fields[2].contains('i'))?;
    let replacement = pattern::replacement(&fields[1], regex.captures_len());
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| if global { regex.replace_all(line, replacement.as_str()).into_owned() } else { regex.replacen(line, 1, replacement.as_str()).into_owned() })
        .collect();
    Ok(Answer::ok(lines.join("\n")))
}

/// Splits at the delimiter, an escaped delimiter kept as itself.
fn split_on(text: &[char], delimiter: char) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut i = 0;
    while i < text.len() {
        if text[i] == '\\' && i + 1 < text.len() && text[i + 1] == delimiter {
            fields.last_mut().unwrap().push(delimiter);
            i += 1;
        } else if text[i] == delimiter {
            fields.push(String::new());
        } else {
            fields.last_mut().unwrap().push(text[i]);
        }
        i += 1;
    }
    fields
}

/// One expression an AWK `print` prints.
enum Expression {
    Line,
    Last,
    Number,
    Count,
    Field(usize),
    Literal(String),
}

impl Expression {
    fn value(&self, line: &str, fields: &[String], nr: usize) -> String {
        match self {
            Expression::Line => line.to_string(),
            Expression::Last => fields.last().cloned().unwrap_or_default(),
            Expression::Number => nr.to_string(),
            Expression::Count => fields.len().to_string(),
            Expression::Field(n) => if *n >= 1 && *n <= fields.len() { fields[n - 1].clone() } else { String::new() },
            Expression::Literal(text) => text.clone(),
        }
    }
}

enum Pattern {
    Every,
    Begin,
    End,
    Regex(Regex),
    Record(String, i32),
}

struct Rule {
    pattern: Pattern,
    /// None when no `{...}` was given: the default action prints the line.
    statements: Option<Vec<Vec<Expression>>>,
}

fn unescape(text: &str) -> String {
    text.replace("\\t", "\t").replace("\\n", "\n")
}

pub fn awk(call: &Call, args: &[String], stdin: Option<String>) -> Run {
    let mut separator: Option<String> = None;
    let mut program: Option<&str> = None;
    let mut path: Option<&str> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-F" && i + 1 < args.len() {
            i += 1;
            separator = Some(unescape(&args[i]));
        } else if args[i].starts_with("-F") && args[i].chars().count() > 2 {
            separator = Some(unescape(&args[i][2..]));
        } else if program.is_none() {
            program = Some(&args[i]);
        } else {
            path = Some(&args[i]);
        }
        i += 1;
    }
    let Some(program) = program else { return Ok(Answer::fail("awk: usage: awk [-F sep] 'program' [path]")) };
    let rules = match parse_program(program)? {
        Ok(rules) => rules,
        Err(words) => return Ok(Answer::fail(format!("awk: {words}"))),
    };
    let text = match (stdin, path) {
        (Some(text), _) => text,
        (None, Some(path)) => match call.resolve(path) {
            Ok(real) => read_all_text(&real)?,
            // The C# names the last argument, which is the path but for a program given after it.
            Err(refusal) => return Ok(Answer::fail(format!("awk: '{}'{}", args[args.len() - 1], refusal.strip_prefix(&format!("'{path}'")).unwrap_or(&refusal)))),
        },
        (None, None) => String::new(),
    };
    let mut lines: Vec<&str> = if text.is_empty() { Vec::new() } else { text.split('\n').collect() };
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let split = |line: &str| -> Vec<String> {
        match &separator {
            None => {
                let trimmed = line.trim();
                if trimmed.is_empty() { Vec::new() } else { trimmed.split(char::is_whitespace).filter(|f| !f.is_empty()).map(str::to_string).collect() }
            }
            Some(separator) => line.split(separator.as_str()).map(str::to_string).collect(),
        }
    };
    let mut output: Vec<String> = Vec::new();
    let mut act = |rule: &Rule, line: &str, fields: &[String], nr: usize| match &rule.statements {
        None => output.push(line.to_string()),
        Some(statements) => {
            for statement in statements {
                output.push(if statement.is_empty() { line.to_string() } else { statement.iter().map(|e| e.value(line, fields, nr)).collect::<Vec<_>>().join(" ") });
            }
        }
    };
    for rule in rules.iter().filter(|r| matches!(r.pattern, Pattern::Begin)) {
        act(rule, "", &[], 0);
    }
    let mut nr = 0;
    for line in &lines {
        nr += 1;
        let fields = split(line);
        for rule in rules.iter().filter(|r| !matches!(r.pattern, Pattern::Begin | Pattern::End)) {
            let selected = match &rule.pattern {
                Pattern::Regex(regex) => regex.is_match(line),
                Pattern::Record(op, value) => compare(op, nr as i64, i64::from(*value)),
                _ => true,
            };
            if selected {
                act(rule, line, &fields, nr);
            }
        }
    }
    let last = lines.last().copied().unwrap_or("");
    let last_fields = split(last);
    for rule in rules.iter().filter(|r| matches!(r.pattern, Pattern::End)) {
        act(rule, last, &last_fields, nr);
    }
    Ok(Answer::ok(output.join("\n")))
}

fn compare(op: &str, nr: i64, value: i64) -> bool {
    match op {
        "==" => nr == value,
        "!=" => nr != value,
        ">=" => nr >= value,
        "<=" => nr <= value,
        ">" => nr > value,
        "<" => nr < value,
        _ => false,
    }
}

/// The program's rules; the outer error is one the C# did not catch (an overflowing number), the inner the words `awk:` refuses with.
fn parse_program(program: &str) -> Result<Result<Vec<Rule>, String>, String> {
    let chars: Vec<char> = program.chars().collect();
    let len = chars.len();
    let mut rules = Vec::new();
    let mut pos = 0;
    loop {
        while pos < len && (chars[pos].is_whitespace() || chars[pos] == ';') {
            pos += 1;
        }
        if pos >= len {
            break;
        }
        let (mut scan, mut quoted) = (pos, false);
        let (mut brace, mut separator) = (None, None);
        while scan < len {
            let c = chars[scan];
            if quoted {
                if c == '\\' {
                    scan += 2;
                    continue;
                }
                if c == '"' {
                    quoted = false;
                }
                scan += 1;
                continue;
            }
            match c {
                '"' => quoted = true,
                '{' => {
                    brace = Some(scan);
                    break;
                }
                ';' | '\n' => {
                    separator = Some(scan);
                    break;
                }
                _ => {}
            }
            scan += 1;
        }
        let text = |from: usize, to: usize| -> String { chars[from..to.min(len)].iter().collect() };
        let pattern_text;
        let mut statements = None;
        if let Some(open) = brace {
            pattern_text = text(pos, open).trim().to_string();
            let Some(close) = matching_brace(&chars, open) else { return Ok(Err(format!("unterminated '{{' in: {}", text(pos, len)))) };
            match parse_statements(text(open + 1, close).trim()) {
                Ok(parsed) => statements = Some(parsed),
                Err(words) => return Ok(Err(words)),
            }
            pos = close + 1;
        } else if let Some(at) = separator {
            pattern_text = text(pos, at).trim().to_string();
            pos = at + 1;
        } else {
            pattern_text = text(pos, len).trim().to_string();
            pos = len;
        }
        if pattern_text.is_empty() && statements.is_none() {
            continue;
        }
        let pattern = match pattern_text.as_str() {
            "BEGIN" => Pattern::Begin,
            "END" => Pattern::End,
            "" => Pattern::Every,
            text => match parse_pattern(text)? {
                Ok(pattern) => pattern,
                Err(words) => return Ok(Err(words)),
            },
        };
        rules.push(Rule { pattern, statements });
    }
    if rules.is_empty() {
        return Ok(Err("empty program".to_string()));
    }
    Ok(Ok(rules))
}

fn matching_brace(chars: &[char], open: usize) -> Option<usize> {
    let (mut depth, mut quoted) = (0, false);
    let mut i = open;
    while i < chars.len() {
        let c = chars[i];
        if quoted {
            if c == '\\' {
                i += 1;
            } else if c == '"' {
                quoted = false;
            }
        } else if c == '"' {
            quoted = true;
        } else if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Splits at a separator only at brace depth 0 and outside a quoted string.
fn split_top_level(text: &str, separator: char) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut parts = vec![String::new()];
    let (mut depth, mut quoted) = (0i32, false);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let part = parts.last_mut().unwrap();
        if quoted {
            part.push(c);
            if c == '\\' && i + 1 < chars.len() {
                i += 1;
                part.push(chars[i]);
            } else if c == '"' {
                quoted = false;
            }
        } else if c == '"' {
            quoted = true;
            part.push(c);
        } else if c == '{' {
            depth += 1;
            part.push(c);
        } else if c == '}' {
            depth -= 1;
            part.push(c);
        } else if depth == 0 && c == separator {
            parts.push(String::new());
        } else {
            part.push(c);
        }
        i += 1;
    }
    parts
}

fn parse_statements(body: &str) -> Result<Vec<Vec<Expression>>, String> {
    let mut statements = Vec::new();
    for raw in split_top_level(body, ';') {
        let statement = raw.trim();
        if statement.is_empty() {
            continue;
        }
        let is_print = statement == "print" || (statement.starts_with("print") && statement[5..].chars().next().is_some_and(char::is_whitespace));
        if !is_print {
            return Err(format!("unsupported awk statement: '{statement}' (only 'print' is supported)"));
        }
        let arguments = statement[5..].trim();
        let expressions = if arguments.is_empty() { Vec::new() } else { split_top_level(arguments, ',').iter().map(|e| parse_expression(e.trim())).collect::<Result<_, _>>()? };
        statements.push(expressions);
    }
    Ok(statements)
}

/// `/regex/` or `NR<op><int>`; the outer error is .NET's for a number past Int32.
fn parse_pattern(text: &str) -> Result<Result<Pattern, String>, String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() >= 2 && chars[0] == '/' && chars[chars.len() - 1] == '/' {
        let inner: String = chars[1..chars.len() - 1].iter().collect();
        return pattern::compile(&inner, false).map(|r| Ok(Pattern::Regex(r)));
    }
    if let Some(rest) = text.strip_prefix("NR") {
        let rest = rest.trim_start_matches(char::is_whitespace);
        for op in ["==", "!=", ">=", "<=", ">", "<"] {
            if let Some(number) = rest.strip_prefix(op) {
                let number = number.trim_start_matches(char::is_whitespace);
                if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()) {
                    return match number.trim().parse::<i32>().ok() {
                        Some(value) => Ok(Ok(Pattern::Record(op.to_string(), value))),
                        None => Err("Value was either too large or too small for an Int32.".to_string()),
                    };
                }
                break;
            }
        }
    }
    Ok(Err(format!("unsupported awk pattern: '{text}' (only /regex/ and NR<op><int> are supported)")))
}

fn parse_expression(text: &str) -> Result<Expression, String> {
    match text {
        "$0" => return Ok(Expression::Line),
        "$NF" => return Ok(Expression::Last),
        "NR" => return Ok(Expression::Number),
        "NF" => return Ok(Expression::Count),
        _ => {}
    }
    if let Some(digits) = text.strip_prefix('$') {
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
            return digits.trim().parse::<i32>().ok().map(|n| Expression::Field(n as usize)).ok_or_else(|| "Value was either too large or too small for an Int32.".to_string());
        }
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() >= 2 && chars[0] == '"' && chars[chars.len() - 1] == '"' {
        return Ok(Expression::Literal(unescape(&chars[1..chars.len() - 1].iter().collect::<String>())));
    }
    if text.trim().parse::<f64>().is_ok() {
        return Ok(Expression::Literal(text.to_string()));
    }
    Err(format!("unsupported awk expression: '{text}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniq_of(line: &str, stdin: &str) -> Answer {
        uniq(&line.split_whitespace().map(str::to_string).collect::<Vec<_>>(), Some(stdin.to_string()))
    }

    #[test]
    fn uniq_d_prints_every_line_of_each_run_and_w_keys_on_the_first_characters() {
        assert_eq!(uniq_of("uniq", "a\na\nb\n").out, "a\nb\n");
        assert_eq!(uniq_of("uniq -c", "a\na\nb").out, "2 a\n1 b");
        assert_eq!(uniq_of("uniq -D", "a\na\nb\nc\nc\nc\nd\n").out, "a\na\nc\nc\nc\n");
        assert_eq!(uniq_of("uniq -D", "a\nb\n").out, "");
        assert_eq!(uniq_of("uniq -D", "x\nx").out, "x\nx");
        assert_eq!(uniq_of("uniq -D -w 3", "abc one\nabc two\nabd three\n").out, "abc one\nabc two\n");
        assert_eq!(uniq_of("uniq -w2", "abX\nabY\ncd\n").out, "abX\ncd\n");
        assert_eq!(uniq_of("uniq -D -w 2", "é1x\né1y\n").out, "é1x\né1y\n");
        assert!(uniq_of("uniq -c -D", "a\n").err.contains("meaningless"));
        assert!(uniq_of("uniq -w", "a\n").err.contains("-w needs"));
    }
}

//! A .NET regular expression run by the `regex` crate: the syntax translated where the two read the same text
//! differently, the replacement written in the crate's syntax, and a refusal worded as .NET's
//! `RegexParseException` words it for the errors the corpus and ordinary use meet. What neither translates nor
//! words alike is recorded in EmuSen_DianaSh.md §13.8 (P3).

use emusen_dotnet::text::utf16_len;
use regex::Regex;
use regex_syntax::ast::ErrorKind;

/// A pattern compiled as `new Regex(pattern, ignoreCase ? IgnoreCase : None)` would read it.
pub fn compile(pattern: &str, ignore_case: bool) -> Result<Regex, String> {
    let translated = translate(pattern);
    let source = if ignore_case { format!("(?i){translated}") } else { translated };
    Regex::new(&source).map_err(|_| refusal(pattern))
}

/// `{` that does not begin a quantifier is a literal to .NET and an error to the crate.
fn translate(pattern: &str) -> String {
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::with_capacity(pattern.len());
    let mut i = 0;
    let mut in_class = false;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            out.push(c);
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        match c {
            '[' if !in_class => in_class = true,
            ']' if in_class => in_class = false,
            '{' if !in_class && !quantifier(&chars[i..]) => {
                out.push_str("\\{");
                i += 1;
                continue;
            }
            '}' if !in_class => {
                let opened = out.rfind('{').is_some_and(|at| !out[..at].ends_with('\\'));
                if !opened {
                    out.push_str("\\}");
                    i += 1;
                    continue;
                }
            }
            _ => {}
        }
        out.push(c);
        i += 1;
    }
    out
}

/// `{n}`, `{n,}` or `{n,m}`.
fn quantifier(rest: &[char]) -> bool {
    let text: String = rest.iter().take_while(|c| **c != '}').collect();
    if rest.len() <= text.chars().count() {
        return false;
    }
    let inner = &text[1..];
    let (low, high) = inner.split_once(',').unwrap_or((inner, "0"));
    !low.is_empty() && low.chars().all(|c| c.is_ascii_digit()) && high.chars().all(|c| c.is_ascii_digit())
}

/// `Invalid pattern '<p>' at offset <n>. <reason>`, as .NET words the ones it shares with the crate.
fn refusal(pattern: &str) -> String {
    let end = utf16_len(pattern);
    let (offset, reason) = match regex_syntax::ast::parse::Parser::new().parse(pattern) {
        Err(error) => {
            let at = utf16_len(&pattern[..error.span().end.offset.min(pattern.len())]);
            match error.kind() {
                ErrorKind::ClassUnclosed => (end, "Unterminated [] set.".to_string()),
                ErrorKind::GroupUnclosed => (end, "Not enough )'s.".to_string()),
                ErrorKind::GroupUnopened => (at, "Too many )'s.".to_string()),
                ErrorKind::RepetitionMissing => {
                    let start = error.span().start.offset.min(pattern.len());
                    let quantifier = pattern[start..].chars().next().unwrap_or('*');
                    (utf16_len(&pattern[..start]) + 1, format!("Quantifier '{quantifier}' following nothing."))
                }
                ErrorKind::EscapeUnexpectedEof => (end, "Illegal \\\\ at end of pattern.".to_string()),
                ErrorKind::ClassRangeInvalid => (at, "[x-y] range in reverse order.".to_string()),
                other => (at, format!("{other}.")),
            }
        }
        Ok(_) => (end, "This pattern uses a construct DianaSh's patterns do not support (look-around or a back-reference).".to_string()),
    };
    format!("Invalid pattern '{pattern}' at offset {offset}. {reason}")
}

/// A sed replacement in the crate's syntax: `&` the whole match, `\N` group N, a `$` literal; a group the pattern
/// does not have stays as the text `$N`, as .NET leaves a substitution it cannot make.
pub fn replacement(text: &str, groups: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '&' => out.push_str("${0}"),
            '\\' if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() => {
                let n = chars[i + 1].to_digit(10).unwrap() as usize;
                if n < groups {
                    out.push_str(&format!("${{{n}}}"));
                } else {
                    out.push_str(&format!("$${n}"));
                }
                i += 1;
            }
            '$' => out.push_str("$$"),
            c => out.push(c),
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refusals_are_worded_as_dotnet_words_them() {
        assert_eq!(compile("[", false).unwrap_err(), "Invalid pattern '[' at offset 1. Unterminated [] set.");
        assert_eq!(compile("(a", false).unwrap_err(), "Invalid pattern '(a' at offset 2. Not enough )'s.");
        assert_eq!(compile("a)", false).unwrap_err(), "Invalid pattern 'a)' at offset 2. Too many )'s.");
        assert_eq!(compile("*a", false).unwrap_err(), "Invalid pattern '*a' at offset 1. Quantifier '*' following nothing.");
    }

    #[test]
    fn a_brace_that_is_no_quantifier_is_a_literal() {
        assert!(compile("a{2}", false).unwrap().is_match("aa"));
        assert!(compile("{x}", false).unwrap().is_match("{x}"));
        assert!(compile("a{", false).unwrap().is_match("a{"));
        assert!(compile("B", true).unwrap().is_match("abc"));
    }

    #[test]
    fn replacements_translate_sed_to_the_crate() {
        let re = compile("(b)", false).unwrap();
        assert_eq!(re.replace("abc", replacement("<\\1>", re.captures_len()).as_str()), "a<b>c");
        assert_eq!(re.replace("abc", replacement("[&]", re.captures_len()).as_str()), "a[b]c");
        assert_eq!(re.replace("abc", replacement("$1", re.captures_len()).as_str()), "a$1c");
        assert_eq!(re.replace("abc", replacement("\\2", re.captures_len()).as_str()), "a$2c");
    }
}

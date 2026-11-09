//! `qml-rules`: EmuSen_Qt.md §1.3's rule that QML is layout only, as a checker over `.qml` text (EmuSen_Kit.md §7.9).
//! It reads text and gives findings, one line each; the emusen.dev command and the harness's test call it.

use std::fmt;
use std::path::Path;

/// What a rule found: where, which rule, and the words at fault.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub rule: Rule,
    pub words: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rule {
    /// A `function` other than a `Connections` handler whose body is one call.
    Function,
    /// `Qt.callLater`.
    CallLater,
    /// An array or object literal built in QML.
    Literal,
    /// A string the player would read, outside the places a string belongs.
    Words,
    /// `Component.onCompleted` doing more than taking the focus.
    Completed,
    /// A `ListModel` with rows.
    RowsInQml,
    /// An import outside the allowed list, or the kit imported without its alias (`import EmuSen.Kit as Kit`).
    Import,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Rule::Function => "function",
            Rule::CallLater => "callLater",
            Rule::Literal => "literal",
            Rule::Words => "words",
            Rule::Completed => "onCompleted",
            Rule::RowsInQml => "rows",
            Rule::Import => "import",
        })
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} {} {}", self.file, self.line, self.rule, self.words)
    }
}

/// The modules a view may import (EmuSen_Qt.md §1.3, §2); a prefix match, so `QtQuick.Controls.impl` is QtQuick's.
pub const ALLOWED_IMPORTS: [&str; 7] =
    ["QtQuick", "QtQuick.Controls", "QtQuick.Templates", "QtQuick.Layouts", "QtQuick.Shapes", "QtQml", "EmuSen."];

/// Imports never allowed, even under an allowed prefix. QtQml.Models is allowed for its DelegateChooser (decided
/// 2026-11-08); its ListModel stays a finding by the rows rule.
pub const FORBIDDEN_IMPORTS: [&str; 1] = ["QtQuick.LocalStorage"];

/// The modules a theme may import, matched whole (EmuSen_Qt.md §3.5): the sandboxed engine's allow-list, read by its host too.
pub const THEME_IMPORTS: [&str; 6] =
    ["QtQuick", "QtQuick.Layouts", "QtQuick.Effects", "QtQuick.Shapes", "QtQuick.Controls", "EmuSen.Theme"];

/// Bindings whose string is a name, a path or a colour rather than words: never findings.
const NAMED_BINDINGS: [&str; 12] = [
    "objectName",
    "role",
    "font.family",
    "source",
    "fragmentShader",
    "vertexShader",
    "property",
    "properties",
    "pragma",
    "Accessible.role",
    "placeholderText",
    "sequence",
];

/// How the check reads a file: as a view, or as a fixture (a test scene, the gallery) whose words, rows and literals stand in for models.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub fixture: bool,
}

fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_block = false;
    let mut in_line = false;
    let mut quote: Option<char> = None;
    while let Some(c) = chars.next() {
        if in_block {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block = false;
                out.push(' ');
            } else if c == '\n' {
                out.push('\n');
            }
            continue;
        }
        if in_line {
            if c == '\n' {
                in_line = false;
                out.push('\n');
            }
            continue;
        }
        if let Some(q) = quote {
            out.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            } else if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '/' if chars.peek() == Some(&'/') => in_line = true,
            '/' if chars.peek() == Some(&'*') => in_block = true,
            '"' | '\'' | '`' => {
                quote = Some(c);
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

fn is_colour(s: &str) -> bool {
    s == "transparent" || (s.starts_with('#') && s.len() >= 4 && s[1..].chars().all(|c| c.is_ascii_hexdigit()))
}

/// Every string literal on a line with the binding it is assigned to, if the words right before it are `name:`.
fn strings_in(line: &str) -> Vec<(String, Option<String>)> {
    let mut found = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' || c == '\'' {
            let mut s = String::new();
            let mut j = i + 1;
            while j < chars.len() && chars[j] != c {
                s.push(chars[j]);
                j += 1;
            }
            found.push((s, binding_before(&chars[..i])));
            i = j + 1;
        } else {
            i += 1;
        }
    }
    found
}

/// The binding name before a value: `name:` with any spaces, read backwards from the value.
fn binding_before(before: &[char]) -> Option<String> {
    let mut k = before.len();
    while k > 0 && before[k - 1].is_whitespace() {
        k -= 1;
    }
    if k == 0 || before[k - 1] != ':' {
        return None;
    }
    k -= 1;
    while k > 0 && before[k - 1].is_whitespace() {
        k -= 1;
    }
    let end = k;
    while k > 0 && (before[k - 1].is_alphanumeric() || before[k - 1] == '.' || before[k - 1] == '_') {
        k -= 1;
    }
    (k < end).then(|| before[k..end].iter().collect())
}

fn is_handler_function(line: &str) -> bool {
    let t = line.trim_start();
    let Some(rest) = t.strip_prefix("function ") else { return false };
    if !rest.starts_with("on") {
        return false;
    }
    let Some(open) = rest.find('{') else { return false };
    let Some(close) = rest.rfind('}') else { return false };
    let body = rest[open + 1..close].trim();
    !body.is_empty()
        && !body.contains(';')
        && !body.contains('{')
        && body.ends_with(')')
        && !body.contains(" if ")
        && !body.starts_with("if")
}

/// Every finding in one file's text.
pub fn check(file: &str, text: &str, options: Options) -> Vec<Finding> {
    check_as(file, text, options, false)
}

/// Every finding in one file of a theme: §1.3's rules as a view's, with the theme's imports (`THEME_IMPORTS`, and quoted relative folders).
pub fn check_theme(file: &str, text: &str) -> Vec<Finding> {
    check_as(file, text, Options::default(), true)
}

/// A file's quoted folder imports (`import "../parts"`), with their lines, for a caller that knows where the file sits.
pub fn local_imports(text: &str) -> Vec<(usize, String)> {
    let clean = strip_comments(text);
    clean
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            line.trim().strip_prefix("import \"").and_then(|rest| rest.split('"').next()).map(|p| (i + 1, p.to_owned()))
        })
        .collect()
}

fn theme_import_allowed(module: &str) -> bool {
    match module.strip_prefix('"').and_then(|m| m.strip_suffix('"')) {
        Some(folder) => !folder.is_empty() && !folder.starts_with('/') && !folder.contains(':'),
        None => THEME_IMPORTS.contains(&module),
    }
}

fn check_as(file: &str, text: &str, options: Options, theme: bool) -> Vec<Finding> {
    let clean = strip_comments(text);
    let mut findings = Vec::new();
    let mut push =
        |line: usize, rule: Rule, words: &str| findings.push(Finding { file: file.to_owned(), line, rule, words: words.trim().to_owned() });
    for (number, line) in clean.lines().enumerate() {
        let n = number + 1;
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(module) = t.strip_prefix("import ") {
            let module = module.split_whitespace().next().unwrap_or("");
            if theme {
                if !theme_import_allowed(module) {
                    push(n, Rule::Import, module);
                }
                continue;
            }
            let allowed = ALLOWED_IMPORTS.iter().any(|a| {
                module == a.trim_end_matches('.') || module.starts_with(a) && (a.ends_with('.') || module[a.len()..].starts_with('.'))
            });
            if !allowed || FORBIDDEN_IMPORTS.iter().any(|f| module == *f || module.starts_with(&format!("{f}."))) {
                push(n, Rule::Import, module);
            } else if module == "EmuSen.Kit" && !t.ends_with(" as Kit") {
                push(n, Rule::Import, "EmuSen.Kit unqualified: import EmuSen.Kit as Kit");
            }
            continue;
        }
        if t.starts_with("pragma ") {
            continue;
        }
        if (t.contains("function ") || t.contains("function(")) && !is_handler_function(t) {
            push(n, Rule::Function, t);
        }
        if t.contains("Qt.callLater") {
            push(n, Rule::CallLater, t);
        }
        if !options.fixture && (t.contains("ListElement") || (t.starts_with("ListModel") && t.contains('{'))) {
            push(n, Rule::RowsInQml, t);
        }
        if t.starts_with("Component.onCompleted") && !t.contains("forceActiveFocus()") {
            push(n, Rule::Completed, t);
        }
        if let Some(colon) = t.find(':') {
            let value = t[colon + 1..].trim_start();
            if !options.fixture && ((value.starts_with('[') && value != "[") || (value.starts_with('{') && value.contains(':'))) {
                push(n, Rule::Literal, t);
            }
        }
        if !options.fixture {
            for (s, binding) in strings_in(t) {
                let binding = binding.unwrap_or_default();
                // A binding ending in Role names a model's role, as ComboBox's textRole and valueRole do.
                let named = binding.ends_with("Role") || NAMED_BINDINGS.iter().any(|b| binding == *b || binding.ends_with(&format!(".{b}")));
                if !named && !s.is_empty() && !is_colour(&s) {
                    push(n, Rule::Words, &format!("\"{s}\""));
                }
            }
        }
    }
    findings
}

/// Every `.qml` under a directory, in path order.
pub fn qml_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return files };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            files.extend(qml_files(&path));
        } else if path.extension().is_some_and(|e| e == "qml") {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// A whole tree: findings, and the last line `ok` or `N findings`.
pub fn check_tree(dir: &Path, options: Options) -> (Vec<Finding>, String) {
    let mut findings = Vec::new();
    for path in qml_files(dir) {
        if let Ok(text) = std::fs::read_to_string(&path) {
            findings.extend(check(&path.display().to_string(), &text, options));
        }
    }
    let summary = if findings.is_empty() { "ok".to_owned() } else { format!("{} findings", findings.len()) };
    (findings, summary)
}

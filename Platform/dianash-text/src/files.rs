//! The commands that read and write files: each resolves its paths through the call, as the C# resolved them
//! through `DianaShSandbox`, and words its refusals the same way.

use crate::{clock, Answer, Run};
use dianash_module::Call;
use emusen_dotnet::file::{io_message, read_all_text};
use emusen_dotnet::number::parse_i32;
use emusen_dotnet::text::{compare_ordinal, equals_ignore_case, is_white_space, simple_upper};
use std::path::Path;

/// A path through the sandbox, or the refusal the command words as `<name>: <refusal>`.
fn resolve(call: &Call, name: &str, path: &str) -> Result<String, Answer> {
    call.resolve(path).map_err(|refusal| Answer::fail(format!("{name}: {refusal}")))
}

/// `File.Exists`: there and not a directory, a dangling link counted as a file; never a path ending in a separator.
fn file_exists(path: &str) -> bool {
    if path.is_empty() || path.ends_with('/') || path.ends_with(std::path::MAIN_SEPARATOR) {
        return false;
    }
    match std::fs::metadata(path) {
        Ok(metadata) => !metadata.is_dir(),
        Err(_) => std::fs::symlink_metadata(path).is_ok_and(|link| link.file_type().is_symlink()),
    }
}

fn directory_exists(path: &str) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_dir())
}

/// `Path.GetFileName`.
fn file_name(path: &str) -> &str {
    path.rsplit(['/', std::path::MAIN_SEPARATOR]).next().unwrap_or(path)
}

/// `Path.Combine` of two parts, the second relative.
fn combine(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        return name.to_string();
    }
    if dir.ends_with('/') || dir.ends_with(std::path::MAIN_SEPARATOR) { format!("{dir}{name}") } else { format!("{dir}{}{name}", std::path::MAIN_SEPARATOR) }
}

pub fn cat(call: &Call, args: &[String], stdin: Option<String>) -> Run {
    if args.len() < 2 {
        return Ok(match stdin {
            Some(text) => Answer::ok(text),
            None => Answer::fail("cat: usage: cat <path>..."),
        });
    }
    let mut text = String::new();
    for path in &args[1..] {
        let resolved = match resolve(call, "cat", path) {
            Ok(r) => r,
            Err(answer) => return Ok(answer),
        };
        if !file_exists(&resolved) {
            return Ok(Answer::fail(format!("cat: no such file: {path}")));
        }
        text.push_str(&read_all_text(&resolved)?);
    }
    Ok(Answer::ok(text.trim_end_matches('\n')))
}

/// `head` and `tail`, which differ only in which end they keep.
pub fn head(call: &Call, args: &[String], stdin: Option<String>, tail: bool) -> Run {
    let name = if tail { "tail" } else { "head" };
    let mut count = 10i32;
    let mut path: Option<&str> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-n" && i + 1 < args.len() {
            i += 1;
            match parse_i32(&args[i]) {
                Some(n) if n >= 0 => count = n,
                _ => return Ok(Answer::fail(format!("{name}: -n expects a non-negative integer"))),
            }
        } else {
            path = Some(&args[i]);
        }
        i += 1;
    }
    let text = match (stdin, path) {
        (Some(text), _) => text,
        (None, Some(path)) => {
            let resolved = match resolve(call, name, path) {
                Ok(r) => r,
                Err(answer) => return Ok(answer),
            };
            if !file_exists(&resolved) {
                return Ok(Answer::fail(format!("{name}: no such file: {path}")));
            }
            read_all_text(&resolved)?
        }
        (None, None) => return Ok(Answer::fail(format!("{name}: usage: {name} [-n N] [path]"))),
    };
    if text.is_empty() {
        return Ok(Answer::ok(""));
    }
    let lines: Vec<&str> = text.split('\n').collect();
    let count = count as usize;
    let kept = if tail { &lines[lines.len().saturating_sub(count)..] } else { &lines[..count.min(lines.len())] };
    Ok(Answer::ok(kept.join("\n")))
}

pub fn wc(call: &Call, args: &[String], stdin: Option<String>) -> Run {
    let (mut lines, mut words, mut bytes) = (false, false, false);
    let mut path: Option<&str> = None;
    for arg in &args[1..] {
        match arg.as_str() {
            "-l" => lines = true,
            "-w" => words = true,
            "-c" => bytes = true,
            other => path = Some(other),
        }
    }
    let mut resolved = None;
    if stdin.is_none() {
        if let Some(path) = path {
            match resolve(call, "wc", path) {
                Ok(r) => resolved = Some(r),
                Err(answer) => return Ok(answer),
            }
        }
    }
    let text = match (stdin, resolved) {
        (Some(text), _) => text,
        (None, Some(file)) => read_all_text(&file)?,
        (None, None) => String::new(),
    };
    // The file's count is one more than the same text piped (§13.4 finding 14): the C# split counts the newline a file ends in.
    let line_count = if text.is_empty() { 0 } else { text.split('\n').count() };
    let word_count = text.split(is_white_space).filter(|w| !w.is_empty()).count();
    let all = !lines && !words && !bytes;
    let mut fields = Vec::new();
    if all || lines {
        fields.push(line_count.to_string());
    }
    if all || words {
        fields.push(word_count.to_string());
    }
    if all || bytes {
        fields.push(text.len().to_string());
    }
    if let Some(path) = path {
        fields.push(path.to_string());
    }
    Ok(Answer::ok(fields.join(" ")))
}

fn entry(dir: &str, name: &str, long: bool) -> String {
    let full = combine(dir, name);
    let is_dir = directory_exists(&full);
    if !long {
        return if is_dir { format!("{name}/") } else { name.to_string() };
    }
    let metadata = std::fs::metadata(&full).ok();
    let size = if is_dir { 0 } else { metadata.as_ref().map_or(0, |m| m.len()) };
    let modified = metadata.and_then(|m| m.modified().ok()).map(clock::local).unwrap_or_else(|| "1601-01-01 00:00:00".to_string());
    format!("{} {size:>10} {modified} {name}", if is_dir { 'd' } else { '-' })
}

/// The names in a directory, in .NET's ordinal order.
fn names(dir: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir).map(|r| r.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect()).unwrap_or_default();
    names.sort_by(|a, b| compare_ordinal(a, b));
    names
}

pub fn ls(call: &Call, args: &[String]) -> Run {
    let (mut all, mut long) = (false, false);
    let mut path = ".";
    for arg in &args[1..] {
        match arg.as_str() {
            "-a" => all = true,
            "-l" => long = true,
            "-al" | "-la" => {
                all = true;
                long = true;
            }
            other => path = other,
        }
    }
    let resolved = match resolve(call, "ls", path) {
        Ok(r) => r,
        Err(answer) => return Ok(answer),
    };
    if file_exists(&resolved) {
        let parent = Path::new(&resolved).parent().map(|p| p.to_string_lossy().into_owned()).filter(|p| !p.is_empty()).unwrap_or_else(|| ".".to_string());
        return Ok(Answer::ok(entry(&parent, file_name(&resolved), long)));
    }
    if !directory_exists(&resolved) {
        return Ok(Answer::fail(format!("ls: no such file or directory: {path}")));
    }
    let lines: Vec<String> = names(&resolved).iter().filter(|n| all || !n.starts_with('.')).map(|n| entry(&resolved, n, long)).collect();
    Ok(Answer::ok(lines.join("\n")))
}

/// `FileSystemName.MatchesSimpleExpression`, case ignored: `*` any run, `?` one character, `\` the next literally.
fn matches(pattern: &[char], name: &[char]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some(('*', rest)) => (0..=name.len()).any(|skip| matches(rest, &name[skip..])),
        Some(('?', rest)) => !name.is_empty() && matches(rest, &name[1..]),
        Some(('\\', rest)) if !rest.is_empty() => !name.is_empty() && simple_upper(rest[0]) == simple_upper(name[0]) && matches(&rest[1..], &name[1..]),
        Some((c, rest)) => !name.is_empty() && simple_upper(*c) == simple_upper(name[0]) && matches(rest, &name[1..]),
    }
}

pub fn find(call: &Call, args: &[String]) -> Run {
    let mut path = ".";
    let mut pattern: Option<Vec<char>> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-name" && i + 1 < args.len() {
            i += 1;
            pattern = Some(args[i].chars().collect());
        } else {
            path = &args[i];
        }
        i += 1;
    }
    let resolved = match resolve(call, "find", path) {
        Ok(r) => r,
        Err(answer) => return Ok(answer),
    };
    if !directory_exists(&resolved) && !file_exists(&resolved) {
        return Ok(Answer::fail(format!("find: no such file or directory: {path}")));
    }
    let mut results = Vec::new();
    walk(call, &resolved, path, pattern.as_deref(), &mut results);
    Ok(Answer::ok(results.join("\n")))
}

fn walk(call: &Call, resolved: &str, shown: &str, pattern: Option<&[char]>, results: &mut Vec<String>) {
    let name: Vec<char> = file_name(resolved).chars().collect();
    if pattern.is_none_or(|p| matches(p, &name)) {
        results.push(shown.to_string());
    }
    if !directory_exists(resolved) || call.cancelled() {
        return;
    }
    for child in names(resolved) {
        let child_shown = if shown == "." { child.clone() } else { combine(shown, &child) };
        walk(call, &combine(resolved, &child), &child_shown, pattern, results);
    }
}

pub fn cp(call: &Call, args: &[String]) -> Run {
    let recursive = args.len() >= 2 && (equals_ignore_case(&args[1], "-r") || equals_ignore_case(&args[1], "-rf"));
    let start = if recursive { 2 } else { 1 };
    if args.len() < start + 2 {
        return Ok(Answer::fail("cp: usage: cp [-r] <src> <dst>"));
    }
    let src = match resolve(call, "cp", &args[start]) {
        Ok(r) => r,
        Err(answer) => return Ok(answer),
    };
    let mut dst = match resolve(call, "cp", &args[start + 1]) {
        Ok(r) => r,
        Err(answer) => return Ok(answer),
    };
    if directory_exists(&dst) {
        dst = combine(&dst, file_name(src.trim_end_matches(['/', '\\'])));
    }
    let copied = if directory_exists(&src) {
        if !recursive {
            return Ok(Answer::fail(format!("cp: '{}' is a directory (use -r to copy directories)", args[start])));
        }
        copy_directory(&src, &dst)
    } else if file_exists(&src) {
        copy_file(&src, &dst)
    } else {
        return Ok(Answer::fail(format!("cp: no such file or directory: {}", args[start])));
    };
    Ok(match copied {
        Ok(()) => Answer::ok(""),
        Err(words) => Answer::fail(format!("cp: {words}")),
    })
}

/// `File.Copy(src, dst, overwrite: true)`, which refuses a file onto itself as being in use rather than truncating it.
fn copy_file(src: &str, dst: &str) -> Result<(), String> {
    let same = std::fs::canonicalize(src).ok().zip(std::fs::canonicalize(dst).ok()).is_some_and(|(a, b)| a == b);
    if same {
        return Err(format!("The process cannot access the file '{dst}' because it is being used by another process."));
    }
    std::fs::copy(src, dst).map(|_| ()).map_err(|e| io_message(&e, dst))
}

fn copy_directory(src: &str, dst: &str) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| io_message(&e, dst))?;
    for name in names(src) {
        let (from, to) = (combine(src, &name), combine(dst, &name));
        if directory_exists(&from) {
            copy_directory(&from, &to)?;
        } else {
            copy_file(&from, &to)?;
        }
    }
    Ok(())
}

/// `mv` and `rm`, parked while a published root holds its own install (`man rm`).
pub fn suspended(name: &str) -> Answer {
    Answer::fail(format!(
        "{name}: suspended - a published build's own binaries live inside this root now (/lib/EmuSen), and nothing here distinguishes them from your data. See 'man {name}' and 'man hier'."
    ))
}

pub fn touch(call: &Call, args: &[String]) -> Run {
    if args.len() < 2 {
        return Ok(Answer::fail("touch: usage: touch <path>"));
    }
    let resolved = match resolve(call, "touch", &args[1]) {
        Ok(r) => r,
        Err(answer) => return Ok(answer),
    };
    if directory_exists(&resolved) {
        return Ok(Answer::fail(format!("touch: '{}' is a directory", args[1])));
    }
    let touched = if file_exists(&resolved) {
        std::fs::OpenOptions::new().write(true).open(&resolved).and_then(|f| f.set_modified(std::time::SystemTime::now()))
    } else {
        std::fs::write(&resolved, [])
    };
    Ok(match touched {
        Ok(()) => Answer::ok(""),
        Err(e) => Answer::fail(format!("touch: {}", io_message(&e, &resolved))),
    })
}

pub fn xxd(call: &Call, args: &[String], stdin: Option<String>) -> Run {
    let data = match stdin {
        Some(text) => text.into_bytes(),
        None => {
            if args.len() < 2 {
                return Ok(Answer::fail("xxd: usage: xxd <path>"));
            }
            let resolved = match resolve(call, "xxd", &args[1]) {
                Ok(r) => r,
                Err(answer) => return Ok(answer),
            };
            if !file_exists(&resolved) {
                return Ok(Answer::fail(format!("xxd: no such file: {}", args[1])));
            }
            std::fs::read(&resolved).map_err(|e| io_message(&e, &resolved))?
        }
    };
    if data.is_empty() {
        return Ok(Answer::ok(""));
    }
    let mut out = String::new();
    for (row, chunk) in data.chunks(16).enumerate() {
        out.push_str(&format!("  {:06X}: ", row * 16));
        let mut ascii = String::new();
        for col in 0..16 {
            match chunk.get(col) {
                Some(b) => {
                    out.push_str(&format!("{b:02X} "));
                    ascii.push(if (0x20..0x7F).contains(b) { *b as char } else { '.' });
                }
                None => out.push_str("   "),
            }
        }
        out.push(' ');
        out.push_str(&ascii);
        out.push('\n');
    }
    Ok(Answer::ok(out.trim_end_matches(is_white_space)))
}

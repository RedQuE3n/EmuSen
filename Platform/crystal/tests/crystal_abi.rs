//! The frontend API held to its header and to its baseline (EmuSen_Backend.md §3.2, §14): every constant's value,
//! every function's signature, every struct's size and every field's offset, the C side read from `emusen_crystal.h`
//! by a C compiler and the Rust side from `emusen_crystal::abi`; the header compiled as C99 and as C++17; each of the
//! session service's functions held to its twin, both ways and in order (§3.8); and any `emusen_crystal_` export of
//! the platform library held to the header. The facts are then held to `Platform/crystal/abi/v1/baseline.txt`: a fact
//! missing from it fails until the baseline is recorded with it (`CRYSTAL_ABI_EMIT=1`), a line of it no longer a fact
//! fails always, and `CRYSTAL_ABI_AGAINST=<file>`, the base branch's baseline, fails on any line of that file this one
//! removed or changed.

use emusen_crystal::abi::*;
use std::collections::{BTreeMap, BTreeSet};
use std::mem::{align_of, offset_of, size_of};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn include_dir() -> PathBuf {
    repo().join("EmuSen.Crystal/include")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn header() -> String {
    read(&include_dir().join("emusen_crystal.h"))
}

fn session_header() -> String {
    read(&include_dir().join("emusen_session.h"))
}

/// A header's code: its comments and its preprocessor lines removed.
fn code(header: &str) -> String {
    let mut out = String::new();
    let mut rest = header;
    while let Some(at) = rest.find("/*") {
        out.push_str(&rest[..at]);
        rest = rest[at..].split_once("*/").map_or("", |(_, after)| after);
    }
    out.push_str(rest);
    out.lines().filter(|l| !l.trim_start().starts_with('#')).collect::<Vec<_>>().join("\n")
}

/// `#define EMUSEN_CRYSTAL_* value` of the header, each value evaluated: numbers, `<<`, `|` and the names before it.
fn defines(header: &str) -> BTreeMap<String, i64> {
    let mut found = BTreeMap::new();
    let mut without_comments = String::new();
    let mut rest = header;
    while let Some(at) = rest.find("/*") {
        without_comments.push_str(&rest[..at]);
        rest = rest[at..].split_once("*/").map_or("", |(_, after)| after);
    }
    without_comments.push_str(rest);
    for line in without_comments.lines() {
        let Some(rest) = line.trim().strip_prefix("#define ") else { continue };
        let Some((name, value)) = rest.split_once(' ') else { continue };
        if !name.starts_with("EMUSEN_CRYSTAL_") || name == "EMUSEN_CRYSTAL_H" {
            continue;
        }
        let value = value.replace(['(', ')'], "");
        let evaluated = evaluate(&value, &found).unwrap_or_else(|| panic!("{name}: cannot read {value}"));
        found.insert(name.to_string(), evaluated);
    }
    found
}

fn evaluate(expression: &str, found: &BTreeMap<String, i64>) -> Option<i64> {
    let operand = |text: &str| -> Option<i64> {
        let text = text.trim();
        let number = text.strip_suffix('u').filter(|n| n.chars().all(|c| c.is_ascii_digit() || c == '-')).unwrap_or(text);
        number.parse().ok().or_else(|| found.get(text).copied())
    };
    let mut value = 0;
    for part in expression.split('|') {
        let mut shifts = part.split("<<");
        let mut term = operand(shifts.next()?)?;
        for shift in shifts {
            term <<= operand(shift)?;
        }
        value |= term;
    }
    Some(value)
}

fn normal(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").replace(" *", "*").replace('*', " *").trim().to_string()
}

/// A parameter's type: its name dropped, its pointer stars kept.
fn parameter(text: &str) -> String {
    let text = text.trim();
    let name_start = text.trim_end_matches(|c: char| c.is_ascii_alphanumeric() || c == '_').len();
    let before = &text[..name_start];
    if before.trim().is_empty() {
        return normal(text);
    }
    normal(before)
}

/// Each function a header's code declares whose name starts with `prefix`, in order, with its return and parameter types.
fn prototypes(code: &str, prefix: &str) -> Vec<(String, String, Vec<String>)> {
    declared(code, prefix, false)
}

/// The same, each parameter its type and its name when `named`, as a twin's must match its table entry's by name too.
fn declared(code: &str, prefix: &str, named: bool) -> Vec<(String, String, Vec<String>)> {
    let mut found = Vec::new();
    let flat = code.replace('\n', " ");
    for statement in flat.split(';') {
        let statement = statement.trim();
        let Some(open) = statement.find('(') else { continue };
        let head = statement[..open].trim();
        let Some(name) = head.rsplit([' ', '*']).next() else { continue };
        if !name.starts_with(prefix) || statement.contains("typedef") || statement.contains('{') {
            continue;
        }
        let returns = normal(head[..head.len() - name.len()].trim());
        let params = statement[open + 1..statement.rfind(')').unwrap()].split(',').map(|p| if named { normal(p) } else { parameter(p.trim()) }).collect();
        found.push((name.to_string(), returns, params));
    }
    found
}

/// The session service's table: each function's name, return type and parameter types after its ctx, in order.
fn table_functions(named: bool) -> Vec<(String, String, Vec<String>)> {
    let text = session_header();
    let start = text.find("typedef struct emusen_session_v1 {").unwrap();
    let end = text.find("} emusen_session_v1;").unwrap();
    let body = code(&text[start..end]).replace('\n', " ");
    let mut found = Vec::new();
    for statement in body.split(';') {
        let statement = statement.trim();
        let Some(star) = statement.find("(*") else { continue };
        let returns = normal(statement[..star].trim().rsplit('{').next().unwrap());
        let name_end = statement[star..].find(')').unwrap() + star;
        let name = statement[star + 2..name_end].to_string();
        let params_text = &statement[statement[name_end..].find('(').unwrap() + name_end + 1..statement.rfind(')').unwrap()];
        let params: Vec<String> = params_text.split(',').map(|p| if named { normal(p) } else { parameter(p.trim()) }).collect();
        assert_eq!(params[0], if named { "void *ctx" } else { "void *" }, "{name}: the table's first parameter is ctx");
        found.push((name, returns, params[1..].to_vec()));
    }
    found
}

/// A C type as the Rust side spells it.
fn rust_type(c: &str) -> String {
    let map = |base: &str| match base {
        "uint8_t" => "u8",
        "int16_t" => "i16",
        "uint32_t" => "u32",
        "int32_t" => "i32",
        "int64_t" => "i64",
        "uint64_t" => "u64",
        "size_t" => "usize",
        "double" => "f64",
        "void" => "()",
        other => other,
    }
    .to_string();
    let constant = c.starts_with("const ");
    let stars = c.matches('*').count();
    let base = c.trim_start_matches("const ").trim_end_matches(['*', ' ']).trim();
    let mut out = map(base);
    for i in 0..stars {
        out = format!("*{} {out}", if constant && i == 0 { "const" } else { "mut" });
    }
    out
}

/// `extern "C" fn emusen_crystal_*(args) -> ret` of a Rust source, as `(ret, [arg types])`.
fn rust_exports(source: &str) -> BTreeMap<String, (String, Vec<String>)> {
    let mut found = BTreeMap::new();
    for chunk in source.split("extern \"C\" fn ").skip(1) {
        let name: String = chunk.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        if !name.starts_with("emusen_crystal_") {
            continue;
        }
        let open = chunk.find('(').unwrap();
        let mut depth = 0;
        let mut close = open;
        for (i, c) in chunk[open..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = open + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let args: Vec<String> = chunk[open + 1..close]
            .split(',')
            .map(str::trim)
            .filter(|a| !a.is_empty())
            .map(|a| a.split_once(':').map_or(a, |(_, ty)| ty).trim().split_whitespace().collect::<Vec<_>>().join(" "))
            .collect();
        let after = &chunk[close + 1..];
        let returns = after.split('{').next().unwrap().trim().strip_prefix("->").map_or("()".to_string(), |r| r.trim().to_string());
        found.insert(name, (returns, if args.is_empty() { vec!["()".to_string()] } else { args }));
    }
    found
}

fn rust_sources(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "target" && n != "tests") {
                rust_sources(&path, into);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}

#[test]
fn every_constant_is_the_header_s() {
    let defines = defines(&header());
    let rust: Vec<(&str, i64)> = vec![
        ("EMUSEN_CRYSTAL_MAJOR", MAJOR.into()),
        ("EMUSEN_CRYSTAL_MINOR", MINOR.into()),
        ("EMUSEN_CRYSTAL_VERSION", VERSION.into()),
        ("EMUSEN_CRYSTAL_NO_ENGINE", status::NO_ENGINE.into()),
        ("EMUSEN_CRYSTAL_FIRMWARE", status::FIRMWARE.into()),
        ("EMUSEN_CRYSTAL_BUSY", status::BUSY.into()),
        ("EMUSEN_CRYSTAL_BAD_STRUCT", status::BAD_STRUCT.into()),
        ("EMUSEN_CRYSTAL_IO", status::IO.into()),
        ("EMUSEN_CRYSTAL_STATE_REFUSED", status::STATE_REFUSED.into()),
        ("EMUSEN_CRYSTAL_OPEN_NO_BATTERY", OPEN_NO_BATTERY.into()),
        ("EMUSEN_CRYSTAL_OPEN_SINGLE_ROWS", OPEN_SINGLE_ROWS.into()),
        ("EMUSEN_CRYSTAL_SPEED_NORMAL", SPEED_NORMAL.into()),
        ("EMUSEN_CRYSTAL_SPEED_TURBO", SPEED_TURBO.into()),
        ("EMUSEN_CRYSTAL_SPEED_SLOW", SPEED_SLOW.into()),
        ("EMUSEN_CRYSTAL_SPEED_DRAW_NONE", SPEED_DRAW_NONE.into()),
        ("EMUSEN_CRYSTAL_SCOPE_CONSOLE", SCOPE_CONSOLE.into()),
        ("EMUSEN_CRYSTAL_SCOPE_GAME", SCOPE_GAME.into()),
        ("EMUSEN_CRYSTAL_SERVICE_READ_ONLY", SERVICE_READ_ONLY.into()),
        ("EMUSEN_CRYSTAL_EVENT_SESSION", EVENT_SESSION.into()),
        ("EMUSEN_CRYSTAL_EVENT_RUN", EVENT_RUN.into()),
        ("EMUSEN_CRYSTAL_EVENT_STATE", EVENT_STATE.into()),
        ("EMUSEN_CRYSTAL_EVENT_REWIND", EVENT_REWIND.into()),
        ("EMUSEN_CRYSTAL_EVENT_SETTINGS", EVENT_SETTINGS.into()),
        ("EMUSEN_CRYSTAL_EVENT_DISC", EVENT_DISC.into()),
        ("EMUSEN_CRYSTAL_EVENT_LOG", EVENT_LOG.into()),
        ("EMUSEN_CRYSTAL_CARDS_LIST", CARDS_LIST.into()),
        ("EMUSEN_CRYSTAL_CARDS_OPEN", CARDS_OPEN.into()),
        ("EMUSEN_CRYSTAL_CARDS_COPY", CARDS_COPY.into()),
        ("EMUSEN_CRYSTAL_CARDS_MOVE", CARDS_MOVE.into()),
        ("EMUSEN_CRYSTAL_CARDS_DELETE", CARDS_DELETE.into()),
        ("EMUSEN_CRYSTAL_CARDS_IMPORT", CARDS_IMPORT.into()),
        ("EMUSEN_CRYSTAL_CARDS_NEW", CARDS_NEW.into()),
        ("EMUSEN_CRYSTAL_CARDS_DUPLICATE", CARDS_DUPLICATE.into()),
        ("EMUSEN_CRYSTAL_CARDS_RENAME", CARDS_RENAME.into()),
    ];
    let rust: BTreeMap<&str, i64> = rust.into_iter().collect();
    assert_eq!(defines.keys().map(String::as_str).collect::<BTreeSet<_>>(), rust.keys().copied().collect(), "every define has a Rust constant");
    for (name, value) in &rust {
        assert_eq!(defines[*name], *value, "{name}");
    }
}

/// The statuses sit in the platform's band after the session service's (EmuSen_Backend.md §3.2, D12).
#[test]
fn the_statuses_follow_the_session_service_s_in_the_band() {
    for (name, value) in defines(&header()) {
        let status_names = ["NO_ENGINE", "FIRMWARE", "BUSY", "BAD_STRUCT", "IO", "STATE_REFUSED"];
        if status_names.iter().any(|s| name == format!("EMUSEN_CRYSTAL_{s}")) {
            assert!((-1139..=-1109).contains(&value), "{name} = {value}: the frontend API's statuses are -1109 to -1139");
        }
    }
}

/// The service's functions whose twin the API already had under another name: 1.3's `games` is `systems` (§3.11).
const NAMED_TWINS: [(&str, &str); 1] = [("games", "systems")];

/// Every session-service function has its twin, the runtime in place of ctx and every other parameter the same, in the
/// table's order; and the header's twins are exactly the table's functions (EmuSen_Backend.md §3.8, SR4).
#[test]
fn every_session_service_function_has_its_twin_both_ways() {
    let h = header();
    let marker = "The twins of emusen_session.h";
    let at = h.find(marker).expect("the header marks where its twins begin");
    let all: BTreeMap<String, (String, Vec<String>)> = declared(&code(&h), "emusen_crystal_", true).into_iter().map(|(n, r, p)| (n, (r, p))).collect();
    let twins: Vec<String> = prototypes(&code(&h[at..]), "emusen_crystal_").into_iter().map(|(n, _, _)| n).collect();
    let table = table_functions(true);
    assert_eq!(table.len(), 82, "the session service's 1.3 table has 82 functions");

    for (name, returns, params) in &table {
        let named = NAMED_TWINS.iter().find(|(n, _)| n == name).map(|(_, t)| *t);
        let twin = format!("emusen_crystal_{}", named.unwrap_or(name));
        let (twin_returns, twin_params) = all.get(&twin).unwrap_or_else(|| panic!("{name} has no twin {twin}"));
        assert_eq!(twin_returns, returns, "{twin}'s return");
        assert_eq!(twin_params[0], "emusen_crystal_runtime *rt", "{twin} takes the runtime first");
        assert_eq!(&twin_params[1..], &params[..], "{twin}'s parameters after the runtime, by type and by name");
    }
    let expected: Vec<String> = table
        .iter()
        .filter(|(n, _, _)| n != "last_error" && !NAMED_TWINS.iter().any(|(t, _)| t == n))
        .map(|(n, _, _)| format!("emusen_crystal_{n}"))
        .collect();
    assert_eq!(twins, expected, "the header's twins, in order, are the table's functions but last_error, declared with the library's calls, and those named elsewhere");
}

/// Any `emusen_crystal_` function the platform's crates export is declared by the header with its signature. At 1.0
/// none is: the C# implementation serves the API until step B7 exports it whole.
#[test]
fn every_export_is_the_header_s() {
    let declared: BTreeMap<String, (String, Vec<String>)> = prototypes(&code(&header()), "emusen_crystal_").into_iter().map(|(n, r, p)| (n, (r, p))).collect();
    let mut sources = Vec::new();
    rust_sources(&repo().join("Platform"), &mut sources);
    for source in sources {
        for (name, (returns, params)) in rust_exports(&read(&source)) {
            let (c_returns, c_params) = declared.get(&name).unwrap_or_else(|| panic!("{} exports {name}, which emusen_crystal.h does not declare", source.display()));
            let expected: Vec<String> = c_params.iter().map(|p| rust_type(p)).collect();
            let expected = if expected == ["()"] || expected.is_empty() { vec!["()".to_string()] } else { expected };
            assert_eq!((rust_type(c_returns), expected), (returns, params), "{name} in {}", source.display());
        }
    }
}

/// Every struct as Rust lays it out: its size and alignment, and each field's offset.
fn rust_layouts() -> Vec<(&'static str, usize, usize, Vec<(&'static str, usize)>)> {
    macro_rules! layout {
        ($c:literal, $t:ty, [$($f:ident),*]) => {
            ($c, size_of::<$t>(), align_of::<$t>(), vec![$((stringify!($f), offset_of!($t, $f))),*])
        };
    }
    vec![
        layout!("emusen_crystal_config", emusen_crystal_config, [size, flags]),
        layout!("emusen_crystal_open_params", emusen_crystal_open_params, [size, flags, path, path_len, console, console_len, engine, engine_len, audio_buffer_max_samples, reserved, device]),
        layout!(
            "emusen_crystal_frame_info",
            emusen_crystal_frame_info,
            [size, ran, halted, rendered, picture_serial, width, height, row_repeat, display_lines, display_aspect, audio_frames, audio_rate, play_audio, speed_percent, frame_interval_ns, total_frames, frame_rate_hz, paused, halted_processor, halted_address, picture_on_device, reserved]
        ),
        layout!("emusen_crystal_raw_info", emusen_crystal_raw_info, [size, element, width, height, stride, reserved]),
    ]
}

/// A C or C++ compiler run on a probe that includes the header; none when the machine has no such compiler.
fn compile(compiler: &str, standard: &str, source: &str, extension: &str) -> Option<PathBuf> {
    let dir = std::env::temp_dir().join(format!("crystal-abi-{}-{}", std::process::id(), standard.replace('+', "p")));
    std::fs::create_dir_all(&dir).ok()?;
    let file = dir.join(format!("probe.{extension}"));
    std::fs::write(&file, source).ok()?;
    let exe = dir.join("probe");
    let built = Command::new(compiler)
        .arg(format!("-std={standard}"))
        .args(["-Wall", "-Wextra", "-Werror", "-pedantic", "-I"])
        .arg(include_dir())
        .arg(&file)
        .arg("-o")
        .arg(&exe)
        .output()
        .ok()?;
    assert!(built.status.success(), "emusen_crystal.h does not compile cleanly as {standard}:\n{}", String::from_utf8_lossy(&built.stderr));
    Some(exe)
}

/// The header as C99 and as C++17, every warning an error, and the layouts as the C compiler gives them.
fn c_layouts() -> Option<BTreeMap<String, String>> {
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let mut probe = String::from("#include <stdio.h>\n#include <stddef.h>\n#include \"emusen_crystal.h\"\nint main(void) {\n");
    for (name, _, _, fields) in rust_layouts() {
        probe += &format!("    printf(\"struct {name} size %lu align %lu\\n\", (unsigned long)sizeof({name}), (unsigned long)offsetof(struct {{ char c; {name} t; }}, t));\n");
        for (field, _) in fields {
            probe += &format!("    printf(\"field {name}.{field} offset %lu\\n\", (unsigned long)offsetof({name}, {field}));\n");
        }
    }
    probe += "    return 0;\n}\n";
    let cxx = std::env::var("CXX").unwrap_or_else(|_| "c++".to_string());
    let plus = "#include \"emusen_crystal.h\"\nint main() { emusen_crystal_frame_info f = emusen_crystal_frame_info(); f.size = sizeof f; return static_cast<int>(f.ran); }\n";
    let _ = compile(&cxx, "c++17", plus, "cpp");
    let exe = compile(&cc, "c99", &probe, "c")?;
    let ran = Command::new(&exe).output().ok()?;
    let _ = std::fs::remove_dir_all(exe.parent().unwrap());
    Some(
        String::from_utf8_lossy(&ran.stdout)
            .lines()
            .map(|line| {
                let mut words = line.splitn(3, ' ');
                let key = format!("{} {}", words.next().unwrap(), words.next().unwrap());
                (key, words.next().unwrap().to_string())
            })
            .collect(),
    )
}

/// Every fact of the API, keyed by kind and name, valued as the baseline records them.
fn facts() -> BTreeMap<String, String> {
    let mut facts = BTreeMap::new();
    for (name, value) in defines(&header()) {
        if name.ends_with("_MINOR") || name.ends_with("_VERSION") {
            continue;
        }
        facts.insert(format!("const {name}"), format!("{value}"));
    }
    for (name, returns, params) in prototypes(&code(&header()), "emusen_crystal_") {
        facts.insert(format!("fn {name}"), format!("{returns} ({})", params.join(", ")));
    }
    for (name, size, align, fields) in rust_layouts() {
        facts.insert(format!("struct {name}"), format!("size {size} align {align}"));
        for (field, offset) in fields {
            facts.insert(format!("field {name}.{field}"), format!("offset {offset}"));
        }
    }
    facts
}

#[test]
fn the_rust_layouts_are_the_c_compiler_s() {
    let Some(c) = c_layouts() else {
        eprintln!("no C compiler: the layouts were not compared");
        return;
    };
    let rust = facts();
    for (key, value) in &c {
        assert_eq!(rust.get(key), Some(value), "{key}");
    }
    assert_eq!(c.len(), rust.keys().filter(|k| k.starts_with("struct ") || k.starts_with("field ")).count());
}

fn baseline_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("abi/v1/baseline.txt")
}

/// A baseline's lines by key, each `(value, tag)`.
fn parse_baseline(text: &str) -> BTreeMap<String, (String, String)> {
    let mut lines = BTreeMap::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let (fact, tag) = line.rsplit_once(" @").unwrap_or_else(|| panic!("no @tag: {line}"));
        let mut words = fact.splitn(3, ' ');
        let key = format!("{} {}", words.next().unwrap(), words.next().unwrap());
        lines.insert(key, (words.next().unwrap_or("").to_string(), tag.to_string()));
    }
    lines
}

/// A struct fact's size and alignment.
fn layout(value: &str) -> Option<(usize, usize)> {
    let words: Vec<&str> = value.split_whitespace().collect();
    match words.as_slice() {
        ["size", size, "align", align] => Some((size.parse().ok()?, align.parse().ok()?)),
        _ => None,
    }
}

/// Whether the header's struct is the baseline's with fields appended: larger, its alignment the same. The baseline's
/// line then stands as the size a callee still accepts, the core ABI's rule (EmuSen_CoreAPI.md §4.2), and the new
/// fields' own lines record the growth, each at or past that size.
fn grown(key: &str, was: &str, now: &str, facts: &BTreeMap<String, String>) -> bool {
    let (Some(name), Some((was_size, was_align)), Some((now_size, now_align))) = (key.strip_prefix("struct "), layout(was), layout(now)) else { return false };
    now_size > was_size && was_align == now_align
        && facts.iter().filter_map(|(k, v)| k.strip_prefix(&format!("field {name}.")).map(|_| v)).all(|v| v.strip_prefix("offset ").and_then(|o| o.parse::<usize>().ok()).is_some())
}

const BASELINE_HEADER: &str = "# The frontend API, emusen_crystal.h, version 1: one fact per line, sorted; struct layouts are a 64-bit target's.\n# Written by `CRYSTAL_ABI_EMIT=1 cargo test -p emusen-crystal --test crystal_abi`; a line is never edited once its minor is released.\n# See EmuSen_Backend.md section 14.\n";

#[test]
fn the_baseline_holds_every_fact_and_loses_none() {
    if size_of::<usize>() != 8 {
        return;
    }
    let facts = facts();
    let path = baseline_path();
    let recorded = std::fs::read_to_string(&path).map(|t| parse_baseline(&t)).unwrap_or_default();
    let minor = format!("@1.{MINOR}");
    if std::env::var("CRYSTAL_ABI_EMIT").as_deref() == Ok("1") {
        let mut text = BASELINE_HEADER.to_string();
        for (key, value) in &facts {
            match recorded.get(key) {
                Some((was, tag)) if was == value || grown(key, was, value, &facts) => text += &format!("{key} {was} @{tag}\n"),
                _ => text += &format!("{key} {value} {minor}\n"),
            }
        }
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        return;
    }
    for (key, (value, tag)) in &recorded {
        let now = facts.get(key);
        assert!(now == Some(value) || now.is_some_and(|n| grown(key, value, n, &facts)), "baseline line `{key} {value} @{tag}` is no longer a fact: a removed or changed line needs a new major");
        if let (Some(name), Some((size, _))) = (key.strip_prefix("struct "), layout(value)) {
            for (field, offset) in facts.iter().filter_map(|(k, v)| k.strip_prefix(&format!("field {name}.")).map(|f| (f, v))) {
                let recorded_field = recorded.contains_key(&format!("field {name}.{field}"));
                let at: usize = offset.trim_start_matches("offset ").parse().unwrap();
                assert!(recorded_field || at >= size, "field {name}.{field} at {at} is inside the struct's recorded size {size}: a field may only be appended");
            }
        }
    }
    for (key, value) in &facts {
        assert!(recorded.contains_key(key), "`{key} {value}` is not in the baseline; record it with CRYSTAL_ABI_EMIT=1 in the same change");
    }
    if let Ok(against) = std::env::var("CRYSTAL_ABI_AGAINST") {
        for (key, (value, tag)) in parse_baseline(&read(Path::new(&against))) {
            assert_eq!(recorded.get(&key), Some(&(value.clone(), tag.clone())), "the base's `{key} {value} @{tag}` was removed or changed");
        }
    }
}

//! DianaSh's two surfaces held to their headers and to their baseline (EmuSen_DianaSh.md §11.4, §13.7): every
//! constant's value, every export's signature, every struct's size and every field's offset, the C side read from
//! `dianash.h` and `dianash_module.h` by a C compiler and the Rust side from the crates. The facts are then held to
//! `Platform/dianash/abi/v1/baseline.txt`: a fact missing from it fails until the baseline is recorded with it
//! (`DIANASH_ABI_EMIT=1`), a line of it no longer a fact fails always, and `DIANASH_ABI_AGAINST=<file>`, the base
//! branch's baseline, fails on any line of that file this one removed or changed.

use emusen_dianash::abi::*;
use std::collections::{BTreeMap, BTreeSet};
use std::mem::{align_of, offset_of, size_of};
use std::path::{Path, PathBuf};
use std::process::Command;

fn dianash_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../dianash")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn headers() -> String {
    read(&dianash_dir().join("include/dianash_module.h")) + &read(&dianash_dir().join("include/dianash.h"))
}

/// The code of a header, its comments removed.
fn code(header: &str) -> String {
    let mut out = String::new();
    let mut rest = header;
    while let Some(at) = rest.find("/*") {
        out.push_str(&rest[..at]);
        rest = rest[at..].split_once("*/").map_or("", |(_, after)| after);
    }
    out.push_str(rest);
    out
}

/// `#define NAME value` of the headers, each value evaluated: numbers, `<<`, `|` and the names defined before it.
fn defines(header: &str) -> BTreeMap<String, i64> {
    let mut found = BTreeMap::new();
    for line in code(header).lines() {
        let Some(rest) = line.trim().strip_prefix("#define ") else { continue };
        let Some((name, value)) = rest.split_once(' ') else { continue };
        if !name.starts_with("DIANASH_") {
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

/// Each `dianash_` function the headers declare, with its return type and parameter types, names dropped.
fn prototypes(header: &str) -> BTreeMap<String, (String, Vec<String>)> {
    let mut found = BTreeMap::new();
    let flat = code(header).replace('\n', " ");
    for statement in flat.split(';') {
        let statement = statement.trim();
        let Some(open) = statement.find('(') else { continue };
        let head = statement[..open].trim();
        let Some(name) = head.rsplit([' ', '*']).next() else { continue };
        if !name.starts_with("dianash_") || statement.contains("typedef") || statement.contains("{") {
            continue;
        }
        let returns = normal(head[..head.len() - name.len()].trim());
        let params = statement[open + 1..statement.rfind(')').unwrap()].split(',').map(|p| parameter(p.trim())).collect();
        found.insert(name.to_string(), (returns, params));
    }
    found
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

/// A C type as the Rust side spells it.
fn rust_type(c: &str) -> String {
    let map = |base: &str| match base {
        "uint8_t" => "u8",
        "uint32_t" => "u32",
        "int32_t" => "i32",
        "int64_t" => "i64",
        "uint64_t" => "u64",
        "size_t" => "usize",
        "dianash_shell" => "DianashShell",
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

/// `pub unsafe extern "C" fn name(args) -> ret` of a Rust source, as `(ret, [arg types])`.
fn rust_exports(source: &str) -> BTreeMap<String, (String, Vec<String>)> {
    let mut found = BTreeMap::new();
    for chunk in source.split("extern \"C\" fn ").skip(1) {
        let name: String = chunk.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
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

#[test]
fn the_host_api_is_exported_as_the_header_declares_it() {
    let declared = prototypes(&headers());
    let exported = rust_exports(include_str!("../src/dianash.rs"));
    let host: BTreeSet<&String> = declared.keys().filter(|n| !n.starts_with("dianash_module_")).collect();
    assert_eq!(host, exported.keys().collect::<BTreeSet<_>>(), "the header's host API and the library's exports");
    for (name, (returns, params)) in &exported {
        let (c_returns, c_params) = &declared[name];
        let expected: Vec<String> = c_params.iter().map(|p| rust_type(p)).collect();
        assert_eq!((rust_type(c_returns), expected), (returns.clone(), params.clone()), "{name}");
    }
}

#[test]
fn every_constant_is_the_header_s() {
    let defines = defines(&headers());
    let rust: Vec<(&str, i64)> = vec![
        ("DIANASH_MODULE_ABI_MAJOR", MODULE_ABI_MAJOR.into()),
        ("DIANASH_MODULE_ABI_MINOR", MODULE_ABI_MINOR.into()),
        ("DIANASH_MODULE_ABI_VERSION", MODULE_ABI_VERSION.into()),
        ("DIANASH_ABI_MAJOR", emusen_dianash::HOST_ABI_MAJOR.into()),
        ("DIANASH_ABI_MINOR", emusen_dianash::HOST_ABI_MINOR.into()),
        ("DIANASH_ABI_VERSION", emusen_dianash::HOST_ABI_VERSION.into()),
        ("DIANASH_OK", status::OK.into()),
        ("DIANASH_NULL", status::NULL.into()),
        ("DIANASH_BAD_STRING", status::BAD_STRING.into()),
        ("DIANASH_BUFFER_TOO_SMALL", status::BUFFER_TOO_SMALL.into()),
        ("DIANASH_NOT_SUPPORTED", status::NOT_SUPPORTED.into()),
        ("DIANASH_BAD_STRUCT", status::BAD_STRUCT.into()),
        ("DIANASH_UNKNOWN_COMMAND", status::UNKNOWN_COMMAND.into()),
        ("DIANASH_SERVICE_MISSING", status::SERVICE_MISSING.into()),
        ("DIANASH_BUSY", status::BUSY.into()),
        ("DIANASH_ABSENT", status::ABSENT.into()),
        ("DIANASH_EXISTS", status::EXISTS.into()),
        ("DIANASH_NO_SUCH_SESSION", status::NO_SUCH_SESSION.into()),
        ("DIANASH_LAST_SESSION", status::LAST_SESSION.into()),
        ("DIANASH_BAD_MANIFEST", status::BAD_MANIFEST.into()),
        ("DIANASH_OUTSIDE", status::OUTSIDE.into()),
        ("DIANASH_FAULT", status::FAULT.into()),
        ("DIANASH_CALL_READ_ONLY_CONTEXT", CALL_READ_ONLY_CONTEXT.into()),
        ("DIANASH_CALL_INTERACTIVE", CALL_INTERACTIVE.into()),
        ("DIANASH_RESOLVE_READ", RESOLVE_READ.into()),
        ("DIANASH_RESOLVE_WRITE", RESOLVE_WRITE.into()),
        ("DIANASH_RESOLVE_REMOVE", RESOLVE_REMOVE.into()),
        ("DIANASH_DONE", DONE.into()),
        ("DIANASH_NEEDS_MORE", NEEDS_MORE.into()),
        ("DIANASH_EXIT", EXIT.into()),
        ("DIANASH_CONFIG_SESSIONS", CONFIG_SESSIONS.into()),
        ("DIANASH_CONFIG_INTERACTIVE", CONFIG_INTERACTIVE.into()),
    ];
    let rust: BTreeMap<&str, i64> = rust.into_iter().collect();
    assert_eq!(defines.keys().map(String::as_str).collect::<BTreeSet<_>>(), rust.keys().copied().collect(), "every define has a Rust constant");
    for (name, value) in &rust {
        assert_eq!(defines[*name], *value, "{name}");
    }
}

/// Every struct of both headers as Rust lays it out: its size and alignment, and each field's offset.
fn rust_layouts() -> Vec<(&'static str, usize, usize, Vec<(&'static str, usize)>)> {
    macro_rules! layout {
        ($c:literal, $t:ty, [$($f:ident),*]) => {
            ($c, size_of::<$t>(), align_of::<$t>(), vec![$((stringify!($f), offset_of!($t, $f))),*])
        };
    }
    vec![
        layout!("dianash_bytes", dianash_bytes, [data, len]),
        layout!("dianash_terminal", dianash_terminal, [size, columns, rows, raw_capable]),
        layout!("dianash_io", dianash_io, [size, ctx, out, err, resolve, display, cancelled, slot, environment, terminal]),
        layout!("dianash_service", dianash_service, [size, name, major, minor, table]),
        layout!("dianash_call", dianash_call, [size, command, argv, argc, stdin_data, stdin_present, flags, cwd, user, io, services, service_count]),
        layout!("dianash_result", dianash_result, [size, exit_status]),
        layout!("dianash_config", dianash_config, [size, flags, root, budget_ms]),
        layout!("dianash_host_result", dianash_host_result, [size, exit_status, request]),
        layout!("dianash_host_commands", dianash_host_commands, [size, manifest, ctx, run, jobs, kill]),
    ]
}

/// The same layouts as a C compiler gives them from the headers, or none on a machine without one.
fn c_layouts() -> Option<BTreeMap<String, String>> {
    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let dir = std::env::temp_dir().join(format!("dianash-abi-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    let mut probe = String::from("#include <stdio.h>\n#include <stddef.h>\n#include \"dianash.h\"\nint main(void) {\n");
    for (name, _, _, fields) in rust_layouts() {
        probe += &format!("    printf(\"struct {name} size %zu align %zu\\n\", sizeof({name}), _Alignof({name}));\n");
        for (field, _) in fields {
            probe += &format!("    printf(\"field {name}.{field} offset %zu\\n\", offsetof({name}, {field}));\n");
        }
    }
    probe += "    return 0;\n}\n";
    std::fs::write(dir.join("probe.c"), probe).ok()?;
    let built = Command::new(&compiler)
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic", "-I"])
        .arg(dianash_dir().join("include"))
        .arg(dir.join("probe.c"))
        .arg("-o")
        .arg(dir.join("probe"))
        .output()
        .ok()?;
    assert!(built.status.success(), "the headers do not compile cleanly:\n{}", String::from_utf8_lossy(&built.stderr));
    let ran = Command::new(dir.join("probe")).output().ok()?;
    let _ = std::fs::remove_dir_all(&dir);
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

/// Every fact of the two surfaces, keyed by kind and name, valued as the baseline records them.
fn facts() -> BTreeMap<String, String> {
    let mut facts = BTreeMap::new();
    for (name, value) in defines(&headers()) {
        if name.ends_with("_MINOR") || name.ends_with("_VERSION") {
            continue;
        }
        facts.insert(format!("const {name}"), format!("{value}"));
    }
    for (name, (returns, params)) in prototypes(&headers()) {
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
}

fn baseline_path() -> PathBuf {
    dianash_dir().join("abi/v1/baseline.txt")
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

const BASELINE_HEADER: &str = "# DianaSh's host API and command module ABI, version 1: one fact per line, sorted; struct layouts are a 64-bit target's.\n# Written by `DIANASH_ABI_EMIT=1 cargo test -p emusen-platform --test dianash_abi`; a line is never edited once its minor is released.\n# See EmuSen_DianaSh.md section 11.4.\n";

#[test]
fn the_baseline_holds_every_fact_and_loses_none() {
    if size_of::<usize>() != 8 {
        return;
    }
    let facts = facts();
    let path = baseline_path();
    let recorded = std::fs::read_to_string(&path).map(|t| parse_baseline(&t)).unwrap_or_default();
    let minor = format!("@1.{}", emusen_dianash::HOST_ABI_MINOR);
    if std::env::var("DIANASH_ABI_EMIT").as_deref() == Ok("1") {
        let mut text = BASELINE_HEADER.to_string();
        for (key, value) in &facts {
            let tag = recorded.get(key).filter(|(v, _)| v == value).map_or(minor.clone(), |(_, tag)| format!("@{tag}"));
            text += &format!("{key} {value} {tag}\n");
        }
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        return;
    }
    for (key, (value, tag)) in &recorded {
        assert_eq!(facts.get(key), Some(value), "baseline line `{key} {value} @{tag}` is no longer a fact: a removed or changed line needs a new major");
    }
    for (key, value) in &facts {
        assert!(recorded.contains_key(key), "`{key} {value}` is not in the baseline; record it with DIANASH_ABI_EMIT=1 in the same change");
    }
    if let Ok(against) = std::env::var("DIANASH_ABI_AGAINST") {
        for (key, (value, tag)) in parse_baseline(&read(Path::new(&against))) {
            assert_eq!(recorded.get(&key), Some(&(value.clone(), tag.clone())), "the base's `{key} {value} @{tag}` was removed or changed");
        }
    }
}

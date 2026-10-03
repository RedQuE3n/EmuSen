//! The core ABI's guard (EmuSen_CoreAPI.md §5.2, §5.4).
//!
//! `emusen-core-abi-check [--check] [--against BASE]` regenerates the facts from the header for every triple and
//! fails on any difference from the committed baseline, and, given the base branch's baseline, on any line it
//! removed or changed. `--emit-baseline` writes the baseline, keeping each line's tag and tagging new lines with
//! the header's minor. The Rust half of the check is this crate's build (`rust_check`).

#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case, dead_code, clippy::all)]

mod c {
    include!(concat!(env!("OUT_DIR"), "/c.rs"));
}
#[allow(unused_imports)]
mod sig {
    include!(concat!(env!("OUT_DIR"), "/sig.rs"));
}
mod macros {
    include!(concat!(env!("OUT_DIR"), "/macros.rs"));
}
#[path = "../../emusen-native/examples/v1_test_core.rs"]
mod test_core;
mod facts;
mod rust_check;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use facts::Facts;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn header_path() -> PathBuf {
    crate_dir().join("../emusen-native/include/emusen_core.h")
}

fn baseline_path() -> PathBuf {
    crate_dir().join("../emusen-native/abi/v1/baseline.txt")
}

/// A baseline line's fact and its tag, by key.
type Baseline = BTreeMap<String, (String, String)>;

fn parse_baseline(text: &str) -> Result<Baseline, String> {
    let mut out = Baseline::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (fact, tag) = line.rsplit_once(" @").ok_or(format!("baseline line {}: no @tag", n + 1))?;
        let mut words = fact.splitn(3, ' ');
        let key = format!("{} {}", words.next().unwrap_or(""), words.next().unwrap_or(""));
        out.insert(key, (words.next().unwrap_or("").to_owned(), tag.to_owned()));
    }
    Ok(out)
}

/// The rule of §4.3 a removal or a change of this kind of fact breaks.
fn rule(key: &str, removed: bool) -> &'static str {
    match key.split(' ').next().unwrap_or("") {
        "fn" if removed => "rule 1, removing or renaming an export",
        "fn" => "rule 2, changing an export's parameters or return",
        "struct" | "field" => "rule 3, changing a struct field's offset, type or meaning, or shrinking a struct",
        "const" | "status" | "enum" => "rule 4, reusing or renumbering a constant",
        "export" if removed => "rule 1, removing an export",
        "export" => "rule 5, making an optional export required, or adding a required one",
        "json" => "rule 7, removing a descriptor field or changing its type",
        _ => "section 4.3",
    }
}

/// Everything the header says, the same for every triple, with the cross-checks against the Rust.
fn generate(problems: &mut Vec<String>) -> Result<Facts, String> {
    clang_sys::load().map_err(|e| format!("libclang: {e}"))?;
    let path = header_path();
    let mut first: Option<facts::Header> = None;
    for triple in facts::TRIPLES {
        let h = facts::header(&path, triple)?;
        problems.extend(h.problems.iter().cloned());
        if let Some(f) = &first {
            for (k, v) in &h.facts {
                if f.facts.get(k) != Some(v) {
                    problems.push(format!("{triple}: {k} {v} differs from {}: {:?}", facts::TRIPLES[0], f.facts.get(k)));
                }
            }
        } else {
            first = Some(h);
        }
    }
    let h = first.unwrap();
    let set = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<BTreeSet<String>>();
    let compare = |what: &str, header: BTreeSet<String>, rust: BTreeSet<String>, problems: &mut Vec<String>| {
        for x in header.difference(&rust) {
            problems.push(format!("{what}: {x} is in the header but not checked against the Rust"));
        }
        for x in rust.difference(&header) {
            problems.push(format!("{what}: {x} is checked but not in the header"));
        }
    };
    let functions: BTreeSet<String> = h.functions.iter().cloned().collect();
    compare("exports assigned", functions.clone(), set(rust_check::ASSIGNED), problems);
    compare("exports listed", functions, sys_exports(), problems);
    compare("struct fields", h.fields.iter().cloned().collect(), set(rust_check::FIELDS), problems);
    let mut constants: BTreeSet<String> = macros::MACROS.iter().map(|m| m.0.to_owned()).collect();
    constants.extend(h.enum_constants.iter().cloned());
    compare("constants", constants, set(rust_check::CONSTANTS), problems);

    let mut all = h.facts;
    all.extend(facts::constants(macros::MACROS));
    all.extend(facts::exports());
    all.extend(facts::descriptors()?);
    Ok(all)
}

fn sys_exports() -> BTreeSet<String> {
    emusen_native::core::sys::EXPORTS.iter().map(|e| e.0.to_owned()).collect()
}

fn minor() -> u32 {
    emusen_native::core::sys::ABI_MINOR
}

fn emit(facts: &Facts, old: &Baseline) -> String {
    let mut lines: Vec<String> = facts
        .iter()
        .map(|(k, v)| {
            let tag = match old.get(k) {
                Some((ov, tag)) if ov == v => tag.clone(),
                _ => format!("1.{}", minor()),
            };
            format!("{k} {v} @{tag}")
        })
        .collect();
    lines.sort();
    let mut text = String::from(
        "# The core ABI, version 1: one fact per line, sorted, the same on every target triple the project builds.\n\
         # Generated by `emusen-core-abi-check --emit-baseline`; a line is never edited once its minor is released.\n\
         # See EmuSen_CoreAPI.md section 5.4.\n",
    );
    for l in lines {
        text.push_str(&l);
        text.push('\n');
    }
    text
}

fn check(facts: &Facts, committed: &Baseline, base: Option<&Baseline>, problems: &mut Vec<String>) {
    for (k, v) in facts {
        match committed.get(k) {
            None => problems.push(format!("added in the header but not in the baseline: {k} {v} (section 5.5: add it with --emit-baseline)")),
            Some((cv, _)) if cv != v => problems.push(format!("changed: {k} was {cv}, the header now says {v}; this breaks version 1 ({})", rule(k, false))),
            _ => {}
        }
    }
    for (k, (v, _)) in committed {
        if !facts.contains_key(k) {
            problems.push(format!("removed: {k} {v} is in the baseline and not in the header; this breaks version 1 ({})", rule(k, true)));
        }
    }
    let Some(base) = base else { return };
    let mut added = false;
    for (k, (v, tag)) in base {
        match committed.get(k) {
            None => problems.push(format!("removed from the baseline: {k} {v} @{tag}; this breaks version 1 ({})", rule(k, true))),
            Some((cv, ct)) if cv != v || ct != tag => problems.push(format!("changed in the baseline: {k} {v} @{tag} became {cv} @{ct}; this breaks version 1 ({})", rule(k, false))),
            _ => {}
        }
    }
    let released = base.values().filter_map(|(_, t)| t.strip_prefix("1.")?.parse::<u32>().ok()).max();
    for (k, (_, tag)) in committed {
        if !base.contains_key(k) {
            added = true;
            if *tag != format!("1.{}", minor()) {
                problems.push(format!("{k} is new and must be tagged @1.{}, the header's minor", minor()));
            }
        }
    }
    if let (true, Some(r)) = (added, released) {
        if minor() <= r {
            problems.push(format!("the baseline gains lines but EMUSEN_CORE_ABI_MINOR is still {}; an addition increments it (section 5.5)", minor()));
        }
    }
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn run() -> Result<Vec<String>, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut problems = Vec::new();
    let facts = generate(&mut problems)?;
    let committed = parse_baseline(&read(&baseline_path()).unwrap_or_default())?;
    if let Some(i) = args.iter().position(|a| a == "--emit-baseline") {
        let out = args.get(i + 1).map(PathBuf::from).unwrap_or_else(baseline_path);
        std::fs::write(&out, emit(&facts, &committed)).map_err(|e| format!("{}: {e}", out.display()))?;
        eprintln!("wrote {} facts to {}", facts.len(), out.display());
        return Ok(problems);
    }
    let base = match args.iter().position(|a| a == "--against") {
        Some(i) => Some(parse_baseline(&read(Path::new(args.get(i + 1).ok_or("--against needs a file")?))?)?),
        None => None,
    };
    check(&facts, &committed, base.as_ref(), &mut problems);
    if problems.is_empty() {
        eprintln!("{} facts agree with the header on {} triples, with the Rust, and with the baseline", facts.len(), facts::TRIPLES.len());
    }
    Ok(problems)
}

fn main() -> ExitCode {
    match run() {
        Ok(p) if p.is_empty() => ExitCode::SUCCESS,
        Ok(p) => {
            for line in p {
                eprintln!("error: {line}");
            }
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(lines: &str) -> Baseline {
        parse_baseline(lines).unwrap()
    }

    #[test]
    fn the_header_and_the_rust_agree_and_the_baseline_is_current() {
        let mut problems = Vec::new();
        let facts = generate(&mut problems).unwrap();
        let committed = base(&read(&baseline_path()).unwrap());
        check(&facts, &committed, None, &mut problems);
        assert_eq!(problems, Vec::<String>::new());
        assert_eq!(facts.keys().filter(|k| k.starts_with("fn ")).count(), 55);
        assert_eq!(facts.get("struct emusen_frame_info").map(String::as_str), Some("size 56 align 8"));
        assert_eq!(facts.get("field emusen_frame_info.serial").map(String::as_str), Some("offset 40 type int64_t"));
        assert_eq!(facts.get("fn emusen_core_advance").map(String::as_str), Some("(emusen_machine*, uint64_t*) -> int32_t"));
        assert_eq!(facts.get("json info.systems[].extensions").map(String::as_str), Some("array<string> required"));
    }

    #[test]
    fn a_removed_or_changed_line_names_the_rule_it_breaks() {
        let old = base("fn emusen_core_x () -> int32_t @1.0\nstruct s size 8 align 8 @1.0\n");
        let new = base("fn emusen_core_x (uint32_t) -> int32_t @1.0\n");
        let facts: Facts = [("fn emusen_core_x".to_owned(), "(uint32_t) -> int32_t".to_owned())].into();
        let mut p = Vec::new();
        check(&facts, &new, Some(&old), &mut p);
        assert_eq!(p.len(), 2);
        assert!(p[0].starts_with("changed in the baseline: fn emusen_core_x") && p[0].contains("rule 2"), "{}", p[0]);
        assert!(p[1].starts_with("removed from the baseline: struct s") && p[1].contains("rule 3"), "{}", p[1]);
    }

    #[test]
    fn an_addition_needs_the_baseline_and_a_new_minor() {
        let old = base("const EMUSEN_A 0x1 @1.0\n");
        let facts: Facts = [("const EMUSEN_A".to_owned(), "0x1".to_owned()), ("const EMUSEN_B".to_owned(), "0x2".to_owned())].into();
        let mut p = Vec::new();
        check(&facts, &old, Some(&old), &mut p);
        assert_eq!(p, vec!["added in the header but not in the baseline: const EMUSEN_B 0x2 (section 5.5: add it with --emit-baseline)"]);
        let new = base("const EMUSEN_A 0x1 @1.0\nconst EMUSEN_B 0x2 @1.0\n");
        let mut p = Vec::new();
        check(&facts, &new, Some(&old), &mut p);
        assert_eq!(p, vec!["the baseline gains lines but EMUSEN_CORE_ABI_MINOR is still 0; an addition increments it (section 5.5)"]);
    }
}

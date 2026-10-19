//! The core ABI's guard (EmuSen_CoreAPI.md §5.2, §5.4).
//!
//! `emusen-core-abi-check [--check] [--against BASE]` regenerates the facts from the header for every triple and
//! fails on any difference from the committed baseline, and, given the base branch's baseline, on any line it
//! removed or changed. `--emit-baseline` writes the baseline, keeping each line's tag and tagging new lines with
//! the header's minor. A struct's line is its size at the minor that introduced it, the least a callee accepts: a
//! struct grown by appended fields keeps its line, and the fields' own lines record the growth (§4.2, §5.4). The
//! Rust half of the check is this crate's build (`rust_check`).

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

/// The header's own revision, which every minor moves up and which is therefore no promise (§5.5).
fn revision(key: &str) -> bool {
    matches!(key, "const EMUSEN_CORE_ABI_MINOR" | "const EMUSEN_CORE_ABI_VERSION")
}

/// A struct fact's size and alignment.
fn layout(value: &str) -> Option<(u64, u64)> {
    let mut words = value.split(' ');
    match (words.next(), words.next(), words.next(), words.next(), words.next()) {
        (Some("size"), Some(size), Some("align"), Some(align), None) => Some((size.parse().ok()?, align.parse().ok()?)),
        _ => None,
    }
}

/// Whether the header's struct is the baseline's with fields appended: larger, its alignment the same (§4.2). The
/// baseline's line then stands as the size a callee still accepts; every earlier field keeps its own line.
fn grown(key: &str, was: &str, now: &str) -> bool {
    key.starts_with("struct ") && matches!((layout(was), layout(now)), (Some((was, a)), Some((now, b))) if now > was && a == b)
}

/// A grown struct's appended field: one at or past the baseline's size, without which the growth is unexplained.
fn appended(facts: &Facts, key: &str, was: &str) -> bool {
    let (Some(name), Some((size, _))) = (key.strip_prefix("struct "), layout(was)) else { return false };
    let prefix = format!("field {name}.");
    facts.iter().any(|(k, v)| k.starts_with(&prefix) && v.strip_prefix("offset ").and_then(|o| o.split(' ').next()?.parse::<u64>().ok()).is_some_and(|o| o >= size))
}

/// A revision fact's number, as the baseline writes it.
fn number(value: &str) -> Option<u64> {
    u64::from_str_radix(value.strip_prefix("0x")?, 16).ok()
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
        .map(|(k, v)| match old.get(k) {
            Some((ov, tag)) if (ov == v || grown(k, ov, v)) && !revision(k) => format!("{k} {ov} @{tag}"),
            _ => format!("{k} {v} @1.{}", minor()),
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
            Some((cv, _)) if cv != v && revision(k) => problems.push(format!("{k} is {v} in the header and {cv} in the baseline (section 5.5: regenerate it with --emit-baseline)")),
            Some((cv, _)) if grown(k, cv, v) && appended(facts, k, cv) => {}
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
    for (k, (v, tag)) in base {
        match committed.get(k) {
            None => problems.push(format!("removed from the baseline: {k} {v} @{tag}; this breaks version 1 ({})", rule(k, true))),
            Some((cv, _)) if revision(k) && number(cv) < number(v) => problems.push(format!("{k} went down from {v} to {cv}; a minor only moves up (section 4.1)")),
            Some(_) if revision(k) => {}
            Some((cv, ct)) if cv != v || ct != tag => problems.push(format!("changed in the baseline: {k} {v} @{tag} became {cv} @{ct}; this breaks version 1 ({})", rule(k, false))),
            _ => {}
        }
    }
    // Each minor's lines arrive together: a new line names a minor the base does not hold yet, no later than the
    // header's, so two minors built side by side may reach the baseline in either order (section 5.5).
    let held: BTreeSet<u32> = base.values().filter_map(|(_, t)| t.strip_prefix("1.")?.parse::<u32>().ok()).collect();
    for (k, (_, tag)) in committed {
        if !base.contains_key(k) && !revision(k) {
            let n = tag.strip_prefix("1.").and_then(|n| n.parse::<u32>().ok());
            if !n.is_some_and(|n| n <= minor() && !held.contains(&n)) {
                problems.push(format!("{k} is new and tagged @{tag}; it must name a minor the baseline does not hold yet, no later than the header's 1.{} (section 5.5)", minor()));
            }
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
        assert_eq!(facts.keys().filter(|k| k.starts_with("fn ")).count(), 65);
        assert_eq!(facts.get("struct emusen_media").map(String::as_str), Some("size 48 align 8"));
        assert_eq!(facts.get("struct emusen_create_params").map(String::as_str), Some("size 112 align 8"), "grown at 1.3, 1.9 and 1.11");
        assert_eq!(committed.get("struct emusen_create_params"), Some(&("size 88 align 8".to_owned(), "1.0".to_owned())), "its 1.0 line stands");
        assert_eq!(committed.get("field emusen_create_params.flags"), Some(&("offset 88 type uint32_t".to_owned(), "1.3".to_owned())));
        assert_eq!(facts.get("const EMUSEN_CAP_BORROW").map(String::as_str), Some("0x100000"));
        assert_eq!(committed.get("json info.systems[].battery"), Some(&("array<object> optional".to_owned(), "1.3".to_owned())));
        assert_eq!(facts.get("const EMUSEN_FILE_MEDIA").map(String::as_str), Some("0x10000"));
        assert_eq!(facts.get("const EMUSEN_CAP_DEBUG_READS").map(String::as_str), Some("0x80000"));
        assert_eq!(facts.get("struct emusen_frame_info").map(String::as_str), Some("size 64 align 8"), "grown at 1.13");
        assert_eq!(committed.get("struct emusen_frame_info"), Some(&("size 56 align 8".to_owned(), "1.0".to_owned())), "its 1.0 line stands");
        assert_eq!(committed.get("field emusen_frame_info.lines"), Some(&("offset 56 type int32_t".to_owned(), "1.13".to_owned())));
        assert_eq!(committed.get("json machine_info.spaces[].byte_order"), Some(&("string optional".to_owned(), "1.14".to_owned())));
        assert_eq!(committed.get("field emusen_create_params.system"), Some(&("offset 104 type const char*".to_owned(), "1.11".to_owned())), "widened at 1.15, not moved");
        assert_eq!(committed.get("const EMUSEN_CORE_ABI_MINOR"), Some(&("0xf".to_owned(), "1.15".to_owned())));
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
        let released = format!("const EMUSEN_A 0x1 @1.{}\n", minor());
        let old = base(&released);
        let facts: Facts = [("const EMUSEN_A".to_owned(), "0x1".to_owned()), ("const EMUSEN_B".to_owned(), "0x2".to_owned())].into();
        let mut p = Vec::new();
        check(&facts, &old, Some(&old), &mut p);
        assert_eq!(p, vec!["added in the header but not in the baseline: const EMUSEN_B 0x2 (section 5.5: add it with --emit-baseline)"]);
        let new = base(&format!("{released}const EMUSEN_B 0x2 @1.{}\n", minor()));
        let mut p = Vec::new();
        check(&facts, &new, Some(&old), &mut p);
        assert_eq!(p, vec![format!("const EMUSEN_B is new and tagged @1.{0}; it must name a minor the baseline does not hold yet, no later than the header's 1.{0} (section 5.5)", minor())]);
    }

    /// §4.2's size rule in the guard: a struct may grow by a field appended at or past its recorded size, and its
    /// line is kept; it may not shrink, change alignment, or grow with nothing appended.
    #[test]
    fn a_struct_grows_only_by_appended_fields_and_keeps_its_line() {
        let old = base("struct s size 16 align 8 @1.0\nfield s.a offset 8 type uint64_t @1.0\n");
        let fact = |size: &str, fields: &[(&str, &str)]| -> Facts {
            std::iter::once(("struct s".to_owned(), size.to_owned())).chain(fields.iter().map(|(k, v)| (format!("field s.{k}"), v.to_string()))).collect()
        };
        let grown = fact("size 24 align 8", &[("a", "offset 8 type uint64_t"), ("b", "offset 16 type uint32_t")]);
        let written = emit(&grown, &old);
        assert!(written.contains("struct s size 16 align 8 @1.0\n"), "{written}");
        assert!(written.contains(&format!("field s.b offset 16 type uint32_t @1.{}\n", minor())), "{written}");
        let mut p = Vec::new();
        check(&grown, &base(&written), Some(&old), &mut p);
        assert_eq!(p, Vec::<String>::new());

        let cases = [
            (fact("size 8 align 8", &[("a", "offset 8 type uint64_t")]), "changed: struct s was size 16 align 8, the header now says size 8 align 8"),
            (fact("size 32 align 16", &[("a", "offset 8 type uint64_t"), ("b", "offset 16 type uint32_t")]), "changed: struct s was size 16 align 8, the header now says size 32 align 16"),
            (fact("size 24 align 8", &[("a", "offset 8 type uint64_t")]), "changed: struct s was size 16 align 8, the header now says size 24 align 8"),
        ];
        for (facts, says) in cases {
            let mut p = Vec::new();
            check(&facts, &old, None, &mut p);
            assert!(p.iter().any(|l| l.starts_with(says) && l.contains("rule 3")), "{p:?}");
        }
    }

    #[test]
    fn two_minors_built_side_by_side_merge_in_either_order() {
        let first = base("const EMUSEN_A 0x1 @1.0\nconst EMUSEN_C 0x3 @1.2\n");
        let both = base("const EMUSEN_A 0x1 @1.0\nconst EMUSEN_B 0x2 @1.1\nconst EMUSEN_C 0x3 @1.2\n");
        let facts: Facts = [("const EMUSEN_A", "0x1"), ("const EMUSEN_B", "0x2"), ("const EMUSEN_C", "0x3")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        let mut p = Vec::new();
        check(&facts, &both, Some(&first), &mut p);
        if minor() >= 2 {
            assert_eq!(p, Vec::<String>::new(), "a 1.1 line arriving after 1.2's is taken");
        }
    }

    #[test]
    fn the_minor_moves_up_and_nothing_else_may() {
        let old = base("const EMUSEN_CORE_ABI_MINOR 0x0 @1.0
const EMUSEN_A 0x1 @1.0
");
        let up = base("const EMUSEN_CORE_ABI_MINOR 0x1 @1.1
const EMUSEN_A 0x1 @1.0
");
        let facts: Facts = [("const EMUSEN_CORE_ABI_MINOR".to_owned(), "0x1".to_owned()), ("const EMUSEN_A".to_owned(), "0x1".to_owned())].into();
        let mut p = Vec::new();
        check(&facts, &up, Some(&old), &mut p);
        assert_eq!(p, Vec::<String>::new());
        let mut p = Vec::new();
        check(&[("const EMUSEN_CORE_ABI_MINOR".to_owned(), "0x0".to_owned()), ("const EMUSEN_A".to_owned(), "0x1".to_owned())].into(), &old, Some(&up), &mut p);
        assert_eq!(p, vec!["const EMUSEN_CORE_ABI_MINOR went down from 0x1 to 0x0; a minor only moves up (section 4.1)"]);
        let mut p = Vec::new();
        check(&facts, &old, None, &mut p);
        assert_eq!(p, vec!["const EMUSEN_CORE_ABI_MINOR is 0x1 in the header and 0x0 in the baseline (section 5.5: regenerate it with --emit-baseline)"]);
    }
}

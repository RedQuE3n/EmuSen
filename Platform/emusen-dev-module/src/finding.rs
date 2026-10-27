//! What `dupes` decides, over plain text: CPD's CSV, a unified diff, the ignore and accept files. No process, no file.

use std::collections::HashMap;

/// A run of lines of one file, both ends included.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub file: String,
    pub start: u32,
    pub end: u32,
}

impl Place {
    pub fn show(&self) -> String {
        format!("{}:{}-{}", self.file, self.start, self.end)
    }
}

/// One duplicated block CPD found: its size and every place it stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub tokens: u32,
    pub places: Vec<Place>,
}

/// A place that duplicates another, and how big the copy is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub tokens: u32,
    pub copies: usize,
    pub subject: Place,
    pub other: Place,
}

impl Finding {
    pub fn line(&self) -> String {
        format!("{}  duplicates  {}  ({} tokens)", self.subject.show(), self.other.show(), self.tokens)
    }
}

/// The lines each file gained, as inclusive ranges.
pub type Added = HashMap<String, Vec<(u32, u32)>>;

fn fields(record: &str) -> Vec<String> {
    let (mut out, mut cur, mut quoted) = (Vec::new(), String::new(), false);
    let mut chars = record.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(std::mem::take(&mut cur)),
            c => cur.push(c),
        }
    }
    out.push(cur);
    out
}

/// CPD's CSV (`lines,tokens,occurrences,` then a line and a file for each place), the files made relative to `root`.
pub fn parse_csv(text: &str, root: &str) -> Vec<Block> {
    let prefix = format!("{}/", root.trim_end_matches('/'));
    let mut blocks = Vec::new();
    for record in text.lines().skip(1) {
        let f = fields(record);
        let nums: Option<Vec<u32>> = f.iter().take(3).map(|n| n.trim().parse().ok()).collect();
        let Some(&[lines, tokens, count]) = nums.as_deref() else { continue };
        let mut places = Vec::new();
        for i in 0..count as usize {
            let (Some(start), Some(file)) = (f.get(3 + 2 * i).and_then(|s| s.trim().parse::<u32>().ok()), f.get(4 + 2 * i)) else { break };
            places.push(Place { file: file.strip_prefix(&prefix).unwrap_or(file).to_string(), start, end: start + lines.max(1) - 1 });
        }
        if places.len() >= 2 {
            blocks.push(Block { tokens, places });
        }
    }
    blocks
}

/// The lines a `git diff -U0` adds, per file.
pub fn parse_diff(text: &str) -> Added {
    let mut added = Added::new();
    let mut file: Option<String> = None;
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("+++ ") {
            file = path.trim_end().strip_prefix("b/").map(str::to_string);
        } else if let (Some(rest), Some(f)) = (line.strip_prefix("@@ -"), &file) {
            let Some(plus) = rest.split(" +").nth(1) else { continue };
            let span = plus.split(' ').next().unwrap_or("");
            let (start, count) = match span.split_once(',') {
                Some((s, c)) => (s.parse::<u32>(), c.parse::<u32>()),
                None => (span.parse::<u32>(), Ok(1)),
            };
            if let (Ok(start), Ok(count)) = (start, count) {
                if count > 0 {
                    added.entry(f.clone()).or_default().push((start, start + count - 1));
                }
            }
        }
    }
    added
}

fn touches(place: &Place, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|&(a, b)| a <= place.end && place.start <= b)
}

/// The ignore file's patterns: `#` comments and blank lines dropped.
pub struct Ignore(Vec<Vec<String>>);

fn glob(pattern: &str, text: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == text,
        Some((head, tail)) => text.strip_prefix(head).is_some_and(|rest| (0..=rest.len()).filter(|&i| rest.is_char_boundary(i)).any(|i| glob(tail, &rest[i..]))),
    }
}

impl Ignore {
    pub fn parse(text: &str) -> Ignore {
        Ignore(text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(|l| l.trim_matches('/').split('/').map(str::to_string).collect()).collect())
    }

    /// A path is ignored when a pattern's parts (`*` stands for any text within one part) match consecutive parts of it.
    pub fn matches(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('/').collect();
        self.0.iter().any(|p| parts.windows(p.len()).any(|w| p.iter().zip(w).all(|(a, b)| glob(a, b))))
    }
}

fn parse_place(text: &str) -> Option<Place> {
    let (file, span) = text.trim().rsplit_once(':')?;
    let (a, b) = span.split_once('-')?;
    Some(Place { file: file.to_string(), start: a.parse().ok()?, end: b.parse().ok()? })
}

/// The accept file: `<place>  duplicates  <place>  <reason>` a line, the places written as a finding prints them.
pub fn parse_accept(text: &str) -> Result<Vec<(Place, Place)>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate().map(|(n, l)| (n + 1, l.trim())) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let bad = || format!("dupes.accept line {n}: expected '<file:a-b>  duplicates  <file:c-d>  <reason>'");
        let (first, rest) = line.split_once("  duplicates  ").ok_or_else(bad)?;
        let (second, reason) = rest.split_once("  ").ok_or_else(bad)?;
        let reason = reason.trim();
        let reason = reason.strip_prefix('(').and_then(|r| r.split_once(')')).filter(|(size, _)| size.ends_with(" tokens")).map_or(reason, |(_, r)| r.trim());
        if reason.is_empty() {
            return Err(format!("dupes.accept line {n}: a reason is required after the places"));
        }
        out.push((parse_place(first).ok_or_else(bad)?, parse_place(second).ok_or_else(bad)?));
    }
    Ok(out)
}

fn accepted(f: &Finding, accept: &[(Place, Place)]) -> bool {
    accept.iter().any(|(a, b)| (a == &f.subject && b == &f.other) || (a == &f.other && b == &f.subject))
}

/// What a run found: the findings at or over the minimum, and those between the warning size and it.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub failures: Vec<Finding>,
    pub warnings: Vec<Finding>,
}

impl Report {
    /// 1 when a finding is at or over the minimum, else 0.
    pub fn status(&self) -> i32 {
        i32::from(!self.failures.is_empty())
    }
}

/// Each pair of places of a block, the first a subject. With `added`, only pairs where a place overlaps the added
/// lines, that place the subject; without it, the block's first place against each other.
fn pairs(block: &Block, added: Option<&Added>) -> Vec<Finding> {
    let copies = block.places.len();
    let make = |subject: &Place, other: &Place| Finding { tokens: block.tokens, copies, subject: subject.clone(), other: other.clone() };
    let Some(added) = added else { return block.places[1..].iter().map(|o| make(&block.places[0], o)).collect() };
    let new: Vec<bool> = block.places.iter().map(|p| added.get(&p.file).is_some_and(|r| touches(p, r))).collect();
    let mut out = Vec::new();
    for i in 0..copies {
        for j in i + 1..copies {
            match (new[i], new[j]) {
                (true, _) => out.push(make(&block.places[i], &block.places[j])),
                (false, true) => out.push(make(&block.places[j], &block.places[i])),
                _ => {}
            }
        }
    }
    out
}

/// The report for `blocks`, ranked by tokens times copies, without the accepted pairs.
pub fn evaluate(blocks: &[Block], added: Option<&Added>, accept: &[(Place, Place)], warn: u32, min: u32) -> Report {
    let mut all: Vec<Finding> = blocks.iter().filter(|b| b.tokens >= warn).flat_map(|b| pairs(b, added)).filter(|f| !accepted(f, accept)).collect();
    all.sort_by(|a, b| (b.tokens as usize * b.copies).cmp(&(a.tokens as usize * a.copies)).then_with(|| a.subject.show().cmp(&b.subject.show())));
    let (failures, warnings) = all.into_iter().partition(|f| f.tokens >= min);
    Report { failures, warnings }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "/r/EmuSen Project";

    fn csv() -> String {
        // Block one: two places, 10 lines, 120 tokens; block two: three places, 6 lines, 80 tokens.
        format!(
            "lines,tokens,occurrences\n10,120,2,5,{ROOT}/a/x.rs,40,{ROOT}/b/y.rs\n6,80,3,1,{ROOT}/a/x.rs,70,{ROOT}/c/z.rs,90,{ROOT}/c/z.rs\n"
        )
    }

    fn diff(text: &str) -> Added {
        parse_diff(text)
    }

    #[test]
    fn csv_places_are_relative_and_end_at_their_last_line() {
        let b = parse_csv(&csv(), ROOT);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].places[0], Place { file: "a/x.rs".into(), start: 5, end: 14 });
        assert_eq!(b[1].places.len(), 3);
        assert_eq!(parse_csv("lines,tokens,occurrences\n3,100,2,1,\"/r/a,b.rs\",2,/r/c.rs\n", "/r")[0].places[0].file, "a,b.rs");
    }

    #[test]
    fn a_diff_gives_the_added_lines_and_drops_pure_deletions() {
        let a = diff("--- a/a/x.rs\n+++ b/a/x.rs\t\n@@ -3,0 +4,2 @@\n+p\n+q\n@@ -9 +11 @@\n-o\n+n\n@@ -20,2 +22,0 @@\n-x\n-y\n");
        assert_eq!(a["a/x.rs"], vec![(4, 5), (11, 11)]);
    }

    #[test]
    fn a_finding_must_overlap_the_added_lines() {
        let b = parse_csv(&csv(), ROOT);
        // The block at a/x.rs 5-14: lines 14 and 15 added touches it, line 15 alone does not.
        let hit = evaluate(&b, Some(&diff("+++ b/a/x.rs\n@@ -0,0 +14,2 @@\n")), &[], 75, 100);
        assert_eq!(hit.failures.len(), 1);
        assert_eq!(hit.failures[0].line(), "a/x.rs:5-14  duplicates  b/y.rs:40-49  (120 tokens)");
        let miss = evaluate(&b, Some(&diff("+++ b/a/x.rs\n@@ -0,0 +15,2 @@\n")), &[], 75, 100);
        assert!(miss.failures.is_empty() && miss.warnings.is_empty());
        let before = evaluate(&b, Some(&diff("+++ b/a/x.rs\n@@ -0,0 +1,4 @@\n")), &[], 75, 100);
        assert!(before.failures.is_empty(), "lines 1-4 end before the block at 5");
    }

    #[test]
    fn the_new_place_is_the_subject_wherever_it_stands() {
        let b = parse_csv(&csv(), ROOT);
        let r = evaluate(&b, Some(&diff("+++ b/b/y.rs\n@@ -0,0 +45 @@\n")), &[], 75, 100);
        assert_eq!(r.failures[0].line(), "b/y.rs:40-49  duplicates  a/x.rs:5-14  (120 tokens)");
    }

    #[test]
    fn without_a_base_the_whole_tree_is_ranked_by_size_times_copies() {
        let b = parse_csv(&csv(), ROOT);
        let r = evaluate(&b, None, &[], 75, 100);
        assert_eq!(r.failures.len(), 1);
        assert_eq!(r.warnings.len(), 2, "the three-place block gives two pairs");
        let mut big = b.clone();
        big[1].tokens = 99;
        let r = evaluate(&big, None, &[], 75, 100);
        assert_eq!(r.failures.len(), 1, "99 tokens is a warning, 120 a failure");
        assert_eq!(r.warnings.len(), 2);
        let mut ranked = b.clone();
        ranked[1].tokens = 98;
        ranked[0].tokens = 100;
        let all = evaluate(&ranked, None, &[], 75, 100);
        assert_eq!(all.failures[0].tokens, 100);
        let order = evaluate(&[ranked[1].clone(), ranked[0].clone()], None, &[], 75, 50);
        assert_eq!((order.failures[0].tokens, order.failures[0].copies), (98, 3), "98 x 3 outranks 100 x 2");
    }

    #[test]
    fn warnings_do_not_fail_the_run() {
        let b = vec![Block { tokens: 80, places: vec![Place { file: "a".into(), start: 1, end: 5 }, Place { file: "b".into(), start: 1, end: 5 }] }];
        let r = evaluate(&b, None, &[], 75, 100);
        assert_eq!((r.failures.len(), r.warnings.len(), r.status()), (0, 1, 0));
        let r = evaluate(&b, None, &[], 75, 80);
        assert_eq!((r.failures.len(), r.status()), (1, 1), "a finding at the minimum fails");
        let r = evaluate(&b, None, &[], 81, 100);
        assert_eq!((r.failures.len(), r.warnings.len(), r.status()), (0, 0, 0), "below the warning size is silent");
    }

    #[test]
    fn the_ignore_file_matches_parts_of_a_path() {
        let i = Ignore::parse("# comment\ntests\nboard_chip.rs\n\nEmuSen.WiseMan\n*.g.cs\nCores/Sega/gen\n");
        assert!(i.matches("EmuSen/Cores/X/tests/a.rs") && i.matches("x/board_chip.rs") && i.matches("EmuSen.WiseMan/a.cs") && i.matches("a/b.g.cs"));
        assert!(i.matches("EmuSen/Cores/Sega/gen/a.rs"));
        assert!(!i.matches("EmuSen/Cores/X/src/contests.rs") && !i.matches("a/board_chip2.rs") && !i.matches("Cores/gen/Sega/a.rs"));
    }

    #[test]
    fn the_accept_file_skips_exactly_the_places_it_names() {
        let acc = parse_accept("# why\na/x.rs:5-14  duplicates  b/y.rs:40-49  (120 tokens)  two ports of one table\n").unwrap();
        let b = parse_csv(&csv(), ROOT);
        let r = evaluate(&b, None, &acc, 75, 100);
        assert!(r.failures.is_empty(), "the pair is accepted");
        assert_eq!(r.warnings.len(), 2, "the other block is not");
        let swapped = parse_accept("b/y.rs:40-49  duplicates  a/x.rs:5-14  same table\n").unwrap();
        assert!(evaluate(&b, None, &swapped, 75, 100).failures.is_empty(), "either way round");
        let moved = parse_accept("a/x.rs:6-15  duplicates  b/y.rs:40-49  same table\n").unwrap();
        assert_eq!(evaluate(&b, None, &moved, 75, 100).failures.len(), 1, "a place that moved is no longer accepted");
        assert!(parse_accept("a/x.rs:5-14  duplicates  b/y.rs:40-49\n").is_err(), "no reason");
        assert!(parse_accept("a/x.rs:5-14  b/y.rs:40-49  because\n").is_err());
    }
}

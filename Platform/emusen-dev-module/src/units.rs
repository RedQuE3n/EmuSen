//! `hwcount`'s attribution: an address to its place (a symbol in the program, a compiled block, a library), a place to
//! a unit by a plain table of patterns, and the sampled addresses summed by unit. Pure text in, numbers out.

use std::collections::HashMap;

/// One mapping of /proc/<pid>/maps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping {
    pub start: u64,
    pub end: u64,
    pub offset: u64,
    pub path: String,
}

pub fn parse_maps(text: &str) -> Vec<Mapping> {
    let mut maps: Vec<Mapping> = text
        .lines()
        .filter_map(|line| {
            let mut p = line.split_whitespace();
            let (range, _perms, offset) = (p.next()?, p.next()?, p.next()?);
            let (_dev, _inode) = (p.next()?, p.next()?);
            let path = p.collect::<Vec<_>>().join(" ");
            let (a, b) = range.split_once('-')?;
            Some(Mapping { start: u64::from_str_radix(a, 16).ok()?, end: u64::from_str_radix(b, 16).ok()?, offset: u64::from_str_radix(offset, 16).ok()?, path })
        })
        .collect();
    maps.sort_by_key(|m| m.start);
    maps
}

/// The rows of `nm -n --defined-only -C`, code only, by address.
pub fn parse_nm(text: &str) -> Vec<(u64, String)> {
    let mut syms: Vec<(u64, String)> = text
        .lines()
        .filter_map(|line| {
            let mut p = line.splitn(3, ' ');
            let (a, kind, name) = (p.next()?, p.next()?, p.next()?);
            matches!(kind, "t" | "T" | "w" | "W").then_some((u64::from_str_radix(a, 16).ok()?, name.to_string()))
        })
        .collect();
    syms.sort_by_key(|s| s.0);
    syms
}

/// The rows of a /tmp/perf-<pid>.map: start, size, name, all hexadecimal but the name.
pub fn parse_perf_map(text: &str) -> Vec<(u64, u64, String)> {
    let mut rows: Vec<(u64, u64, String)> = text
        .lines()
        .filter_map(|line| {
            let mut p = line.splitn(3, ' ');
            Some((u64::from_str_radix(p.next()?, 16).ok()?, u64::from_str_radix(p.next()?, 16).ok()?, p.next()?.trim().to_string()))
        })
        .collect();
    rows.sort_by_key(|r| r.0);
    rows
}

/// The table of `pattern  unit` lines; `#` starts a comment; the first pattern found in a place's name wins.
pub fn parse_table(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        match line.rsplit_once("  ") {
            Some((pattern, unit)) => rows.push((pattern.trim().to_string(), unit.trim().to_string())),
            None => return Err(format!("the units table's line {} has a pattern and no unit", i + 1)),
        }
    }
    Ok(rows)
}

/// Addresses to places. `program` is the base name of the program whose symbols are `symbols`.
pub struct Places {
    maps: Vec<Mapping>,
    symbols: Vec<(u64, String)>,
    compiled: Vec<(u64, u64, String)>,
    program: String,
    base: u64,
}

impl Places {
    pub fn new(maps: Vec<Mapping>, symbols: Vec<(u64, String)>, compiled: Vec<(u64, u64, String)>, program: &str) -> Places {
        let base = maps.iter().filter(|m| base_name(&m.path) == program).map(|m| m.start - m.offset).min().unwrap_or(0);
        Places { maps, symbols, compiled, program: program.to_string(), base }
    }

    /// The name of what holds an address: a symbol, `compiled <name>`, `compiled ?` or a library's file name.
    pub fn place(&self, ip: u64) -> String {
        let i = self.maps.partition_point(|m| m.start <= ip);
        let Some(m) = i.checked_sub(1).map(|i| &self.maps[i]).filter(|m| ip < m.end) else { return "?".to_string() };
        if base_name(&m.path) == self.program {
            let off = ip - self.base;
            let j = self.symbols.partition_point(|s| s.0 <= off);
            return j.checked_sub(1).map_or("?".to_string(), |j| self.symbols[j].1.clone());
        }
        if m.path.is_empty() || m.path.starts_with("[anon") || m.path.contains("memfd") {
            let k = self.compiled.partition_point(|r| r.0 <= ip);
            return match k.checked_sub(1).map(|k| &self.compiled[k]).filter(|r| ip < r.0 + r.1) {
                Some(r) => format!("compiled {}", r.2),
                None => "compiled ?".to_string(),
            };
        }
        base_name(&m.path).to_string()
    }
}

fn base_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// A place's unit by the table, else `compiled` for compiled code, else `other`.
pub fn unit_of<'a>(place: &'a str, table: &'a [(String, String)]) -> &'a str {
    match table.iter().find(|(p, _)| place.contains(p.as_str())) {
        Some((_, unit)) => unit,
        None if place.starts_with("compiled") => "compiled",
        None => "other",
    }
}

/// One row of a sum: samples of the two events.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    pub instr: u64,
    pub cycles: u64,
}

impl Row {
    pub fn ipc(self) -> f64 {
        if self.cycles == 0 { 0.0 } else { self.instr as f64 / self.cycles as f64 }
    }
}

/// Sums sampled addresses, (address, is the instruction event), by place, each address classified once.
pub fn by_place(samples: &[(u64, bool)], places: &Places) -> HashMap<String, Row> {
    let mut named: HashMap<u64, String> = HashMap::new();
    let mut rows: HashMap<String, Row> = HashMap::new();
    for &(ip, instr) in samples {
        let name = named.entry(ip).or_insert_with(|| places.place(ip));
        let row = rows.entry(name.clone()).or_default();
        if instr {
            row.instr += 1;
        } else {
            row.cycles += 1;
        }
    }
    rows
}

/// Places summed by unit, largest instruction count first.
pub fn by_unit(places: &HashMap<String, Row>, table: &[(String, String)]) -> Vec<(String, Row)> {
    let mut units: HashMap<String, Row> = HashMap::new();
    for (place, row) in places {
        let u = units.entry(unit_of(place, table).to_string()).or_default();
        u.instr += row.instr;
        u.cycles += row.cycles;
    }
    let mut out: Vec<(String, Row)> = units.into_iter().collect();
    out.sort_by(|a, b| b.1.instr.cmp(&a.1.instr).then(a.0.cmp(&b.0)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAPS: &str = "1000-2000 r--p 00000000 fd:00 11 /x/bench\n2000-5000 r-xp 00001000 fd:00 11 /x/bench\n7000-9000 r-xp 00000000 00:00 0 \nA000-B000 r-xp 00000000 fd:00 12 /lib/libc.so.6\n";

    fn places() -> Places {
        let syms = parse_nm("0000000000001000 t a::vif::run\n0000000000002000 T a::vu::step\n0000000000002800 d data\n");
        Places::new(parse_maps(MAPS), syms, parse_perf_map("7000 100 sapphire_vu_12\n"), "bench")
    }

    #[test]
    fn an_address_finds_its_symbol_block_or_library() {
        let p = places();
        assert_eq!(p.place(0x2004), "a::vif::run");
        assert_eq!(p.place(0x3010), "a::vu::step");
        assert_eq!(p.place(0x7040), "compiled sapphire_vu_12");
        assert_eq!(p.place(0x8000), "compiled ?");
        assert_eq!(p.place(0xA800), "libc.so.6");
        assert_eq!(p.place(0x6000), "?");
    }

    #[test]
    fn samples_sum_by_the_first_pattern_that_matches() {
        let table = parse_table("# a comment\n::vu::  VU\n::vif  VIF # the interface\ncompiled sapphire_vu  VU compiled\n").unwrap();
        let samples = [(0x2004, true), (0x2008, false), (0x3010, true), (0x3011, true), (0x7040, true), (0xA800, true)];
        let rows = by_unit(&by_place(&samples, &places()), &table);
        assert_eq!(rows[0], ("VU".to_string(), Row { instr: 2, cycles: 0 }));
        let vif = rows.iter().find(|r| r.0 == "VIF").unwrap().1;
        assert_eq!((vif, vif.ipc()), (Row { instr: 1, cycles: 1 }, 1.0));
        assert!(rows.iter().any(|r| r.0 == "VU compiled") && rows.iter().any(|r| r.0 == "other"));
    }

    #[test]
    fn a_table_line_without_a_unit_is_refused() {
        assert!(parse_table("::vif\n").unwrap_err().contains("line 1"));
    }
}

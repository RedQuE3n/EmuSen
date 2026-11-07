//! A golden trace: per frame eight bytes of SHA-256 of the state, the picture and the sound, or a chain written every
//! so many frames or instructions, as text (Moon_Native.md §9.2, VenusRT_Native.md §68.2).

use std::collections::BTreeMap;

use emusen_core_host::sha256::{Sha256, sha256_hex};

pub const VERSION: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Trace {
    pub heading: String,
    pub name: String,
    pub rom: String,
    pub input: String,
    pub rendering: String,
    pub recorded_by: String,
    pub reason: String,
    /// "frame": rows (index, state, picture, sound); "chain": rows (index, chain) over frames; "step": over instructions.
    pub unit: String,
    pub rows: Vec<Vec<String>>,
    pub transcript: BTreeMap<String, String>,
}

/// The first sixteen hex digits of SHA-256, upper case, as the traces hold them.
pub fn hash16(bytes: &[u8]) -> String {
    sha256_hex(bytes)[..16].to_ascii_uppercase()
}

/// chain = the first eight bytes of SHA-256(chain ‖ link), where link is eight bytes of a hash.
pub fn fold(chain: &mut [u8; 8], link: &[u8]) {
    let mut h = Sha256::default();
    h.update(chain);
    h.update(&link[..8]);
    chain.copy_from_slice(&h.finish()[..8]);
}

/// Sixteen hex digits as eight bytes.
pub fn unhex8(text: &str) -> [u8; 8] {
    let mut out = [0u8; 8];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&text[2 * i..2 * i + 2], 16).expect("a hash of hex digits");
    }
    out
}

pub fn hex_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

impl Trace {
    pub fn new(unit: &str) -> Trace {
        Trace { unit: unit.to_string(), ..Trace::default() }
    }

    pub fn text(&self) -> String {
        let mut t = String::new();
        if !self.heading.is_empty() {
            t.push_str(&format!("# {}\n", self.heading));
        }
        t.push_str(&format!(
            "golden {VERSION}\nname {}\nrom {}\ninput {}\nrendering {}\nunit {}\nrecorded {}\nreason {}\n",
            self.name, self.rom, self.input, self.rendering, self.unit, self.recorded_by, self.reason
        ));
        let mut last: Option<&Vec<String>> = None;
        for row in &self.rows {
            t.push_str(&row[0]);
            for i in 1..row.len() {
                t.push(' ');
                let same = last.is_some_and(|l| l.len() == row.len() && l[i] == row[i]);
                t.push_str(if same { "=" } else { &row[i] });
            }
            t.push('\n');
            last = Some(row);
        }
        for (k, v) in &self.transcript {
            t.push_str(&format!("{k} {v}\n"));
        }
        t
    }

    pub fn parse(text: &str) -> Trace {
        let mut trace = Trace::default();
        let mut last: Option<Vec<String>> = None;
        for raw in text.split('\n') {
            let line = raw.trim_end_matches('\r');
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split(' ').collect();
            if line.as_bytes()[0].is_ascii_digit() {
                let row: Vec<String> = parts
                    .iter()
                    .enumerate()
                    .map(|(i, v)| if *v == "=" { last.as_ref().map_or(v.to_string(), |l| l[i].clone()) } else { v.to_string() })
                    .collect();
                trace.rows.push(row.clone());
                last = Some(row);
                continue;
            }
            let value = if line.len() > parts[0].len() { line[parts[0].len() + 1..].to_string() } else { String::new() };
            match parts[0] {
                "golden" => assert_eq!(value.parse::<u32>().ok(), Some(VERSION), "a trace of another version"),
                "name" => trace.name = value,
                "rom" => trace.rom = value,
                "input" => trace.input = value,
                "rendering" => trace.rendering = value,
                "unit" => trace.unit = value,
                "recorded" => trace.recorded_by = value,
                "reason" => trace.reason = value,
                key => {
                    trace.transcript.insert(key.to_string(), value);
                }
            }
        }
        trace
    }

    /// The first difference from another trace of the same run, or none: the header, then row by row, then the transcript.
    pub fn first_difference(&self, now: &Trace) -> Option<String> {
        for (what, a, b) in [("rom", &self.rom, &now.rom), ("input", &self.input, &now.input), ("rendering", &self.rendering, &now.rendering), ("unit", &self.unit, &now.unit)] {
            if a != b {
                return Some(format!("{what}: recorded {a}, now {b}"));
            }
        }
        let parts: &[&str] = if self.unit == "frame" { &["", "state", "picture", "sound"] } else { &["", "chain"] };
        let unit = if self.unit == "step" { "step" } else { "frame" };
        for (r, n) in self.rows.iter().zip(&now.rows) {
            if r == n {
                continue;
            }
            let at = (0..r.len()).find(|&k| k >= n.len() || r[k] != n[k]).unwrap_or(r.len().saturating_sub(1));
            let got = n.get(at).map_or("nothing", String::as_str);
            return Some(format!("{unit} {}: the {} differs (recorded {}, now {got})", r[0], parts[at.min(parts.len() - 1)], r[at]));
        }
        if self.rows.len() != now.rows.len() {
            return Some(format!("recorded {} rows, now {}", self.rows.len(), now.rows.len()));
        }
        let keys: std::collections::BTreeSet<&String> = self.transcript.keys().chain(now.transcript.keys()).collect();
        for k in keys {
            let (a, b) = (self.transcript.get(k), now.transcript.get(k));
            if a != b {
                return Some(format!("{k}: recorded {}, now {}", a.map_or("nothing", String::as_str), b.map_or("nothing", String::as_str)));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trace_reads_back_as_written_and_a_changed_hash_is_named() {
        let mut t = Trace::new("frame");
        t.name = "x".into();
        t.rows.push(vec!["1".into(), "AA".into(), "BB".into(), "CC".into()]);
        t.rows.push(vec!["2".into(), "AA".into(), "BD".into(), "CC".into()]);
        t.transcript.insert("verdict".into(), "Passed 2".into());
        let back = Trace::parse(&t.text());
        assert_eq!(back, Trace { heading: String::new(), ..t.clone() });
        let mut changed = back.clone();
        changed.rows[1][2] = "BE".into();
        assert_eq!(back.first_difference(&changed).as_deref(), Some("frame 2: the picture differs (recorded BD, now BE)"));
    }
}

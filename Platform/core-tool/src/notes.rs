//! `cores-notes.tsv`, shipped with the build: the libretro builds the project tested, each by its SHA-256, so Cores can
//! say "this exact build was tested"; information, never a gate (EmuSen_Cores.md §4.6, §11.4).

/// The file as this build ships it.
pub const SHIPPED: &str = include_str!("../data/cores-notes.tsv");

/// One tested build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tested {
    pub stem: String,
    pub date: String,
    pub sha256: String,
    pub kit: String,
    pub note: String,
}

/// How a core's build stands against the tested ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Match {
    /// This exact build, by its SHA-256.
    Exact,
    /// Another build of the same core.
    Other,
    /// No build of it.
    None,
}

impl Match {
    pub fn word(self) -> &'static str {
        match self {
            Match::Exact => "exact",
            Match::Other => "other",
            Match::None => "none",
        }
    }
}

/// The lines of the file, `#` lines, blank lines and lines of another shape skipped.
pub fn parse(text: &str) -> Vec<Tested> {
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let [stem, date, sha256, kit, note] = f.as_slice() else { return None };
            let sha256 = sha256.trim().to_ascii_lowercase();
            (sha256.len() == 64 && sha256.bytes().all(|b| b.is_ascii_hexdigit())).then(|| Tested { stem: stem.trim().into(), date: date.trim().into(), sha256, kit: kit.trim().into(), note: note.trim().into() })
        })
        .collect()
}

/// A core's standing, and the tested build it is told by: the exact one, else the latest tested.
pub fn standing<'a>(notes: &'a [Tested], stem: &str, sha256: Option<&str>) -> (Match, Option<&'a Tested>) {
    let mine: Vec<&Tested> = notes.iter().filter(|t| t.stem == stem).collect();
    if let Some(exact) = sha256.and_then(|h| mine.iter().find(|t| t.sha256 == h)) {
        return (Match::Exact, Some(exact));
    }
    match mine.into_iter().max_by(|a, b| a.date.cmp(&b.date)) {
        Some(t) => (Match::Other, Some(t)),
        None => (Match::None, None),
    }
}

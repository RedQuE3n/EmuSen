//! Which (element, property) pairs the scene and its painter draw, from the data file mapping.tsv, so a count of what
//! is supported is a count of one table (EmuSen_BigPicture.md §13.3; EmuSen_Qt.md §4.5.5).

use std::collections::HashMap;
use std::sync::OnceLock;

/// What the scene does with a pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Drawn,
    /// Set by a theme with no visible effect in EmuSen; counted apart from drawn until the tester rules on it.
    Accepted,
    /// Not drawn yet; the family that will draw it.
    Later(String),
}

fn table() -> &'static Vec<(String, String, Status)> {
    static TABLE: OnceLock<Vec<(String, String, Status)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        include_str!("mapping.tsv")
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|line| {
                let mut f = line.split('\t');
                let (kind, property, status) = (f.next(), f.next(), f.next());
                let (Some(kind), Some(property), Some(status), None) = (kind, property, status, f.next()) else {
                    panic!("mapping.tsv: element, property, status: {line}")
                };
                let status = match status {
                    "drawn" => Status::Drawn,
                    "accepted" => Status::Accepted,
                    later => Status::Later(later.strip_prefix("later:").unwrap_or_else(|| panic!("mapping.tsv: {later}")).to_owned()),
                };
                (kind.to_owned(), property.to_owned(), status)
            })
            .collect()
    })
}

fn index() -> &'static HashMap<(&'static str, &'static str), &'static Status> {
    static INDEX: OnceLock<HashMap<(&'static str, &'static str), &'static Status>> = OnceLock::new();
    INDEX.get_or_init(|| table().iter().map(|(k, p, s)| ((k.as_str(), p.as_str()), s)).collect())
}

/// Every pair in the file's order.
pub fn pairs() -> impl Iterator<Item = (&'static str, &'static str, &'static Status)> {
    table().iter().map(|(k, p, s)| (k.as_str(), p.as_str(), s))
}

pub fn status(kind: &str, property: &str) -> Option<&'static Status> {
    index().get(&(kind, property)).copied()
}

/// Whether the element kind is drawn at all: any of its pairs is; sound is played, not drawn, and gameselector draws nothing itself.
pub fn is_drawn(kind: &str) -> bool {
    table().iter().any(|(k, _, s)| k == kind && *s == Status::Drawn)
}

/// Whether a property set on an element is in the table as drawn or accepted (the check counts the two apart).
pub fn is_mapped(kind: &str, property: &str) -> bool {
    matches!(status(kind, property), Some(Status::Drawn | Status::Accepted))
}

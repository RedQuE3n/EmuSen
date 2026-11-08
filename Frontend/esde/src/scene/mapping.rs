//! Which (element, property) pairs the scene and its painter draw, from the data file mapping.tsv, so a count of what
//! is supported is a count of one table (EmuSen_BigPicture.md §13.3; EmuSen_Qt.md §4.4).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

struct Mapping {
    common: HashSet<String>,
    specific: HashMap<String, HashSet<String>>,
}

fn mapping() -> &'static Mapping {
    static MAPPING: OnceLock<Mapping> = OnceLock::new();
    MAPPING.get_or_init(|| {
        let mut common = HashSet::new();
        let mut specific = HashMap::new();
        for line in include_str!("mapping.tsv").lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let (kind, names) = line.split_once('\t').expect("mapping.tsv: element, tab, properties");
            let names: HashSet<String> = names.split(' ').map(str::to_owned).collect();
            if kind == "common" {
                common = names;
            } else {
                specific.insert(kind.to_owned(), names);
            }
        }
        Mapping { common, specific }
    })
}

/// Whether the element kind is drawn at all; sound is not, and gameselector draws nothing itself.
pub fn is_drawn(kind: &str) -> bool {
    mapping().specific.contains_key(kind)
}

/// Whether a property set on an element is drawn.
pub fn is_mapped(kind: &str, property: &str) -> bool {
    mapping().specific.get(kind).is_some_and(|own| own.contains(property) || mapping().common.contains(property))
}

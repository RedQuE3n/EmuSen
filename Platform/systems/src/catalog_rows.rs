//! The catalog's hand rows (`data/catalog-rows.tsv`): the console rows the runtime lists ahead of those discovery adds,
//! each with its systems' shelves, the words a player types for it, its console's sole engine and the console as a
//! refusal names it (EmuSen_Backend.md §29.2).

use std::sync::OnceLock;

/// One hand row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandRow {
    pub name: String,
    /// Each system's id and its shelf, the first the row's console.
    pub systems: Vec<(String, String)>,
    pub keys: Vec<String>,
    pub sole_engine: String,
    /// The console as the refusal names it when the sole engine is missing.
    pub refusal: String,
}

/// The rows as this build ships them.
pub const SHIPPED: &str = include_str!("../data/catalog-rows.tsv");

/// The lines of a list, `#` and blank lines skipped; a line of another shape is an error naming it.
pub fn parse(text: &str) -> Result<Vec<HandRow>, String> {
    let mut rows = Vec::new();
    for (n, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty() && !l.starts_with('#')) {
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        let [name, systems, keys, sole_engine, refusal] = fields.as_slice() else {
            return Err(format!("catalog-rows.tsv line {} has not five fields", n + 1));
        };
        let mut pairs = Vec::new();
        for pair in systems.split(',') {
            let Some((id, shelf)) = pair.split_once('=').filter(|(i, s)| !i.trim().is_empty() && !s.trim().is_empty()) else {
                return Err(format!("catalog-rows.tsv line {}: {pair} is not id=shelf", n + 1));
            };
            pairs.push((id.trim().to_string(), shelf.trim().to_string()));
        }
        let keys = keys.split(',').map(str::trim).filter(|k| !k.is_empty()).map(str::to_string).collect();
        rows.push(HandRow { name: name.to_string(), systems: pairs, keys, sole_engine: sole_engine.to_string(), refusal: refusal.to_string() });
    }
    Ok(rows)
}

/// The shipped rows, read once; empty only if the file was committed broken, which the tests refuse.
pub fn hand_rows() -> &'static [HandRow] {
    static ROWS: OnceLock<Vec<HandRow>> = OnceLock::new();
    ROWS.get_or_init(|| parse(SHIPPED).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_rows_name_systems_of_the_table() {
        let rows = parse(SHIPPED).unwrap();
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["SNES (Venus)", "NES (Moon)", "Game Boy (Mercury)", "Nintendo 64 (Mars)"]);
        for row in &rows {
            for (id, _) in &row.systems {
                assert!(crate::find(id).is_some(), "{} names {id}, which the table lacks", row.name);
            }
        }
        assert_eq!(rows[2].systems[1], ("gbc".to_string(), "Game Boy Color (Mercury)".to_string()));
        assert_eq!(rows[2].keys, ["mercury", "gb", "gbc"]);
    }

    #[test]
    fn a_bad_line_is_refused() {
        assert!(parse("A\tsnes\tk\tE\tR\n").unwrap_err().contains("is not id=shelf"));
        assert!(parse("A\tsnes=A\tk\tE\n").unwrap_err().contains("not five fields"));
    }
}

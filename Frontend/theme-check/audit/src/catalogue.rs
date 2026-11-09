//! A catalogue a theme's console screen is recorded over (man theme-check, --catalogue): plain text, one console a line, in
//! place of the library's own consoles, so a design's capacity states (one console, thirty, 120, one maker of forty) are
//! fed through the real clients' shapes as the program feeds them.

use emusen_crystal_client::shapes::SystemRow;
use emusen_platform_client::library::GameFile;
use emusen_qt_models::esde_shelves::{MediaIndex, Plays};
use emusen_qt_models::gamelists::Collection;
use emusen_qt_models::library::{Engine, LibrarySource};
use serde_json::Value;
use std::path::Path;

/// One console of a catalogue: its maker, its name, how many games it holds, the engine it runs on (none when empty), whether
/// that is a libretro core, and the table's console whose look it takes (its body), else the first shelved console's.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub maker: String,
    pub name: String,
    pub games: usize,
    pub engine: String,
    pub libretro: bool,
    pub like: String,
}

/// The entries of a catalogue's text: tab-separated `maker name games [engine [emusen|libretro [like]]]`, `#` a comment,
/// blank lines skipped; an empty maker is written as `-`. The error names the line.
pub fn parse(text: &str) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let cells: Vec<&str> = line.split('\t').collect();
        let at = |i: usize| cells.get(i).map_or("", |c| c.trim());
        let games = at(2).parse::<usize>().map_err(|_| format!("line {}: the games count {:?} is not a number", n + 1, at(2)))?;
        if at(1).is_empty() {
            return Err(format!("line {}: a console needs a name", n + 1));
        }
        let libretro = match at(4) {
            "" | "emusen" => false,
            "libretro" => true,
            other => return Err(format!("line {}: the engine's kind {other:?} is neither emusen nor libretro", n + 1)),
        };
        let maker = if at(0) == "-" { String::new() } else { at(0).to_owned() };
        entries.push(Entry { maker, name: at(1).to_owned(), games, engine: at(3).to_owned(), libretro, like: at(5).to_owned() });
    }
    Ok(entries)
}

/// The clients' answers with the catalogue's consoles and games in place of the library's own.
pub struct Catalogue<'a> {
    inner: &'a dyn LibrarySource,
    systems: Vec<SystemRow>,
    games: Vec<GameFile>,
    engines: Vec<(String, Engine)>,
}

impl<'a> Catalogue<'a> {
    /// Each entry copies its `like` row of the table under its own name and id, else the first shelved console's under a made-up
    /// year and hue as well.
    pub fn new(inner: &'a dyn LibrarySource, root: &Path, entries: &[Entry]) -> Result<Catalogue<'a>, String> {
        let table = inner.systems()?;
        let shelved = inner.games()?;
        let first = table
            .iter()
            .find(|s| shelved.iter().any(|g| g.system.as_ref() == Some(&s.id)))
            .or(table.first())
            .ok_or("the table has no console")?;
        let (mut systems, mut games, mut engines) = (Vec::new(), Vec::new(), Vec::new());
        for (n, entry) in entries.iter().enumerate() {
            let like = table.iter().find(|s| !entry.like.is_empty() && s.id == entry.like).unwrap_or(first);
            let id = format!("catalogue{n:03}");
            let mut row = like.clone();
            if entry.like.is_empty() {
                row.look.hue = (n * 37 % 360) as f64;
                row.release_year = 1977 + (n % 30) as i32;
            }
            (row.id, row.name, row.display_name, row.console, row.esde_system, row.esde_full_name, row.manufacturer) = (
                id.clone(),
                entry.name.clone(),
                entry.name.clone(),
                entry.name.clone(),
                id.clone(),
                entry.name.clone(),
                entry.maker.clone(),
            );
            row.extensions = vec![format!(".c{n:03}")];
            for g in 0..entry.games {
                let path = root.join(format!("Roms/{} {g:05}.c{n:03}", entry.name));
                games.push(GameFile { path: path.to_string_lossy().into_owned(), system: Some(id.clone()), ..GameFile::default() });
            }
            if !entry.engine.is_empty() {
                engines.push((id, Engine { id: entry.engine.clone(), name: entry.engine.clone(), libretro: entry.libretro }));
            }
            systems.push(row);
        }
        Ok(Catalogue { inner, systems, games, engines })
    }
}

impl LibrarySource for Catalogue<'_> {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        Ok(self.systems.clone())
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        Ok(self.games.clone())
    }
    fn favourites(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        self.inner.favourites(games)
    }
    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        self.inner.recent(games)
    }
    fn engines(&self, _systems: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        Ok(self.engines.clone())
    }
    fn media(&self, systems: &[SystemRow], games: &[GameFile], types: &[&str]) -> Result<Vec<(String, String)>, String> {
        self.inner.media(systems, games, types)
    }
    fn played(&self, games: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        self.inner.played(games)
    }
    fn shelves(&self) -> Result<Vec<String>, String> {
        Ok(self.systems.iter().map(|s| s.id.clone()).collect())
    }
    fn settings(&self) -> Result<Value, String> {
        self.inner.settings()
    }
    fn plays(&self, games: &[GameFile]) -> Result<Vec<(String, Plays)>, String> {
        self.inner.plays(games)
    }
    fn media_index(&self, systems: &[String]) -> Result<MediaIndex, String> {
        self.inner.media_index(systems)
    }
    fn collections(&self) -> Result<Vec<Collection>, String> {
        self.inner.collections()
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, Entry};

    #[test]
    fn a_catalogue_reads_its_lines_and_names_a_bad_one() {
        let text = "# a comment\nSega\tGenesis\t947\tNephrite\temusen\tmd\n-\tLoose\t3\n\nSony\tPS2\t56\tPCSX2\tlibretro\n";
        let entries = parse(text).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], Entry { maker: "Sega".into(), name: "Genesis".into(), games: 947, engine: "Nephrite".into(), libretro: false, like: "md".into() });
        assert_eq!((entries[1].maker.as_str(), entries[1].engine.as_str()), ("", ""), "a maker of - is none, and no engine is none");
        assert!(entries[2].libretro);
        assert_eq!(parse("Sega\tGenesis\tmany\n").unwrap_err(), "line 1: the games count \"many\" is not a number");
        assert_eq!(parse("Sega\t\t3\n").unwrap_err(), "line 1: a console needs a name");
        assert!(parse("Sega\tGenesis\t3\tX\tmame\n").unwrap_err().contains("neither emusen nor libretro"));
        assert!(parse("# nothing\n").unwrap().is_empty());
    }
}

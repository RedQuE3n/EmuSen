//! Which name runs which command, the aliases, which commands are read-only, and the order `help` lists them in:
//! one table per session, shared with the sessions `tmux new` makes from it. See EmuSen_DianaSh.md §5.2 and §13.5.

use crate::builtins::Builtin;
use crate::dotnet;
use std::sync::Arc;

/// Who answers a name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// One of the shell's own.
    Builtin(Builtin),
    /// A host's command, served by the registration with this index.
    Host(usize),
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub usage: String,
    pub read_only: bool,
    pub kind: Kind,
}

#[derive(Clone, Debug, Default)]
pub struct Table {
    /// In the order `help` lists them.
    entries: Vec<Entry>,
    /// Name to command, as typed in lower case.
    aliases: Vec<(String, String)>,
    /// `ps` and `kill`'s id prefixes, each with the host registration that lists and removes them.
    claims: Vec<(String, usize)>,
    /// Lines a host adds to `help`'s notes.
    notes: Vec<String>,
}

impl Table {
    /// The shell's own commands, in the order the C# shell has always listed them.
    pub fn builtins() -> Arc<Table> {
        let entries = Builtin::ALL.iter().map(|b| Entry { name: b.name().to_string(), usage: b.usage().to_string(), read_only: b.read_only(), kind: Kind::Builtin(*b) }).collect();
        let aliases = vec![("quit".to_string(), "shutdown".to_string()), ("jobs".to_string(), "ps".to_string())];
        Arc::new(Table { entries, aliases, claims: Vec::new(), notes: Vec::new() })
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn find(&self, name: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| dotnet::same(&e.name, name))
    }

    /// A name lower-cased, then mapped through the aliases.
    pub fn canonical(&self, name: &str) -> String {
        let lower = dotnet::to_lower(name);
        match self.aliases.iter().find(|(alias, _)| *alias == lower) {
            Some((_, command)) => command.clone(),
            None => lower,
        }
    }

    pub fn is_read_only(&self, name: &str) -> bool {
        self.find(&self.canonical(name)).is_some_and(|e| e.read_only)
    }

    pub fn claims(&self) -> &[(String, usize)] {
        &self.claims
    }

    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    pub fn note(&mut self, line: &str) {
        self.notes.push(line.to_string());
    }

    /// A host's command takes the place of any command of its name, a builtin included, as a frontend's always has.
    pub fn add(&mut self, entry: Entry) {
        self.entries.retain(|e| !dotnet::same(&e.name, &entry.name));
        self.entries.push(entry);
    }

    pub fn alias(&mut self, alias: &str, command: &str) {
        let alias = dotnet::to_lower(alias);
        self.aliases.retain(|(a, _)| *a != alias);
        self.aliases.push((alias, dotnet::to_lower(command)));
    }

    pub fn claim(&mut self, prefix: &str, registration: usize) {
        self.claims.push((prefix.to_string(), registration));
    }

    /// The listing's order as a host gives it; a name it does not give keeps its place after the ones it does.
    pub fn order(&mut self, names: &[String]) {
        let rank = |e: &Entry| names.iter().position(|n| dotnet::same(n, &e.name)).unwrap_or(usize::MAX);
        self.entries.sort_by_key(rank);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_map_after_lower_casing_and_order_is_the_hosts() {
        let mut table = (*Table::builtins()).clone();
        assert_eq!(table.canonical("QUIT"), "shutdown");
        assert_eq!(table.canonical("Echo"), "echo");
        assert!(table.is_read_only("JOBS"));
        assert!(!table.is_read_only("cd"));
        table.order(&["pwd".to_string(), "echo".to_string()]);
        assert_eq!(table.entries()[0].name, "pwd");
        assert_eq!(table.entries()[1].name, "echo");
        assert_eq!(table.entries()[2].name, "cd");
    }
}

//! The shell's accounts: identity only, with a hash nothing gates on. `root` always exists and cannot be removed.
//! One registry per shell, shared by its sessions. See `man su`, `man useradd` and EmuSen_DianaSh.md §4.1.

use crate::dotnet;
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Accounts {
    /// In the order added, compared ignoring case, each with its password's hash.
    entries: Vec<(String, Option<String>)>,
}

impl Default for Accounts {
    fn default() -> Self {
        Accounts { entries: vec![("root".to_string(), None)] }
    }
}

impl Accounts {
    fn find(&self, name: &str) -> Option<usize> {
        self.entries.iter().position(|(n, _)| dotnet::same(n, name))
    }

    pub fn exists(&self, name: &str) -> bool {
        self.find(name).is_some()
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(n, _)| n.as_str())
    }

    pub fn add(&mut self, name: &str) -> bool {
        if self.exists(name) {
            return false;
        }
        self.entries.push((name.to_string(), None));
        true
    }

    pub fn remove(&mut self, name: &str) -> bool {
        if dotnet::same(name, "root") {
            return false;
        }
        match self.find(name) {
            Some(at) => {
                self.entries.remove(at);
                true
            }
            None => false,
        }
    }

    /// Sets the hash, adding the account if it was not there, as the C# indexer does.
    pub fn set_password(&mut self, name: &str, password: &str) {
        let hash: String = Sha256::digest(password.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
        match self.find(name) {
            Some(at) => self.entries[at].1 = Some(hash),
            None => self.entries.push((name.to_string(), Some(hash))),
        }
    }

    pub fn password_hash(&self, name: &str) -> Option<&str> {
        self.find(name).and_then(|at| self.entries[at].1.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_stays_and_names_ignore_case() {
        let mut accounts = Accounts::default();
        assert!(!accounts.remove("ROOT"));
        assert!(accounts.add("Tester"));
        assert!(!accounts.add("tester"));
        assert!(accounts.exists("TESTER"));
        accounts.set_password("tester", "pw");
        assert_eq!(accounts.password_hash("Tester"), Some("30c952fab122c3f9759f02a6d95c3758b246b4fee239957b2d4fee46e26170c4"));
        assert!(accounts.remove("TESTER"));
        assert_eq!(accounts.names().collect::<Vec<_>>(), ["root"]);
    }
}

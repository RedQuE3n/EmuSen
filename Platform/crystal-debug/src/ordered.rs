//! A map that enumerates its entries in the order they were first added, as a .NET `Dictionary` does while nothing is
//! removed from it; the registries that use one only ever add to it or clear it.

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct Ordered<K, V> {
    entries: Vec<(K, V)>,
    index: HashMap<K, usize>,
}

impl<K, V> Default for Ordered<K, V> {
    fn default() -> Self {
        Ordered { entries: Vec::new(), index: HashMap::new() }
    }
}

impl<K: Eq + Hash + Clone, V> Ordered<K, V> {
    pub fn new() -> Self {
        Ordered { entries: Vec::new(), index: HashMap::new() }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.index.get(key).map(|&i| &self.entries[i].1)
    }

    /// The entry's value, added with `make` where absent.
    pub fn entry(&mut self, key: K, make: impl FnOnce() -> V) -> &mut V {
        let i = match self.index.get(&key) {
            Some(&i) => i,
            None => {
                self.entries.push((key.clone(), make()));
                self.index.insert(key, self.entries.len() - 1);
                self.entries.len() - 1
            }
        };
        &mut self.entries[i].1
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().map(|(k, v)| (k, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(k, _)| k)
    }
}

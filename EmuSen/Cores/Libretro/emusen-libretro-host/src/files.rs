//! The core's own folders, persistent beside its entry, and its saves: every file in its save directory is a named
//! battery file the host owns, passed in before load and reported when it changes; what the core keeps in its system
//! and cache folders is its working data and no one's save (EmuSen_LibretroHost.md §4.5, §4.8).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use emusen_native::core::sys::flags;
use emusen_native::core::{BatteryKind, NamedFile, request};

/// `<cores>/libretro/.data/<stem>/`, with `system/`, `save/` and `cache/`.
pub struct Folders {
    pub root: PathBuf,
    pub system: PathBuf,
    pub save: PathBuf,
    pub cache: PathBuf,
}

impl Folders {
    /// The folders of the entry at `folder`, made where missing.
    pub fn for_entry(folder: &Path, stem: &str) -> std::io::Result<Folders> {
        let libretro = folder.parent().unwrap_or(folder);
        let root = libretro.join(".data").join(stem);
        let f = Folders { system: root.join("system"), save: root.join("save"), cache: root.join("cache"), root };
        for d in [&f.system, &f.save, &f.cache] {
            std::fs::create_dir_all(d)?;
        }
        Ok(f)
    }
}

/// One save file as last reported.
struct Seen {
    which: u32,
    len: u64,
    modified: Option<SystemTime>,
    bytes: Vec<u8>,
    /// The bytes the host last marked saved.
    saved: Vec<u8>,
    removed: bool,
}

/// The core's save files, by name relative to the save folder.
pub struct Saves {
    folder: PathBuf,
    files: BTreeMap<String, Seen>,
    next: u32,
}

/// What a scan found, for the host's events.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Changes {
    pub changed: Vec<u32>,
    pub listed: bool,
}

fn walk(base: &Path, at: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(read) = std::fs::read_dir(at) else { return };
    for item in read.flatten() {
        let path = item.path();
        let Ok(kind) = item.file_type() else { continue };
        if kind.is_dir() {
            walk(base, &path, out);
        } else if kind.is_file()
            && let Ok(rel) = path.strip_prefix(base)
            && let Some(name) = rel.to_str()
        {
            let name = name.replace('\\', "/");
            if request::valid_name(&name) {
                out.push((name, path));
            }
        }
    }
}

impl Saves {
    pub fn new(folder: PathBuf) -> Saves {
        Saves { folder, files: BTreeMap::new(), next: flags::FILE_NAMED }
    }

    /// The files the host passed, written into place by name before the core loads; a file already there and not
    /// passed is kept and reported at the first scan.
    pub fn place(&mut self, passed: &[(u32, String, &[u8])]) -> std::io::Result<()> {
        for (which, name, data) in passed {
            if !request::valid_name(name) {
                continue;
            }
            let path = self.folder.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, data)?;
            let meta = std::fs::metadata(&path)?;
            self.files.insert(name.clone(), Seen { which: *which, len: meta.len(), modified: meta.modified().ok(), bytes: data.to_vec(), saved: data.to_vec(), removed: false });
            self.next = self.next.max(which + 1);
        }
        Ok(())
    }

    /// The folder compared with what was last reported.
    pub fn scan(&mut self) -> Changes {
        let mut found = Vec::new();
        walk(&self.folder, &self.folder, &mut found);
        let mut c = Changes::default();
        for (name, path) in &found {
            let Ok(meta) = std::fs::metadata(path) else { continue };
            let (len, modified) = (meta.len(), meta.modified().ok());
            match self.files.get_mut(name) {
                Some(s) if !s.removed && s.len == len && s.modified == modified => {}
                Some(s) => {
                    if let Ok(bytes) = std::fs::read(path) {
                        let was_removed = s.removed;
                        (s.len, s.modified, s.removed) = (len, modified, false);
                        if bytes != s.bytes || was_removed {
                            s.bytes = bytes;
                            c.changed.push(s.which);
                        }
                        c.listed |= was_removed;
                    }
                }
                None => {
                    if self.next >= flags::FILE_MEDIA {
                        continue;
                    }
                    if let Ok(bytes) = std::fs::read(path) {
                        let which = self.next;
                        self.next += 1;
                        self.files.insert(name.clone(), Seen { which, len, modified, bytes, saved: Vec::new(), removed: false });
                        c.changed.push(which);
                        c.listed = true;
                    }
                }
            }
        }
        for (name, s) in self.files.iter_mut() {
            if !s.removed && !found.iter().any(|(n, _)| n == name) {
                s.removed = true;
                c.listed = true;
            }
        }
        c
    }

    /// Machine info's entries for the named files.
    pub fn listed(&self) -> Vec<NamedFile> {
        self.files.iter().map(|(name, s)| NamedFile { which: s.which, name: name.clone(), kind: BatteryKind::Console, removed: s.removed }).collect()
    }

    /// A named file's bytes and whether they changed since the host last saved them.
    pub fn bytes(&self, which: u32) -> Option<(&[u8], bool)> {
        self.files.values().find(|s| s.which == which).map(|s| (if s.removed { &[][..] } else { &s.bytes[..] }, s.bytes != s.saved || s.removed))
    }

    pub fn saved(&mut self, which: u32) -> bool {
        match self.files.values_mut().find(|s| s.which == which) {
            Some(s) => {
                s.saved = s.bytes.clone();
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_changed_and_removed_file_are_each_reported() {
        let dir = std::env::temp_dir().join(format!("emusen-libretro-saves-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut s = Saves::new(dir.clone());
        s.place(&[(flags::FILE_NAMED, "card.bin".into(), b"one")]).unwrap();
        assert_eq!(s.scan(), Changes::default());
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub/new.txt"), b"x").unwrap();
        let c = s.scan();
        assert_eq!((c.changed, c.listed), (vec![flags::FILE_NAMED + 1], true));
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(dir.join("card.bin"), b"two!").unwrap();
        assert_eq!(s.scan().changed, vec![flags::FILE_NAMED]);
        assert_eq!(s.bytes(flags::FILE_NAMED), Some((&b"two!"[..], true)));
        assert!(s.saved(flags::FILE_NAMED));
        assert_eq!(s.bytes(flags::FILE_NAMED), Some((&b"two!"[..], false)));
        std::fs::remove_file(dir.join("sub/new.txt")).unwrap();
        assert!(s.scan().listed);
        assert!(s.listed().iter().any(|n| n.name == "sub/new.txt" && n.removed));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

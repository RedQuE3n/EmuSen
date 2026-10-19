//! Which libraries are loaded at all: `EmuSen_CoreAPI.md` §7.2's rule 3, checked before loading without running
//! anything, and rules 1 and 2, a hash in the installation's `cores.manifest` or among the player's approvals, whose
//! file, `cores.approved`, has the manifest's format (EmuSen_Backend.md §27.5, §29.40.3).

use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The manifest's file name in an installation directory.
pub const MANIFEST: &str = "cores.manifest";

/// The approvals' file's name in the config directory: the hashes the player approved, one `sha256sum` line each.
pub const APPROVALS: &str = "cores.approved";

/// The largest library rule 3 lets through.
pub const MAX_LIBRARY_BYTES: u64 = 512 << 20;

/// One line of a manifest: a library's file name and its hash, lowercase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub sha256: String,
}

/// A manifest's lines in `sha256sum`'s format, `<64 hex digits>  <file name>` (or ` *<file name>`); others are skipped.
pub fn parse_manifest(text: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some((hash, rest)) = line.split_at_checked(64) else { continue };
        let name = rest.strip_prefix("  ").or_else(|| rest.strip_prefix(" *"));
        if let Some(name) = name.filter(|n| !n.is_empty() && !n.contains('/'))
            && hash.bytes().all(|b| b.is_ascii_hexdigit())
        {
            out.push(Entry { name: name.to_owned(), sha256: hash.to_ascii_lowercase() });
        }
    }
    out
}

/// Whether `text` is a SHA-256 written as 64 hex digits, in either case.
pub fn is_hash(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The hashes an approvals' file at `path` lists, none where it is absent; refused, in words, where others may write it.
pub fn read_approvals(path: &Path) -> Result<Vec<String>, String> {
    let Ok(meta) = std::fs::metadata(path) else { return Ok(Vec::new()) };
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.mode() & 0o002 != 0 {
            return Err(format!("{} is writable by others; its approvals were not read", path.display()));
        }
    }
    #[cfg(not(unix))]
    let _ = meta;
    let text = std::fs::read(path).map_err(|e| format!("{}: {e}; its approvals were not read", path.display()))?;
    Ok(parse_manifest(&String::from_utf8_lossy(&text)).into_iter().map(|e| e.sha256).collect())
}

/// The approvals' text with `sha256` added under the library's file name, every other line kept; none where it is there.
pub fn with_approval(text: &str, sha256: &str, name: &str) -> Option<String> {
    let hash = sha256.to_ascii_lowercase();
    if parse_manifest(text).iter().any(|e| e.sha256 == hash) {
        return None;
    }
    let name = if name.is_empty() || name.contains('/') || name.chars().any(char::is_control) { "-" } else { name };
    let mut out = text.to_owned();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!("{hash}  {name}\n"));
    Some(out)
}

/// The approvals' text without the lines of `sha256`, every other line kept; none where it has none.
pub fn without_approval(text: &str, sha256: &str) -> Option<String> {
    let hash = sha256.to_ascii_lowercase();
    let mut removed = false;
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        if parse_manifest(line).first().is_some_and(|e| e.sha256 == hash) {
            removed = true;
        } else {
            out.push_str(line);
        }
    }
    removed.then_some(out)
}

/// How far a library is trusted, as the listing says it (EmuSen_Backend.md §29.40.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    Manifest,
    Approved,
    NeedsApproval,
    NotRead,
}

impl Standing {
    pub fn word(self) -> &'static str {
        match self {
            Standing::Manifest => "manifest",
            Standing::Approved => "approved",
            Standing::NeedsApproval => "needs_approval",
            Standing::NotRead => "not_read",
        }
    }
}

/// The installation's manifests and the player's approvals, over the cores directories.
#[derive(Clone, Debug, Default)]
pub struct Trust {
    /// Each installation directory resolved, beside the entries of the manifest it holds.
    directories: Vec<(PathBuf, Vec<Entry>)>,
    /// The player's directories resolved, whose manifests are not read.
    player: Vec<PathBuf>,
    approvals: HashSet<String>,
}

#[cfg(unix)]
unsafe extern "C" {
    fn geteuid() -> u32;
}

impl Trust {
    /// The trust of the installation directories `installation`, each one's `cores.manifest` read where it has one, and
    /// the hashes the player approved.
    pub fn new(installation: &[PathBuf], approvals: &[String]) -> Trust {
        let directories = installation
            .iter()
            .filter_map(|d| {
                let real = std::fs::canonicalize(d).ok()?;
                let entries = std::fs::read_to_string(real.join(MANIFEST)).map(|t| parse_manifest(&t)).unwrap_or_default();
                Some((real, entries))
            })
            .collect();
        Trust { directories, player: Vec::new(), approvals: approvals.iter().map(|a| a.to_ascii_lowercase()).collect() }
    }

    /// The same trust with the player's directories, which rule 3 counts as cores directories and whose manifests it
    /// does not read.
    pub fn with_player(mut self, player: &[PathBuf]) -> Trust {
        self.player = player.iter().filter_map(|d| std::fs::canonicalize(d).ok()).collect();
        self
    }

    /// Rule 3 but the hash, in words that follow the library's name; nothing of the file is run.
    pub fn check_file(&self, library: &Path) -> Result<(), String> {
        let real = std::fs::canonicalize(library).map_err(|_| "cannot be resolved".to_owned())?;
        let meta = std::fs::metadata(&real).map_err(|_| "cannot be read".to_owned())?;
        if !meta.is_file() {
            return Err("is not a regular file".to_owned());
        }
        let parent = real.parent();
        let Some(home) = self.directories.iter().map(|(d, _)| d).chain(self.player.iter()).find(|d| parent == Some(d.as_path())) else {
            return Err("lies outside the cores directories once its links are resolved".to_owned());
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            // SAFETY: geteuid has no preconditions.
            let me = unsafe { geteuid() };
            let installation = std::fs::metadata(home).map(|m| m.uid()).unwrap_or(u32::MAX);
            if meta.uid() != me && meta.uid() != 0 && meta.uid() != installation {
                return Err("is owned by neither the player nor the installation".to_owned());
            }
            if meta.mode() & 0o002 != 0 {
                return Err("is writable by others".to_owned());
            }
            if let Some(open) = home.ancestors().find(|d| std::fs::metadata(d).is_ok_and(|m| m.mode() & 0o002 != 0 && m.mode() & 0o1000 == 0)) {
                return Err(format!("lies under {}, which others may write", open.display()));
            }
        }
        #[cfg(not(unix))]
        let _ = home;
        if meta.len() > MAX_LIBRARY_BYTES {
            return Err("is larger than 512 MB".to_owned());
        }
        let mut head = [0u8; 4096];
        let n = std::fs::File::open(&real).and_then(|mut f| f.read(&mut head)).map_err(|_| "cannot be read".to_owned())?;
        if !header_names_this_machine(&head[..n]) {
            return Err("is not a library for this machine's architecture".to_owned());
        }
        Ok(())
    }

    /// Rules 1 and 2: the hash in the manifest of the library's own directory under its name, or approved.
    pub fn vouches(&self, library: &Path, sha256: &str) -> bool {
        self.approved(sha256) || self.in_manifest(library, sha256)
    }

    /// Rule 2: the player approved the hash.
    pub fn approved(&self, sha256: &str) -> bool {
        self.approvals.contains(&sha256.to_ascii_lowercase())
    }

    /// Rule 1: the manifest of the library's own installation directory lists it under its name with this hash.
    pub fn in_manifest(&self, library: &Path, sha256: &str) -> bool {
        let hash = sha256.to_ascii_lowercase();
        let (Some(dir), Some(name)) = (library.parent().and_then(|d| std::fs::canonicalize(d).ok()), library.file_name().and_then(|n| n.to_str())) else { return false };
        self.directories.iter().any(|(d, entries)| *d == dir && entries.iter().any(|e| e.name == name && e.sha256 == hash))
    }

    /// Where the listing places a library that has a sidecar: by its manifest, its approval, or neither.
    pub fn standing(&self, library: &Path, sha256: &str) -> Standing {
        if self.in_manifest(library, sha256) {
            Standing::Manifest
        } else if self.approved(sha256) {
            Standing::Approved
        } else {
            Standing::NeedsApproval
        }
    }
}

fn u16_at(b: &[u8], at: usize, little: bool) -> Option<u16> {
    let x: [u8; 2] = b.get(at..at + 2)?.try_into().ok()?;
    Some(if little { u16::from_le_bytes(x) } else { u16::from_be_bytes(x) })
}

fn u32_at(b: &[u8], at: usize, little: bool) -> Option<u32> {
    let x: [u8; 4] = b.get(at..at + 4)?.try_into().ok()?;
    Some(if little { u32::from_le_bytes(x) } else { u32::from_be_bytes(x) })
}

/// Whether an executable header is this system's format and names this process's architecture: ELF's `e_machine`,
/// Mach-O's `cputype` (any slice of a universal file), PE's `Machine`.
pub fn header_names_this_machine(b: &[u8]) -> bool {
    if cfg!(target_os = "macos") {
        macho_matches(b)
    } else if cfg!(windows) {
        pe_matches(b)
    } else {
        elf_matches(b)
    }
}

fn elf_matches(b: &[u8]) -> bool {
    const MACHINE: u16 = if cfg!(target_arch = "x86_64") {
        62
    } else if cfg!(target_arch = "aarch64") {
        183
    } else if cfg!(target_arch = "x86") {
        3
    } else if cfg!(target_arch = "arm") {
        40
    } else if cfg!(target_arch = "riscv64") {
        243
    } else {
        0
    };
    let class = if cfg!(target_pointer_width = "64") { 2 } else { 1 };
    let little = cfg!(target_endian = "little");
    b.starts_with(b"\x7fELF") && b.get(4) == Some(&class) && b.get(5) == Some(&if little { 1 } else { 2 }) && u16_at(b, 18, little) == Some(MACHINE)
}

fn macho_matches(b: &[u8]) -> bool {
    const CPU: u32 = if cfg!(target_arch = "aarch64") { 0x0100_000c } else { 0x0100_0007 };
    match u32_at(b, 0, true) {
        Some(0xfeed_facf) => u32_at(b, 4, true) == Some(CPU),
        Some(0xbeba_feca) => {
            let n = u32_at(b, 4, false).unwrap_or(0).min(64) as usize;
            (0..n).any(|i| u32_at(b, 8 + i * 20, false) == Some(CPU))
        }
        _ => false,
    }
}

fn pe_matches(b: &[u8]) -> bool {
    const MACHINE: u16 = if cfg!(target_arch = "aarch64") { 0xaa64 } else if cfg!(target_arch = "x86") { 0x14c } else { 0x8664 };
    if !b.starts_with(b"MZ") {
        return false;
    }
    let Some(at) = u32_at(b, 0x3c, true).map(|v| v as usize) else { return false };
    b.get(at..at + 4) == Some(b"PE\0\0") && u16_at(b, at + 4, true) == Some(MACHINE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_manifest_reads_sha256sums_two_forms_and_skips_the_rest() {
        let h = "AB".repeat(32);
        let text = format!("{h}  libone.so\n{} *libtwo.so\n# a remark\nshort  libx.so\n{h}  sub/lib.so\n{h} libsingle.so\n", "cd".repeat(32));
        let e = parse_manifest(&text);
        assert_eq!(e, vec![Entry { name: "libone.so".into(), sha256: "ab".repeat(32) }, Entry { name: "libtwo.so".into(), sha256: "cd".repeat(32) }]);
    }

    #[test]
    fn approvals_are_added_once_and_removed_with_every_other_line_kept() {
        let (a, b) = ("ab".repeat(32), "cd".repeat(32));
        let text = format!("# kept\n{a}  libone.so");
        let added = with_approval(&text, &b.to_ascii_uppercase(), "libtwo.so").unwrap();
        assert_eq!(added, format!("# kept\n{a}  libone.so\n{b}  libtwo.so\n"));
        assert_eq!(with_approval(&added, &a, "other.so"), None);
        assert_eq!(with_approval("", &a, "bad\nname"), Some(format!("{a}  -\n")));
        assert_eq!(without_approval(&added, &a.to_ascii_uppercase()), Some(format!("# kept\n{b}  libtwo.so\n")));
        assert_eq!(without_approval(&added, &"ef".repeat(32)), None);
        assert!(is_hash(&a.to_ascii_uppercase()) && !is_hash("ab") && !is_hash(&"zz".repeat(32)));
    }

    #[test]
    fn the_header_must_be_this_machines() {
        let me = std::fs::read(std::env::current_exe().unwrap()).unwrap();
        assert!(header_names_this_machine(&me[..4096.min(me.len())]));
        let mut other = me[..64].to_vec();
        if cfg!(target_os = "linux") {
            other[18] ^= 0x55;
            assert!(!header_names_this_machine(&other));
        }
        assert!(!header_names_this_machine(b"#!/bin/sh\n"));
        assert!(!header_names_this_machine(b""));
    }
}

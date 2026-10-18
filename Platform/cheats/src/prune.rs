//! Pruning the cheat database, the C# `CheatDatabasePruner`: handed the system folders to keep, it plans the removal of
//! the rest, refusing the cases where an empty or unmatched keep-set would wipe everything, and applies a plan only
//! inside the tree it was planned against (EmuSen_Settings_Reference.md §4.16).

use crate::database::{Database, visit_files};
use emusen_dotnet::text::equals_ignore_case;
use emusen_galaxia::dotnet_path::{self as path, Style};

/// One system folder the plan removes, with its files' count and their bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pruned {
    pub system: String,
    pub path: String,
    pub files: usize,
    pub bytes: u64,
}

/// A plan: what goes, what stays, and why it cannot apply where it cannot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub removing: Vec<Pruned>,
    pub keeping: Vec<String>,
    pub reason: Option<String>,
}

/// `CheatDatabasePruner.Plan`.
pub fn plan(db: &Database, keep: &[String]) -> Plan {
    let kept = |system: &str| keep.iter().any(|k| equals_ignore_case(k, system));
    let present = db.systems();
    let keeping: Vec<String> = present.iter().filter(|(s, _)| kept(s)).map(|(s, _)| s.clone()).collect();
    let removing: Vec<&(String, usize)> = present.iter().filter(|(s, _)| !kept(s)).collect();
    let reason = if keep.is_empty() {
        Some("no core in this build claims any cheat system, so there is nothing to prune against.".to_string())
    } else if present.is_empty() {
        Some(format!("no cheat files in {}.", db.directory))
    } else if keeping.is_empty() {
        Some("none of the systems on disk match any core in this build - refusing to delete the whole database.".to_string())
    } else if removing.is_empty() {
        Some("every system on disk is already one a core can use.".to_string())
    } else {
        None
    };
    let removing = if reason.is_none() { removing.into_iter().map(|(s, n)| measure(&db.directory, s, *n)).collect() } else { Vec::new() };
    Plan { removing, keeping, reason }
}

/// A system folder's bytes, every file under it counted as .NET measures one; what cannot be read counts nothing.
fn measure(root: &str, system: &str, files: usize) -> Pruned {
    let folder = path::combine(Style::HOST, root, system);
    let mut bytes = 0u64;
    let _ = visit_files(&folder, |file, _| {
        if let Ok(m) = std::fs::symlink_metadata(file) {
            bytes += m.len();
        }
    });
    Pruned { system: system.to_string(), path: folder, files, bytes }
}

/// `CheatDatabasePruner.Apply`: each planned folder removed where it lies inside the tree, failures said and not thrown.
pub fn apply(db: &Database, removing: &[Pruned]) -> (usize, Vec<String>) {
    let mut failed = Vec::new();
    let mut removed = 0;
    for system in removing {
        if !is_inside(&db.directory, &system.path) {
            failed.push(format!("{} (outside {})", system.system, db.directory));
            continue;
        }
        match remove_tree(&system.path) {
            Ok(()) => removed += 1,
            Err(e) => failed.push(format!("{} ({})", system.system, emusen_dotnet::file::io_message(&e, &system.path))),
        }
    }
    (removed, failed)
}

/// Resolved on both sides, so neither `..` nor a link reaches out of the tree.
fn is_inside(root: &str, candidate: &str) -> bool {
    let full_root = path::full_path(root);
    let full_candidate = path::full_path(candidate);
    let (r, c) = (path::trim_ending_separator(Style::HOST, &full_root), path::trim_ending_separator(Style::HOST, &full_candidate));
    if std::fs::symlink_metadata(c).is_ok_and(|m| m.file_type().is_symlink()) {
        return false;
    }
    c.len() > r.len() && c.starts_with(r) && c[r.len()..].starts_with(Style::HOST.separator())
}

/// `Directory.Delete(path, recursive: true)` on Unix: every entry tried, links removed and not followed, the first
/// failure given after the rest were tried, and the folder itself last.
fn remove_tree(folder: &str) -> std::io::Result<()> {
    let mut first: Option<std::io::Error> = None;
    match std::fs::read_dir(folder) {
        Ok(entries) => {
            for entry in entries {
                let result = entry.and_then(|entry| {
                    let p = entry.path();
                    if entry.file_type()?.is_dir() { remove_tree(&p.to_string_lossy()) } else { std::fs::remove_file(&p) }
                });
                if let Err(e) = result {
                    first.get_or_insert(e);
                }
            }
        }
        Err(e) => {
            first.get_or_insert(e);
        }
    }
    match first {
        Some(e) => Err(e),
        None => std::fs::remove_dir(folder),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plan_keeps_the_named_systems_and_refuses_to_remove_everything() {
        let root = std::env::temp_dir().join(format!("emusen-cheats-prune-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for dir in ["Keep", "Drop", "Drop/Sub"] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        std::fs::write(root.join("Keep/a.cht"), b"1234").unwrap();
        std::fs::write(root.join("Drop/b.cht"), b"12").unwrap();
        std::fs::write(root.join("Drop/Sub/c.txt"), b"123").unwrap();
        let db = Database::scan(&root.to_string_lossy());
        assert_eq!(plan(&db, &[]).reason.as_deref(), Some("no core in this build claims any cheat system, so there is nothing to prune against."));
        assert!(plan(&db, &["Other".into()]).reason.unwrap().starts_with("none of the systems"));
        let p = plan(&db, &["keep".into()]);
        assert_eq!((p.reason, p.keeping.as_slice(), p.removing.len(), p.removing[0].bytes), (None, ["Keep".to_string()].as_slice(), 1, 5));
        assert_eq!(apply(&db, &p.removing), (1, Vec::new()));
        assert!(!root.join("Drop").exists() && root.join("Keep/a.cht").exists());
        std::fs::remove_dir_all(&root).unwrap();
    }
}

//! A tree walked and a file measured: every file under a folder by walkdir, links followed, and a file's existence,
//! length and time in the 100 ns ticks the library and the unpacked folders store. Moved out of `emusen-cheats` for the
//! library to share; see EmuSen_RustPlatform.md §47.17 and EmuSen_Backend.md §28.2.

use crate::dotnet_path::{self as path, Style};

/// What a walk passes over: hidden entries, and folders that refuse to be read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub skip_hidden: bool,
    pub ignore_inaccessible: bool,
}

/// A folder and a name in it joined by one separator.
pub fn join(folder: &str, name: &str) -> String {
    if folder.is_empty() || path::ends_in_separator(Style::HOST, folder) { format!("{folder}{name}") } else { format!("{folder}{}{name}", Style::HOST.separator()) }
}

/// Whether a listed entry is a directory: a link is taken as what it links to.
pub fn listed_as_directory(entry: &std::fs::DirEntry) -> bool {
    std::fs::metadata(entry.path()).is_ok_and(|m| m.is_dir())
}

/// Whether a name is hidden on Unix: it begins with a period.
pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// Every file under a folder, depth first and each folder in the system's order, handed over with its name: links to
/// folders followed, a link that loops back to a folder it is in passed over, a link whose target cannot be read handed
/// over as a file; a hidden entry skipped, folder or file, where the options say; a folder that refuses to be read
/// passed over where they say so. Any other error ends the walk, after what was handed over before it.
pub fn visit_files_with(folder: &str, options: Options, mut each: impl FnMut(&str, &str)) -> std::io::Result<()> {
    let hidden = |name: &std::ffi::OsStr| options.skip_hidden && is_hidden(&name.to_string_lossy());
    let walk = walkdir::WalkDir::new(folder).follow_links(true).min_depth(1).into_iter().filter_entry(|e| !hidden(e.file_name()));
    for entry in walk {
        let error = match entry {
            Ok(e) if e.file_type().is_dir() => continue,
            Ok(e) => {
                each(&e.path().to_string_lossy(), &e.file_name().to_string_lossy());
                continue;
            }
            Err(e) => e,
        };
        let dangling = error.path().filter(|at| std::fs::metadata(at).is_err() && std::fs::symlink_metadata(at).is_ok_and(|m| m.file_type().is_symlink()));
        if let Some(at) = dangling {
            let name = at.file_name().unwrap_or_default();
            if !hidden(name) {
                each(&at.to_string_lossy(), &name.to_string_lossy());
            }
        } else if error.loop_ancestor().is_none() && !(options.ignore_inaccessible && error.io_error().is_some_and(|x| x.kind() == std::io::ErrorKind::PermissionDenied)) {
            return Err(error.into());
        }
    }
    Ok(())
}

/// Every file under a folder, nothing skipped.
pub fn visit_files(folder: &str, each: impl FnMut(&str, &str)) -> std::io::Result<()> {
    visit_files_with(folder, Options::default(), each)
}

/// The ticks of 1 January 1970 in the stored count of 100 ns from the year 1.
pub const UNIX_EPOCH_TICKS: i64 = 621_355_968_000_000_000;

/// A file's length and modification time in ticks, both of a link as the link's own (`lstat`); none where nothing is
/// there or it is a folder.
pub fn file_info(at: &str) -> Option<(i64, i64)> {
    let own = std::fs::symlink_metadata(at).ok()?;
    let directory = if own.file_type().is_symlink() { std::fs::metadata(at).is_ok_and(|m| m.is_dir()) } else { own.is_dir() };
    if directory {
        return None;
    }
    Some((own.len() as i64, write_ticks(&own)))
}

/// A modification time as ticks since the year 1, in UTC.
pub fn write_ticks(metadata: &std::fs::Metadata) -> i64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        UNIX_EPOCH_TICKS + metadata.mtime() * 10_000_000 + metadata.mtime_nsec() / 100
    }
    #[cfg(not(unix))]
    {
        match metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()) {
            Some(d) => UNIX_EPOCH_TICKS + (d.as_nanos() / 100) as i64,
            None => UNIX_EPOCH_TICKS,
        }
    }
}

/// A folder, or a link to one.
pub fn directory_exists(at: &str) -> bool {
    std::fs::metadata(at).is_ok_and(|m| m.is_dir())
}

/// A file, or a link to one; a folder is not.
pub fn file_exists(at: &str) -> bool {
    std::fs::metadata(at).is_ok_and(|m| !m.is_dir())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn a_walk_skips_hidden_entries_and_unreadable_folders_where_asked() {
        let temp = TempDir::new("walk");
        for f in ["a.nes", ".hidden.nes", "sub/b.nes", ".dir/c.nes", "sub/deep/d.nes"] {
            let at = temp.join(f);
            std::fs::create_dir_all(std::path::Path::new(&at).parent().unwrap()).unwrap();
            std::fs::write(at, b"").unwrap();
        }
        let mut all = Vec::new();
        visit_files(temp.path(), |at, _| all.push(at.to_string())).unwrap();
        assert_eq!(all.len(), 5);
        let mut seen = Vec::new();
        visit_files_with(temp.path(), Options { skip_hidden: true, ignore_inaccessible: true }, |_, name| seen.push(name.to_string())).unwrap();
        seen.sort();
        assert_eq!(seen, ["a.nes", "b.nes", "d.nes"]);
    }

    #[cfg(unix)]
    #[test]
    fn a_walk_follows_links_lists_dangling_ones_and_passes_over_loops_and_closed_folders() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let temp = TempDir::new("links");
        let root = std::path::Path::new(temp.path());
        std::fs::create_dir_all(root.join("a/closed")).unwrap();
        std::fs::write(root.join("a/real.nes"), b"").unwrap();
        std::fs::write(root.join("a/closed/inside.nes"), b"").unwrap();
        symlink(root.join("a"), root.join("linked")).unwrap();
        symlink(root.join("missing"), root.join("dangling.nes")).unwrap();
        symlink(root.join("missing"), root.join(".hidden-dangling.nes")).unwrap();
        symlink(root, root.join("a/back")).unwrap();
        std::fs::set_permissions(root.join("a/closed"), std::fs::Permissions::from_mode(0o000)).unwrap();
        let walked = |options: Options| {
            let mut seen = Vec::new();
            let done = visit_files_with(temp.path(), options, |at, name| seen.push((at[temp.path().len() + 1..].to_string(), name.to_string())));
            seen.sort();
            (done.is_ok(), seen)
        };
        let names = |list: &[&str]| list.iter().map(|n| (n.to_string(), n.rsplit('/').next().unwrap().to_string())).collect::<Vec<_>>();
        let quiet = walked(Options { skip_hidden: true, ignore_inaccessible: true });
        let loud = walked(Options::default());
        std::fs::set_permissions(root.join("a/closed"), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(quiet, (true, names(&["a/real.nes", "dangling.nes", "linked/real.nes"])));
        assert!(!loud.0, "a closed folder ends a walk that does not pass it over");
    }

    #[test]
    fn a_file_is_measured_with_its_time_in_ticks() {
        let temp = TempDir::new("info");
        let at = temp.join("f.bin");
        std::fs::write(&at, b"abcd").unwrap();
        let (length, ticks) = file_info(&at).unwrap();
        assert_eq!(length, 4);
        assert!(ticks > UNIX_EPOCH_TICKS);
        assert_eq!(file_info(temp.path()), None);
        assert_eq!(file_info(&temp.join("absent")), None);
    }
}

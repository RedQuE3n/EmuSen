//! A tree walked and a file measured as .NET does: `Directory.EnumerateFiles` with its `EnumerationOptions`, and
//! `FileInfo`'s existence, length and `LastWriteTimeUtc`. Moved out of `emusen-cheats` for the library to share; see
//! EmuSen_Backend.md §15.3 and §28.2.

use crate::dotnet_path::{self as path, Style};
use std::collections::VecDeque;

/// The `EnumerationOptions` a walk takes: `AttributesToSkip` holding `Hidden`, and `IgnoreInaccessible`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub skip_hidden: bool,
    pub ignore_inaccessible: bool,
}

/// `Path.Join` of a folder and a name in it, as .NET's enumeration builds each path it gives.
pub fn join(folder: &str, name: &str) -> String {
    if folder.is_empty() || path::ends_in_separator(Style::HOST, folder) { format!("{folder}{name}") } else { format!("{folder}{}{name}", Style::HOST.separator()) }
}

/// Whether a listed entry is a directory as .NET's enumeration takes it: a link is taken as what it links to.
pub fn listed_as_directory(entry: &std::fs::DirEntry) -> bool {
    std::fs::metadata(entry.path()).is_ok_and(|m| m.is_dir())
}

/// Whether a name is `FileAttributes.Hidden` on Unix: it begins with a period.
pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

fn inaccessible(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::PermissionDenied
}

/// Every file under a folder, handed over as `Directory.EnumerateFiles(folder, "*", options)` with
/// `RecurseSubdirectories` yields them: a folder's files in the system's order, its folders queued and visited after,
/// links to folders followed; a hidden entry skipped, folder or file, where the options say; a folder that refuses to
/// be read passed over where they say so. Any other error ends the walk, after what was handed over before it, as it
/// ends .NET's lazy enumeration.
pub fn visit_files_with(folder: &str, options: Options, mut each: impl FnMut(&str, &str)) -> std::io::Result<()> {
    let mut queue = VecDeque::from([folder.to_string()]);
    while let Some(dir) = queue.pop_front() {
        let listing = match std::fs::read_dir(&dir) {
            Ok(listing) => listing,
            Err(e) if options.ignore_inaccessible && inaccessible(&e) => continue,
            Err(e) => return Err(e),
        };
        for entry in listing {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if options.skip_hidden && is_hidden(&name) {
                continue;
            }
            let at = join(&dir, &name);
            if listed_as_directory(&entry) {
                queue.push_back(at);
            } else {
                each(&at, &name);
            }
        }
    }
    Ok(())
}

/// `Directory.EnumerateFiles(folder, pattern, AllDirectories)`: every file, nothing skipped.
pub fn visit_files(folder: &str, each: impl FnMut(&str, &str)) -> std::io::Result<()> {
    visit_files_with(folder, Options::default(), each)
}

/// The ticks of 1 January 1970 in .NET's count of 100 ns from the year 1.
pub const UNIX_EPOCH_TICKS: i64 = 621_355_968_000_000_000;

/// A file as `FileInfo` sees it: its length and its `LastWriteTimeUtc.Ticks`, both of a link as the link's own, as
/// .NET's Unix `FileStatus` takes them from `lstat`; none where `Exists` is false (nothing there, or a folder).
pub fn file_info(at: &str) -> Option<(i64, i64)> {
    let own = std::fs::symlink_metadata(at).ok()?;
    let directory = if own.file_type().is_symlink() { std::fs::metadata(at).is_ok_and(|m| m.is_dir()) } else { own.is_dir() };
    if directory {
        return None;
    }
    Some((own.len() as i64, write_ticks(&own)))
}

/// A modification time as .NET's ticks since the year 1, in UTC.
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

/// `Directory.Exists`: a folder, or a link to one.
pub fn directory_exists(at: &str) -> bool {
    std::fs::metadata(at).is_ok_and(|m| m.is_dir())
}

/// `File.Exists`: a file, or a link to one; a folder is not.
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

    #[test]
    fn a_file_is_measured_with_its_time_in_dotnet_ticks() {
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

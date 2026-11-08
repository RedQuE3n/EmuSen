//! The key a stored path is filed under: the one lexical normalisation, so states.db, games.db and the unpacked folder name keep their text (EmuSen_RustPlatform.md §47.18).

use std::path::{Component, Path, PathBuf};

/// Whether the text ends with a separator, which `Path::components` drops and a key keeps.
pub fn ends_in_separator(path: &Path) -> bool {
    path.as_os_str().as_encoded_bytes().last().is_some_and(|&b| std::path::is_separator(b as char))
}

/// The path made absolute against `working`, then lexically: `.` parts and repeated separators gone, `..` unwinding one part and stopping at the root, a trailing separator kept; links are not followed.
pub fn key_from(path: &Path, working: &Path) -> PathBuf {
    let rooted = if path.has_root() { path.to_path_buf() } else { working.join(path) };
    let mut key = PathBuf::new();
    for part in rooted.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                key.pop();
            }
            other => key.push(other),
        }
    }
    if ends_in_separator(&rooted) && key.parent().is_some() {
        key.push("");
    }
    key
}

/// [`key_from`] against the working directory, or the root where it cannot be read.
pub fn key(path: impl AsRef<Path>) -> PathBuf {
    let working = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(std::path::MAIN_SEPARATOR_STR));
    key_from(path.as_ref(), &working)
}

/// [`key`] as the text a database row or a folder-name hash holds.
pub fn key_text(path: impl AsRef<Path>) -> String {
    key(path).to_string_lossy().into_owned()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn k(path: &str, working: &str) -> String {
        key_from(Path::new(path), Path::new(working)).to_string_lossy().into_owned()
    }

    #[test]
    fn relative_parts_go_and_a_trailing_separator_stays() {
        assert_eq!(k("/a/./b/../c", "/w"), "/a/c");
        assert_eq!(k("/a/b/", "/w"), "/a/b/");
        assert_eq!(k("/a/b/.", "/w"), "/a/b");
        assert_eq!(k("/a/b/..", "/w"), "/a");
        assert_eq!(k("/a/b/../", "/w"), "/a/");
        assert_eq!(k("/a/..", "/w"), "/");
        assert_eq!(k("/..", "/w"), "/");
        assert_eq!(k("/", "/w"), "/");
        assert_eq!(k("//", "/w"), "/");
        assert_eq!(k("/../a", "/w"), "/a");
        assert_eq!(k("//a///b", "/w"), "/a/b");
        assert_eq!(k("x/../y", "/w"), "/w/y");
        assert_eq!(k("..", "/w/v"), "/w");
        assert_eq!(k("/a/...", "/w"), "/a/...");
        assert_eq!(k("/a/.b/..c", "/w"), "/a/.b/..c");
        assert_eq!(k("/s/Game.state.", "/w"), "/s/Game.state.");
    }
}

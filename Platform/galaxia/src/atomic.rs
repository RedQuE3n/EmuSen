//! Whole-file reads and writes that an interrupted save cannot truncate: the C# `AtomicFile`. See EmuSen_Galaxia.md §4.
//!
//! The guarantee is the C#'s and no more: the bytes go to `<path>.tmp` and are renamed over the live file, so a reader
//! or a killed process sees the whole old file or the whole new one. Nothing is flushed to the device, so it is not a
//! promise against power loss; EmuSen_RustPlatform.md §10.4 records why that was left as it is.

use crate::dotnet_path::{self as path, Style};
use crate::tree::file_exists;
use std::io::Read;

pub const TEMP_SUFFIX: &str = ".tmp";

/// An error's words as a sentence, for a diagnostic that goes on to say what was done about it.
pub fn words(error: &std::io::Error) -> String {
    let text = error.to_string();
    if text.ends_with('.') { text } else { format!("{text}.") }
}

/// The file's length, or none where `File.Exists` is false: missing, a directory, or not to be examined.
pub fn length(file: &str) -> Option<u64> {
    if !file_exists(file) {
        return None;
    }
    // A file that exists and will not be examined (a dangling link) is given a length, so that reading it is tried and says why it failed.
    Some(std::fs::metadata(file).map_or(0, |m| m.len()))
}

/// The file's bytes; none for a missing file, and the diagnostic for one that is there and will not read.
pub fn try_read(file: &str) -> Result<Option<Vec<u8>>, String> {
    if !file_exists(file) {
        return Ok(None);
    }
    std::fs::read(file).map(Some).map_err(|error| format!("{file}: {} Treating it as absent.", words(&error)))
}

/// The file read into the caller's buffer: its whole length, with the buffer filled only when it holds all of it.
pub fn read_into(file: &str, out: &mut [u8]) -> Result<Option<usize>, String> {
    if !file_exists(file) {
        return Ok(None);
    }
    let mut read = || -> std::io::Result<usize> {
        let mut source = std::fs::File::open(file)?;
        let mut filled = 0;
        while filled < out.len() {
            match source.read(&mut out[filled..])? {
                0 => return Ok(filled),
                n => filled += n,
            }
        }
        // The buffer is full: whatever is left is counted, so the caller can come back with room for it.
        let mut rest = [0u8; 1 << 16];
        loop {
            match source.read(&mut rest)? {
                0 => return Ok(filled),
                n => filled += n,
            }
        }
    };
    read().map(Some).map_err(|error| format!("{file}: {} Treating it as absent.", words(&error)))
}

/// The bytes written beside the file and renamed over it; the diagnostic when any step failed, the live file then being as it was.
pub fn write(file: &str, contents: &[u8]) -> Result<(), String> {
    let write = || -> std::io::Result<()> {
        if let Some(directory) = path::directory_name(Style::HOST, file).filter(|d| !d.is_empty()) {
            std::fs::create_dir_all(directory)?;
        }
        let temp = format!("{file}{TEMP_SUFFIX}");
        std::fs::write(&temp, contents)?;
        std::fs::rename(&temp, file)
    };
    write().map_err(|error| format!("{file}: {} The previous file is unchanged.", words(&error)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn a_write_makes_its_directory_replaces_the_file_and_leaves_no_temp() {
        let temp = TempDir::new("atomic");
        let file = temp.join("Saves/SNES/game.srm");
        write(&file, b"first").unwrap();
        write(&file, b"second, longer").unwrap();
        assert_eq!(try_read(&file).unwrap().as_deref(), Some(&b"second, longer"[..]));
        assert!(!file_exists(&format!("{file}{TEMP_SUFFIX}")));
        assert_eq!(length(&file), Some(14));
    }

    #[test]
    fn an_abandoned_temp_file_never_becomes_the_live_file() {
        let temp = TempDir::new("atomic");
        let file = temp.join("game.srm");
        std::fs::write(format!("{file}{TEMP_SUFFIX}"), b"half a sa").unwrap();
        assert_eq!(try_read(&file), Ok(None));
        write(&file, b"whole").unwrap();
        assert_eq!(try_read(&file).unwrap().as_deref(), Some(&b"whole"[..]));
    }

    #[test]
    fn a_missing_file_and_a_directory_are_absent_without_a_word() {
        let temp = TempDir::new("atomic");
        assert_eq!(try_read(&temp.join("none.srm")), Ok(None));
        assert_eq!(try_read(temp.path()), Ok(None));
        assert_eq!(length(temp.path()), None);
        assert_eq!(try_read(""), Ok(None));
    }

    #[cfg(unix)]
    #[test]
    fn a_dangling_link_is_there_to_be_read_and_says_why_it_cannot_be() {
        let temp = TempDir::new("atomic");
        let link = temp.join("dangling.srm");
        std::os::unix::fs::symlink(temp.join("nowhere"), &link).unwrap();
        assert_eq!(length(&link), Some(0));
        let message = try_read(&link).unwrap_err();
        assert!(message.starts_with(&link) && message.ends_with(". Treating it as absent."), "{message}");
        assert!(read_into(&link, &mut [0u8; 8]).is_err());
    }

    #[test]
    fn a_path_that_cannot_be_written_says_so_and_does_not_panic() {
        let temp = TempDir::new("atomic");
        let blocker = temp.join("blocker");
        std::fs::write(&blocker, b"a file where a directory is wanted").unwrap();
        let message = write(&temp.join("blocker/game.srm"), b"x").unwrap_err();
        assert!(message.starts_with(&temp.join("blocker/game.srm")), "{message}");
        assert!(message.ends_with(". The previous file is unchanged."), "{message}");
    }

    #[test]
    fn the_bytes_go_to_the_temp_file_first_which_a_write_over_a_directory_strands() {
        let temp = TempDir::new("atomic");
        let taken = temp.join("taken.srm");
        std::fs::create_dir_all(&taken).unwrap();
        assert!(write(&taken, b"bytes").is_err());
        assert_eq!(std::fs::read(format!("{taken}{TEMP_SUFFIX}")).unwrap(), b"bytes");
        assert!(crate::tree::directory_exists(&taken));
    }

    #[test]
    fn a_buffer_too_small_is_told_the_whole_length_and_one_large_enough_is_filled() {
        let temp = TempDir::new("atomic");
        let file = temp.join("big.state");
        let contents: Vec<u8> = (0..200_000u32).map(|i| (i * 7) as u8).collect();
        write(&file, &contents).unwrap();
        let mut small = [0u8; 100];
        assert_eq!(read_into(&file, &mut small), Ok(Some(200_000)));
        let mut exact = vec![0u8; 200_000];
        assert_eq!(read_into(&file, &mut exact), Ok(Some(200_000)));
        assert_eq!(exact, contents);
        let mut roomy = vec![0u8; 200_001];
        assert_eq!(read_into(&file, &mut roomy), Ok(Some(200_000)));
        assert_eq!(read_into(&temp.join("none"), &mut small), Ok(None));
    }
}

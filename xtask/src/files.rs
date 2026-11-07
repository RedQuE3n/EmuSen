//! The file operations the build host needs: a SHA-256, a copy that keeps the source's time, and a link for staging.

use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;

pub fn sha256_hex(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("{}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Copies unless the destination has the source's size and time already, and gives the copy the source's time.
pub fn copy_keeping_time(from: &Path, to: &Path) -> Result<(), String> {
    let meta = fs::metadata(from).map_err(|e| format!("{}: {e}", from.display()))?;
    if let Ok(there) = fs::metadata(to) {
        if there.len() == meta.len() && there.modified().ok() == meta.modified().ok() {
            return Ok(());
        }
    }
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::copy(from, to).map_err(|e| format!("{} to {}: {e}", from.display(), to.display()))?;
    let modified = meta.modified().map_err(|e| format!("{}: {e}", from.display()))?;
    fs::File::options().write(true).open(to).and_then(|f| f.set_modified(modified)).map_err(|e| format!("{}: {e}", to.display()))
}

/// Puts `from` at `to` as a hard link, or a copy where the two are on different file systems.
pub fn stage(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let _ = fs::remove_file(to);
    if fs::hard_link(from, to).is_ok() {
        return Ok(());
    }
    copy_keeping_time(from, to)
}

/// Writes `text` unless the file holds it already, so an unchanged manifest keeps its time.
pub fn write_if_different(path: &Path, text: &str) -> Result<(), String> {
    if fs::read_to_string(path).is_ok_and(|t| t == text) {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hash_is_sha256sum_s() {
        let dir = std::env::temp_dir().join(format!("xtask-files-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let f = dir.join("abc");
        fs::write(&f, b"abc").unwrap();
        assert_eq!(sha256_hex(&f).unwrap(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        let copy = dir.join("deep/copy");
        copy_keeping_time(&f, &copy).unwrap();
        assert_eq!(fs::metadata(&copy).unwrap().modified().unwrap(), fs::metadata(&f).unwrap().modified().unwrap());
        let _ = fs::remove_dir_all(&dir);
    }
}

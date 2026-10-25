//! The pinned chdman: `chdman.pin` read, a binary's SHA-256 taken, and a binary refused with words naming both hashes
//! when it is not the one pinned for this platform (EmuSen_GamePackages.md §4.9, item 4).

use sha2::{Digest, Sha256};
use std::io::Read;

/// The pin as the repository records it.
pub const TEXT: &str = include_str!("../chdman.pin");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    pub release: String,
    pub tag: String,
    pub source: String,
    pub source_sha256: String,
    /// Each platform's binary, by runtime identifier.
    pub binaries: Vec<(String, String)>,
}

impl Pin {
    /// The SHA-256 pinned for a runtime identifier.
    pub fn binary(&self, rid: &str) -> Option<&str> {
        self.binaries.iter().find(|(r, _)| r == rid).map(|(_, h)| h.as_str())
    }
}

fn sha256_text(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A pin's text: `#` comments, then one `key value` a line.
pub fn parse(text: &str) -> Result<Pin, String> {
    let mut pin = Pin { release: String::new(), tag: String::new(), source: String::new(), source_sha256: String::new(), binaries: Vec::new() };
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(char::is_whitespace) else { return Err(format!("chdman.pin line {}: a key with no value", n + 1)) };
        let value = value.trim().to_string();
        match key {
            "release" => pin.release = value,
            "tag" => pin.tag = value,
            "source" => pin.source = value,
            "source-sha256" if sha256_text(&value) => pin.source_sha256 = value,
            rid if rid.contains('-') && sha256_text(&value) => pin.binaries.push((rid.to_string(), value)),
            _ => return Err(format!("chdman.pin line {}: {line} is not a pin", n + 1)),
        }
    }
    if pin.release.is_empty() || pin.tag.is_empty() {
        return Err("chdman.pin names no release or tag".into());
    }
    Ok(pin)
}

/// The pin this build carries.
pub fn pinned() -> &'static Pin {
    static PIN: std::sync::OnceLock<Pin> = std::sync::OnceLock::new();
    PIN.get_or_init(|| parse(TEXT).expect("chdman.pin is the repository's own and parses"))
}

/// This build's runtime identifier, as .NET and the published tree name it.
pub fn rid() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("macos", "aarch64") => "osx-arm64",
        ("windows", "x86_64") => "win-x64",
        _ => "unknown",
    }
}

/// The file's SHA-256 as 64 lower-case hexadecimal digits.
pub fn sha256(path: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// The binary at `path` against the hash `expected` pinned for it: its SHA-256 when they are equal, else the words
/// that refuse it. `None` is a platform the pin has no binary for.
pub fn check(path: &str, expected: Option<&str>, release: &str) -> Result<String, String> {
    let Some(expected) = expected else {
        return Err(format!("chdman.pin has no chdman for {}: nothing was converted.", rid()));
    };
    let actual = sha256(path).map_err(|e| format!("chdman was not found at {path} ({e}): nothing was converted."))?;
    if actual != expected {
        return Err(format!("chdman at {path} has SHA-256 {actual}, but this build is pinned to {expected} (MAME {release}, {}): it was not run, and nothing was converted.", rid()));
    }
    Ok(actual)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_repositorys_pin_parses_and_names_this_platform() {
        let pin = pinned();
        assert_eq!((pin.release.as_str(), pin.tag.as_str()), ("0.289", "mame0289"));
        assert!(pin.binary("linux-x64").is_some());
        assert!(parse("release 1\n").is_err());
        assert!(parse("release 1\ntag t\nlinux-x64 nothex\n").is_err());
    }

    #[test]
    fn a_binary_that_is_not_the_pinned_one_is_refused_naming_both_hashes() {
        let dir = std::env::temp_dir().join(format!("emusen-pin-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("chdman");
        std::fs::write(&fake, b"not chdman").unwrap();
        let fake = fake.to_string_lossy().into_owned();
        let actual = sha256(&fake).unwrap();
        let pinned_hash = "55abb0d257ba692ff14c4bd20073f1717ac54f362e2bdc56ee3e637783c87e0f";
        let words = check(&fake, Some(pinned_hash), "0.289").unwrap_err();
        assert!(words.contains(&actual) && words.contains(pinned_hash) && words.contains("nothing was converted"), "{words}");
        assert_eq!(check(&fake, Some(&actual), "0.289").unwrap(), actual);
        assert!(check(&fake, None, "0.289").unwrap_err().contains("no chdman for"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

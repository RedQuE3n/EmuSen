//! An installed libretro core's folder under the player's `/Cores/libretro/<stem>/` and its `NOTICE`: one reader and
//! writer, owned by Cores and used by the libretro host and discovery alike (EmuSen_Cores.md §5.3, §7; decided
//! 2026-10-31). The folder's files are named here and nowhere else, on the core's file stem, never its `corename`.

use sha2::{Digest, Sha256};
use std::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The folder under `/Cores` that holds one folder per installed libretro core.
pub const LIBRETRO: &str = "libretro";
/// Inside [`LIBRETRO`]: the install's working folder and the build kept for Roll Back, never listed as entries.
pub const STAGING: &str = ".staging";
pub const PREVIOUS: &str = ".previous";
/// The entry's record, its proxy's target and its data files.
pub const NOTICE: &str = "NOTICE";
pub const TARGET: &str = "broker.target";
pub const ASSETS: &str = "assets";
/// The project's proxy and adapter, copied into each entry (EmuSen_LibretroHost.md §1.1), without the library suffix.
pub const BROKER: &str = "emusen-core-broker";
pub const ADAPTER: &str = "emusen-libretro-host";
/// The one `NOTICE` format this crate reads and writes.
pub const FORMAT: u32 = 1;

/// The platform's shared library suffix, as the buildbot names a core's file.
pub const fn library_suffix() -> &'static str {
    if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

/// Whether a file stem (`pcsx2` of `pcsx2_libretro.so`) can name an entry's folder and files: 1 to 64 of
/// `[A-Za-z0-9_.-]`, not starting with a dot, so it is never a path, a hidden folder, `.` or `..`.
pub fn valid_stem(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('.')
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

fn stem_before<'a>(file_name: &'a str, tail: &str) -> Option<&'a str> {
    file_name.strip_suffix(tail).filter(|s| valid_stem(s))
}

/// The stem of the buildbot's zip, `<stem>_libretro.<suffix>.zip`, for this platform's suffix.
pub fn stem_of_zip(file_name: &str) -> Option<&str> {
    stem_before(file_name, &format!("_libretro.{}.zip", library_suffix()))
}

/// The stem of a core information file, `<stem>_libretro.info`.
pub fn stem_of_info(file_name: &str) -> Option<&str> {
    stem_before(file_name, "_libretro.info")
}

/// The stem of the libretro core itself, `<stem>_libretro.<suffix>`.
pub fn stem_of_core(file_name: &str) -> Option<&str> {
    stem_before(file_name, &format!("_libretro.{}", library_suffix()))
}

/// What a `NOTICE` records of a downloaded core (EmuSen_Cores.md §5.2 step 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// The file stem the folder and files are named on (`pcsx2`), from the zip's and the `.info`'s names.
    pub stem: String,
    /// The `.info`'s `display_name`.
    pub name: String,
    /// The `.info`'s `corename` (`LRPS2`), information only: it names nothing on disk.
    pub corename: String,
    /// The `.info`'s `authors`, split on `|`.
    pub authors: Vec<String>,
    /// The `.info`'s `license`, as it writes it.
    pub license: String,
    /// The address the zip was fetched from, `https://` only.
    pub source: String,
    /// The build's date in the buildbot's index, `YYYY-MM-DD`.
    pub build: String,
    /// The zip's CRC-32 in the index, which the update check compares.
    pub index_crc32: u32,
    /// The libretro core's own SHA-256, lowercase hex.
    pub sha256: String,
    /// The date it was fetched, `YYYY-MM-DD`.
    pub fetched: String,
    /// Keys a later format added, kept in order and written back unchanged.
    pub extra: Vec<(String, String)>,
}

/// Why a `NOTICE` was refused, in words for the player and the log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused(pub String);

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Refused {}

fn refused<T>(text: impl Into<String>) -> Result<T, Refused> {
    Err(Refused(text.into()))
}

const KEYS: [&str; 11] = ["format", "stem", "name", "corename", "authors", "license", "source", "build", "index_crc32", "sha256", "fetched"];

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        && matches!(s[5..7].parse::<u8>(), Ok(1..=12))
        && matches!(s[8..10].parse::<u8>(), Ok(1..=31))
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn clean(key: &str, value: &str) -> Result<(), Refused> {
    if value.is_empty() {
        return refused(format!("NOTICE's {key} is empty"));
    }
    if value.chars().any(char::is_control) {
        return refused(format!("NOTICE's {key} holds a control character"));
    }
    if value.contains('\u{FEFF}') {
        return refused(format!("NOTICE's {key} holds a byte-order mark"));
    }
    Ok(())
}

impl Notice {
    /// A `NOTICE`'s text read: `key: value` lines, `#` lines and blank lines skipped, every key of format 1 once.
    pub fn parse(text: &str) -> Result<Notice, Refused> {
        let mut seen: Vec<(String, String)> = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                return refused(format!("NOTICE's line {} is not key: value", n + 1));
            };
            let (key, value) = (key.trim(), value.trim());
            if key.is_empty() || !key.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
                return refused(format!("NOTICE's line {} has no key of a-z, 0-9 and _", n + 1));
            }
            if seen.iter().any(|(k, _)| k == key) {
                return refused(format!("NOTICE names {key} twice"));
            }
            clean(key, value)?;
            seen.push((key.to_string(), value.to_string()));
        }
        let take = |key: &str| -> Result<String, Refused> {
            seen.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()).ok_or_else(|| Refused(format!("NOTICE has no {key}")))
        };
        let format = take("format")?;
        match format.parse::<u32>() {
            Ok(FORMAT) => {}
            Ok(f) if f > FORMAT => return refused(format!("NOTICE is format {f}, newer than this build reads ({FORMAT})")),
            _ => return refused(format!("NOTICE's format is not {FORMAT}")),
        }
        let crc = take("index_crc32")?;
        if crc.len() != 8 || !crc.bytes().all(|b| b.is_ascii_hexdigit()) {
            return refused("NOTICE's index_crc32 is not eight hex digits");
        }
        let notice = Notice {
            stem: take("stem")?,
            name: take("name")?,
            corename: take("corename")?,
            authors: take("authors")?.split('|').map(str::trim).filter(|a| !a.is_empty()).map(str::to_string).collect(),
            license: take("license")?,
            source: take("source")?,
            build: take("build")?,
            index_crc32: u32::from_str_radix(&crc, 16).map_err(|_| Refused("NOTICE's index_crc32 is not eight hex digits".into()))?,
            sha256: take("sha256")?.to_ascii_lowercase(),
            fetched: take("fetched")?,
            extra: seen.into_iter().filter(|(k, _)| !KEYS.contains(&k.as_str())).collect(),
        };
        notice.check()?;
        Ok(notice)
    }

    /// The rules every field holds to, so that what is written reads back.
    pub fn check(&self) -> Result<(), Refused> {
        for (key, value) in [("name", &self.name), ("corename", &self.corename), ("license", &self.license), ("source", &self.source)] {
            clean(key, value)?;
        }
        if !valid_stem(&self.stem) {
            return refused(format!("NOTICE's stem {:?} cannot name a folder", self.stem));
        }
        if self.authors.is_empty() {
            return refused("NOTICE has no authors");
        }
        for a in &self.authors {
            clean("authors", a)?;
            if a.contains('|') || a.trim() != a {
                return refused("NOTICE's authors hold one that cannot be written back");
            }
        }
        if !self.source.starts_with("https://") {
            return refused("NOTICE's source is not an https:// address");
        }
        if self.source.rsplit('/').next().and_then(stem_of_zip) != Some(self.stem.as_str()) {
            return refused(format!("NOTICE's source is not {}_libretro.{}.zip", self.stem, library_suffix()));
        }
        if !is_date(&self.build) {
            return refused("NOTICE's build is not a date YYYY-MM-DD");
        }
        if !is_date(&self.fetched) {
            return refused("NOTICE's fetched is not a date YYYY-MM-DD");
        }
        if !is_sha256(&self.sha256) {
            return refused("NOTICE's sha256 is not 64 lowercase hex digits");
        }
        for (k, v) in &self.extra {
            if KEYS.contains(&k.as_str()) || k.is_empty() || !k.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
                return refused(format!("NOTICE's extra key {k:?} cannot be written"));
            }
            clean(k, v)?;
        }
        Ok(())
    }

    /// The `NOTICE` text, keys in format 1's order, then the extra keys.
    pub fn to_text(&self) -> Result<String, Refused> {
        self.check()?;
        let mut t = String::from("# Installed by EmuSen's Cores at the player's request; the core keeps its own licence.\n");
        let fields: [(&str, String); 11] = [
            ("format", FORMAT.to_string()),
            ("stem", self.stem.clone()),
            ("name", self.name.clone()),
            ("corename", self.corename.clone()),
            ("authors", self.authors.join("|")),
            ("license", self.license.clone()),
            ("source", self.source.clone()),
            ("build", self.build.clone()),
            ("index_crc32", format!("{:08x}", self.index_crc32)),
            ("sha256", self.sha256.clone()),
            ("fetched", self.fetched.clone()),
        ];
        for (k, v) in fields.iter().map(|(k, v)| (*k, v.as_str())).chain(self.extra.iter().map(|(k, v)| (k.as_str(), v.as_str()))) {
            t.push_str(k);
            t.push_str(": ");
            t.push_str(v);
            t.push('\n');
        }
        Ok(t)
    }
}

/// A file's SHA-256, lowercase hex, read in pieces.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// One installed core's folder, `<cores>/libretro/<stem>/`, and the names of its files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub folder: PathBuf,
    pub stem: String,
}

impl Entry {
    /// The entry for a file stem under a `/Cores` folder; refused where the stem cannot name a folder.
    pub fn new(cores: &Path, stem: &str) -> Result<Entry, Refused> {
        if !valid_stem(stem) {
            return refused(format!("{stem:?} cannot name a core's folder"));
        }
        Ok(Entry { folder: cores.join(LIBRETRO).join(stem), stem: stem.to_string() })
    }

    /// The entry a `NOTICE` describes under a `/Cores` folder, named on its stem.
    pub fn for_notice(cores: &Path, notice: &Notice) -> Result<Entry, Refused> {
        Entry::new(cores, &notice.stem)
    }

    /// The same entry's files in another folder, such as its staging or previous copy.
    pub fn at(folder: PathBuf, stem: &str) -> Result<Entry, Refused> {
        if !valid_stem(stem) {
            return refused(format!("{stem:?} cannot name a core's folder"));
        }
        Ok(Entry { folder, stem: stem.to_string() })
    }

    pub fn notice_path(&self) -> PathBuf {
        self.folder.join(NOTICE)
    }

    /// `<stem>_libretro.<suffix>`, the libretro core as downloaded.
    pub fn core_path(&self) -> PathBuf {
        self.folder.join(format!("{}_libretro.{}", self.stem, library_suffix()))
    }

    /// `<stem>_libretro.info`, its published core information.
    pub fn info_path(&self) -> PathBuf {
        self.folder.join(format!("{}_libretro.info", self.stem))
    }

    pub fn broker_path(&self) -> PathBuf {
        self.folder.join(format!("{BROKER}.{}", library_suffix()))
    }

    pub fn adapter_path(&self) -> PathBuf {
        self.folder.join(format!("{ADAPTER}.{}", library_suffix()))
    }

    pub fn target_path(&self) -> PathBuf {
        self.folder.join(TARGET)
    }

    pub fn assets_path(&self) -> PathBuf {
        self.folder.join(ASSETS)
    }

    /// `broker.target`'s one line: the adapter copy's file name.
    pub fn target_text() -> String {
        format!("{ADAPTER}.{}\n", library_suffix())
    }

    /// The entry's `NOTICE`, read and checked, its stem the folder's.
    pub fn read_notice(&self) -> Result<Notice, Refused> {
        let text = std::fs::read_to_string(self.notice_path()).map_err(|e| Refused(format!("{}: {e}", self.notice_path().display())))?;
        let notice = Notice::parse(&text)?;
        if notice.stem != self.stem {
            return refused(format!("NOTICE names {} but its folder is {}", notice.stem, self.stem));
        }
        Ok(notice)
    }

    /// The libretro core hashed against its `NOTICE`; the hash when they agree. Integrity only, never trust: the folder
    /// is the player's to write, so trust is the hash in `cores.approved`, checked before the core is started.
    pub fn verify(&self) -> Result<String, Refused> {
        let notice = self.read_notice()?;
        let path = self.core_path();
        let hash = sha256_file(&path).map_err(|e| Refused(format!("{}: {e}", path.display())))?;
        if hash != notice.sha256 {
            return refused(format!("{} is not the file NOTICE records (SHA-256 {hash}, recorded {})", path.display(), notice.sha256));
        }
        Ok(hash)
    }

    /// The entries under a `/Cores` folder, by stem, the staging and previous folders and anything not a valid stem
    /// left out; none where there is no `libretro` folder.
    pub fn list(cores: &Path) -> std::io::Result<Vec<Entry>> {
        let root = cores.join(LIBRETRO);
        let read = match std::fs::read_dir(&root) {
            Ok(r) => r,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let mut entries = Vec::new();
        for item in read {
            let item = item?;
            if !item.file_type()?.is_dir() {
                continue;
            }
            if let Some(name) = item.file_name().to_str()
                && valid_stem(name)
            {
                entries.push(Entry { folder: item.path(), stem: name.to_string() });
            }
        }
        entries.sort_by(|a, b| a.stem.cmp(&b.stem));
        Ok(entries)
    }
}

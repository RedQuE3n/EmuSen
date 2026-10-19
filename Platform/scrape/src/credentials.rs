//! EmuSen's developer identity at ScreenScraper and the player's member account, `ScreenScraperCredentials` ported
//! (§30.9 Q4): the developer file in Galaxia's config directory, then the legacy one, then the program's embedded blob;
//! the member's `screenscraper.json`, written whole with mode 0600 and moved into place. Each value is registered with
//! the redactor the moment it is read, and nothing here prints one. See EmuSen_BigPicture.md §5.7, §17.7 and §28.

use crate::redact;
use emusen_dotnet::json::{self as dj, Value};

pub const DEVELOPER_FILE: &str = "screenscraper-developer.json";
pub const MEMBER_FILE: &str = "screenscraper.json";
pub const NONE_HERE: &str = "this build carries no developer credentials and EmuSen's developer file is not on this computer";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    TreeFile,
    UserFile,
    Embedded,
}

impl Origin {
    pub fn word(self) -> &'static str {
        match self {
            Origin::TreeFile => "tree",
            Origin::UserFile => "user",
            Origin::Embedded => "embedded",
        }
    }
}

pub struct Developer {
    pub id: String,
    pub password: String,
    pub soft_name: String,
    pub origin: Origin,
}

impl std::fmt::Debug for Developer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScreenScraper developer credentials for {}", self.soft_name)
    }
}

fn escape_data(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `Uri.EscapeDataString`: every byte of the UTF-8 that is not unreserved, escaped in upper case.
pub fn escape(text: &str) -> String {
    escape_data(text)
}

fn register(secret: &str) {
    redact::register(secret);
    redact::register(&escape_data(secret));
}

fn read_text(value: &Value, name: &str) -> Option<String> {
    match value.get(name) {
        Some(Value::String(t)) if !t.broken => Some(t.text.clone()),
        _ => None,
    }
}

fn parse_developer(json: &str, origin: Origin) -> Option<Developer> {
    let root = dj::parse(json).ok()?;
    let (id, password, soft) = (read_text(&root, "devid")?, read_text(&root, "devpassword")?, read_text(&root, "softname")?);
    if [&id, &password, &soft].iter().any(|v| emusen_dotnet::text::is_blank(Some(v))) {
        return None;
    }
    redact::register(&id);
    register(&password);
    Some(Developer { id, password, soft_name: soft, origin })
}

/// What `ScreenScraperDeveloper.targets` writes: "ESSD", a version, the key's length, the key, then the JSON XORed with it.
pub fn decode(blob: &[u8]) -> Option<Developer> {
    let key_length = *blob.get(5)? as usize;
    if blob.len() <= 6 + key_length || key_length == 0 || &blob[..4] != b"ESSD" || blob[4] != 1 {
        return None;
    }
    let key = &blob[6..6 + key_length];
    let mut body: Vec<u8> = blob[6 + key_length..].iter().enumerate().map(|(i, b)| b ^ key[i % key_length]).collect();
    let text = String::from_utf8_lossy(&body).into_owned();
    let found = parse_developer(&text, Origin::Embedded);
    body.fill(0);
    found
}

/// The developer file's two places, then the embedded blob; `files` false reads no file of the machine's.
pub fn developer(files: bool, embedded: Option<&[u8]>) -> Option<Developer> {
    if files {
        let tree = emusen_galaxia::tree::tree();
        let places = [Some((tree.config_path(None, DEVELOPER_FILE), Origin::TreeFile)), tree.legacy_config_path(DEVELOPER_FILE).map(|p| (p, Origin::UserFile))];
        drop(tree);
        for (path, origin) in places.into_iter().flatten() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Some(found) = parse_developer(&text, origin) {
                    return Some(found);
                }
            }
        }
    }
    embedded.and_then(decode)
}

/// The player's member account; empty values where there is none.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Member {
    pub user: String,
    pub password: String,
    /// When ScreenScraper last accepted it, UTC ticks; none for a file the old two boxes wrote.
    pub verified: Option<i64>,
}

impl std::fmt::Debug for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.is_set() { "a ScreenScraper member account" } else { "no ScreenScraper member account" })
    }
}

impl Member {
    pub fn new(user: &str, password: &str) -> Member {
        redact::register(user);
        register(password);
        Member { user: user.to_string(), password: password.to_string(), verified: None }
    }

    pub fn is_set(&self) -> bool {
        !self.user.is_empty() && !self.password.is_empty()
    }

    pub fn path() -> String {
        emusen_galaxia::tree::tree().config_path(None, MEMBER_FILE)
    }

    /// The account file, or an empty account where it is missing or does not read.
    pub fn load() -> Member {
        let Ok(text) = std::fs::read_to_string(Member::path()) else { return Member::default() };
        let Ok(root) = dj::parse(&text) else { return Member::default() };
        let mut member = Member::new(&read_text(&root, "ssid").unwrap_or_default(), &read_text(&root, "sspassword").unwrap_or_default());
        member.verified = read_text(&root, "verified").and_then(|t| emusen_dotnet::date::instant(&t));
        member
    }

    /// Written whole beside its final name with mode 0600 from the moment it exists, then moved over the old file.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Member::path();
        if let Some(dir) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(dir)?;
        }
        let temp = format!("{path}.part");
        let mut body = String::from("{\"ssid\":");
        body.push_str(&dj::quoted(&self.user));
        body.push_str(",\"sspassword\":");
        body.push_str(&dj::quoted(&self.password));
        if let Some(at) = self.verified {
            body.push_str(",\"verified\":");
            body.push_str(&dj::quoted(&round_trip_utc(at)));
        }
        body.push('}');
        {
            use std::io::Write;
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create(true).truncate(true);
            #[cfg(unix)]
            std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
            let mut file = options.open(&temp)?;
            file.write_all(body.as_bytes())?;
        }
        #[cfg(unix)]
        std::fs::set_permissions(&temp, std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
        std::fs::rename(&temp, &path)
    }

    /// Log Out: the file is deleted, not emptied; true when there was one.
    pub fn delete() -> std::io::Result<bool> {
        match std::fs::remove_file(Member::path()) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
}

/// `DateTime.ToString("O")` of a UTC time: `2026-10-18T12:00:00.0000000Z`.
pub fn round_trip_utc(ticks: i64) -> String {
    let millis = emusen_dotnet::date::utc_millis(ticks);
    let fraction = ticks.rem_euclid(emusen_dotnet::date::TICKS_PER_SECOND);
    format!("{}.{fraction:07}Z", &millis[..19])
}

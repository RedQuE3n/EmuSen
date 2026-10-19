//! OpenEmu's sources as the failover, `OpenVgdb`, `OpenVgdbDownload` and `CoverFetcher` ported: OpenVGDB names a file
//! (by its name, else by the MD5 of the bytes the systems table's rule says OpenVGDB hashed), libretro's thumbnail
//! server has its box under that name, and OpenVGDB's own address is tried last. Asked only inside a run the player
//! started. See EmuSen_Settings_Reference.md §4.39, EmuSen_BigPicture.md §26 and EmuSen_Backend.md §30.6.

use crate::client::{TIMEOUT, Wire};
use crate::consoles::Core;
use emusen_galaxia::dotnet_files::file_exists;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_sqlite::{Bound, Connection, Sqlite};
use std::sync::Arc;
use std::time::Duration;

pub const FILE_NAME: &str = "openvgdb.sqlite";
pub const THUMBNAIL_SERVER: &str = "https://thumbnails.libretro.com/";
pub const LATEST_RELEASE: &str = "https://api.github.com/repos/OpenVGDB/OpenVGDB/releases/latest";
/// ES-DE's default ceiling for hashing a file, so an enormous image is looked up by name only.
pub const HASH_LIMIT: u64 = 384 << 20;
/// Between requests to either server.
pub const SPACING: Duration = Duration::from_millis(250);

const UNSAFE: [char; 11] = ['&', '*', '/', ':', '`', '<', '>', '?', '\\', '|', '"'];

/// libretro-thumbnails' substitution.
pub fn safe(name: &str) -> String {
    name.chars().map(|c| if UNSAFE.contains(&c) { '_' } else { c }).collect()
}

/// The exact name, then with trailing tags dropped one at a time, keeping the first.
pub fn names(rom_name: &str) -> Vec<String> {
    let mut tags: Vec<usize> = Vec::new();
    let chars: Vec<(usize, char)> = rom_name.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let mut q = i;
        while q < chars.len() && chars[q].1.is_whitespace() {
            q += 1;
        }
        if q < chars.len() && chars[q].1 == '(' {
            if let Some(r) = (q + 1..chars.len()).find(|&r| chars[r].1 == ')') {
                tags.push(chars[i].0);
                i = r + 1;
                continue;
            }
        }
        i += 1;
    }
    let mut out = vec![rom_name.to_string()];
    let mut tries = 1;
    let mut keep = tags.len() as i64 - 1;
    while keep >= 1 && tries < 3 {
        out.push(rom_name[..tags[keep as usize]].to_string());
        keep -= 1;
        tries += 1;
    }
    out
}

pub struct Match {
    pub rom_name: String,
    pub cover_url: Option<String>,
}

pub struct Database {
    db: Connection,
}

impl Database {
    /// None for a file that is absent or is not an OpenVGDB.
    pub fn open(sqlite: Arc<Sqlite>, at: &str) -> Option<Database> {
        if !file_exists(at) {
            return None;
        }
        let db = Connection::open_read_only(sqlite, at).ok()?;
        db.query("SELECT COUNT(*) FROM ROMs JOIN SYSTEMS USING (systemID) LIMIT 1", &[], |_| {}).ok()?;
        Some(Database { db })
    }

    fn find(&self, condition: &str, key: &str, systems: &[String]) -> Option<Match> {
        if systems.is_empty() {
            return None;
        }
        let names: Vec<String> = (0..systems.len()).map(|i| format!("$s{i}")).collect();
        let sql = format!(
            "SELECT r.romExtensionlessFileName, l.releaseTitleName, l.releaseCoverFront, s.systemShortName FROM ROMs r JOIN SYSTEMS s ON s.systemID = r.systemID LEFT JOIN RELEASES l ON l.romID = r.romID WHERE {condition} AND s.systemShortName IN ({}) ORDER BY l.releaseCoverFront IS NULL, l.releaseID LIMIT 1",
            names.join(", ")
        );
        let mut parameters: Vec<(&str, Bound)> = vec![("$key", Bound::Text(key))];
        for (i, s) in systems.iter().enumerate() {
            parameters.push((names[i].as_str(), Bound::Text(s)));
        }
        let mut found = None;
        self.db
            .query(&sql, &parameters, |row| {
                if found.is_none() && !row.is_null(0) {
                    found = Some(Match { rom_name: row.text(0), cover_url: row.text_or_null(2) });
                }
            })
            .ok()?;
        found
    }

    pub fn by_name(&self, stem: &str, systems: &[String]) -> Option<Match> {
        self.find("r.romExtensionlessFileName = $key", stem, systems)
    }

    pub fn by_md5(&self, md5: &str, systems: &[String]) -> Option<Match> {
        self.find("r.romHashMD5 = $key", &md5.to_ascii_uppercase(), systems)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Found,
    Unknown,
    NoArt,
    Failed,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Outcome::Found => "Found",
            Outcome::Unknown => "Unknown",
            Outcome::NoArt => "NoArt",
            Outcome::Failed => "Failed",
        }
    }
}

pub struct Result {
    pub rom: String,
    pub outcome: Outcome,
    pub rom_name: Option<String>,
    pub saved: Option<String>,
    pub detail: Option<String>,
}

/// One lookup: the waits between requests are the real clock's, as `CoverFetcher`'s are.
pub fn look_up(db: &Database, wire: &Wire, rom: &str, core: &Core, artwork: &str, stop: &crate::clock::Stop) -> std::result::Result<Result, Option<String>> {
    let style = Style::HOST;
    let stem = path::file_name_without_extension(style, rom).to_string();
    let mut found = db.by_name(&stem, &core.openvgdb_systems);
    if found.is_none() {
        let size = std::fs::metadata(rom).map(|m| m.len()).map_err(|e| Some(emusen_dotnet::file::io_message(&e, rom)))?;
        if size <= HASH_LIMIT {
            let bytes = std::fs::read(rom).map_err(|e| Some(emusen_dotnet::file::io_message(&e, rom)))?;
            let hashed = core.systems.first().and_then(|s| emusen_systems::openvgdb::bytes(s, &bytes)).unwrap_or(bytes);
            found = db.by_md5(&emusen_galaxia::rom_hash::hashes_of(&hashed).md5, &core.openvgdb_systems);
        }
    }
    let Some(m) = found else { return Ok(Result { rom: rom.into(), outcome: Outcome::Unknown, rom_name: None, saved: None, detail: None }) };
    let target = path::combine_all(style, &[artwork, &core.console, &format!("{}.png", safe(&stem))]);
    if file_exists(&target) {
        return Ok(Result { rom: rom.into(), outcome: Outcome::Found, rom_name: Some(m.rom_name), saved: Some(target), detail: Some("already there".into()) });
    }
    for name in names(&m.rom_name) {
        for system in &core.cheat_systems {
            let url = format!("{THUMBNAIL_SERVER}{}/Named_Boxarts/{}.png", crate::credentials::escape(system), crate::credentials::escape(&safe(&name)));
            if let Some(saved) = fetch(wire, &url, &target, stop)? {
                return Ok(Result { rom: rom.into(), outcome: Outcome::Found, rom_name: Some(m.rom_name), saved: Some(saved), detail: Some(url) });
            }
        }
    }
    if let Some(cover) = m.cover_url.as_deref().filter(|c| c.starts_with("https://") || c.starts_with("http://")) {
        let ext = path::extension(style, cover.split(['?', '#']).next().unwrap_or(cover));
        let fallback = path::change_extension(style, &target, if ext == ".jpg" || ext == ".jpeg" { ".jpg" } else { ".png" });
        if let Some(saved) = fetch(wire, cover, &fallback, stop)? {
            return Ok(Result { rom: rom.into(), outcome: Outcome::Found, rom_name: Some(m.rom_name), saved: Some(saved), detail: Some(cover.to_string()) });
        }
    }
    Ok(Result { rom: rom.into(), outcome: Outcome::NoArt, rom_name: Some(m.rom_name), saved: None, detail: None })
}

/// Only an image of 80 bytes or more is kept, written beside its final name and moved into place, never over a file.
fn fetch(wire: &Wire, url: &str, target: &str, stop: &crate::clock::Stop) -> std::result::Result<Option<String>, Option<String>> {
    if stop.wait(SPACING) {
        return Err(None);
    }
    let request = wire.request(url, TIMEOUT);
    let Some(fetched) = crate::clock::detached(stop, move || emusen_net::fetch_answer(&request, &emusen_net::Progress::default())) else { return Err(None) };
    let answer = fetched.map_err(|f| Some(f.message))?;
    if answer.status != 200 {
        return Ok(None);
    }
    let media = answer.content_type.as_deref().map(|t| t.split(';').next().unwrap_or("").trim().to_string());
    if !media.is_some_and(|t| t.starts_with("image/")) || answer.body.len() < 80 {
        return Ok(None);
    }
    if let Some(dir) = std::path::Path::new(target).parent() {
        std::fs::create_dir_all(dir).map_err(|e| Some(e.to_string()))?;
    }
    let temp = format!("{target}.part");
    std::fs::write(&temp, &answer.body).map_err(|e| Some(e.to_string()))?;
    if file_exists(target) {
        let _ = std::fs::remove_file(&temp);
        return Err(Some(format!("The file '{target}' already exists.")));
    }
    std::fs::rename(&temp, target).map_err(|e| Some(e.to_string()))?;
    Ok(Some(target.to_string()))
}

/// OpenVGDB's newest release from GitHub, as OpenEmu fetches it on first use: the release's tag, the database written
/// beside its final name and moved into place, opened as a database first.
pub fn download(sqlite: Arc<Sqlite>, wire: &Wire, target: &str, cancel: &emusen_net::Progress) -> std::result::Result<String, String> {
    let release = emusen_net::fetch(&wire.request(LATEST_RELEASE, TIMEOUT), cancel).map_err(|f| f.message)?;
    let text = String::from_utf8_lossy(&release).into_owned();
    let doc = emusen_dotnet::json_document::parse(&text)?;
    let tag = doc.get("tag_name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
    let zip = match doc.get("assets") {
        Some(emusen_dotnet::json::Value::Array(assets)) => assets
            .iter()
            .filter(|a| a.get("name").and_then(|n| n.as_str()).is_some_and(|n| emusen_dotnet::text::ends_with_ignore_case(n, ".zip")))
            .find_map(|a| a.get("browser_download_url").and_then(|u| u.as_str()).map(str::to_string)),
        _ => None,
    };
    let Some(zip) = zip else { return Err(format!("OpenVGDB {tag} has no zip to download.")) };
    let archive = emusen_net::fetch(&wire.request(&zip, TIMEOUT), cancel).map_err(|f| f.message)?;
    let part = format!("{target}.part");
    if let Some(dir) = std::path::Path::new(target).parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let zip_part = format!("{target}.zip.part");
    std::fs::write(&zip_part, &archive).map_err(|e| e.to_string())?;
    let named = emusen_galaxia::archive::entries_of(&zip_part).ok().and_then(|entries| {
        entries.into_iter().map(|e| e.full_name).find(|n| emusen_dotnet::text::equals_ignore_case(n.rsplit('/').next().unwrap_or(n), FILE_NAME))
    });
    let entry = named.and_then(|n| emusen_galaxia::archive::read_entry(&zip_part, &n, usize::MAX).ok());
    let _ = std::fs::remove_file(&zip_part);
    let entry = entry.ok_or_else(|| format!("OpenVGDB {tag}'s zip holds no {FILE_NAME}."))?;
    std::fs::write(&part, entry).map_err(|e| e.to_string())?;
    if Database::open(sqlite, &part).is_none() {
        let _ = std::fs::remove_file(&part);
        return Err(format!("OpenVGDB {tag} did not open as a database."));
    }
    std::fs::rename(&part, target).map_err(|e| e.to_string())?;
    Ok(tag)
}

/// Where OpenVGDB is kept: Galaxia's `home/Library`.
pub fn default_path() -> String {
    let library = emusen_galaxia::tree::tree().directory(emusen_galaxia::tree::Directory::Library).unwrap_or_default();
    path::combine(Style::HOST, &library, FILE_NAME)
}

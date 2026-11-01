//! libretro's catalogue: the core information archive and this platform's nightly index, fetched only when the player
//! asks and cached in `/Cores/libretro/.catalogue/` with the date fetched (EmuSen_Cores.md §4.2); and the rows Cores
//! lists, joining it with the installed entries.

use crate::index::{self, IndexLine};
use crate::info::CoreInfo;
use crate::{Context, Source};
use emusen_core_entry::{stem_of_info, Entry, Notice, CATALOGUE, LIBRETRO};
use emusen_galaxia::{archive, atomic, clock};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The largest information archive and index taken; libretro's were 294 KB and about 20 KB on 2026-10-31.
pub const INFO_LIMIT: usize = 16 << 20;
pub const INDEX_LIMIT: usize = 4 << 20;
/// The largest `.info` read out of the archive.
const ENTRY_LIMIT: usize = 64 << 10;

/// The cache's folder under a `/Cores` folder.
pub fn folder(cores: &Path) -> PathBuf {
    cores.join(LIBRETRO).join(CATALOGUE)
}

fn index_name(source: &Source) -> String {
    format!("index-{}", source.platform.replace('/', "-"))
}

/// What was fetched, and when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetched {
    pub cores: usize,
    pub skipped: usize,
    pub date: String,
}

fn get(source: &Source, url: String, limit: usize, stop: &dyn Fn() -> bool) -> Result<Vec<u8>, String> {
    if stop() {
        return Err("stopped".into());
    }
    let mut request = emusen_net::Request::new(url.clone(), Duration::from_secs(120));
    request.same_host = true;
    request.test_roots = source.test_roots.clone();
    request.test_connect = source.test_connect;
    request.offline = source.offline;
    let bytes = emusen_net::fetch(&request, &emusen_net::Progress::default()).map_err(|f| format!("{url}: {}", f.message))?;
    if bytes.len() > limit {
        return Err(format!("{url}: {} bytes, more than the {limit} a catalogue may be", bytes.len()));
    }
    Ok(bytes)
}

/// The archive and the index fetched again, checked, and written in place of the cached ones only when both read.
pub fn fetch(ctx: &Context, stop: &dyn Fn() -> bool) -> Result<Fetched, String> {
    let source = ctx.source.as_ref().ok_or("no buildbot listing is known for this platform")?;
    let info = get(source, source.info_url(), INFO_LIMIT, stop)?;
    let index_bytes = get(source, source.index_url(), INDEX_LIMIT, stop)?;
    let infos = archive::entries_in(&info).map_err(|e| format!("info.zip does not read: {e}"))?;
    if !infos.iter().any(|e| stem_of_info(&e.full_name).is_some()) {
        return Err("info.zip holds no core information".into());
    }
    let text = String::from_utf8(index_bytes).map_err(|_| "the index is not text".to_string())?;
    let (lines, skipped) = index::parse(&text);
    if lines.is_empty() {
        return Err("the index lists no core for this platform".into());
    }
    let dir = folder(&ctx.cores);
    let date = clock::utc_now().chars().take(10).collect::<String>();
    let at = |name: &str| dir.join(name).to_string_lossy().into_owned();
    atomic::write(&at("info.zip"), &info)?;
    atomic::write(&at(&index_name(source)), text.as_bytes())?;
    atomic::write(&at("fetched"), format!("{date}\n").as_bytes())?;
    Ok(Fetched { cores: lines.len(), skipped, date })
}

/// The cached catalogue.
#[derive(Clone, Debug, Default)]
pub struct Catalogue {
    pub infos: Vec<CoreInfo>,
    pub index: Vec<IndexLine>,
    /// The date it was fetched, none where it never was.
    pub fetched: Option<String>,
}

/// The cached catalogue read; empty where nothing was fetched yet.
pub fn load(ctx: &Context) -> Result<Catalogue, String> {
    let dir = folder(&ctx.cores);
    let fetched = std::fs::read_to_string(dir.join("fetched")).ok().map(|t| t.trim().to_string()).filter(|t| !t.is_empty());
    let Some(source) = ctx.source.as_ref() else { return Ok(Catalogue { fetched, ..Catalogue::default() }) };
    let zip = dir.join("info.zip");
    let mut infos = Vec::new();
    if zip.exists() {
        let at = zip.to_string_lossy().into_owned();
        for entry in archive::entries_of(&at)? {
            let Some(stem) = stem_of_info(&entry.full_name) else { continue };
            if entry.length > ENTRY_LIMIT as u64 {
                continue;
            }
            let bytes = archive::read_entry(&at, &entry.full_name, ENTRY_LIMIT)?;
            infos.push(CoreInfo::parse(stem, &String::from_utf8_lossy(&bytes)));
        }
    }
    let index = std::fs::read_to_string(dir.join(index_name(source))).map(|t| index::parse(&t).0).unwrap_or_default();
    Ok(Catalogue { infos, index, fetched })
}

/// Where a row stands (EmuSen_Cores.md §4.2). Approval and refresh are the runtime's trust, joined by the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Installed,
    Update,
    Available,
    Unsupported,
    /// A folder under `/Cores/libretro` with no `NOTICE`: the player's own, never removed by Cores.
    Added,
}

impl State {
    pub fn word(self) -> &'static str {
        match self {
            State::Installed => "installed",
            State::Update => "update",
            State::Available => "available",
            State::Unsupported => "unsupported",
            State::Added => "added",
        }
    }
}

/// One libretro core as Cores lists it.
#[derive(Clone, Debug)]
pub struct Row {
    pub stem: String,
    pub info: Option<CoreInfo>,
    pub index: Option<IndexLine>,
    pub notice: Option<Notice>,
    /// EmuSen's systems that run it, from the placement; empty where none does or none was given.
    pub systems: Vec<String>,
    pub state: State,
    pub reason: Option<String>,
    /// The installed libretro core's size in bytes.
    pub size: Option<u64>,
}

impl Row {
    pub fn name(&self) -> String {
        self.info.as_ref().map(|i| i.name().to_string()).or_else(|| self.notice.as_ref().map(|n| n.name.clone())).unwrap_or_else(|| self.stem.clone())
    }
}

/// Every libretro core the catalogue offers for this platform, and every installed entry, by stem.
pub fn rows(ctx: &Context, catalogue: &Catalogue) -> Result<Vec<Row>, String> {
    let entries = Entry::list(&ctx.cores).map_err(|e| e.to_string())?;
    let mut stems: Vec<String> = catalogue.index.iter().filter(|l| catalogue.infos.iter().any(|i| i.stem == l.stem) && ctx.refuses(&l.stem).is_none()).map(|l| l.stem.clone()).collect();
    stems.extend(entries.iter().map(|e| e.stem.clone()));
    stems.sort();
    stems.dedup();
    let mut rows = Vec::new();
    for stem in stems {
        let entry = entries.iter().find(|e| e.stem == stem);
        let notice = entry.and_then(|e| e.read_notice().ok());
        let installed_info = entry.and_then(|e| std::fs::read_to_string(e.info_path()).ok()).map(|t| CoreInfo::parse(&stem, &t));
        let info = catalogue.infos.iter().find(|i| i.stem == stem).cloned().or(installed_info);
        let index = catalogue.index.iter().find(|l| l.stem == stem).cloned();
        let hosted = ctx.hosted.as_ref().and_then(|list| list.iter().find(|h| h.stem == stem));
        let systems = match (&ctx.placement, &info) {
            _ if hosted.is_some() => hosted.map(|h| h.systems.clone()).unwrap_or_default(),
            (Some(place), Some(info)) => {
                let databases = info.list("database");
                let refs: Vec<&str> = databases.iter().map(String::as_str).collect();
                place(info.get("systemid").unwrap_or(""), &refs)
            }
            _ => Vec::new(),
        };
        let size = entry.and_then(|e| std::fs::metadata(e.core_path()).ok()).map(|m| m.len());
        let (state, reason) = match (entry, &notice) {
            (Some(_), _) if ctx.refuses(&stem).is_some() => (State::Unsupported, ctx.refuses(&stem)),
            (Some(_), None) => (State::Added, None),
            (Some(_), Some(n)) => match &index {
                Some(l) if l.crc32 != n.index_crc32 || l.date != n.build => (State::Update, None),
                _ => (State::Installed, None),
            },
            (None, _) => match info.as_ref().and_then(CoreInfo::unsupported) {
                Some(why) => (State::Unsupported, Some(why)),
                None if ctx.placement.is_some() && systems.is_empty() => (State::Unsupported, Some("no EmuSen console runs this system".into())),
                None => (State::Available, None),
            },
        };
        rows.push(Row { stem, info, index, notice, systems, state, reason, size });
    }
    Ok(rows)
}

//! A game's page's source over the client crates (EmuSen_Qt.md §7.8): the metadata and pictures from the scraper's
//! sources-alone handle, the record and the edits from games.db, the states by Galaxia's save paths, the engines from
//! the runtime, the profiles from Endymion's store, the cheats from Galaxia's cheat file, the crop's and the television
//! standard's offer from Serenity's catalogue, the standard from games.db, the player's screenshots from Galaxia's
//! folder, the package from the library tool's `pkg-ls` and Galaxia's claim index, and its job (`pkg-make` or `pkg-check`)
//! started through the library tool; the crop's choices are Galaxia's Overscan row's. Nothing here needs a session.

use crate::clients::{app_settings, engine_shown, ClientSource};
use crate::display::Crop;
use crate::game_page::{
    cover_words, media_word, Change, Choice, CoverCard, CoverChoice, CropOffer, Detail, GamePageSource, Media, MediaInfo, MediaKind, Package, PackageJob, Read, Slot, State, ALL_DETAILS, CHECK,
    DETAILS, MAKE,
};
use crate::library::{console_name, title_of, HERO};
use emusen_crystal_client::Scope;
use emusen_platform_client::bindings::consts::PROFILES;
use emusen_platform_client::galaxia::consts::{
    DIR_SCREENSHOTS, SAVE_PACKAGE_RESUME_STATE, SAVE_PACKAGE_STATE, SAVE_PICTURE, SAVE_RESUME_STATE, SAVE_STATE,
};
use emusen_platform_client::library::GameFile;
use emusen_platform_client::packages::Roots;
use emusen_platform_client::serenity;
use emusen_platform_client::tools::{Step, ToolJob};
use serde_json::{json, Value};
use std::path::Path;

fn words(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// The media store's types the Media section shows, in Mistress's order, the PDF manual last.
const MEDIA: [(&str, MediaKind); 6] = [
    ("screenshot", MediaKind::Screenshot),
    ("titlescreen", MediaKind::TitleScreen),
    ("video", MediaKind::Clip),
    ("marquee", MediaKind::Marquee),
    ("fanart", MediaKind::Fanart),
    ("manual", MediaKind::Manual),
];

/// The words for a kind of picture, by its media store type and folder: the Media section's for a kind it shows, else the
/// label of the Scraping settings' switch for the kind in Galaxia's rows (`scrape_run::kind_switch`), else its type.
fn kind_words(rows: &Value, (kind, folder): (&str, &str)) -> String {
    if let Some((_, shown)) = MEDIA.iter().find(|(t, _)| *t == kind) {
        return media_word(*shown).to_owned();
    }
    let switch = crate::scrape_run::kind_switch(folder);
    let all = rows.as_object().into_iter().flat_map(|m| m.values()).flat_map(|list| list.as_array().into_iter().flatten());
    let row = all.into_iter().find(|r| r["key"].as_str().is_some_and(|k| k.eq_ignore_ascii_case(&switch)));
    row.and_then(|r| r["label"].as_str()).map_or_else(|| kind.to_owned(), str::to_owned)
}

/// A file's modified time in seconds since 1970; none for no file.
fn modified(path: &Path) -> Option<i64> {
    let at = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(at.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64))
}

/// Strips `//` and `/* */` comments and trailing commas outside string literals so JSONC package manifests parse cleanly.
fn strip_jsonc(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut in_str = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            out.push(b);
            if b == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1]);
                i += 2;
                continue;
            }
            if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_str = true;
            out.push(b);
            i += 1;
        } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
        } else if b == b',' {
            let mut j = i + 1;
            loop {
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j + 1 < bytes.len() && bytes[j] == b'/' && bytes[j + 1] == b'/' {
                    j += 2;
                    while j < bytes.len() && bytes[j] != b'\n' {
                        j += 1;
                    }
                    continue;
                }
                if j + 1 < bytes.len() && bytes[j] == b'/' && bytes[j + 1] == b'*' {
                    j += 2;
                    while j + 1 < bytes.len() && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
                        j += 1;
                    }
                    j = (j + 2).min(bytes.len());
                    continue;
                }
                break;
            }
            if j < bytes.len() && matches!(bytes[j], b']' | b'}') {
                i += 1;
            } else {
                out.push(b);
                i += 1;
            }
        } else {
            out.push(b);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| raw.to_owned())
}

/// Computes lowercase SHA-1 hex digest in pure Rust for package save-key derivation.
fn sha1_hex(data: &[u8]) -> String {
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    let [mut h0, mut h1, mut h2, mut h3, mut h4] = [0x6745_2301u32, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476, 0xC3D2_E1F0];
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }
    format!("{h0:08x}{h1:08x}{h2:08x}{h3:08x}{h4:08x}")
}

fn decode_text(bytes: &[u8]) -> String {
    match bytes {
        [0xFE, 0xFF, rest @ ..] => char::decode_utf16(rest.chunks_exact(2).map(|b| u16::from_be_bytes([b[0], b[1]])))
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect(),
        [0xFF, 0xFE, 0, 0, rest @ ..] => rest
            .chunks_exact(4)
            .map(|b| char::from_u32(u32::from_le_bytes([b[0], b[1], b[2], b[3]])).unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect(),
        [0xFF, 0xFE, rest @ ..] => char::decode_utf16(rest.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])))
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect(),
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        [0, 0, 0xFE, 0xFF, rest @ ..] => rest
            .chunks_exact(4)
            .map(|b| char::from_u32(u32::from_be_bytes([b[0], b[1], b[2], b[3]])).unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect(),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

fn valid_package_version(doc: &Value) -> bool {
    let Some(v) = doc.get("version").and_then(Value::as_str) else { return false };
    let Some((major, minor)) = v.split_once('.') else { return false };
    major == "1" && !minor.is_empty() && minor.bytes().all(|b| b.is_ascii_digit())
}

fn valid_hex_opt(v: Option<&Value>, digits: usize) -> bool {
    match v {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) => s.len() == digits && s.bytes().all(|b| b.is_ascii_hexdigit()),
        _ => false,
    }
}

fn valid_patch_entry(entry: &Value) -> bool {
    let Some(obj) = entry.as_object() else { return false };
    let has_file = obj.get("file").and_then(Value::as_str).is_some_and(|s| !s.is_empty());
    let has_fmt = obj.get("format").and_then(Value::as_str).is_some_and(|f| matches!(f, "ips" | "bps" | "ups"));
    has_file && has_fmt && valid_hex_opt(obj.get("sha1"), 40)
}

/// Derives Galaxia's 8-hex-digit save key from `package.json`'s `patches` array (matching `emusen_galaxia::package::save_key_at`).
pub fn package_save_key(folder: &Path) -> Option<String> {
    let bytes = std::fs::read(folder.join("package.json")).ok()?;
    let raw = decode_text(&bytes);
    let doc: Value = serde_json::from_str(&strip_jsonc(&raw)).ok()?;
    let valid_mode = doc.get("mode").and_then(Value::as_str).is_some_and(|m| matches!(m, "linked" | "self-contained"));
    let valid_sys = doc.get("system").and_then(Value::as_str).is_some_and(|s| !s.is_empty());
    if !valid_package_version(&doc) || !valid_sys || !valid_mode || !doc.get("game").is_some_and(Value::is_object) {
        return None;
    }
    let entries = doc.get("patches").and_then(Value::as_array).filter(|a| !a.is_empty())?;
    let mut lines = String::new();
    for entry in entries {
        if !valid_patch_entry(entry) {
            return None;
        }
        let fmt = entry.get("format").and_then(Value::as_str)?;
        let id = entry.get("sha1").and_then(Value::as_str).or_else(|| entry.get("file").and_then(Value::as_str))?;
        lines.push_str(fmt);
        lines.push(' ');
        lines.push_str(&id.to_ascii_lowercase());
        lines.push('\n');
    }
    Some(sha1_hex(lines.as_bytes())[..8].to_owned())
}

/// Reads PNG, GIF, BMP, WebP or JPEG pixel dimensions from the first 4 KiB without decoding whole images.
fn image_dimensions(path: &Path) -> (u32, u32) {
    use std::io::Read;
    let Ok(file) = std::fs::File::open(path) else { return (0, 0) };
    let mut bytes = Vec::with_capacity(4096);
    if file.take(4096).read_to_end(&mut bytes).is_err() {
        return (0, 0);
    }
    if bytes.len() >= 24 && &bytes[..8] == b"\x89PNG\r\n\x1a\n" {
        return (u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]), u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]));
    }
    if bytes.len() >= 10 && (&bytes[..6] == b"GIF87a" || &bytes[..6] == b"GIF89a") {
        return (u16::from_le_bytes([bytes[6], bytes[7]]) as u32, u16::from_le_bytes([bytes[8], bytes[9]]) as u32);
    }
    if bytes.len() >= 26 && &bytes[..2] == b"BM" {
        let w = i32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]).unsigned_abs();
        let h = i32::from_le_bytes([bytes[22], bytes[23], bytes[24], bytes[25]]).unsigned_abs();
        return (w, h);
    }
    if bytes.len() >= 30 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        if &bytes[12..16] == b"VP8X" {
            let w = 1 + (bytes[24] as u32 | ((bytes[25] as u32) << 8) | ((bytes[26] as u32) << 16));
            let h = 1 + (bytes[27] as u32 | ((bytes[28] as u32) << 8) | ((bytes[29] as u32) << 16));
            return (w, h);
        }
        if &bytes[12..16] == b"VP8 " && bytes[23..26] == [0x9d, 0x01, 0x2a] {
            let w = (u16::from_le_bytes([bytes[26], bytes[27]]) & 0x3fff) as u32;
            let h = (u16::from_le_bytes([bytes[28], bytes[29]]) & 0x3fff) as u32;
            return (w, h);
        }
        if &bytes[12..16] == b"VP8L" && bytes[20] == 0x2f {
            let bits = u32::from_le_bytes([bytes[21], bytes[22], bytes[23], bytes[24]]);
            return (1 + (bits & 0x3fff), 1 + ((bits >> 14) & 0x3fff));
        }
    }
    if bytes.len() >= 4 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        let mut i = 2;
        while i + 9 < bytes.len() {
            if bytes[i] != 0xFF {
                break;
            }
            let marker = bytes[i + 1];
            if marker == 0xD9 || marker == 0xDA {
                break;
            }
            let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
            if matches!(marker, 0xC0..=0xC3) {
                let h = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]) as u32;
                let w = u16::from_be_bytes([bytes[i + 7], bytes[i + 8]]) as u32;
                return (w, h);
            }
            if len < 2 {
                break;
            }
            i += 2 + len;
        }
    }
    (0, 0)
}

/// Resolves the console display name of a borrowed cover's source game from the systems table.
fn borrowed_console_name(systems: &[emusen_crystal_client::shapes::SystemRow], from: &str, picture: &str) -> Option<String> {
    let pic = Path::new(picture);
    if let Some(sys) = pic.parent().and_then(Path::parent).and_then(Path::file_name).and_then(|n| n.to_str()) {
        if let Some(s) = systems.iter().find(|s| s.esde_system.eq_ignore_ascii_case(sys)) {
            return Some(console_name(s));
        }
    }
    let from_path = Path::new(from);
    if let Some(folder) = from_path.parent().and_then(Path::file_name).and_then(|n| n.to_str()) {
        if let Some(s) = systems.iter().find(|s| s.id.eq_ignore_ascii_case(folder) || s.esde_system.eq_ignore_ascii_case(folder) || s.console.eq_ignore_ascii_case(folder)) {
            return Some(console_name(s));
        }
    }
    None
}

impl ClientSource {
    /// Looks up a console display name by its ScreenScraper system id from the runtime's systems table.
    pub fn console_by_screenscraper(&self, id: u32) -> Option<String> {
        let systems = self.runtime.systems().ok()?;
        systems.iter().find(|s| s.screenscraper_system == Some(id)).map(console_name)
    }
    fn media_file(&self, game: &GameFile, esde_system: &str, kind: &str) -> String {
        let Some(scraper) = self.scraper() else { return String::new() };
        let found = scraper.get(&json!({ "what": "media", "path": game.path, "system": esde_system, "type": kind }));
        found.ok().and_then(|f| f.get("path").and_then(Value::as_str).map(str::to_owned)).unwrap_or_default()
    }

    /// Cover art's cards, the one in use, and why there are none where the scraper that finds them could not be opened: a
    /// cover borrowed from another game while it is shown, the cover found where one is, then each kind of picture the
    /// scraper's `kinds` names (besides the cover's own) that the game has a file of; what is in use as `media` answers
    /// the cover. A game on no console the systems table knows has no pictures looked for, so no cards.
    fn covers(&self, game: &GameFile, esde_system: &str) -> Result<(Vec<CoverCard>, CoverChoice, Option<String>), String> {
        let Some(scraper) = self.scraper() else { return Ok((Vec::new(), CoverChoice::Found, Some(crate::clients::NO_SCRAPER.into()))) };
        if esde_system.is_empty() {
            return Ok((Vec::new(), CoverChoice::Found, None));
        }
        let read = |choices: bool| scraper.get(&json!({ "what": "media", "path": game.path, "system": esde_system, "type": crate::library::COVER, "choices": choices }));
        let (shown, found) = (read(true).map_err(words)?, read(false).map_err(words)?);
        let path = |v: &Value| v["path"].as_str().filter(|p| !p.is_empty()).map(str::to_owned);
        let in_use = match (shown["from"].as_str(), shown["type"].as_str()) {
            (Some(from), _) => CoverChoice::Borrowed(from.to_owned()),
            (None, Some(t)) if t != crate::library::COVER => CoverChoice::Own(t.to_owned()),
            _ => CoverChoice::Found,
        };
        let card = |choice: CoverChoice, text: String, picture: String| CoverCard { text, choice, picture };
        let mut cards = Vec::new();
        if let (CoverChoice::Borrowed(_), Some(picture)) = (&in_use, path(&shown)) {
            cards.push(card(in_use.clone(), cover_words(&in_use), picture));
        }
        if let Some(picture) = path(&found) {
            cards.push(card(CoverChoice::Found, cover_words(&CoverChoice::Found), picture));
        }
        let kinds = scraper.get(&json!({ "what": "kinds" })).map_err(words)?;
        let rows = self.platform.settings_rows().map_err(words)?;
        let pictures = kinds.as_array().into_iter().flatten().filter(|k| k["payload"] == "image").filter(|k| k["type"] != crate::library::COVER);
        for kind in pictures.filter_map(|k| Some((k["type"].as_str()?, k["folder"].as_str()?))) {
            let file = self.media_file(game, esde_system, kind.0);
            if !file.is_empty() {
                cards.push(card(CoverChoice::Own(kind.0.to_owned()), kind_words(&rows, kind), file));
            }
        }
        Ok((cards, in_use, None))
    }

    /// The states, slot by slot and then the automatic one, each where its file is, the most recent first.
    pub(crate) fn states(&self, game: &GameFile, settings: &Value) -> Result<Vec<State>, String> {
        let directory = settings.get("StateDirectory").and_then(Value::as_str).filter(|d| !d.trim().is_empty());
        let path_of = |slot: Slot| {
            let (n, slot_no) = match slot {
                Slot::Number(n) => (true, n as i32),
                Slot::Auto => (false, 0),
            };
            match &game.package {
                Some(folder) => {
                    let save_key = package_save_key(Path::new(folder));
                    self.platform.save_path(if n { SAVE_PACKAGE_STATE } else { SAVE_PACKAGE_RESUME_STATE }, folder, save_key.as_deref(), slot_no)
                }
                None => self.platform.save_path(if n { SAVE_STATE } else { SAVE_RESUME_STATE }, &game.path, directory, slot_no),
            }
        };
        let slots = (1..=crate::pause::SLOTS).map(Slot::Number).chain([Slot::Auto]);
        let mut out = Vec::new();
        for slot in slots {
            let file = path_of(slot).map_err(words)?;
            let Some(when) = modified(&file) else { continue };
            let file = file.to_string_lossy().into_owned();
            let picture = self.platform.save_path(SAVE_PICTURE, &file, None, 0).map_err(words)?;
            let picture = if picture.exists() { picture.to_string_lossy().into_owned() } else { String::new() };
            out.push(State { path: file, slot, picture, when });
        }
        out.sort_by_key(|s| std::cmp::Reverse(s.when));
        Ok(out)
    }

    /// The player's own screenshots of the game, the newest first.
    fn screenshots(&self, game: &GameFile) -> Result<Vec<Media>, String> {
        let Some(folder) = self.platform.directory(DIR_SCREENSHOTS).map_err(words)? else { return Ok(Vec::new()) };
        Ok(crate::screenshots::of_game(&folder, &game.path).into_iter().map(|(path, _)| Media { path, kind: MediaKind::Taken }).collect())
    }

    /// The game's package and its layers: the package the scan found the game in, else one that claims the game but
    /// refused it, its file changed (Mistress's `GamePackageDetail`, `Refused`); each read by the library tool's `pkg-ls`.
    fn package(&self, game: &GameFile) -> Result<(Package, usize, Option<PackageJob>), String> {
        let (folder, refused) = match &game.package {
            Some(folder) => (folder.clone(), false),
            None => {
                let scan = self.scan_request()?;
                let library = scan.roots.first().cloned();
                // A game the library folder holds is made a package by its path there (the library tool's `pkg-make`).
                let make = library.as_deref().and_then(|root| std::path::Path::new(&game.path).strip_prefix(root).ok()).map(|rel| PackageJob {
                    key: MAKE,
                    request: json!({ "op": "pkg-make", "paths": [rel.to_string_lossy()] }),
                });
                let (Some(packages), Some(compressed)) = (scan.packages.as_deref(), scan.compressed.as_deref()) else { return Ok((Package::None, 0, make)) };
                let roots = Roots { packages, library: library.as_deref(), compressed };
                let index = self.platform.package_index(roots).map_err(words)?;
                let claims = |p: &&Value| p["claims"].as_array().is_some_and(|c| c.iter().any(|c| c.as_str() == Some(game.path.as_str())));
                match index["packages"].as_array().and_then(|all| all.iter().find(claims)).and_then(|p| p["folder"].as_str()) {
                    Some(folder) => (folder.to_owned(), true),
                    None => return Ok((Package::None, 0, make)),
                }
            }
        };
        let check = PackageJob { key: CHECK, request: json!({ "op": "pkg-check", "paths": [folder] }) };
        let job = ToolJob::start(&self.platform, &json!({ "op": "pkg-ls", "paths": [folder] })).map_err(words)?;
        let mut listed = Value::Null;
        loop {
            match job.next(1000).map_err(words)? {
                Step::Event(e) if e["event"] == "package" => listed = e,
                Step::Event(_) | Step::Waiting => {}
                Step::Ended => break,
            }
        }
        let layers = listed["patches"].as_array().map_or(0, Vec::len);
        // A package made since the scan claims the game too: it is the game's while pkg-ls finds the game as it recorded.
        let kind = match (refused && listed["state"] != "ok", listed["mode"].as_str()) {
            (true, _) => Package::Changed,
            (false, Some("self-contained")) => Package::SelfContained,
            (false, _) => Package::Linked,
        };
        Ok((kind, layers, Some(check)))
    }

    /// The game's crop as Display crop offers it: Galaxia's Overscan row's choices that hide a fixed part, each as
    /// Serenity's catalogue sizes it, and the game's own crop from games.db.
    fn crop(&self, game: &GameFile, catalogue: &Value) -> Result<CropOffer, String> {
        let rows = self.platform.settings_rows().map_err(words)?;
        let row = rows["GraphicsConfig.Consoles"].as_array().and_then(|all| all.iter().find(|r| r["key"] == "Overscan")).cloned().unwrap_or(Value::Null);
        let texts = |k: &str| row[k].as_array().into_iter().flatten().map(|v| v.as_str().unwrap_or_default().to_owned()).collect::<Vec<_>>();
        let label = row["label"].as_str().unwrap_or_default().to_owned();
        let mut offer = CropOffer { label, most: crate::display::most(catalogue), ..CropOffer::default() };
        for (value, label) in texts("choices").into_iter().zip(texts("choice_labels")) {
            match value.as_str() {
                crate::display::CUSTOM => offer.custom = label,
                _ => offer.choices.push((label, crate::display::overscan(catalogue, &value, &|_| String::new()))),
            }
        }
        offer.own = Crop::from_record(&self.library.get(&json!({ "what": "crop", "path": game.path })).map_err(words)?);
        Ok(offer)
    }

    /// Whether a scrape run is going, by the scrape service's progress.
    fn scraping(&self) -> bool {
        self.scraper().and_then(|s| s.progress(u64::MAX).ok()).is_some_and(|p| p["state"] == "running")
    }

    /// The console's profiles where it has more than one, the console's own first, and the game's choice.
    fn profiles(&self, console: &str, path: &str) -> (Option<Vec<Choice>>, Option<String>) {
        let Ok(model) = self.platform.bindings_load(PROFILES, &[console]) else { return (None, None) };
        let of = &model[console];
        let names: Vec<String> = of["Profiles"].as_object().map(|p| p.keys().cloned().collect()).unwrap_or_default();
        let chosen = of["Games"][title_of(path)].as_str().map(str::to_owned);
        if names.len() < 2 {
            return (None, chosen);
        }
        let default = of["Default"].as_str().unwrap_or("Default");
        let mut choices = vec![Choice { value: None, text: format!("Console's ({default})") }];
        choices.extend(names.into_iter().map(|n| Choice { value: Some(n.clone()), text: n }));
        (Some(choices), chosen)
    }
}

impl GamePageSource for ClientSource {
    fn read(&self, game: &GameFile) -> Result<Read, String> {
        let systems = self.runtime.systems().map_err(words)?;
        let system = systems.iter().find(|s| Some(&s.id) == game.system.as_ref());
        let settings = app_settings(&self.platform)?;
        let names = settings.get("ScrapeGameNames").and_then(Value::as_bool).unwrap_or(false);
        let metadata = match self.scraper() {
            Some(s) => s.get(&json!({ "what": "metadata", "path": game.path, "scraped_names": names })).map_err(words)?,
            None => Value::Null,
        };
        let value = |key: &str| metadata.get(key).and_then(|v| v.get("value")).and_then(Value::as_str).unwrap_or("").to_owned();
        let record = self.library.record(&game.path).map_err(words)?.unwrap_or_default();
        let esde = system.map(|s| s.esde_system.clone()).unwrap_or_default();
        let media = |kind: &str| if esde.is_empty() { String::new() } else { self.media_file(game, &esde, kind) };
        let mut read = Read {
            title: Some(value("name")).filter(|n| !n.is_empty()).unwrap_or_else(|| title_of(&game.path)),
            console: system.map(console_name).unwrap_or_default(),
            year: crate::esde_shelves::year_of(&value(crate::esde_shelves::RELEASE_DATE)).unwrap_or_default(),
            developer: value("developer"),
            publisher: value("publisher"),
            description: value("description"),
            played_seconds: record.play_seconds as i64,
            last_played: record.last_played.as_ref().and_then(Value::as_str).and_then(crate::words::parse_utc),
            favourite: record.favourite,
            cover_aspect: crate::library::cover_shape(system),
            cover: media(crate::library::COVER),
            hero: HERO.iter().map(|k| media(k)).find(|f| !f.is_empty()).unwrap_or_default(),
            screenshot: media("screenshot"),
            // Settings ▸ Display ▸ Play videos off shows the still alone (EmuSen_Interface.md §5.9).
            clip: if settings.get("PlayVideos").and_then(Value::as_bool).unwrap_or(true) { media("video") } else { String::new() },
            clip_volume: crate::esde_shelves::volume_of(&settings),
            states: self.states(game, &settings)?,
            ..Read::default()
        };
        let edited = |key: &str| metadata.get(key).and_then(|v| v.get("source")).and_then(Value::as_str) == Some("edited");
        read.details = DETAILS.iter().map(|(field, _)| Detail { field, value: value(field), edited: edited(field) }).collect();
        read.all_details = ALL_DETAILS.iter().map(|(field, _, _)| Detail { field, value: value(field), edited: edited(field) }).collect();
        read.esde_system = esde.clone();
        read.media = self.screenshots(game)?;
        read.media.extend(MEDIA.iter().filter_map(|(kind, what)| Some(media(kind)).filter(|f| !f.is_empty()).map(|path| Media { path, kind: *what })));
        read.media_info = read
            .media
            .iter()
            .map(|m| {
                let p = Path::new(&m.path);
                let (width, height) = if matches!(m.kind, MediaKind::Clip | MediaKind::Manual) { (0, 0) } else { image_dimensions(p) };
                MediaInfo { when: modified(p), width, height }
            })
            .collect();
        read.engine = Some(value(crate::edits::ALT_EMULATOR)).filter(|e| !e.is_empty());
        (read.package, read.layers, read.package_job) = self.package(game)?;
        read.scraping = self.scraping();
        (read.covers, read.cover_in_use, read.covers_unread) = self.covers(game, &esde)?;
        if let CoverChoice::Borrowed(from) = &read.cover_in_use {
            let pic = read.covers.iter().find(|c| matches!(c.choice, CoverChoice::Borrowed(_))).map_or("", |c| c.picture.as_str());
            read.borrowed_console = borrowed_console_name(&systems, from, pic);
        }
        let Some(s) = system else { return Ok(read) };
        // The engines: the console's own first, then each it may run on (Mistress's IGamePageHost.Engines).
        let cores = self.runtime.cores().map_err(words)?;
        let console = crate::consoles::settings_console(s);
        let schema_engines: Vec<String> = self
            .runtime
            .settings_schema(0, Some(console), None)
            .unwrap_or_default()
            .into_iter()
            .find(|r| r.key == crate::consoles::ENGINE_KEY)
            .and_then(|r| r.choices)
            .unwrap_or_default();
        let engines = if schema_engines.len() > s.engines.len() { &schema_engines } else { &s.engines };
        let stored = self.runtime.settings_get(0, Scope::Console, Some(console), crate::consoles::ENGINE_KEY).map_err(words)?;
        let own = stored.filter(|e| !e.is_empty()).or_else(|| engines.first().cloned());
        let name = |id: &str| engine_shown(id, &cores).name;
        read.engines = match (&own, engines.len()) {
            (None, _) => Vec::new(),
            (Some(own), 0 | 1) => vec![Choice { value: None, text: format!("{} (the only one)", name(own)) }],
            (Some(own), _) => std::iter::once(Choice { value: None, text: format!("{} (default)", name(own)) })
                .chain(engines.iter().map(|e| Choice {
                    value: Some(e.clone()),
                    text: if e == own { format!("{} (the console's)", name(e)) } else { name(e) },
                }))
                .collect(),
        };
        (read.profiles, read.profile) = self.profiles(&s.console, &game.path);
        read.cheats = crate::cheats::List::of(&self.platform, &s.console, &game.path)?.map(|l| l.switches()).unwrap_or_default();
        let catalogue = serenity::catalogue(&self.platform).map_err(words)?;
        if crate::display::television(&catalogue, &s.console) {
            read.crop = Some(self.crop(game, &catalogue)?);
            // The standard is offered for a console that drove a television, as every filter of a tube is (Mistress's `StandardOffered`).
            let standard = self.library.get(&json!({ "what": "standard", "path": game.path })).map_err(words)?;
            let values: Vec<String> = standard["values"].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect();
            read.standards = Some(crate::game_page::standard_choices(&values));
            read.standard = standard["value"].as_str().map(str::to_owned);
        }
        Ok(read)
    }

    fn start_job(&self, request: &Value) -> Result<Box<dyn crate::tools::cores::backend::Job + Send>, String> {
        crate::tools::cores::backend::sent_job(&self.platform, request)
    }

    fn other_copy(&self, game: &GameFile, state: &str) -> Result<Option<String>, String> {
        // A record the runtime cannot read is not known, as Mistress's `StateRecordsOnApi.Read` answers it.
        let Ok(Some(record)) = self.runtime.state_record(state) else { return Ok(None) };
        let Some(saved) = record.rom_md5.filter(|m| !m.is_empty()) else { return Ok(None) };
        let Ok(identity) = self.library.identity(&json!({ "path": game.path })) else { return Ok(None) };
        let same = identity["md5"].as_str().is_none_or(|md5| md5.eq_ignore_ascii_case(&saved));
        Ok((!same).then(|| record.rom_file.unwrap_or_default()))
    }

    fn change(&self, game: &GameFile, change: &Change) -> Result<(), String> {
        let path = game.path.as_str();
        match change {
            Change::Favourite => self.library.change(&json!({ "op": "toggle_favourite", "path": path })).map(drop).map_err(words),
            Change::Engine(engine) => {
                let at = crate::words::utc(crate::words::now());
                self.library.change(&json!({ "op": "save_edits", "path": path, "changes": { crate::edits::ALT_EMULATOR: engine }, "at": at })).map(drop).map_err(words)
            }
            Change::Standard(standard) => {
                let at = crate::words::utc(crate::words::now());
                self.library.change(&json!({ "op": "set_standard", "path": path, "standard": standard, "at": at })).map(drop).map_err(words)
            }
            Change::DeleteState(state) => {
                // The state, its picture by Galaxia's name for it, and the runtime's record of it (Mistress's DeleteState).
                let picture = self.platform.save_path(SAVE_PICTURE, state, None, 0).map_err(words)?;
                std::fs::remove_file(state).map_err(words)?;
                if picture.exists() {
                    std::fs::remove_file(&picture).map_err(words)?;
                }
                self.runtime.state_forget(state).map_err(words)
            }
            Change::Edit(field, value) => {
                let at = crate::words::utc(crate::words::now());
                self.library.change(&json!({ "op": "save_edits", "path": path, "changes": { *field: value }, "at": at })).map(drop).map_err(words)
            }
            Change::Cheat(index, on) => {
                let systems = self.runtime.systems().map_err(words)?;
                let console = systems.iter().find(|s| Some(&s.id) == game.system.as_ref()).map(|s| s.console.clone()).unwrap_or_default();
                let mut list = crate::cheats::List::of(&self.platform, &console, path)?.ok_or_else(|| "the game has no cheat list".to_string())?;
                if !list.set(*index, *on) {
                    return Err(format!("no cheat {index}"));
                }
                list.save(&self.platform)
            }
            Change::Crop(Some(crop)) => self.library.change(&crop.set_record(path)).map(drop).map_err(words),
            Change::Crop(None) => self.library.change(&json!({ "op": "clear_crop", "path": path })).map(drop).map_err(words),
            Change::Cover(kind) => {
                let scraper = self.scraper().ok_or(crate::clients::NO_SCRAPER)?;
                let systems = self.runtime.systems().map_err(words)?;
                let system = systems.iter().find(|s| Some(&s.id) == game.system.as_ref()).ok_or("the game's console is not known")?;
                let at = crate::words::utc(crate::words::now());
                scraper.change(&json!({ "op": "cover", "path": path, "system": system.esde_system, "type": kind, "at": at })).map(drop).map_err(words)
            }
            Change::Hide => {
                let at = crate::words::utc(crate::words::now());
                self.library.change(&json!({ "op": "save_edits", "path": path, "changes": { crate::edits::HIDDEN: crate::edits::YES }, "at": at })).map(drop).map_err(words)
            }
            Change::ClearScraped => {
                let systems = self.runtime.systems().map_err(words)?;
                let system = systems.iter().find(|s| Some(&s.id) == game.system.as_ref()).ok_or("the game's console is not known")?;
                self.scrape_forget(path, &system.esde_system)
            }
            Change::DeleteMedia(file) => std::fs::remove_file(file).map_err(words),
            Change::Profile(profile) => {
                let systems = self.runtime.systems().map_err(words)?;
                let Some(s) = systems.iter().find(|s| Some(&s.id) == game.system.as_ref()) else { return Ok(()) };
                let mut model = self.platform.bindings_load(PROFILES, &[s.console.as_str()]).map_err(words)?;
                let games = &mut model[s.console.as_str()]["Games"];
                if !games.is_object() {
                    *games = json!({});
                }
                let key = title_of(path);
                if let Some(p) = profile {
                    games[&key] = json!(p);
                } else if let Some(m) = games.as_object_mut() {
                    m.remove(&key);
                }
                self.platform.bindings_save(PROFILES, &model).map_err(words)
            }
        }
    }
}

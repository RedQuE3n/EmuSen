//! ScreenScraper's API v2 over the platform's one HTTP client, `ScreenScraperClient` ported: `jeuInfos` by hashes, size
//! and name, `jeuRecherche`, `ssuserInfos`, and the media an answer names, each request as the C# built it and each
//! answer read as it read it. Every request goes through `emusen-net` (SR9), with the program's `User-Agent` and the
//! client's 30 s. See EmuSen_BigPicture.md §5.1, §17.3, §38 and EmuSen_Backend.md §30.5.

use crate::answer::{self, Game, Member, Quota};
use crate::credentials::{self, Developer};
use crate::kinds::Payload;
use crate::quota::Status;
use crate::redact::redact;
use crate::{Settings, Test};
use emusen_dotnet::date::TICKS_PER_SECOND;
use emusen_net::{Progress, Request};
use std::io::Write;
use std::time::{Duration, Instant};

pub const API: &str = "https://api.screenscraper.fr/api2/";
/// ES-DE's threshold: anything smaller is an error page, not a picture.
pub const SMALLEST_IMAGE: u64 = 80;
/// The largest file kept.
pub const LARGEST_FILE: u64 = 128 << 20;
/// Mistress's `HttpClient.Timeout`.
pub const TIMEOUT: Duration = Duration::from_secs(30);

/// What the client sends with every request, and how: the program's name, and the seams of a test.
#[derive(Clone, Debug, Default)]
pub struct Wire {
    pub user_agent: Option<String>,
    pub test: Test,
}

impl Wire {
    pub fn of(settings: &Settings) -> Wire {
        Wire { user_agent: settings.user_agent.clone(), test: settings.test.clone() }
    }

    pub fn request(&self, url: &str, timeout: Duration) -> Request {
        let mut r = Request::new(url, timeout);
        if let Some(agent) = &self.user_agent {
            r.headers.push(("User-Agent".into(), agent.clone()));
        }
        r.test_connect = self.test.connect;
        r.offline = self.test.offline;
        if let Some(pem) = &self.test.roots {
            r.test_roots = emusen_net::pem_certificates(pem).unwrap_or_default();
        }
        r
    }

    /// `GetAsync` then `ReadAsStringAsync`: the status, the body and how long it took, or the failure's words.
    pub fn get_text(&self, url: &str, cancel: &Progress) -> (u16, Result<String, String>, Duration) {
        let started = Instant::now();
        match emusen_net::fetch_answer(&self.request(url, TIMEOUT), cancel) {
            Ok(answer) => match emusen_net::text(answer.content_type.as_deref(), &answer.body) {
                Ok(text) => (answer.status, Ok(text), started.elapsed()),
                Err(f) => (0, Err(f.message), started.elapsed()),
            },
            Err(f) => (0, Err(f.message), started.elapsed()),
        }
    }
}

/// The start of a body, for the status line: never more than a line of 160 characters, and never a credential.
pub fn snippet(body: &str) -> String {
    let line = body.split('\n').next().unwrap_or("").trim_matches(char::is_whitespace);
    let cut: String = if emusen_dotnet::text::utf16_len(line) > 160 { take_utf16(line, 160) } else { line.to_string() };
    redact(&cut)
}

fn take_utf16(text: &str, units: usize) -> String {
    let mut out = String::new();
    let mut n = 0;
    for c in text.chars() {
        let w = c.len_utf16();
        if n + w > units {
            break;
        }
        out.push(c);
        n += w;
    }
    out
}

pub struct JeuInfos {
    pub status: Status,
    pub game: Option<Game>,
    pub quota: Option<Quota>,
    pub detail: String,
    pub body: Option<String>,
}

pub struct Search {
    pub status: Status,
    pub games: Vec<Game>,
    pub quota: Option<Quota>,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaOutcome {
    Saved,
    AlreadyThere,
    NoMedia,
    NotAnImage,
    TooSmall,
    Failed,
    Unchanged,
    TooLarge,
}

pub struct MediaAnswer {
    pub outcome: MediaOutcome,
    pub path: Option<String>,
    pub bytes: u64,
    pub sha1: Option<String>,
    pub detail: String,
    pub took: Duration,
}

/// The three hashes `jeuInfos` asks for, with the size.
pub struct Hashes {
    pub md5: String,
    pub crc32: String,
    pub sha1: String,
    pub size: u64,
}

pub struct Client {
    pub wire: Wire,
    pub developer: Developer,
    /// Read at every request, so signing out mid-run sends no `ssid` or `sspassword` from the next one.
    pub member: std::sync::RwLock<Option<credentials::Member>>,
}

impl Client {
    fn append(url: &mut String, name: &str, value: &str) {
        url.push_str(name);
        url.push('=');
        url.push_str(&credentials::escape(value));
        url.push('&');
    }

    /// The credentials first, as every call carries them, then `output=json` and the query.
    pub fn url(&self, endpoint: &str, query: &[(&str, &str)]) -> String {
        let mut url = format!("{API}{endpoint}?");
        Client::append(&mut url, "devid", &self.developer.id);
        Client::append(&mut url, "devpassword", &self.developer.password);
        Client::append(&mut url, "softname", &self.developer.soft_name);
        Client::append(&mut url, "output", "json");
        if let Some(m) = self.member.read().unwrap_or_else(|p| p.into_inner()).as_ref().filter(|m| m.is_set()) {
            Client::append(&mut url, "ssid", &m.user);
            Client::append(&mut url, "sspassword", &m.password);
        }
        for (name, value) in query {
            Client::append(&mut url, name, value);
        }
        url.pop();
        url
    }

    pub fn jeu_infos_url(&self, system_id: i32, hashes: &Hashes, file_name: &str, game_id: Option<i64>) -> String {
        let system = system_id.to_string();
        let size = hashes.size.to_string();
        let name = emusen_galaxia::dotnet_path::file_name(emusen_galaxia::dotnet_path::Style::HOST, file_name).to_string();
        let id = game_id.map(|g| g.to_string());
        let mut query: Vec<(&str, &str)> = vec![
            ("romtype", "rom"),
            ("systemeid", &system),
            ("crc", &hashes.crc32),
            ("md5", &hashes.md5),
            ("sha1", &hashes.sha1),
            ("romtaille", &size),
            ("romnom", &name),
        ];
        if let Some(id) = &id {
            query.push(("gameid", id));
        }
        self.url("jeuInfos.php", &query)
    }

    pub fn jeu_infos(&self, system_id: i32, hashes: &Hashes, file_name: &str, game_id: Option<i64>, cancel: &Progress) -> JeuInfos {
        let (code, body, _) = self.wire.get_text(&self.jeu_infos_url(system_id, hashes, file_name, game_id), cancel);
        let body = match body {
            Ok(b) => b,
            Err(words) => return JeuInfos { status: Status::Failed, game: None, quota: None, detail: snippet(&words), body: Some(words) },
        };
        let status = Status::of(code);
        if status != Status::Found {
            return JeuInfos { status, game: None, quota: None, detail: snippet(&body), body: Some(body) };
        }
        match answer::jeu_infos(&body) {
            Ok((None, quota)) => JeuInfos { status: Status::NotFound, game: None, quota, detail: "no game in the answer".into(), body: Some(body) },
            Ok((Some(game), quota)) => JeuInfos { status: Status::Found, game: Some(game), quota, detail: String::new(), body: Some(body) },
            Err(words) => JeuInfos { status: Status::Malformed, game: None, quota: None, detail: redact(&words), body: Some(body) },
        }
    }

    pub fn search(&self, system_id: i32, name: &str, cancel: &Progress) -> Search {
        let system = system_id.to_string();
        let url = self.url("jeuRecherche.php", &[("systemeid", &system), ("recherche", name)]);
        let (code, body, _) = self.wire.get_text(&url, cancel);
        let body = match body {
            Ok(b) => b,
            Err(words) => return Search { status: Status::Failed, games: Vec::new(), quota: None, detail: snippet(&words) },
        };
        let status = Status::of(code);
        if status != Status::Found {
            return Search { status, games: Vec::new(), quota: None, detail: snippet(&body) };
        }
        match answer::jeu_recherche(&body) {
            Ok((games, quota)) => Search { status: if games.is_empty() { Status::NotFound } else { Status::Found }, games, quota, detail: String::new() },
            Err(words) => Search { status: Status::Malformed, games: Vec::new(), quota: None, detail: redact(&words) },
        }
    }

    /// One `ssuserInfos`: the status and the body, for a sign-in to read.
    pub fn user_infos(&self, cancel: &Progress) -> (Status, String) {
        let (code, body, _) = self.wire.get_text(&self.url("ssuserInfos.php", &[]), cancel);
        match body {
            Ok(b) => (Status::of(code), b),
            Err(words) => (Status::Failed, words),
        }
    }

    /// A media address carries the account its lookup was made with; with no member now, `ssid` and `sspassword` go.
    pub fn media_url(&self, url: &str) -> String {
        if self.member.read().unwrap_or_else(|p| p.into_inner()).as_ref().is_some_and(|m| m.is_set()) {
            return url.to_string();
        }
        let Some(q) = url.find('?') else { return url.to_string() };
        let kept: Vec<&str> = url[q + 1..]
            .split('&')
            .filter(|p| !(emusen_dotnet::text::starts_with_ignore_case(p, "ssid=") || emusen_dotnet::text::starts_with_ignore_case(p, "sspassword=")))
            .collect();
        format!("{}{}", &url[..q + 1], kept.join("&"))
    }

    /// ES-DE's `ScraperTransferTimeout`, 120 s, plus a second for every 64 KB a large file declares.
    pub fn transfer_timeout(declared: Option<i64>) -> Duration {
        Duration::from_secs(120 + (declared.unwrap_or(0).max(0) / 65536) as u64)
    }

    /// A picture by its content type; a manual by the `%PDF-` it starts with; a clip by MP4's `ftyp` box or a video type.
    pub fn is_payload(payload: Payload, content_type: Option<&str>, head: &[u8]) -> bool {
        match payload {
            Payload::Pdf => head.starts_with(b"%PDF-"),
            Payload::Video => (head.len() >= 8 && &head[4..8] == b"ftyp") || content_type.is_some_and(|t| t.starts_with("video/")),
            Payload::Image => content_type.is_some_and(|t| t.starts_with("image/")),
        }
    }

    /// A media file streamed beside its final name and moved into place, as the kind asks.
    #[allow(clippy::too_many_arguments)]
    pub fn download(&self, url: &str, target: &str, cancel: &Progress, payload: Payload, local_sha1: Option<&str>, replace: bool, declared: Option<i64>) -> Result<MediaAnswer, ()> {
        let started = Instant::now();
        let done = |outcome, path: Option<String>, bytes, sha1, detail: String| MediaAnswer { outcome, path, bytes, sha1, detail, took: started.elapsed() };
        if !replace && emusen_galaxia::dotnet_files::file_exists(target) {
            let len = std::fs::metadata(target).map(|m| m.len()).unwrap_or(0);
            return Ok(done(MediaOutcome::AlreadyThere, Some(target.to_string()), len, None, String::new()));
        }
        let mut url = self.media_url(url);
        if let Some(sha1) = local_sha1.filter(|s| !s.is_empty()) {
            url.push(if url.contains('?') { '&' } else { '?' });
            url.push_str("sha1=");
            url.push_str(&credentials::escape(sha1));
        }
        let temp = format!("{target}.part");
        let mut file: Option<std::fs::File> = None;
        let mut head: Vec<u8> = Vec::with_capacity(64);
        let mut length: u64 = 0;
        let mut sha = Sha1Stream::default();
        let mut write_error: Option<String> = None;
        let result = emusen_net::fetch_into(&self.wire.request(&url, TIMEOUT), Client::transfer_timeout(declared), cancel, &mut |chunk| {
            if file.is_none() {
                if let Some(dir) = std::path::Path::new(target).parent() {
                    std::fs::create_dir_all(dir)?;
                }
                file = Some(std::fs::File::create(&temp)?);
            }
            if head.len() < 64 {
                let take = chunk.len().min(64 - head.len());
                head.extend_from_slice(&chunk[..take]);
            }
            length += chunk.len() as u64;
            if length > LARGEST_FILE {
                return Ok(false);
            }
            sha.update(chunk);
            if let Err(e) = file.as_mut().expect("made above").write_all(chunk) {
                write_error = Some(e.to_string());
                return Err(e);
            }
            Ok(true)
        });
        drop(file);
        let streamed = match result {
            Ok(s) => s,
            Err(f) if f.kind == emusen_net::Kind::Cancelled => {
                let _ = std::fs::remove_file(&temp);
                return Err(());
            }
            Err(f) => {
                let _ = std::fs::remove_file(&temp);
                return Ok(done(MediaOutcome::Failed, None, 0, None, redact(&f.message)));
            }
        };
        if streamed.status != 200 {
            return Ok(done(MediaOutcome::Failed, None, 0, None, format!("HTTP {}", streamed.status)));
        }
        if !std::path::Path::new(&temp).exists() {
            if let Some(dir) = std::path::Path::new(target).parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::File::create(&temp);
        }
        let text = if length < 64 { String::from_utf8_lossy(&head).trim_matches(char::is_whitespace).to_string() } else { String::new() };
        let media_type = streamed.content_type.as_deref().map(|t| t.split(';').next().unwrap_or("").trim().to_string());
        let refused = match text.as_str() {
            "NOMEDIA" => Some(MediaOutcome::NoMedia),
            "SHA1OK" | "MD5OK" | "CRCOK" => Some(MediaOutcome::Unchanged),
            _ if length > LARGEST_FILE => Some(MediaOutcome::TooLarge),
            _ if !Client::is_payload(payload, media_type.as_deref(), &head) => Some(MediaOutcome::NotAnImage),
            _ if length < SMALLEST_IMAGE => Some(MediaOutcome::TooSmall),
            _ => None,
        };
        if let Some(no) = refused {
            let _ = std::fs::remove_file(&temp);
            let detail = match no {
                MediaOutcome::NotAnImage => redact(if !text.is_empty() { &text } else { media_type.as_deref().unwrap_or("no type") }),
                MediaOutcome::TooSmall | MediaOutcome::TooLarge => format!("{length} bytes"),
                _ => text.clone(),
            };
            let path = (no == MediaOutcome::Unchanged && emusen_galaxia::dotnet_files::file_exists(target)).then(|| target.to_string());
            return Ok(done(no, path, if no == MediaOutcome::NotAnImage { length } else { 0 }, None, detail));
        }
        if !replace && emusen_galaxia::dotnet_files::file_exists(target) {
            let _ = std::fs::remove_file(&temp);
            let len = std::fs::metadata(target).map(|m| m.len()).unwrap_or(0);
            return Ok(done(MediaOutcome::AlreadyThere, Some(target.to_string()), len, None, String::new()));
        }
        if let Err(e) = std::fs::rename(&temp, target) {
            let _ = std::fs::remove_file(&temp);
            return Ok(done(MediaOutcome::Failed, None, 0, None, redact(&e.to_string())));
        }
        let _ = write_error;
        Ok(done(MediaOutcome::Saved, Some(target.to_string()), length, Some(sha.hex()), String::new()))
    }
}

/// SHA-1 taken as a body streams, as `IncrementalHash` takes it.
#[derive(Default)]
struct Sha1Stream {
    bytes: Vec<u8>,
}

impl Sha1Stream {
    fn update(&mut self, chunk: &[u8]) {
        self.bytes.extend_from_slice(chunk);
    }

    fn hex(&self) -> String {
        emusen_galaxia::rom_hash::hashes_of(&self.bytes).sha1.unwrap_or_default()
    }
}

/// A sign-in's result, `SignInResult`, with what the server said.
pub fn sign_in_result(status: Status, body: &str) -> (&'static str, Option<Member>, String) {
    let said = snippet(body);
    match status {
        Status::Found => match answer::member(body) {
            Ok(Some(m)) => ("signed_in", Some(m), said),
            Ok(None) => ("wrong_account", None, said),
            Err(_) => ("unreadable", None, said),
        },
        Status::BadCredentials if emusen_dotnet::text::contains_ignore_case(body, "velop") => ("developer_refused", None, said),
        Status::BadCredentials => ("wrong_account", None, said),
        Status::ServerBusy => ("busy", None, said),
        Status::ApiClosed => ("closed", None, said),
        Status::Blacklisted => ("blocked", None, said),
        Status::TooManyRequests | Status::DailyQuota | Status::DailyKoQuota => ("too_many", None, said),
        Status::Failed => ("unreachable", None, said),
        _ => ("unreadable", None, said),
    }
}

/// Seconds as ticks, for the clock.
pub fn ticks(d: Duration) -> i64 {
    d.as_nanos() as i64 / 100
}

pub const _TICKS: i64 = TICKS_PER_SECOND;

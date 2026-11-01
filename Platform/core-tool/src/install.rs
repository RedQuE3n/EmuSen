//! A libretro core downloaded, checked and installed, and the installed ones updated, rolled back, removed and verified
//! (EmuSen_Cores.md §5.2-§5.5): every change built in `/Cores/libretro/.staging/` and put in place by renames, so a job
//! stopped or failed at any point leaves the installed folders as they were.

use crate::catalogue::{self, Catalogue};
use crate::Context;
use emusen_core_entry::{built_name, library_suffix, sha256_file, valid_stem, Entry, Notice, ADAPTER, BROKER, HOST_LIBRARY, LIBRETRO, PREVIOUS, PROXY_LIBRARY, STAGING};
use emusen_galaxia::{archive, atomic, clock};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The largest libretro core taken, as the zip declares it, as it inflates, and as the zip itself: the runtime's bound
/// on a core library (EmuSen_CoreAPI.md §7.2 rule 3).
pub const CORE_LIMIT: u64 = 512 << 20;
/// The largest `.info` copied out of the catalogue.
const INFO_LIMIT: usize = 64 << 10;
/// The bytes of a library's header read to name its architecture: a universal Mach-O's 64 slices fit.
const HEADER: usize = 4096;
/// How often a download reports its progress.
const PROGRESS_STEP: u64 = 1 << 20;

/// What a step tells its job as it goes.
pub enum Note<'a> {
    /// A download's or an unpacking's bytes so far, and the whole where it is known.
    Progress { step: &'a str, bytes: u64, total: Option<u64> },
    /// One check of EmuSen_Cores.md §5.2 passed, by name.
    Checked(&'a str),
}

/// What an install, an update or a roll back left in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installed {
    pub stem: String,
    /// The libretro core's SHA-256 now in place, which the player approves.
    pub sha256: String,
    pub build: String,
    /// The build kept for Roll Back, by its hash, where there is one.
    pub previous: Option<String>,
    /// Hashes of builds no longer on disk, whose approvals the face revokes (EmuSen_Cores.md §5.5).
    pub revoke: Vec<String>,
}

/// What `verify` found of one installed core (EmuSen_Cores.md §5.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verified {
    /// The libretro core is the file its `NOTICE` records and the project's copies are the installation's.
    Ok { sha256: String },
    /// As `Ok`, after the named project files were copied again from the installation.
    Refreshed { sha256: String, files: Vec<String> },
    /// The libretro core is not the file its `NOTICE` records; nothing is changed, and it needs approval again.
    Changed { reason: String },
    /// A folder with no `NOTICE`, the player's own, which Cores neither checks nor changes.
    Added,
}

fn io(path: &Path, e: std::io::Error) -> String {
    format!("{}: {}", path.display(), atomic::words(&e))
}

fn root(ctx: &Context) -> PathBuf {
    ctx.cores.join(LIBRETRO)
}

#[cfg(unix)]
fn mode(path: &Path, folder: bool) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(if folder { 0o755 } else { 0o644 })).map_err(|e| io(path, e))
}

#[cfg(not(unix))]
fn mode(_: &Path, _: bool) -> Result<(), String> {
    Ok(())
}

/// A folder made where there is none, 0755; one already there is left as it is.
fn make_folder(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(path).map_err(|e| io(path, e))?;
    mode(path, true)
}

/// The lock every changing job holds on `/Cores/libretro`, so two jobs, in this process or another, never rename the
/// same folders at once; refused, not waited for, while another holds it.
pub fn lock(ctx: &Context) -> Result<File, String> {
    let dir = root(ctx);
    make_folder(&dir)?;
    let path = dir.join(".lock");
    let file = File::options().create(true).truncate(false).write(true).open(&path).map_err(|e| io(&path, e))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => Err("another Cores job is changing /Cores/libretro; try again when it ends".into()),
        Err(std::fs::TryLockError::Error(e)) => Err(io(&path, e)),
    }
}

/// What a stopped or failed job left in the staging folder, removed; called with the lock held, so nothing there is
/// another job's.
pub fn clear_staging(ctx: &Context) {
    let _ = std::fs::remove_dir_all(root(ctx).join(STAGING));
}

fn staging(ctx: &Context, stem: &str, what: &str) -> PathBuf {
    root(ctx).join(STAGING).join(format!("{stem}-{}{what}", std::process::id()))
}

fn previous(ctx: &Context, stem: &str) -> Result<Entry, String> {
    Entry::at(root(ctx).join(PREVIOUS).join(stem), stem).map_err(|e| e.0)
}

/// The hashes a folder's build is known by: its `NOTICE`'s and its file's, where they differ.
fn hashes(entry: &Entry) -> Vec<String> {
    let mut out: Vec<String> = entry.read_notice().map(|n| vec![n.sha256]).unwrap_or_default();
    if let Ok(h) = sha256_file(&entry.core_path())
        && !out.contains(&h)
    {
        out.push(h);
    }
    out
}

/// A folder removed by renaming it into staging first, so it vanishes whole; its hashes are given back.
fn take_away(ctx: &Context, entry: &Entry, what: &str) -> Result<Vec<String>, String> {
    if !entry.folder.exists() {
        return Ok(Vec::new());
    }
    let known = hashes(entry);
    let gone = staging(ctx, &entry.stem, what);
    make_folder(gone.parent().unwrap_or(&gone))?;
    std::fs::rename(&entry.folder, &gone).map_err(|e| io(&entry.folder, e))?;
    let _ = std::fs::remove_dir_all(&gone);
    Ok(known)
}

/// The project's proxy and libretro host as this installation builds them, each with the name its copy takes in a core's
/// folder; both required, since a libretro core runs only through the proxy (EmuSen_Cores.md §5.3, EmuSen_LibretroHost.md §10d).
fn project_files(ctx: &Context) -> Result<Vec<(PathBuf, String)>, String> {
    let dir = ctx.installation.as_ref().ok_or("this EmuSen build names no installation folder, so it has no libretro host to install a core beside")?;
    let mut files = Vec::new();
    for (library, copy) in [(HOST_LIBRARY, ADAPTER), (PROXY_LIBRARY, BROKER)] {
        let built = built_name(library);
        if !dir.join(&built).is_file() {
            return Err(format!("this EmuSen build has no {built} beside it, so a libretro core installed now could not run; nothing was downloaded"));
        }
        files.push((dir.join(&built), format!("{copy}.{}", library_suffix())));
    }
    Ok(files)
}

/// A file copied into a folder by a copy beside it and a rename over it.
fn copy_in(from: &Path, folder: &Path, name: &str) -> Result<(), String> {
    let temp = folder.join(format!(".{name}.new"));
    std::fs::copy(from, &temp).map_err(|e| io(from, e))?;
    mode(&temp, false)?;
    std::fs::rename(&temp, folder.join(name)).map_err(|e| io(&temp, e))
}

/// The folder's project files brought to the installation's: each copied where its hash differs or it is missing, and
/// `broker.target` written. The names changed.
fn refresh(ctx: &Context, entry: &Entry) -> Result<Vec<String>, String> {
    let files = project_files(ctx)?;
    let mut changed = Vec::new();
    for (from, name) in &files {
        let at = entry.folder.join(name);
        let same = at.is_file() && sha256_file(&at).ok() == sha256_file(from).ok();
        if !same {
            copy_in(from, &entry.folder, name)?;
            changed.push(name.clone());
        }
    }
    let target = entry.target_path();
    if std::fs::read_to_string(&target).ok().as_deref() != Some(Entry::target_text().as_str()) {
        let temp = entry.folder.join(".broker.target.new");
        std::fs::write(&temp, Entry::target_text()).map_err(|e| io(&temp, e))?;
        mode(&temp, false)?;
        std::fs::rename(&temp, &target).map_err(|e| io(&temp, e))?;
        changed.push(emusen_core_entry::TARGET.to_string());
    }
    Ok(changed)
}

/// The zip fetched into `zip`, bounded at [`CORE_LIMIT`].
fn download(ctx: &Context, url: &str, zip: &Path, stop: &dyn Fn() -> bool, note: &mut dyn FnMut(Note)) -> Result<(), String> {
    let source = ctx.source.as_ref().ok_or("no buildbot listing is known for this platform")?;
    let mut request = emusen_net::Request::new(url.to_string(), Duration::from_secs(120));
    request.same_host = true;
    request.test_roots = source.test_roots.clone();
    request.test_connect = source.test_connect;
    request.offline = source.offline;
    let file = File::create(zip).map_err(|e| io(zip, e))?;
    let mut out = BufWriter::new(file);
    let progress = emusen_net::Progress::default();
    let (mut received, mut told, mut over, mut stopped) = (0u64, 0u64, false, false);
    let streamed = emusen_net::fetch_into(&request, Duration::from_secs(3600), &progress, &mut |chunk| {
        if stop() {
            stopped = true;
            return Ok(false);
        }
        received += chunk.len() as u64;
        if received > CORE_LIMIT || progress.total().is_some_and(|t| t > CORE_LIMIT) {
            over = true;
            return Ok(false);
        }
        out.write_all(chunk)?;
        if received - told >= PROGRESS_STEP {
            told = received;
            note(Note::Progress { step: "download", bytes: received, total: progress.total() });
        }
        Ok(true)
    })
    .map_err(|f| format!("{url}: {}", f.message))?;
    out.flush().map_err(|e| io(zip, e))?;
    if stopped {
        return Err("stopped".into());
    }
    if over {
        return Err(format!("{url} is larger than the {} MB a core may be", CORE_LIMIT >> 20));
    }
    if streamed.status != 200 {
        return Err(format!("{url}: the buildbot answered {}", streamed.status));
    }
    note(Note::Progress { step: "download", bytes: received, total: Some(received) });
    Ok(())
}

/// The core unpacked from a checked zip into `core`: exactly one member, named `<stem>_libretro.<suffix>` whole, whose
/// CRC-32 as the zip records it is the index line's, no link, at most [`CORE_LIMIT`] declared and inflated, its bytes'
/// CRC-32 checked, its header naming this machine.
fn unpack(stem: &str, zip: &Path, core: &Path, index_crc32: u32, note: &mut dyn FnMut(Note)) -> Result<(), String> {
    let entries = archive::entries_of(&zip.to_string_lossy()).map_err(|e| format!("the zip does not read: {e}"))?;
    let want = format!("{stem}_libretro.{}", library_suffix());
    let [entry] = entries.as_slice() else {
        return Err(format!("the zip holds {} members, and a core's zip holds one, {want}", entries.len()));
    };
    if entry.full_name != want {
        return Err(format!("the zip's member is {:?}, not {want}", entry.full_name));
    }
    if entry.crc32 != index_crc32 {
        return Err(format!("the zip records {want}'s CRC-32 as {:08x}, and the index the plan was made from says {index_crc32:08x}: the build changed since cores fetch, or the bytes did; cores fetch reads the index again", entry.crc32));
    }
    note(Note::Checked("crc32"));
    if entry.platform == 3 && (entry.external >> 16) & 0o170000 == 0o120000 {
        return Err(format!("the zip's {want} is a link, not a file"));
    }
    note(Note::Checked("shape"));
    if entry.length > CORE_LIMIT {
        return Err(format!("the zip declares {want} at {} bytes, more than the {} MB a core may be", entry.length, CORE_LIMIT >> 20));
    }
    note(Note::Checked("size"));
    let file = File::create(core).map_err(|e| io(core, e))?;
    let mut out = BufWriter::new(file);
    archive::copy_entry(&zip.to_string_lossy(), entry, &mut out)?;
    out.flush().map_err(|e| io(core, e))?;
    drop(out);
    mode(core, false)?;
    note(Note::Checked("members"));
    let mut head = vec![0u8; HEADER];
    let n = File::open(core).and_then(|mut f| f.read(&mut head)).map_err(|e| io(core, e))?;
    if !emusen_core_host::trust::header_names_this_machine(&head[..n]) {
        return Err(format!("{want} is not a library for this machine's system and architecture"));
    }
    note(Note::Checked("architecture"));
    Ok(())
}

/// A new build of a core staged whole: downloaded, checked, unpacked, its `.info` and `NOTICE` written and the project's
/// files copied; the staging folder, ready to be renamed into place.
fn stage(ctx: &Context, cat: &Catalogue, stem: &str, stop: &dyn Fn() -> bool, note: &mut dyn FnMut(Note)) -> Result<(PathBuf, Notice), String> {
    let source = ctx.source.as_ref().ok_or("no buildbot listing is known for this platform")?;
    let line = cat.index.iter().find(|l| l.stem == stem).ok_or_else(|| format!("{stem} is not in this platform's index; cores fetch reads it again"))?;
    let info = cat.infos.iter().find(|i| i.stem == stem).ok_or_else(|| format!("{stem} has no core information in the catalogue; cores fetch reads it again"))?;
    if let Some(why) = info.unsupported() {
        return Err(format!("{stem} {why}"));
    }
    project_files(ctx)?;
    let url = source.zip_url(&line.file);
    if !url.starts_with("https://") {
        return Err(format!("{url} is not an https:// address"));
    }
    let folder = staging(ctx, stem, "");
    let zip = staging(ctx, stem, ".zip");
    make_folder(&folder)?;
    download(ctx, &url, &zip, stop, note)?;
    note(Note::Checked("transport"));
    let entry = Entry::at(folder.clone(), stem).map_err(|e| e.0)?;
    unpack(stem, &zip, &entry.core_path(), line.crc32, note)?;
    let _ = std::fs::remove_file(&zip);
    let sha256 = sha256_file(&entry.core_path()).map_err(|e| io(&entry.core_path(), e))?;
    note(Note::Checked("sha256"));
    let cached = catalogue::folder(&ctx.cores).join("info.zip");
    let info_bytes = archive::read_entry(&cached.to_string_lossy(), &format!("{stem}_libretro.info"), INFO_LIMIT)?;
    std::fs::write(entry.info_path(), &info_bytes).map_err(|e| io(&entry.info_path(), e))?;
    mode(&entry.info_path(), false)?;
    let authors = info.list("authors");
    let notice = Notice {
        stem: stem.to_string(),
        name: info.name().to_string(),
        corename: info.get("corename").unwrap_or(stem).to_string(),
        authors: if authors.is_empty() { vec!["not stated".into()] } else { authors },
        license: info.get("license").unwrap_or("not stated").to_string(),
        source: url,
        build: line.date.clone(),
        index_crc32: line.crc32,
        sha256,
        fetched: clock::utc_now().chars().take(10).collect(),
        extra: Vec::new(),
    };
    let text = notice.to_text().map_err(|e| e.0)?;
    std::fs::write(entry.notice_path(), text).map_err(|e| io(&entry.notice_path(), e))?;
    mode(&entry.notice_path(), false)?;
    refresh(ctx, &entry)?;
    Ok((folder, notice))
}

/// `core-get`: a core not yet installed downloaded, checked and installed by one rename (EmuSen_Cores.md §5.3).
pub fn get(ctx: &Context, cat: &Catalogue, stem: &str, stop: &dyn Fn() -> bool, note: &mut dyn FnMut(Note)) -> Result<Installed, String> {
    let dest = Entry::new(&ctx.cores, stem).map_err(|e| e.0)?;
    if dest.folder.exists() {
        return Err(match dest.read_notice() {
            Ok(_) => format!("{stem} is installed; cores update {stem} replaces it with the index's build"),
            Err(_) => format!("/Cores/libretro/{stem} is a folder Cores did not make; it is never replaced"),
        });
    }
    let (folder, notice) = stage(ctx, cat, stem, stop, note).inspect_err(|_| discard(ctx, stem))?;
    if stop() {
        discard(ctx, stem);
        return Err("stopped".into());
    }
    std::fs::rename(&folder, &dest.folder).map_err(|e| io(&dest.folder, e)).inspect_err(|_| discard(ctx, stem))?;
    Ok(Installed { stem: stem.into(), sha256: notice.sha256, build: notice.build, previous: None, revoke: Vec::new() })
}

fn discard(ctx: &Context, stem: &str) {
    let _ = std::fs::remove_dir_all(staging(ctx, stem, ""));
    let _ = std::fs::remove_file(staging(ctx, stem, ".zip"));
}

/// `core-update`: the index's build staged, the installed one moved to `.previous/` for Roll Back and the new one renamed
/// into place, every rename undone on failure; the build kept before is dropped and its hashes given to revoke
/// (EmuSen_Cores.md §5.4).
pub fn update(ctx: &Context, cat: &Catalogue, stem: &str, stop: &dyn Fn() -> bool, note: &mut dyn FnMut(Note)) -> Result<Installed, String> {
    let dest = Entry::new(&ctx.cores, stem).map_err(|e| e.0)?;
    let current = dest.read_notice().map_err(|_| {
        if dest.folder.exists() { format!("/Cores/libretro/{stem} is a folder Cores did not make; it is never replaced") } else { format!("{stem} is not installed; cores get {stem} installs it") }
    })?;
    let (folder, notice) = stage(ctx, cat, stem, stop, note).inspect_err(|_| discard(ctx, stem))?;
    if stop() {
        discard(ctx, stem);
        return Err("stopped".into());
    }
    let prev = previous(ctx, stem)?;
    make_folder(prev.folder.parent().unwrap_or(&prev.folder))?;
    let old = staging(ctx, stem, "-old");
    let dropped = if prev.folder.exists() {
        let known = hashes(&prev);
        std::fs::rename(&prev.folder, &old).map_err(|e| io(&prev.folder, e)).inspect_err(|_| discard(ctx, stem))?;
        known
    } else {
        Vec::new()
    };
    let undo_old = || {
        if old.exists() {
            let _ = std::fs::rename(&old, &prev.folder);
        }
    };
    if let Err(e) = std::fs::rename(&dest.folder, &prev.folder) {
        undo_old();
        discard(ctx, stem);
        return Err(io(&dest.folder, e));
    }
    if let Err(e) = std::fs::rename(&folder, &dest.folder) {
        let _ = std::fs::rename(&prev.folder, &dest.folder);
        undo_old();
        discard(ctx, stem);
        return Err(io(&folder, e));
    }
    let _ = std::fs::remove_dir_all(&old);
    let kept: Vec<String> = [current.sha256.clone(), notice.sha256.clone()].into();
    let revoke = dropped.into_iter().filter(|h| !kept.contains(h)).collect();
    Ok(Installed { stem: stem.into(), sha256: notice.sha256, build: notice.build, previous: Some(current.sha256), revoke })
}

/// `core-rollback`: the kept build's project files refreshed, then it and the installed one swapped by renames, so a
/// second roll back undoes the first; both keep their approvals (EmuSen_Cores.md §5.4).
pub fn rollback(ctx: &Context, stem: &str) -> Result<Installed, String> {
    let dest = Entry::new(&ctx.cores, stem).map_err(|e| e.0)?;
    let prev = previous(ctx, stem)?;
    let back = prev.read_notice().map_err(|_| format!("no earlier build of {stem} is kept; Roll Back follows an update"))?;
    prev.verify().map_err(|e| format!("the kept build of {stem} is not the file its NOTICE records, so it is not put back: {}", e.0))?;
    let now = dest.read_notice().map_err(|_| format!("{stem} is not installed by Cores, so there is nothing to roll back from"))?;
    refresh(ctx, &prev)?;
    let swap = staging(ctx, stem, "-swap");
    make_folder(swap.parent().unwrap_or(&swap))?;
    std::fs::rename(&dest.folder, &swap).map_err(|e| io(&dest.folder, e))?;
    if let Err(e) = std::fs::rename(&prev.folder, &dest.folder) {
        let _ = std::fs::rename(&swap, &dest.folder);
        return Err(io(&prev.folder, e));
    }
    if let Err(e) = std::fs::rename(&swap, &prev.folder) {
        let _ = std::fs::rename(&dest.folder, &prev.folder);
        let _ = std::fs::rename(&swap, &dest.folder);
        return Err(io(&swap, e));
    }
    Ok(Installed { stem: stem.into(), sha256: back.sha256, build: back.build, previous: Some(now.sha256), revoke: Vec::new() })
}

/// `core-rm`: a folder Cores made removed whole, with the build kept for Roll Back; the player's saves and the core's
/// settings live in EmuSen's stores and stay. The hashes are given to revoke (EmuSen_Cores.md §5.5).
pub fn remove(ctx: &Context, stem: &str) -> Result<Vec<String>, String> {
    let dest = Entry::new(&ctx.cores, stem).map_err(|e| e.0)?;
    let prev = previous(ctx, stem)?;
    if dest.folder.exists() {
        if let Err(e) = dest.read_notice() {
            return Err(if dest.notice_path().exists() {
                format!("/Cores/libretro/{stem}'s NOTICE does not read ({}); Cores removes only folders it made", e.0)
            } else {
                format!("/Cores/libretro/{stem} is a folder Cores did not make; it is never removed")
            });
        }
    } else if !prev.folder.exists() {
        return Err(format!("{stem} is not installed"));
    }
    let mut revoke = take_away(ctx, &dest, "-rm")?;
    for h in take_away(ctx, &prev, "-rm-previous")? {
        if !revoke.contains(&h) {
            revoke.push(h);
        }
    }
    Ok(revoke)
}

/// `core-verify`: the libretro core hashed against its `NOTICE`, and where it agrees the project's files refreshed from
/// the installation; a core that differs is reported and left alone (EmuSen_Cores.md §5.4).
pub fn verify(ctx: &Context, stem: &str) -> Result<Verified, String> {
    let dest = Entry::new(&ctx.cores, stem).map_err(|e| e.0)?;
    if !dest.folder.is_dir() {
        return Err(format!("{stem} is not installed"));
    }
    if !dest.notice_path().exists() {
        return Ok(Verified::Added);
    }
    let sha256 = match dest.verify() {
        Ok(h) => h,
        Err(e) => return Ok(Verified::Changed { reason: format!("{}; it needs approval again before it runs, and Verify never clears that", e.0) }),
    };
    let files = refresh(ctx, &dest)?;
    Ok(if files.is_empty() { Verified::Ok { sha256 } } else { Verified::Refreshed { sha256, files } })
}

/// Whether a name can be a core's stem in a request.
pub fn named(stem: &str) -> Result<(), String> {
    if valid_stem(stem) { Ok(()) } else { Err(format!("{stem:?} cannot name a core")) }
}

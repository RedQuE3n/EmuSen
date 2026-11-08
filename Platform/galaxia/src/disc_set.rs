//! A game's discs as its library file names them, the C# `DiscSet`, `DiscFiles` and `ArchiveImages`' unpacking: an
//! `.m3u`'s discs in its order, else the one image, each unpacked into the Unpacked folder where it is an archive; a
//! disc opened is its image and the files beside it mapped and held, with a hold on the unpacked folder so that a
//! pruner of either half leaves it while a game reads it (EmuSen_Galaxia.md §3.7, §3.8).

use crate::path_key;
use crate::mapped::Mapped;
use crate::tree::{Directory, Tree};
use crate::{archive, image_sets};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// What the folder keeps beside the archive being opened; past it, the least recently opened archives go.
pub const KEPT_BYTES: u64 = 4 << 30;

/// A partial unpacking older than this was left by a process that stopped.
pub const PARTIAL_AGE: Duration = Duration::from_secs(3600);

const MARKER: &str = ".unpacked";

fn unpacked(tree: &Tree) -> PathBuf {
    tree.directory(Directory::Unpacked).unwrap_or_default()
}

/// The folder an archive unpacks to, named by its full path, length and time, `ArchiveImages.FolderFor`; an archive
/// that is not there has none, as `FileInfo.Length` throws.
pub fn folder_for(tree: &Tree, archive_path: impl AsRef<Path>) -> Result<PathBuf, String> {
    let full = path_key::key_text(archive_path);
    let (length, ticks) = crate::files::file_info(&full).ok_or_else(|| format!("Could not find file '{full}'."))?;
    Ok(unpacked(tree).join(folder_name(&full, length, ticks)))
}

/// The unpacked folder's name: the first eight bytes of the SHA-256 of `full path|length|ticks`, in hex. A stored name, so its rule never changes.
pub fn folder_name(full: &str, length: i64, ticks: i64) -> String {
    let key = Sha256::digest(format!("{full}|{length}|{ticks}").as_bytes());
    key[..8].iter().map(|b| format!("{b:02x}")).collect()
}

/// An entry's name in the unpacked folder: the chosen image and an `.sbi` of its stem take the archive's stem.
fn renamed(entry: &str, chosen: &str, archive_path: &Path) -> PathBuf {
    let stem = archive_path.file_stem().unwrap_or_default();
    let dir = Path::new(chosen).parent().unwrap_or(Path::new(""));
    if entry == chosen {
        let mut name = stem.to_os_string();
        if let Some(extension) = Path::new(chosen).extension() {
            name.push(".");
            name.push(extension);
        }
        return dir.join(name);
    }
    let sbi = Path::new(entry).extension().is_some_and(|e| e.eq_ignore_ascii_case("sbi"));
    if sbi && Path::new(entry).with_extension("") == Path::new(chosen).with_extension("") {
        let mut name = stem.to_os_string();
        name.push(".sbi");
        dir.join(name)
    } else {
        PathBuf::from(entry)
    }
}

/// The archive unpacked, once, its image named by the archive's stem; the image's path, `ArchiveImages.Unpack`.
pub fn unpack(tree: &Tree, archive_path: impl AsRef<Path>, is_image: &dyn Fn(&str) -> bool) -> Result<PathBuf, String> {
    let archive_path = archive_path.as_ref();
    let folder = folder_for(tree, archive_path)?;
    let marker = folder.join(MARKER);
    if !crate::files::file_exists(&marker) {
        if crate::files::directory_exists(&folder) {
            let _ = std::fs::remove_dir_all(&folder);
        }
        let mut partial = folder.clone().into_os_string();
        partial.push(format!(".partial-{}", std::process::id()));
        let partial = PathBuf::from(partial);
        if crate::files::directory_exists(&partial) {
            std::fs::remove_dir_all(&partial).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&partial).map_err(|e| e.to_string())?;
        let names: Vec<String> = archive::entries_of(archive_path)?.into_iter().map(|e| e.full_name).filter(|n| !n.ends_with('/')).collect();
        let chosen = archive::image_entry(archive_path, is_image)?
            .ok_or_else(|| format!("{} holds no image a console here opens.", archive_path.file_name().unwrap_or_default().to_string_lossy()))?;
        let root = path_key::key(&partial);
        for name in &names {
            let target = path_key::key(partial.join(renamed(name, &chosen, archive_path)));
            if !target.starts_with(&root) || target == root {
                continue;
            }
            if let Some(dir) = target.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            let bytes = archive::read_entry(archive_path, name, usize::MAX)?;
            std::fs::write(&target, bytes).map_err(|e| e.to_string())?;
        }
        let words = format!("{}\n{}\n", path_key::key_text(archive_path), renamed(&chosen, &chosen, archive_path).to_string_lossy());
        std::fs::write(partial.join(MARKER), words).map_err(|e| e.to_string())?;
        if std::fs::rename(&partial, &folder).is_err() {
            if crate::files::file_exists(&marker) {
                let _ = std::fs::remove_dir_all(&partial);
            } else {
                return Err(format!("{} could not be put in place", folder.display()));
            }
        }
    }
    let _ = File::open(&folder).and_then(|f| f.set_modified(SystemTime::now()));
    prune(tree, Some(&folder), KEPT_BYTES);
    let marker_text = std::fs::read_to_string(&marker).map_err(|e| e.to_string())?;
    let image = marker_text.lines().nth(1).unwrap_or("");
    Ok(folder.join(image))
}

/// A hold on the unpacked folder a file is in, a shared lock on its marker as .NET's `FileShare.Read` takes one; none
/// for a file in no unpacked folder, `ArchiveImages.Hold`.
pub fn hold(tree: &Tree, file: impl AsRef<Path>) -> Result<Option<File>, String> {
    let root = path_key::key(unpacked(tree));
    let full = path_key::key(file);
    let Ok(rest) = full.strip_prefix(&root) else { return Ok(None) };
    let Some(first) = rest.components().next() else { return Ok(None) };
    let marker = root.join(first).join(MARKER);
    let f = File::open(&marker).map_err(|e| e.to_string())?;
    f.lock_shared().map_err(|e| e.to_string())?;
    Ok(Some(f))
}

fn folder_bytes(dir: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            match e.file_type() {
                Ok(t) if t.is_dir() => total += folder_bytes(&e.path()),
                Ok(_) => total += e.metadata().map(|m| m.len()).unwrap_or(0),
                Err(_) => {}
            }
        }
    }
    total
}

/// The folders past `kept` bytes, least recently opened first, and partial ones a stopped process left; `keep` and a
/// folder a game holds are never removed, `ArchiveImages.Prune`.
pub fn prune(tree: &Tree, keep: Option<&Path>, kept: u64) {
    let root = unpacked(tree);
    let Ok(read) = std::fs::read_dir(&root) else { return };
    let keep = keep.map(path_key::key);
    let mut folders: Vec<(PathBuf, SystemTime)> = read
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| (path_key::key(e.path()), e.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH)))
        .collect();
    let now = SystemTime::now();
    let partial = |d: &Path| d.file_name().is_some_and(|n| n.to_string_lossy().contains(".partial-"));
    for (dir, modified) in folders.iter().filter(|(d, _)| partial(d)) {
        if now.duration_since(*modified).unwrap_or_default() > PARTIAL_AGE {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
    folders.retain(|(d, _)| !partial(d));
    folders.sort_by(|a, b| (Some(&b.0) == keep.as_ref()).cmp(&(Some(&a.0) == keep.as_ref())).then(b.1.cmp(&a.1)));
    let mut total = 0;
    for (dir, _) in &folders {
        total += folder_bytes(dir);
        if total > kept && Some(dir) != keep.as_ref() {
            remove_unless_held(dir);
        }
    }
}

fn remove_unless_held(dir: &Path) {
    let marker = dir.join(MARKER);
    if crate::files::file_exists(&marker) {
        let Ok(alone) = File::open(&marker) else { return };
        if alone.try_lock().is_err() {
            return;
        }
        if std::fs::remove_file(&marker).is_err() {
            return;
        }
    }
    let _ = std::fs::remove_dir_all(dir);
}

/// A game's discs: each disc's archive where it came from one, and its image.
pub struct DiscSet {
    discs: Vec<(Option<PathBuf>, PathBuf)>,
}

impl DiscSet {
    /// The set a library file stands for, `DiscSet.Of`; `is_image` says which extensions an archive's entry may have.
    pub fn of(tree: &Tree, at: impl AsRef<Path>, is_image: &dyn Fn(&str) -> bool) -> Result<DiscSet, String> {
        let disc = |file: &Path| -> Result<(Option<PathBuf>, PathBuf), String> {
            if archive::is_archive(file) { Ok((Some(file.to_path_buf()), unpack(tree, file, is_image)?)) } else { Ok((None, file.to_path_buf())) }
        };
        let first = disc(at.as_ref())?;
        let discs = if image_sets::is_playlist(&first.1) { image_sets::discs(&first.1)?.iter().map(|d| disc(d)).collect::<Result<_, _>>()? } else { vec![first] };
        Ok(DiscSet { discs })
    }

    /// Each disc's image, an archive's where it is unpacked.
    pub fn images(&self) -> Vec<PathBuf> {
        self.discs.iter().map(|(_, i)| i.clone()).collect()
    }

    pub fn count(&self) -> usize {
        self.discs.len()
    }

    /// Disc `index` mapped and held; an archive's unpacked again where its folder has gone since.
    pub fn open(&self, tree: &Tree, index: usize, is_image: &dyn Fn(&str) -> bool) -> Result<DiscFiles, String> {
        let (archive, image) = &self.discs[index];
        let Some(archive) = archive else { return DiscFiles::open(tree, image) };
        let mut attempt = 0;
        loop {
            let image = unpack(tree, archive, is_image)?;
            match DiscFiles::open(tree, &image) {
                Ok(files) => return Ok(files),
                Err(e) if attempt >= 2 => return Err(e),
                Err(_) => attempt += 1,
            }
        }
    }
}

/// A disc's image and the files beside it, mapped and held with the unpacked folder they are in.
pub struct DiscFiles {
    pub path: PathBuf,
    pub image: Mapped,
    /// A sheet's tracks and then an `.sbi`, in the order a core numbers them.
    pub companions: Vec<Mapped>,
    _hold: Option<File>,
}

impl DiscFiles {
    /// The image at `image_path` with its companions; nothing of any file is read but a sheet's own text.
    pub fn open(tree: &Tree, image_path: impl AsRef<Path>) -> Result<DiscFiles, String> {
        let image_path = image_path.as_ref();
        let hold = hold(tree, image_path)?;
        let image = Mapped::open(image_path).map_err(|e| e.to_string())?;
        let companions = image_sets::files_for(image_path)?.iter().map(|f| Mapped::open(f).map_err(|e| e.to_string())).collect::<Result<_, _>>()?;
        Ok(DiscFiles { path: image_path.to_path_buf(), image, companions, _hold: hold })
    }

    /// No disc: an empty image and no companions, which a console started empty is made with.
    pub fn empty() -> DiscFiles {
        DiscFiles { path: PathBuf::new(), image: Mapped::empty(), companions: Vec::new(), _hold: None }
    }

    /// The image's own file, where the bytes mapped are that file's and not a copy unpacked from an archive, which a
    /// core reading by path could find pruned (EmuSen_CoreAPI.md §6.41, the path only for an unaltered file).
    pub fn unaltered_path(&self) -> Option<&Path> {
        (self._hold.is_none() && !self.path.as_os_str().is_empty()).then_some(self.path.as_path())
    }

    /// Every byte of the disc on its medium.
    pub fn length(&self) -> u64 {
        self.image.bytes().len() as u64 + self.companions.iter().map(|c| c.bytes().len() as u64).sum::<u64>()
    }
}

//! The library tool (EmuSen_GamePackages.md §2.4, §4): the library's disc games converted to CHD by the shipped,
//! pinned chdman into `Compressed`, never inside the library; each verified by chdman and by two checks of EmuSen's own
//! reader; each given a linked package; listed and checked again. A request runs as a job whose events, one JSON line
//! each, are read by the shell's `emusen.library` module and by Mistress's window alike. §12 records what was built.

pub mod chdman;
pub mod checks;
pub mod convert;
pub mod discs;
pub mod job;
pub mod packages;
pub mod patches;
pub mod pin;

use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree::{Directory, Tree};

/// Where a job reads and writes, and the chdman it runs: everything the tool knows of the machine.
#[derive(Clone, Debug)]
pub struct Context {
    /// Galaxia's tree, for `Compressed`, `Packages`, `Logs` and `Unpacked`.
    pub tree: Tree,
    /// `AppSettings.RomDirectory`, the library; read, never written.
    pub library: Option<String>,
    /// The chdman binary, and the SHA-256 pinned for it, with the pin's release.
    pub chdman: String,
    pub pin: Option<String>,
    pub release: String,
    /// `AppSettings.StateDirectory`, where `pkg migrate` finds a game's states; none for the tree's own.
    pub state_folder: Option<String>,
    /// The process's SQLite, for `pkg migrate` to read states.db's records and games.db's crop; none copies neither.
    pub sqlite: Option<std::sync::Arc<emusen_sqlite::Sqlite>>,
    /// `AppSettings.ChdOutputDirectory` as written, where CHDs go; none or blank for `Compressed` (§20).
    pub output: Option<String>,
}

/// Where a conversion writes its CHDs (EmuSen_GamePackages.md §20): a folder, `Compressed` or the player's, at the
/// library's relative path; or beside each original in the library, a choice made for one batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    Folder { root: String, compressed: bool },
    Beside { library: String },
}

impl Destination {
    pub fn root(&self) -> &str {
        match self {
            Destination::Folder { root, .. } => root,
            Destination::Beside { library } => library,
        }
    }

    pub fn beside(&self) -> bool {
        matches!(self, Destination::Beside { .. })
    }
}

impl Context {
    /// A context over the pin this build carries, for this platform.
    pub fn pinned(tree: Tree, library: Option<String>, chdman: String) -> Context {
        let pin = pin::pinned();
        Context { tree, library, chdman, pin: pin.binary(pin::rid()).map(str::to_string), release: pin.release.clone(), state_folder: None, sqlite: None, output: None }
    }

    /// The folders a package's paths are relative to (EmuSen_GamePackages.md §14).
    pub fn places(&self) -> emusen_galaxia::claims::Places {
        emusen_galaxia::claims::Places { packages: self.packages(), library: self.library.clone(), compressed: self.compressed() }
    }

    fn directory(&self, which: Directory) -> String {
        self.tree.directory(which).unwrap_or_default()
    }

    pub fn compressed(&self) -> String {
        self.directory(Directory::Compressed)
    }

    pub fn packages(&self) -> String {
        self.directory(Directory::Packages)
    }

    pub fn logs(&self) -> String {
        self.directory(Directory::Logs)
    }

    pub fn unpacked(&self) -> String {
        self.directory(Directory::Unpacked)
    }

    /// The library, refused with words where it is unset or missing, and where a folder the tool writes lies inside
    /// it or holds it: checked before anything is written (§4.2, §4.5).
    pub fn checked_library(&self) -> Result<String, String> {
        let library = self.library.clone().filter(|l| !l.trim().is_empty()).ok_or("AppSettings.RomDirectory is not set: there is no library to convert.")?;
        if !std::path::Path::new(&library).is_dir() {
            return Err(format!("The library {library} is not a folder."));
        }
        let real = canonical(&library);
        for (name, at) in [("Compressed", self.compressed()), ("Packages", self.packages()), ("Logs", self.logs()), ("Unpacked", self.unpacked())] {
            if at.is_empty() {
                return Err(format!("The tree names no {name} folder."));
            }
            let target = canonical(&at);
            if inside(&target, &real) || inside(&real, &target) {
                return Err(format!("{name} ({at}) and the library ({library}) overlap: the library is never written, so nothing was done."));
            }
        }
        Ok(library)
    }

    /// Where CHDs go: beside the originals when the batch asks, else the folder the request or the setting names,
    /// else `Compressed`. A chosen folder is refused with words where it is not a full path, is a file, or overlaps
    /// the library either way (§20.1).
    pub fn destination(&self, library: &str, asked: Option<&str>, beside: bool) -> Result<Destination, String> {
        if beside {
            return Ok(Destination::Beside { library: library.to_string() });
        }
        let compressed = self.compressed();
        let named = emusen_galaxia::setting_folders::checked(asked.or(self.output.as_deref()), &emusen_galaxia::tree::user_home());
        use emusen_galaxia::setting_folders::FolderState;
        let root = match (named.state, named.path) {
            (FolderState::Unset, _) => return Ok(Destination::Folder { root: compressed, compressed: true }),
            (FolderState::Relative, _) => return Err(format!("The output folder \"{}\" is not a full path, so nothing was done: name a full path, or one starting with ~.", named.written)),
            (FolderState::File, _) => return Err(format!("The output folder \"{}\" is a file, not a folder, so nothing was done.", named.written)),
            (_, Some(p)) => p,
            (_, None) => return Ok(Destination::Folder { root: compressed, compressed: true }),
        };
        let (target, real) = (canonical(&root), canonical(library));
        if inside(&target, &real) || inside(&real, &target) {
            return Err(format!("The output folder ({root}) and the library ({library}) overlap: the library is never written, so nothing was done."));
        }
        let is_compressed = target == canonical(&compressed);
        Ok(Destination::Folder { root, compressed: is_compressed })
    }
}

/// A path with every link along it followed, as far as it exists; the rest as written.
pub fn canonical(at: &str) -> String {
    let full = path::full_path(at);
    let mut existing = std::path::PathBuf::from(&full);
    let mut rest: Vec<std::ffi::OsString> = Vec::new();
    while !existing.exists() {
        match (existing.file_name().map(|n| n.to_owned()), existing.parent().map(|p| p.to_path_buf())) {
            (Some(name), Some(parent)) => {
                rest.push(name);
                existing = parent;
            }
            _ => return full,
        }
    }
    let mut real = std::fs::canonicalize(&existing).unwrap_or(existing);
    for part in rest.iter().rev() {
        real.push(part);
    }
    real.to_string_lossy().into_owned()
}

/// Whether `child` is `parent` or lies under it, both canonical.
pub fn inside(child: &str, parent: &str) -> bool {
    let parent = path::trim_ending_separator(Style::HOST, parent);
    child == parent || child.strip_prefix(parent).is_some_and(|rest| rest.starts_with(std::path::MAIN_SEPARATOR))
}

//! Where this program's tree starts and where each kind of file goes in it: `ConfigRoot`, `ConfigStore` and
//! `DataStore` of the C# Galaxia. See EmuSen_Galaxia.md §3 and `man hier`.

use crate::files::{directory_exists, file_exists};
use crate::path_key;
use std::cell::RefCell;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// The fallback root beside the binary, when neither marker turns up.
pub const PUBLISHED_ROOT_DIR_NAME: &str = "DianaShRoot";
/// Written to a published tree's root by the publish.
pub const ROOT_MARKER_FILE_NAME: &str = ".dianashroot";
/// The fallback root and the marker a tree published before the shell was renamed carries, still honoured.
pub const LEGACY_PUBLISHED_ROOT_DIR_NAME: &str = "DianaOSRoot";
pub const LEGACY_ROOT_MARKER_FILE_NAME: &str = ".dianaosroot";
/// The checkout's marker.
pub const SOLUTION_FILE_NAME: &str = "EmuSen.sln";
/// The folder under `~/Library/Application Support` that holds a Mac's whole tree.
pub const MAC_DATA_DIR_NAME: &str = "EmuSen";
pub const HOME_DIR_NAME: &str = "home";
pub const CONFIG_DIR_NAME: &str = "etc";
pub const PROGRAM_DIR_NAME: &str = "EmuSen";

/// `ConfigRoot.MacDataDirectoryFor`.
pub fn mac_data_directory_for(user_home: &Path) -> PathBuf {
    user_home.join("Library").join("Application Support").join(MAC_DATA_DIR_NAME)
}

/// `ConfigRoot.BundleContentsFor`: the bundle's `Contents` when `base_directory` is its `Contents/MacOS`.
pub fn bundle_contents_for(base_directory: &Path) -> Option<PathBuf> {
    let dir = path_key::key(base_directory);
    let contents = dir.parent()?;
    let bundle = contents.parent()?;
    let named = |p: &Path, name: &str| p.file_name().is_some_and(|n| n.eq_ignore_ascii_case(name));
    let in_bundle = named(&dir, "MacOS") && named(contents, "Contents") && bundle.extension().is_some_and(|e| e.eq_ignore_ascii_case("app"));
    in_bundle.then(|| contents.to_path_buf())
}

/// `ConfigRoot.SeedDirectoryFor`: the read-only skeleton a bundle carries, `Contents/Resources/home`.
pub fn seed_directory_for(base_directory: &Path, mac_os: bool) -> Option<PathBuf> {
    if !mac_os {
        return None;
    }
    bundle_contents_for(base_directory).map(|contents| contents.join("Resources").join(HOME_DIR_NAME))
}

/// `ConfigRoot.ComputeFor`: the bundle rule, then the walk up to a marker, then the fallback. See EmuSen_Galaxia.md §3.4.
pub fn root_for(base_directory: &Path, mac_os: bool, user_home: &Path) -> PathBuf {
    if mac_os && bundle_contents_for(base_directory).is_some() {
        return mac_data_directory_for(user_home);
    }
    let mut dir = path_key::key(base_directory);
    for _ in 0..10 {
        if [SOLUTION_FILE_NAME, ROOT_MARKER_FILE_NAME, LEGACY_ROOT_MARKER_FILE_NAME].iter().any(|marker| file_exists(dir.join(marker))) {
            return dir;
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }
    if mac_os {
        return mac_data_directory_for(user_home);
    }
    let legacy = path_key::key(base_directory.join(LEGACY_PUBLISHED_ROOT_DIR_NAME));
    let fallback = path_key::key(base_directory.join(PUBLISHED_ROOT_DIR_NAME));
    if !directory_exists(&fallback) && directory_exists(&legacy) { legacy } else { fallback }
}

/// `DataMigration.LegacyRootFor`: where the tree lived before it collapsed to `<root>/home`.
pub fn legacy_root_for(config_root: &Path) -> PathBuf {
    config_root.join("EmuSen.DianaSh").join("DianaSh").join("Usr").join("Home")
}

/// A directory of the tree, by name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Directory {
    Root = 0,
    Seed = 1,
    Config = 2,
    ConfigPrevious = 3,
    ConfigLegacy = 4,
    Home = 5,
    Logs = 6,
    Saves = 7,
    SaveStates = 8,
    Library = 9,
    Screenshots = 10,
    Artwork = 11,
    Media = 12,
    Firmware = 13,
    Shaders = 14,
    Themes = 15,
    Cheats = 16,
    Games = 17,
    LegacyRoot = 18,
    /// `ErrorLog.DefaultRoot`.
    LogDefault = 19,
    /// `DataStore.Unpacked`: archives opened for play, unpacked and kept by their own rule (EmuSen_Galaxia.md §3.7).
    Unpacked = 20,
    /// `DataStore.Cores`: the player's own core libraries, `/Cores` (EmuSen_Backend.md §29.40.4).
    Cores = 21,
    /// `DataStore.Packages`: game packages, a folder per game (EmuSen_GamePackages.md §3.1).
    Packages = 22,
    /// `DataStore.Compressed`: CHDs converted from the library, never inside it (EmuSen_GamePackages.md §4.2).
    Compressed = 23,
}

impl Directory {
    pub const ALL: [Directory; 24] = [
        Directory::Root,
        Directory::Seed,
        Directory::Config,
        Directory::ConfigPrevious,
        Directory::ConfigLegacy,
        Directory::Home,
        Directory::Logs,
        Directory::Saves,
        Directory::SaveStates,
        Directory::Library,
        Directory::Screenshots,
        Directory::Artwork,
        Directory::Media,
        Directory::Firmware,
        Directory::Shaders,
        Directory::Themes,
        Directory::Cheats,
        Directory::Games,
        Directory::LegacyRoot,
        Directory::LogDefault,
        Directory::Unpacked,
        Directory::Cores,
        Directory::Packages,
        Directory::Compressed,
    ];

    pub fn from_u32(value: u32) -> Option<Directory> {
        Directory::ALL.get(value as usize).copied()
    }
}

/// One of the three redirects the tests move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Override {
    Config = 0,
    Data = 1,
    Legacy = 2,
}

impl Override {
    pub fn from_u32(value: u32) -> Option<Override> {
        [Override::Config, Override::Data, Override::Legacy].get(value as usize).copied()
    }
}

/// The tree as one value: its root and the overrides, with every path computed on demand so an override moved underneath takes effect at once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tree {
    /// `ConfigRoot.Directory`.
    pub root: PathBuf,
    /// `ConfigRoot.SeedDirectory`.
    pub seed: Option<PathBuf>,
    /// The system's folder for a program's settings, under which config sat before Galaxia.
    pub application_data: PathBuf,
    /// `ConfigStore.OverrideDirectory`.
    pub config_override: Option<PathBuf>,
    /// `DataStore.OverrideDirectory`.
    pub data_override: Option<PathBuf>,
    /// `ConfigStore.OverrideLegacyDirectory`.
    pub legacy_override: Option<PathBuf>,
}

impl Tree {
    /// `DataStore.UsrHome`: the shell's own `/`.
    pub fn home(&self) -> PathBuf {
        self.data_override.clone().unwrap_or_else(|| self.root.join(HOME_DIR_NAME))
    }

    /// `ConfigStore.Directory`, built from the root and not from the data override, so neither redirect moves the other.
    pub fn config(&self) -> PathBuf {
        self.config_override.clone().unwrap_or_else(|| self.root.join(HOME_DIR_NAME).join(CONFIG_DIR_NAME).join(PROGRAM_DIR_NAME))
    }

    /// `ConfigStore.PreviousDirectory`: where config sat before it moved under home.
    pub fn config_previous(&self) -> PathBuf {
        self.root.join(CONFIG_DIR_NAME).join(PROGRAM_DIR_NAME)
    }

    /// `ConfigStore.LegacyDirectory`.
    pub fn config_legacy(&self) -> PathBuf {
        self.legacy_override.clone().unwrap_or_else(|| self.application_data.join(PROGRAM_DIR_NAME))
    }

    /// `ConfigStore.For`, with or without a category.
    pub fn config_path(&self, category: Option<&str>, file_name: &str) -> PathBuf {
        match category {
            Some(category) => self.config().join(category).join(file_name),
            None => self.config().join(file_name),
        }
    }

    /// `ConfigStore.LegacyPathFor`: none when config is redirected and the legacy directory is not, so a test never reaches a real one.
    pub fn legacy_config_path(&self, file_name: &str) -> Option<PathBuf> {
        if self.config_override.is_some() && self.legacy_override.is_none() {
            return None;
        }
        Some(self.config_legacy().join(file_name))
    }

    /// A named directory; only the seed can be absent.
    pub fn directory(&self, which: Directory) -> Option<PathBuf> {
        let home = || self.home();
        Some(match which {
            Directory::Root => self.root.clone(),
            Directory::Seed => return self.seed.clone(),
            Directory::Config => self.config(),
            Directory::ConfigPrevious => self.config_previous(),
            Directory::ConfigLegacy => self.config_legacy(),
            Directory::Home => home(),
            Directory::Logs => home().join("Logs"),
            Directory::Saves => self.saves(),
            Directory::SaveStates => self.save_states(),
            Directory::Library => home().join("Library"),
            Directory::Screenshots => home().join("Screenshots"),
            Directory::Artwork => home().join("Artwork"),
            Directory::Media => home().join("Media"),
            Directory::Firmware => home().join("Firmware"),
            Directory::Shaders => home().join("Shaders"),
            Directory::Themes => home().join("Themes"),
            Directory::Cheats => home().join("Cheats"),
            Directory::Games => home().join("Games"),
            Directory::LegacyRoot => legacy_root_for(&self.root),
            Directory::LogDefault => self.application_data.join(PROGRAM_DIR_NAME).join("Logs"),
            Directory::Unpacked => home().join("Unpacked"),
            Directory::Cores => home().join("Cores"),
            Directory::Packages => home().join("Packages"),
            Directory::Compressed => home().join("Compressed"),
        })
    }

    /// `DataStore.Saves`.
    pub fn saves(&self) -> PathBuf {
        self.home().join("Saves")
    }

    /// `DataStore.SaveStates`.
    pub fn save_states(&self) -> PathBuf {
        self.saves().join("Save States")
    }

    pub fn set_override(&mut self, which: Override, directory: Option<PathBuf>) {
        match which {
            Override::Config => self.config_override = directory,
            Override::Data => self.data_override = directory,
            Override::Legacy => self.legacy_override = directory,
        }
    }
}

/// The home directory of whoever runs the program, as .NET's `SpecialFolder.UserProfile` finds it.
pub fn user_home() -> PathBuf {
    std::env::home_dir().unwrap_or_default()
}

/// The system's folder for a program's settings when the host passes none: `$XDG_CONFIG_HOME` where it is absolute, else `~/.config`; `~/Library/Application Support` on macOS; the roaming application data on Windows. Empty where the system has none.
pub fn application_data() -> PathBuf {
    dirs::config_dir().unwrap_or_default()
}

/// What the host said before the tree was first asked for: the program's directory and .NET's application-data folder.
static HOST: OnceLock<(PathBuf, Option<PathBuf>)> = OnceLock::new();
static TREE: OnceLock<RwLock<Tree>> = OnceLock::new();

/// Names the program's directory, which a managed host knows and `current_exe` does not; the first call stands. False when it came too late.
pub fn init(base_directory: &Path, application_data: Option<&Path>) -> bool {
    HOST.set((base_directory.to_path_buf(), application_data.map(Path::to_path_buf))).is_ok()
}

/// `ConfigRoot.ShellModulesDirName`: the folder beside the program that holds the shell's shipped command modules and their manuals.
pub const SHELL_MODULES_DIR_NAME: &str = "dianash";

/// The program's directory: what the host named, else the executable's own (`AppContext.BaseDirectory`).
pub fn base_directory() -> PathBuf {
    HOST.get_or_init(|| (own_directory(), None)).0.clone()
}

/// `ConfigRoot.ShellModulesDirectory`, outside the shell's sandbox as the program's own files are. See EmuSen_DianaSh.md §12.4.
pub fn shell_modules_directory() -> PathBuf {
    base_directory().join(SHELL_MODULES_DIR_NAME)
}

fn own_directory() -> PathBuf {
    std::env::current_exe().ok().and_then(|exe| exe.parent().map(Path::to_path_buf)).unwrap_or_else(|| PathBuf::from("."))
}

fn process_tree() -> &'static RwLock<Tree> {
    TREE.get_or_init(|| {
        let (base, data) = HOST.get_or_init(|| (own_directory(), None)).clone();
        let mac_os = cfg!(target_os = "macos");
        RwLock::new(Tree {
            root: root_for(&base, mac_os, &user_home()),
            seed: seed_directory_for(&base, mac_os),
            application_data: data.unwrap_or_else(application_data),
            ..Tree::default()
        })
    })
}

/// The three redirects a test has scoped to one thread, in place of the process tree's - see EmuSen_Galaxia.md §3.14.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThreadScope {
    pub config: Option<PathBuf>,
    pub data: Option<PathBuf>,
    pub legacy: Option<PathBuf>,
}

thread_local! {
    static SCOPE: RefCell<Option<ThreadScope>> = const { RefCell::new(None) };
}

/// Scopes the redirects to the calling thread, or with `None` returns it to the process tree; no other thread, the library's own included, sees it.
pub fn set_thread_scope(scope: Option<ThreadScope>) {
    SCOPE.with(|s| *s.borrow_mut() = scope);
}

/// The tree a call on this thread resolves against: the process's, or a copy of it under this thread's scope.
pub enum TreeRef {
    Process(RwLockReadGuard<'static, Tree>),
    Scoped(Tree),
}

impl Deref for TreeRef {
    type Target = Tree;
    fn deref(&self) -> &Tree {
        match self {
            TreeRef::Process(guard) => guard,
            TreeRef::Scoped(tree) => tree,
        }
    }
}

/// The process's tree, computed once on first use as `ConfigRoot.Directory` is, under the calling thread's scope when it has one.
pub fn tree() -> TreeRef {
    let process = process_tree().read().unwrap_or_else(|poisoned| poisoned.into_inner());
    match SCOPE.with(|s| s.borrow().clone()) {
        None => TreeRef::Process(process),
        Some(scope) => {
            let mut tree = process.clone();
            tree.config_override = scope.config;
            tree.data_override = scope.data;
            tree.legacy_override = scope.legacy;
            TreeRef::Scoped(tree)
        }
    }
}

/// The process's tree, to move an override.
pub fn tree_mut() -> RwLockWriteGuard<'static, Tree> {
    process_tree().write().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_thread_scope_is_seen_by_its_thread_alone_and_leaves_the_process_tree_as_it_was() {
        let before = tree().directory(Directory::Config);
        set_thread_scope(Some(ThreadScope { config: Some("/scoped/etc".into()), data: Some("/scoped/home".into()), legacy: None }));
        assert_eq!(tree().directory(Directory::Config), Some("/scoped/etc".into()));
        assert_eq!(std::thread::spawn(|| tree().directory(Directory::Config)).join().unwrap(), before);
        set_thread_scope(None);
        assert_eq!(tree().directory(Directory::Config), before);
        assert!(matches!(tree(), TreeRef::Process(_)));
    }
    use crate::test_support::TempDir;

    #[cfg(unix)]
    fn tree_at(root: &str) -> Tree {
        Tree { root: root.into(), application_data: "/appdata".into(), ..Tree::default() }
    }

    #[cfg(unix)]
    #[test]
    fn every_directory_sits_where_the_shell_shows_it() {
        let tree = tree_at("/r");
        let at = |which| tree.directory(which);
        assert_eq!(at(Directory::Root), Some("/r".into()));
        assert_eq!(at(Directory::Seed), None);
        assert_eq!(at(Directory::Config), Some("/r/home/etc/EmuSen".into()));
        assert_eq!(at(Directory::ConfigPrevious), Some("/r/etc/EmuSen".into()));
        assert_eq!(at(Directory::ConfigLegacy), Some("/appdata/EmuSen".into()));
        assert_eq!(at(Directory::Home), Some("/r/home".into()));
        assert_eq!(at(Directory::SaveStates), Some("/r/home/Saves/Save States".into()));
        assert_eq!(at(Directory::Shaders), Some("/r/home/Shaders".into()));
        assert_eq!(at(Directory::LegacyRoot), Some("/r/EmuSen.DianaSh/DianaSh/Usr/Home".into()));
        assert_eq!(tree.config_path(None, "audio.json"), Path::new("/r/home/etc/EmuSen/audio.json"));
        assert_eq!(tree.config_path(Some("cheats"), "x.json"), Path::new("/r/home/etc/EmuSen/cheats/x.json"));
    }

    #[cfg(unix)]
    #[test]
    fn the_two_overrides_do_not_move_each_other() {
        let mut tree = tree_at("/r");
        tree.set_override(Override::Data, Some("/d".into()));
        assert_eq!(tree.home(), Path::new("/d"));
        assert_eq!(tree.saves(), Path::new("/d/Saves"));
        assert_eq!(tree.config(), Path::new("/r/home/etc/EmuSen"));
        tree.set_override(Override::Data, None);
        tree.set_override(Override::Config, Some("/c".into()));
        assert_eq!(tree.config(), Path::new("/c"));
        assert_eq!(tree.home(), Path::new("/r/home"));
        assert_eq!(tree.root, Path::new("/r"));
    }

    #[cfg(unix)]
    #[test]
    fn a_redirected_config_reaches_no_real_legacy_directory() {
        let mut tree = tree_at("/r");
        assert_eq!(tree.legacy_config_path("a.json"), Some("/appdata/EmuSen/a.json".into()));
        tree.set_override(Override::Config, Some("/c".into()));
        assert_eq!(tree.legacy_config_path("a.json"), None);
        tree.set_override(Override::Legacy, Some("/l".into()));
        assert_eq!(tree.legacy_config_path("a.json"), Some("/l/a.json".into()));
    }

    #[test]
    fn the_walk_stops_at_the_nearest_marker_of_either_kind() {
        let temp = TempDir::new("root");
        let nested = temp.join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(temp.join(ROOT_MARKER_FILE_NAME), "").unwrap();
        assert_eq!(root_for(&nested, false, Path::new("unused")), temp.path());
        std::fs::write(temp.join(&format!("a/{SOLUTION_FILE_NAME}")), "").unwrap();
        assert_eq!(root_for(&nested, false, Path::new("unused")), temp.join("a"));
        // A directory named as a marker is not one: File.Exists is false for it.
        std::fs::create_dir_all(temp.join(&format!("a/b/{ROOT_MARKER_FILE_NAME}"))).unwrap();
        assert_eq!(root_for(&nested, false, Path::new("unused")), temp.join("a"));
    }

    #[test]
    fn a_tree_published_under_the_old_names_is_still_found() {
        let temp = TempDir::new("legacy");
        let nested = temp.join("lib/EmuSen");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(temp.join(LEGACY_ROOT_MARKER_FILE_NAME), "").unwrap();
        assert_eq!(root_for(&nested, false, Path::new("unused")), temp.path());

        let unmarked = TempDir::new("legacy-bare");
        let bare = unmarked.join("bare");
        std::fs::create_dir_all(bare.join(LEGACY_PUBLISHED_ROOT_DIR_NAME)).unwrap();
        assert_eq!(root_for(&bare, false, Path::new("unused")), bare.join(LEGACY_PUBLISHED_ROOT_DIR_NAME));
        std::fs::create_dir_all(bare.join(PUBLISHED_ROOT_DIR_NAME)).unwrap();
        assert_eq!(root_for(&bare, false, Path::new("unused")), bare.join(PUBLISHED_ROOT_DIR_NAME));
    }

    #[test]
    fn the_walk_looks_in_ten_directories_and_no_more() {
        let temp = TempDir::new("deep");
        let tenth = temp.join("1/2/3/4/5/6/7/8/9/10");
        std::fs::create_dir_all(&tenth).unwrap();
        std::fs::write(temp.join(ROOT_MARKER_FILE_NAME), "").unwrap();
        assert_eq!(root_for(&tenth, false, Path::new("unused")), tenth.join(PUBLISHED_ROOT_DIR_NAME));
        assert_eq!(root_for(&temp.join("1/2/3/4/5/6/7/8/9"), false, Path::new("unused")), temp.path());
    }

    /// .NET's `SpecialFolder.ApplicationData` is the system's known folder on Windows and does not read the
    /// environment; a host that passes none still finds it with `APPDATA` unset. The test runs itself again without it.
    #[cfg(windows)]
    #[test]
    fn the_application_data_folder_does_not_depend_on_the_environment() {
        const ASKED: &str = "EMUSEN_GALAXIA_TEST_APPLICATION_DATA";
        if std::env::var_os(ASKED).is_some() {
            println!("<<{}>>", application_data().display());
            return;
        }
        let name = "tree::tests::the_application_data_folder_does_not_depend_on_the_environment";
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([name, "--exact", "--nocapture", "--test-threads=1"])
            .env(ASKED, "1")
            .env_remove("APPDATA")
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&child.stdout);
        let without = said.split_once("<<").and_then(|(_, rest)| rest.split_once(">>")).map(|(folder, _)| folder.to_string());
        assert_eq!(without.as_deref(), application_data().to_str(), "{said}");
        assert!(application_data().is_absolute() && application_data() == Path::new(&std::env::var("APPDATA").unwrap()));
    }

    #[test]
    fn a_bundle_roots_in_application_support_before_any_walk() {
        let temp = TempDir::new("bundle");
        std::fs::write(temp.join(SOLUTION_FILE_NAME), "").unwrap();
        let mac_os = temp.join("out/EmuSen.app/Contents/MacOS");
        std::fs::create_dir_all(&mac_os).unwrap();
        let support = mac_data_directory_for(Path::new("/Users/p"));
        assert_eq!(root_for(&mac_os, true, Path::new("/Users/p")), support);
        assert_eq!(root_for(&mac_os.join(""), true, Path::new("/Users/p")), support);
        assert_eq!(root_for(&mac_os, false, Path::new("/Users/p")), temp.path());
        assert_eq!(bundle_contents_for(&mac_os), Some(temp.join("out/EmuSen.app/Contents")));
        assert_eq!(seed_directory_for(&mac_os, true), Some(temp.join("out/EmuSen.app/Contents/Resources/home")));
        assert_eq!(seed_directory_for(&mac_os, false), None);
        assert_eq!(bundle_contents_for(&temp.join("out/EmuSen.app/Contents")), None);
        assert_eq!(root_for(&temp.join("out"), true, Path::new("/Users/p")), temp.path());
    }
}

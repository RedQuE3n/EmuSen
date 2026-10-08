//! Where this program's tree starts and where each kind of file goes in it: `ConfigRoot`, `ConfigStore` and
//! `DataStore` of the C# Galaxia. See EmuSen_Galaxia.md §3 and `man hier`.

use crate::dotnet_path::{self as path, Style};
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// The fallback root beside the binary, when neither marker turns up.
pub const PUBLISHED_ROOT_DIR_NAME: &str = "DianaOSRoot";
/// Written to a published tree's root by the publish.
pub const ROOT_MARKER_FILE_NAME: &str = ".dianaosroot";
/// The checkout's marker.
pub const SOLUTION_FILE_NAME: &str = "EmuSen.sln";
/// The folder under `~/Library/Application Support` that holds a Mac's whole tree.
pub const MAC_DATA_DIR_NAME: &str = "EmuSen";
pub const HOME_DIR_NAME: &str = "home";
pub const CONFIG_DIR_NAME: &str = "etc";
pub const PROGRAM_DIR_NAME: &str = "EmuSen";

/// `File.Exists`: there and not a directory, with a link judged by what it names and a dangling link counted as a file; never a path that ends in a separator.
pub fn file_exists(file: &str) -> bool {
    if file.is_empty() || path::ends_in_separator(Style::HOST, file) {
        return false;
    }
    match std::fs::metadata(file) {
        Ok(metadata) => !metadata.is_dir(),
        Err(_) => std::fs::symlink_metadata(file).is_ok_and(|link| link.file_type().is_symlink()),
    }
}

/// `Directory.Exists`.
pub fn directory_exists(path: &str) -> bool {
    !path.is_empty() && std::fs::metadata(path).is_ok_and(|m| m.is_dir())
}

/// `ConfigRoot.MacDataDirectoryFor`.
pub fn mac_data_directory_for(style: Style, user_home: &str) -> String {
    path::combine_all(style, &[user_home, "Library", "Application Support", MAC_DATA_DIR_NAME])
}

/// `ConfigRoot.BundleContentsFor`: the bundle's `Contents` when `base_directory` is its `Contents/MacOS`.
pub fn bundle_contents_for(base_directory: &str) -> Option<String> {
    let style = Style::HOST;
    let full = path::full_path(base_directory);
    let dir = path::trim_ending_separator(style, &full);
    let contents = path::directory_name(style, dir)?;
    let bundle = path::directory_name(style, &contents)?;
    let in_bundle = path::equals_ignore_case(path::file_name(style, dir), "MacOS")
        && path::equals_ignore_case(path::file_name(style, &contents), "Contents")
        && path::ends_with_ignore_case(&bundle, ".app");
    in_bundle.then_some(contents)
}

/// `ConfigRoot.SeedDirectoryFor`: the read-only skeleton a bundle carries, `Contents/Resources/home`.
pub fn seed_directory_for(base_directory: &str, mac_os: bool) -> Option<String> {
    if !mac_os {
        return None;
    }
    bundle_contents_for(base_directory).map(|contents| path::combine_all(Style::HOST, &[&contents, "Resources", HOME_DIR_NAME]))
}

/// `ConfigRoot.ComputeFor`: the bundle rule, then the walk up to a marker, then the fallback. See EmuSen_Galaxia.md §3.4.
pub fn root_for(base_directory: &str, mac_os: bool, user_home: &str) -> String {
    let style = Style::HOST;
    if mac_os && bundle_contents_for(base_directory).is_some() {
        return mac_data_directory_for(style, user_home);
    }
    let mut dir = path::full_path(base_directory);
    for _ in 0..10 {
        if file_exists(&path::combine(style, &dir, SOLUTION_FILE_NAME)) || file_exists(&path::combine(style, &dir, ROOT_MARKER_FILE_NAME)) {
            return dir;
        }
        match path::directory_name(style, path::trim_ending_separator(style, &dir)) {
            Some(parent) if parent != dir => dir = parent,
            _ => break,
        }
    }
    if mac_os { mac_data_directory_for(style, user_home) } else { path::full_path(&path::combine(style, base_directory, PUBLISHED_ROOT_DIR_NAME)) }
}

/// `DataMigration.LegacyRootFor`: where the tree lived before it collapsed to `<root>/home`.
pub fn legacy_root_for(config_root: &str) -> String {
    path::combine_all(Style::HOST, &[config_root, "EmuSen.DianaOS", "DianaOS", "Usr", "Home"])
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
}

impl Directory {
    pub const ALL: [Directory; 19] = [
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
    pub root: String,
    /// `ConfigRoot.SeedDirectory`.
    pub seed: Option<String>,
    /// .NET's `SpecialFolder.ApplicationData`, under which config sat before Galaxia.
    pub application_data: String,
    /// `ConfigStore.OverrideDirectory`.
    pub config_override: Option<String>,
    /// `DataStore.OverrideDirectory`.
    pub data_override: Option<String>,
    /// `ConfigStore.OverrideLegacyDirectory`.
    pub legacy_override: Option<String>,
}

impl Tree {
    const STYLE: Style = Style::HOST;

    fn under(&self, base: &str, name: &str) -> String {
        path::combine(Self::STYLE, base, name)
    }

    /// `DataStore.UsrHome`: the shell's own `/`.
    pub fn home(&self) -> String {
        self.data_override.clone().unwrap_or_else(|| self.under(&self.root, HOME_DIR_NAME))
    }

    /// `ConfigStore.Directory`, built from the root and not from the data override, so neither redirect moves the other.
    pub fn config(&self) -> String {
        self.config_override.clone().unwrap_or_else(|| path::combine_all(Self::STYLE, &[&self.root, HOME_DIR_NAME, CONFIG_DIR_NAME, PROGRAM_DIR_NAME]))
    }

    /// `ConfigStore.PreviousDirectory`: where config sat before it moved under home.
    pub fn config_previous(&self) -> String {
        path::combine_all(Self::STYLE, &[&self.root, CONFIG_DIR_NAME, PROGRAM_DIR_NAME])
    }

    /// `ConfigStore.LegacyDirectory`.
    pub fn config_legacy(&self) -> String {
        self.legacy_override.clone().unwrap_or_else(|| self.under(&self.application_data, PROGRAM_DIR_NAME))
    }

    /// `ConfigStore.For`, with or without a category.
    pub fn config_path(&self, category: Option<&str>, file_name: &str) -> String {
        match category {
            Some(category) => path::combine_all(Self::STYLE, &[&self.config(), category, file_name]),
            None => self.under(&self.config(), file_name),
        }
    }

    /// `ConfigStore.LegacyPathFor`: none when config is redirected and the legacy directory is not, so a test never reaches a real one.
    pub fn legacy_config_path(&self, file_name: &str) -> Option<String> {
        if self.config_override.is_some() && self.legacy_override.is_none() {
            return None;
        }
        Some(self.under(&self.config_legacy(), file_name))
    }

    /// A named directory; only the seed can be absent.
    pub fn directory(&self, which: Directory) -> Option<String> {
        let home = || self.home();
        Some(match which {
            Directory::Root => self.root.clone(),
            Directory::Seed => return self.seed.clone(),
            Directory::Config => self.config(),
            Directory::ConfigPrevious => self.config_previous(),
            Directory::ConfigLegacy => self.config_legacy(),
            Directory::Home => home(),
            Directory::Logs => self.under(&home(), "Logs"),
            Directory::Saves => self.saves(),
            Directory::SaveStates => self.save_states(),
            Directory::Library => self.under(&home(), "Library"),
            Directory::Screenshots => self.under(&home(), "Screenshots"),
            Directory::Artwork => self.under(&home(), "Artwork"),
            Directory::Media => self.under(&home(), "Media"),
            Directory::Firmware => self.under(&home(), "Firmware"),
            Directory::Shaders => self.under(&home(), "Shaders"),
            Directory::Themes => self.under(&home(), "Themes"),
            Directory::Cheats => self.under(&home(), "Cheats"),
            Directory::Games => self.under(&home(), "Games"),
            Directory::LegacyRoot => legacy_root_for(&self.root),
        })
    }

    /// `DataStore.Saves`.
    pub fn saves(&self) -> String {
        self.under(&self.home(), "Saves")
    }

    /// `DataStore.SaveStates`.
    pub fn save_states(&self) -> String {
        self.under(&self.saves(), "Save States")
    }

    pub fn set_override(&mut self, which: Override, directory: Option<String>) {
        match which {
            Override::Config => self.config_override = directory,
            Override::Data => self.data_override = directory,
            Override::Legacy => self.legacy_override = directory,
        }
    }
}

/// The home directory of whoever runs the program, as .NET's `SpecialFolder.UserProfile` finds it.
pub fn user_home() -> String {
    std::env::home_dir().map(|h| h.to_string_lossy().into_owned()).unwrap_or_default()
}

/// .NET's `SpecialFolder.ApplicationData` when the host passes none: `$XDG_CONFIG_HOME` or `~/.config`, `%APPDATA%` on Windows.
pub fn application_data() -> String {
    if cfg!(windows) {
        return std::env::var("APPDATA").unwrap_or_default();
    }
    let home = user_home();
    if cfg!(target_os = "macos") {
        return path::combine_all(Style::HOST, &[&home, "Library", "Application Support"]);
    }
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(config) if config.starts_with('/') => config,
        _ => path::combine(Style::HOST, &home, ".config"),
    }
}

/// What the host said before the tree was first asked for: the program's directory and .NET's application-data folder.
static HOST: OnceLock<(String, Option<String>)> = OnceLock::new();
static TREE: OnceLock<RwLock<Tree>> = OnceLock::new();

/// Names the program's directory, which a managed host knows and `current_exe` does not; the first call stands. False when it came too late.
pub fn init(base_directory: &str, application_data: Option<&str>) -> bool {
    HOST.set((base_directory.to_string(), application_data.map(str::to_string))).is_ok()
}

fn own_directory() -> String {
    std::env::current_exe().ok().and_then(|exe| exe.parent().map(|dir| dir.to_string_lossy().into_owned())).unwrap_or_else(|| ".".to_string())
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

/// The process's tree, computed once on first use as `ConfigRoot.Directory` is.
pub fn tree() -> RwLockReadGuard<'static, Tree> {
    process_tree().read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The process's tree, to move an override.
pub fn tree_mut() -> RwLockWriteGuard<'static, Tree> {
    process_tree().write().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn tree_at(root: &str) -> Tree {
        Tree { root: root.to_string(), application_data: "/appdata".to_string(), ..Tree::default() }
    }

    #[cfg(unix)]
    #[test]
    fn every_directory_sits_where_the_shell_shows_it() {
        let tree = tree_at("/r");
        let at = |which| tree.directory(which);
        assert_eq!(at(Directory::Root).as_deref(), Some("/r"));
        assert_eq!(at(Directory::Seed), None);
        assert_eq!(at(Directory::Config).as_deref(), Some("/r/home/etc/EmuSen"));
        assert_eq!(at(Directory::ConfigPrevious).as_deref(), Some("/r/etc/EmuSen"));
        assert_eq!(at(Directory::ConfigLegacy).as_deref(), Some("/appdata/EmuSen"));
        assert_eq!(at(Directory::Home).as_deref(), Some("/r/home"));
        assert_eq!(at(Directory::SaveStates).as_deref(), Some("/r/home/Saves/Save States"));
        assert_eq!(at(Directory::Shaders).as_deref(), Some("/r/home/Shaders"));
        assert_eq!(at(Directory::LegacyRoot).as_deref(), Some("/r/EmuSen.DianaOS/DianaOS/Usr/Home"));
        assert_eq!(tree.config_path(None, "audio.json"), "/r/home/etc/EmuSen/audio.json");
        assert_eq!(tree.config_path(Some("cheats"), "x.json"), "/r/home/etc/EmuSen/cheats/x.json");
    }

    #[cfg(unix)]
    #[test]
    fn the_two_overrides_do_not_move_each_other() {
        let mut tree = tree_at("/r");
        tree.set_override(Override::Data, Some("/d".to_string()));
        assert_eq!(tree.home(), "/d");
        assert_eq!(tree.saves(), "/d/Saves");
        assert_eq!(tree.config(), "/r/home/etc/EmuSen");
        tree.set_override(Override::Data, None);
        tree.set_override(Override::Config, Some("/c".to_string()));
        assert_eq!(tree.config(), "/c");
        assert_eq!(tree.home(), "/r/home");
        assert_eq!(tree.root, "/r");
    }

    #[cfg(unix)]
    #[test]
    fn a_redirected_config_reaches_no_real_legacy_directory() {
        let mut tree = tree_at("/r");
        assert_eq!(tree.legacy_config_path("a.json").as_deref(), Some("/appdata/EmuSen/a.json"));
        tree.set_override(Override::Config, Some("/c".to_string()));
        assert_eq!(tree.legacy_config_path("a.json"), None);
        tree.set_override(Override::Legacy, Some("/l".to_string()));
        assert_eq!(tree.legacy_config_path("a.json").as_deref(), Some("/l/a.json"));
    }

    #[test]
    fn the_walk_stops_at_the_nearest_marker_of_either_kind() {
        let temp = TempDir::new("root");
        let nested = temp.join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(temp.join(ROOT_MARKER_FILE_NAME), "").unwrap();
        assert_eq!(root_for(&nested, false, "unused"), temp.path());
        std::fs::write(temp.join(&format!("a/{SOLUTION_FILE_NAME}")), "").unwrap();
        assert_eq!(root_for(&nested, false, "unused"), temp.join("a"));
        // A directory named as a marker is not one: File.Exists is false for it.
        std::fs::create_dir_all(temp.join(&format!("a/b/{ROOT_MARKER_FILE_NAME}"))).unwrap();
        assert_eq!(root_for(&nested, false, "unused"), temp.join("a"));
    }

    #[test]
    fn the_walk_looks_in_ten_directories_and_no_more() {
        let temp = TempDir::new("deep");
        let tenth = temp.join("1/2/3/4/5/6/7/8/9/10");
        std::fs::create_dir_all(&tenth).unwrap();
        std::fs::write(temp.join(ROOT_MARKER_FILE_NAME), "").unwrap();
        assert_eq!(root_for(&tenth, false, "unused"), path::combine(Style::HOST, &tenth, PUBLISHED_ROOT_DIR_NAME));
        assert_eq!(root_for(&temp.join("1/2/3/4/5/6/7/8/9"), false, "unused"), temp.path());
    }

    #[cfg(unix)]
    #[test]
    fn a_file_exists_as_dotnet_says_it_does() {
        let temp = TempDir::new("exists");
        let file = temp.join("file.bin");
        std::fs::write(&file, "x").unwrap();
        std::os::unix::fs::symlink(&file, temp.join("to-file")).unwrap();
        std::os::unix::fs::symlink(temp.path(), temp.join("to-directory")).unwrap();
        std::os::unix::fs::symlink(temp.join("nowhere"), temp.join("dangling")).unwrap();
        assert!(file_exists(&file));
        assert!(file_exists(&temp.join("to-file")));
        assert!(file_exists(&temp.join("dangling")), "a dangling link counts as a file");
        assert!(!file_exists(&temp.join("to-directory")));
        assert!(!file_exists(temp.path()));
        assert!(!file_exists(&format!("{file}/")), "a path that ends in a separator is never a file");
        assert!(!file_exists(&temp.join("absent")));
        assert!(!file_exists(""));
        assert!(directory_exists(&temp.join("to-directory")));
        assert!(directory_exists(&format!("{}/", temp.path())));
        assert!(!directory_exists(&file));
    }

    #[test]
    fn a_bundle_roots_in_application_support_before_any_walk() {
        let temp = TempDir::new("bundle");
        std::fs::write(temp.join(SOLUTION_FILE_NAME), "").unwrap();
        let mac_os = temp.join("out/EmuSen.app/Contents/MacOS");
        std::fs::create_dir_all(&mac_os).unwrap();
        let support = mac_data_directory_for(Style::HOST, "/Users/p");
        assert_eq!(root_for(&mac_os, true, "/Users/p"), support);
        assert_eq!(root_for(&format!("{mac_os}{}", Style::HOST.separator()), true, "/Users/p"), support);
        assert_eq!(root_for(&mac_os, false, "/Users/p"), temp.path());
        assert_eq!(bundle_contents_for(&mac_os), Some(temp.join("out/EmuSen.app/Contents")));
        assert_eq!(seed_directory_for(&mac_os, true), Some(temp.join("out/EmuSen.app/Contents/Resources/home")));
        assert_eq!(seed_directory_for(&mac_os, false), None);
        assert_eq!(bundle_contents_for(&temp.join("out/EmuSen.app/Contents")), None);
        assert_eq!(root_for(&temp.join("out"), true, "/Users/p"), temp.path());
    }
}

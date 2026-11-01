//! The player's `/Cores` followed at each scan where the runtime was given no directories: a data folder moved after
//! the runtime was made is the one the next rescan lists, as `CoreDiscovery.PlayerDirectories` reads `DataStore.Cores`.

use std::path::PathBuf;

use emusen_crystal::runtime::{Config, Runtime};
use emusen_galaxia::tree::{Override, tree_mut};

fn data(root: &std::path::Path) {
    tree_mut().set_override(Override::Data, Some(root.join("Home").to_string_lossy().into_owned()));
    tree_mut().set_override(Override::Config, Some(root.join("Config").to_string_lossy().into_owned()));
}

#[test]
fn a_moved_data_folders_cores_are_the_ones_the_next_rescan_lists() {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("player-cores-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (first, second) = (base.join("first"), base.join("second"));
    data(&first);
    let rt = Runtime::new(&Config { cores: None, player_cores: None, development: Some(false), sqlite: None, lender: None, fetch: None });
    let cores = emusen_galaxia::tree::tree().directory(emusen_galaxia::tree::Directory::Cores).unwrap();
    assert!(cores.starts_with(&*second.parent().unwrap().to_string_lossy()), "{cores}");
    data(&second);
    let moved = PathBuf::from(emusen_galaxia::tree::tree().directory(emusen_galaxia::tree::Directory::Cores).unwrap());
    std::fs::create_dir_all(&moved).unwrap();
    std::fs::write(moved.join(format!("libplayer.{}", if cfg!(target_os = "macos") { "dylib" } else { "so" })), b"a library of the player's").unwrap();
    rt.rescan();
    let listed: Vec<PathBuf> = rt.discovery().unread().iter().map(|u| u.library_path.clone()).collect();
    assert_eq!(listed.len(), 1, "{listed:?}");
    assert!(listed[0].starts_with(&moved), "{listed:?}");
    let _ = std::fs::remove_dir_all(&base);
}

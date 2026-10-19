//! A cartridge's `.bin` shelved by the scan's routing: a 32X's by its header where the 32X is a named system, the
//! folder's where it is not; and the tester's library where `EMUSEN_ROUTING_LIBRARY` names it, read only
//! (EmuSen_Backend.md §32).

use emusen_library::scan::{Systems, route};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn cartridge(system: &str) -> Vec<u8> {
    let mut image = vec![0u8; 0x20000];
    image[0x100..0x110].copy_from_slice(format!("{system:<16}").as_bytes());
    image
}

fn named(ids: &[&str]) -> Systems {
    Systems::named(&ids.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}

#[test]
fn a_32x_cartridge_named_bin_is_shelved_by_its_header_where_the_32x_is_named() {
    let root = std::env::temp_dir().join(format!("emusen-routing-32x-{}", std::process::id()));
    let put = |folder: &str, name: &str, bytes: &[u8]| {
        let at = root.join(folder).join(name);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(&at, bytes).unwrap();
        at.to_string_lossy().into_owned()
    };
    let every = named(&["md", "mcd", "32x", "psx"]);
    let without = named(&["md", "mcd", "psx"]);
    for folder in ["genesis", "sega32x", "roms"] {
        let mars = put(folder, "Mars.bin", &cartridge("SEGA 32X"));
        let genesis = put(folder, "Genesis.bin", &cartridge("SEGA GENESIS"));
        assert_eq!(route(&mars, &every).as_deref(), Some("32x"), "{folder}");
        assert_eq!(route(&genesis, &every).as_deref(), Some("md"), "{folder}");
        assert_eq!(route(&genesis, &without).as_deref(), Some("md"), "{folder}");
        assert_eq!(route(&mars, &without).as_deref(), if folder == "genesis" { Some("md") } else { None }, "{folder}: no named system claims it, so its folder decides");
    }
    let _ = std::fs::remove_dir_all(&root);
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, out);
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("bin")) {
            out.push(path);
        }
    }
}

#[test]
fn the_testers_bin_files_route_by_their_headers() {
    let Some(library) = std::env::var_os("EMUSEN_ROUTING_LIBRARY").map(PathBuf::from).filter(|l| l.is_dir()) else {
        println!("skipped: EMUSEN_ROUTING_LIBRARY names the tester's library, read only");
        return;
    };
    let every = named(&["nes", "snes", "md", "mcd", "32x", "psx", "gb", "gbc", "n64", "sms", "sg1000"]);
    let mut found = Vec::new();
    files(&library, &mut found);
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for path in &found {
        let folder = path.strip_prefix(&library).ok().and_then(|p| p.components().next()).map(|c| c.as_os_str().to_string_lossy().into_owned()).unwrap_or_default();
        let system = route(&path.to_string_lossy(), &every).unwrap_or_else(|| "none".into());
        *counts.entry((folder, system)).or_default() += 1;
    }
    for ((folder, system), n) in &counts {
        println!("{folder} -> {system}: {n}");
    }
    let named_32x = |p: &PathBuf| p.file_name().is_some_and(|n| n.to_string_lossy().to_lowercase().contains("32x"));
    for path in found.iter().filter(|p| p.strip_prefix(&library).is_ok_and(|r| r.starts_with("genesis"))) {
        let system = route(&path.to_string_lossy(), &every);
        assert_eq!(system.as_deref(), Some(if named_32x(path) { "32x" } else { "md" }), "{}", path.display());
    }
}

//! A file's route: the console and engine that open it, and why none does, as the C# `CoreCatalog.Route` and
//! `ManagedRuntime.RouteJson` decide it, routing by content through the same probe and recognisers the library's scan
//! reads a file by (EmuSen_Backend.md §28.7, §29.2). Reads the file; runs no core.

use emusen_core_host::Discovery;
use emusen_core_host::entry::Waiting;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::image_probe::{self, ImageProbe};
use emusen_systems::System;
use emusen_systems::probe::{self, Candidate};

use crate::catalog::{self, Catalog};
use crate::json::Json;

/// The decision with its words: a system, or none with the reason.
pub struct Routed {
    pub system: Option<&'static System>,
    pub reason: Option<String>,
}

/// An extension some console's engine opens: a catalog row's, or a discovered engine's, `IsImageExtension`.
pub fn is_image_extension(catalog: &Catalog, discovery: &Discovery, extension: &str) -> bool {
    catalog.rows.iter().any(|r| r.supports(extension)) || discovery.for_extension(&extension.to_lowercase()).next().is_some()
}

pub fn image_extension(catalog: &Catalog, discovery: &Discovery, at: &str) -> String {
    image_probe::image_extension(at, &|e| is_image_extension(catalog, discovery, e))
}

/// The row a file runs as and the decision, `CoreCatalog.Route`.
pub fn route(catalog: &Catalog, discovery: &Discovery, at: &str) -> (Option<usize>, Option<Routed>) {
    let extension = image_extension(catalog, discovery, at);
    let mut rows: Vec<(Option<usize>, Option<&'static System>)> =
        catalog.rows.iter().enumerate().filter(|(_, r)| r.supports(&extension)).map(|(i, r)| (Some(i), catalog::pack_of(r.console()))).collect();
    for c in discovery.for_extension(&extension) {
        for s in c.sidecar.info.systems.iter().filter(|s| discovery.offers(c, s) && s.extensions.iter().any(|e| e.eq_ignore_ascii_case(&extension)) && takes(c, &s.id, at)) {
            let Some(entry) = emusen_systems::find(&s.id) else { continue };
            if catalog::hand_console_of(&entry.id).is_none()
                && !rows.iter().any(|(r, e)| e.is_some_and(|e| std::ptr::eq(e, entry)) || r.is_some_and(|r| catalog.rows[r].console() == entry.console))
                && entry.extensions.iter().any(|e| e.eq_ignore_ascii_case(&extension))
            {
                rows.push((None, Some(entry)));
            }
        }
    }
    if rows.len() <= 1 {
        let decision = match rows.first() {
            Some((None, entry)) => Some(Routed { system: *entry, reason: None }),
            _ => None,
        };
        return (rows.first().and_then(|(r, _)| *r), decision);
    }
    let candidates: Vec<&'static System> = rows.iter().filter_map(|(_, e)| *e).collect();
    let routed = decide(catalog, discovery, at, &candidates);
    let row = rows.iter().find(|(_, e)| e.is_some_and(|e| routed.system.is_some_and(|s| std::ptr::eq(s, e)))).and_then(|(r, _)| *r);
    (row, Some(routed))
}

/// `SystemRouting.Decide` among the candidates, each a system that takes the file's extension.
fn decide(catalog: &Catalog, discovery: &Discovery, at: &str, candidates: &[&'static System]) -> Routed {
    let claims = if candidates.len() < 2 {
        Vec::new()
    } else {
        match ImageProbe::of(at, &|e| is_image_extension(catalog, discovery, e)) {
            Ok(p) => probe::claims(&candidates.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), &mut |n| p.read(n)),
            Err(_) => Vec::new(),
        }
    };
    let name = path::file_name(Style::HOST, at);
    let asked = candidates.len() >= 2 && claims.len() != 1;
    let folder = if asked { path::directory_name(Style::HOST, &path::full_path(at)).map(|d| path::file_name(Style::HOST, &d).to_string()) } else { None };
    let named: Vec<Candidate> = candidates.iter().map(|s| Candidate { id: &s.id, name: &s.name, esde_system: &s.esde_system }).collect();
    let r = probe::route(name, folder.as_deref(), &named, &claims);
    Routed { system: r.system.as_deref().and_then(emusen_systems::find), reason: r.reason }
}

/// The system a file runs as, the table's id or an engine's own, with the row and the decision.
fn routed(catalog: &Catalog, discovery: &Discovery, at: &str) -> (Option<String>, Option<usize>, Option<Routed>) {
    let (row, decision) = route(catalog, discovery, at);
    let extension = image_extension(catalog, discovery, at);
    let system = decision
        .as_ref()
        .and_then(|d| d.system.map(|s| s.id.clone()))
        .or_else(|| {
            row.map(|r| {
                let console = catalog.rows[r].console();
                catalog::system_ids_for(console)
                    .first()
                    .map(|s| s.to_string())
                    .or_else(|| emusen_systems::table().unwrap_or(&[]).iter().find(|s| s.console == console).map(|s| s.id.clone()))
            })
            .flatten()
        })
        .or_else(|| {
            if decision.is_some() {
                return None;
            }
            discovery.for_extension(&extension).find_map(|c| c.sidecar.info.system_for(&extension).map(|s| s.id.clone()))
        });
    (system, row, decision)
}

/// The system a file runs as, `SystemOfFile`.
pub fn system_of_file(catalog: &Catalog, discovery: &Discovery, at: &str) -> Option<String> {
    routed(catalog, discovery, at).0
}

/// A `.gbc` file, or a `.gb` whose header asks for the Color, `CoreCatalog.IsGameBoyColor`.
fn is_game_boy_color(at: &str) -> bool {
    let byte = image_probe::head_of(at, 0x144).ok().filter(|h| h.len() > 0x143).map(|h| h[0x143]);
    let extension = path::extension(Style::HOST, at);
    if emusen_systems::find("gbc").is_some_and(|s| s.extensions.iter().any(|e| e.eq_ignore_ascii_case(&extension))) {
        return true;
    }
    matches!(byte, Some(0x80 | 0xC0))
}

/// `{"system","console","engine","reason","cartridge","shelf","waiting_default","default_held"}`.
/// `engine_for` is the engine the settings' layers choose, given the file, the row's console and the system; `approved`
/// whether the player's approvals name a hash; `held` why an entry, by its stem, could not be read.
pub fn route_json(
    catalog: &Catalog,
    discovery: &Discovery,
    at: &str,
    engine_for: &dyn Fn(&str, Option<&str>, &str) -> Option<String>,
    approved: &dyn Fn(&str) -> bool,
    held: &dyn Fn(&str) -> Option<String>,
) -> String {
    let (system, row, decision) = routed(catalog, discovery, at);
    let extension = image_extension(catalog, discovery, at);
    let console: Option<String> = row
        .map(|r| catalog.rows[r].console().to_string())
        .or_else(|| decision.as_ref().and_then(|d| d.system.map(|s| s.console.clone())))
        .or_else(|| system.as_deref().and_then(Catalog::discovered_console_for_system).map(str::to_string));
    let chosen = system.as_deref().and_then(|sys| engine_for(at, row.map(|r| catalog.rows[r].console()), sys));
    let waiting = system.as_deref().and_then(|sys| waiting_default(discovery, sys, at, chosen.is_some(), approved));
    let kept_back = system.as_deref().and_then(|sys| default_held(discovery, sys, at, chosen.is_some(), approved, held));
    let engine = system.as_deref().and_then(|sys| {
        chosen
            .clone()
            .or_else(|| console.as_deref().and_then(catalog::sole_engine).map(str::to_string))
            .or_else(|| discovery.for_system(sys).next().map(|c| c.engine_name().to_string()))
    });
    let reason = if system.is_none() {
        Some(decision.as_ref().and_then(|d| d.reason.clone()).unwrap_or_else(|| format!("no engine in this build takes {extension} files")))
    } else {
        None
    };
    let mut j = Json::new();
    j.begin_object().str_or_null("system", system.as_deref()).str_or_null("console", console.as_deref()).str_or_null("engine", engine.as_deref()).str_or_null("reason", reason.as_deref());
    cartridge(&mut j, system.as_deref(), at);
    let shelf = row.map(|r| {
        let row = &catalog.rows[r];
        let colour = row.systems.iter().position(|s| s.id == "gbc").filter(|_| !row.discovered && is_game_boy_color(at));
        colour.map_or_else(|| row.display_name.clone(), |i| row.shelves[i].clone())
    });
    j.str_or_null("shelf", shelf.as_deref());
    match waiting {
        Some((stem, sha256)) => {
            j.key_object("waiting_default").str("stem", &stem).str("sha256", &sha256).end_object();
        }
        None => {
            j.null("waiting_default");
        }
    }
    match kept_back {
        Some((stem, sha256, reason)) => {
            j.key_object("default_held").str("stem", &stem).str("sha256", &sha256).str("reason", &reason).end_object();
        }
        None => {
            j.null("default_held");
        }
    }
    j.end_object();
    j.finish()
}

/// The system's hosted default where it is an installed entry not yet read that takes the file, and no layer chose an
/// engine, so that the route's engine is only the fallback.
fn default_entry<'d>(discovery: &'d Discovery, system: &str, at: &str, chosen: bool) -> Option<(&'static str, &'d Waiting)> {
    let stem = emusen_systems::libretro_cores::default_for(system).filter(|_| !chosen)?;
    if discovery.for_system(system).any(|c| c.entry.as_ref().is_some_and(|e| e.stem == stem)) {
        return None;
    }
    let w = discovery.waiting().iter().find(|w| w.entry.stem == stem)?;
    entry_takes(&w.entry, system, at).then_some((stem, w))
}

/// The default entry where it waits for the player's approval: its stem and its core's hash, which an approval names
/// (EmuSen_LibretroHost.md §10t).
pub fn waiting_default(discovery: &Discovery, system: &str, at: &str, chosen: bool, approved: &dyn Fn(&str) -> bool) -> Option<(String, String)> {
    default_entry(discovery, system, at, chosen).filter(|(_, w)| !approved(&w.sha256)).map(|(stem, w)| (stem.to_string(), w.sha256.clone()))
}

/// The default entry where its core is approved but it could not be read, so the fallback opens the game for a stated
/// reason, never silently: its stem, its core's hash and why (EmuSen_LibretroHost.md §10v).
pub fn default_held(discovery: &Discovery, system: &str, at: &str, chosen: bool, approved: &dyn Fn(&str) -> bool, held: &dyn Fn(&str) -> Option<String>) -> Option<(String, String, String)> {
    let (stem, w) = default_entry(discovery, system, at, chosen).filter(|(_, w)| approved(&w.sha256))?;
    let why = held(stem).unwrap_or_else(|| "it has not been read since its approval".to_string());
    Some((stem.to_string(), w.sha256.clone(), why))
}

/// What the NES pack reads of an iNES header; null for any other system or a file it cannot read.
fn cartridge(j: &mut Json, system: Option<&str>, at: &str) {
    use emusen_systems::nes::header;
    let read = (system == Some("nes")).then(|| std::fs::read(at).ok()).flatten().and_then(|bytes| header::read(&bytes, bytes.len()).ok());
    match read {
        Some(h) => {
            const CHR_BANK: i64 = 0x2000;
            j.key_object("cartridge")
                .int("board", h.mapper as i64)
                .str("header_trust", match h.trust { 2 => "archaic", 1 => "unverifiable", _ => "clean" })
                .int("prg_bytes", h.prg_banks as i64 * header::PRG_BANK)
                .int("chr_bytes", if h.chr_banks == 0 { CHR_BANK } else { h.chr_banks as i64 * CHR_BANK })
                .bool("battery", h.battery)
                .end_object();
        }
        None => {
            j.null("cartridge");
        }
    }
}

/// The v1 engine a file opens on, `CoreFactory.V1Library`: the one asked for by name, else the console's sole engine,
/// else the routed system's first, else the extension's first; the decision's words where several consoles take the
/// file and none is chosen.
pub fn v1_engine<'d>(catalog: &Catalog, discovery: &'d Discovery, at: &str, engine: Option<&str>) -> Result<Option<&'d emusen_core_host::Discovered>, String> {
    let extension = image_extension(catalog, discovery, at);
    let (row, decision) = route(catalog, discovery, at);
    if engine.is_none() && row.is_none() {
        if let Some(Routed { system: None, reason: Some(why) }) = &decision {
            return Err(why.clone());
        }
    }
    let routed = decision.as_ref().and_then(|d| d.system.map(|s| s.id.clone())).or_else(|| {
        row.filter(|r| catalog.rows[*r].discovered)
            .and_then(|r| emusen_systems::table().unwrap_or(&[]).iter().find(|s| s.console == catalog.rows[r].console()).map(|s| s.id.clone()))
    });
    let sole = row.and_then(|r| catalog::sole_engine(catalog.rows[r].console()));
    let claims = |c: &emusen_core_host::Discovered| c.sidecar.info.system_for(&extension).is_some_and(|s| takes(c, &s.id, at));
    let named = |name: &str| discovery.for_extension(&extension).find(|c| c.engine_name() == name && claims(c));
    let found = engine.and_then(named).or_else(|| match sole {
        Some(sole) => named(sole),
        None if engine.is_some() => None,
        None if catalog.rows.iter().any(|r| !r.discovered && r.supports(&extension)) => None,
        None => match &routed {
            Some(system) => discovery.for_system(system).find(|c| takes(c, system, at)),
            None => discovery.for_extension(&extension).find(|c| claims(c)),
        },
    });
    Ok(found)
}

/// Whether an engine takes the game as one of `system`: a hosted core the list names by its installed `.info`'s own
/// extensions (`emusen_systems::libretro_extensions`, EmuSen_LibretroHost.md §4.1a), every other engine as its info claims.
pub fn takes(core: &emusen_core_host::Discovered, system: &str, at: &str) -> bool {
    let Some(entry) = &core.entry else { return true };
    if !emusen_systems::libretro_cores::hosted().iter().any(|h| h.stem == entry.stem) {
        return true;
    }
    entry_takes(entry, system, at)
}

/// Whether an entry the hosted list names takes the game by its installed `.info`'s own extensions.
fn entry_takes(entry: &emusen_core_host::entry::Entry, system: &str, at: &str) -> bool {
    let info = std::fs::read_to_string(entry.info_path()).unwrap_or_default();
    emusen_systems::libretro_extensions::takes(&entry.stem, &emusen_systems::libretro_extensions::info_extensions(&info), system, std::path::Path::new(at))
}

#[cfg(test)]
mod hosted_tests {
    use emusen_core_host::Discovery;
    use emusen_core_host::entry::Entry;

    #[test]
    fn a_listed_hosted_core_takes_only_what_its_installed_info_lists_and_an_unlisted_one_what_its_info_claims() {
        let root = std::env::temp_dir().join(format!("crystal-takes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let cores = root.join("Cores");
        for (stem, info) in [("pcee2", "supported_extensions = \"iso|chd|cso|zso|gz|bin|mdf|nrg|elf|irx\"\n"), ("testcore", "")] {
            let entry = Entry::new(&cores, stem).unwrap();
            std::fs::create_dir_all(&entry.folder).unwrap();
            std::fs::write(entry.info_path(), info).unwrap();
            let sidecar = format!(r#"{{"sidecar":1,"library":"emusen-core-broker.so","sha256":"","abi":"1.17","info":{{"id":"{stem}","name":"{stem}","systems":[{{"id":"ps2","name":"PlayStation 2","extensions":[".iso",".cue"]}}]}}}}"#);
            std::fs::write(emusen_core_host::sidecar::path_for(&entry.broker_path()), sidecar).unwrap();
        }
        let d = Discovery::scan_with_player(&[], &[cores], false);
        let core = |stem: &str| d.found().iter().find(|c| c.entry.as_ref().is_some_and(|e| e.stem == stem)).unwrap();
        assert!(super::takes(core("pcee2"), "ps2", "/games/Disc.ISO"));
        assert!(!super::takes(core("pcee2"), "ps2", "/games/Disc.cue"), "PCEE2's .info lists no cue");
        assert!(super::takes(core("testcore"), "ps2", "/games/Disc.cue"), "a core the list names nowhere takes what its info claims");
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod waiting_default_tests {
    use emusen_core_host::Discovery;
    use emusen_core_host::entry::Entry;

    // The tester's run on 2026-11-01: PCEE2 installed and not approved, Sapphire installed, no engine chosen.
    fn laid(tag: &str) -> (std::path::PathBuf, Discovery, String) {
        let root = std::env::temp_dir().join(format!("crystal-waiting-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (install, cores) = (root.join("Install"), root.join("Cores"));
        std::fs::create_dir_all(&install).unwrap();
        std::fs::write(install.join("libsapphire.so.core.json"), r#"{"sidecar":1,"library":"libsapphire.so","sha256":"","abi":"1.17","info":{"id":"sapphire","name":"Sapphire","systems":[{"id":"ps2","name":"PlayStation 2","extensions":[".iso",".cue"]}]}}"#).unwrap();
        let entry = Entry::new(&cores, "pcee2").unwrap();
        std::fs::create_dir_all(&entry.folder).unwrap();
        std::fs::write(entry.core_path(), b"the core").unwrap();
        std::fs::write(entry.info_path(), "supported_extensions = \"iso|chd|cso|zso|gz|bin|mdf|nrg|elf|irx\"\n").unwrap();
        let d = Discovery::scan_with_player(&[install], &[cores], false);
        let hash = emusen_core_host::entry::core_hash(&entry).unwrap();
        (root, d, hash)
    }

    #[test]
    fn the_systems_waiting_default_is_named_where_no_layer_chose_an_engine() {
        let (root, d, hash) = laid("named");
        assert_eq!(d.for_system("ps2").map(|c| c.engine_name().to_string()).collect::<Vec<_>>(), ["Sapphire"], "the route passes the waiting core over");
        assert_eq!(super::waiting_default(&d, "ps2", "/games/Disc.iso", false, &|_| false), Some(("pcee2".to_string(), hash.clone())));
        assert_eq!(super::waiting_default(&d, "ps2", "/games/Disc.iso", true, &|_| false), None, "the player chose an engine");
        assert_eq!(super::waiting_default(&d, "ps2", "/games/Disc.iso", false, &|h| h == hash), None, "its hash is approved already");
        assert_eq!(super::waiting_default(&d, "ps2", "/games/Disc.cue", false, &|_| false), None, "PCEE2's .info lists no cue");
        assert_eq!(super::waiting_default(&d, "psx", "/games/Disc.iso", false, &|_| false), None, "no hosted default for the system");
        let _ = std::fs::remove_dir_all(&root);
    }

    // The tester's run on 2026-11-01: PCEE2 approved, but its copies were an earlier build's, so it could not be read and Jak opened on Sapphire with nothing said.
    #[test]
    fn an_approved_default_that_could_not_be_read_is_named_with_why() {
        let (root, d, hash) = laid("held");
        let approved = |h: &str| h == hash;
        let why = |stem: &str| (stem == "pcee2").then(|| "the entry needs refresh".to_string());
        assert_eq!(super::default_held(&d, "ps2", "/games/Disc.iso", false, &approved, &why), Some(("pcee2".to_string(), hash.clone(), "the entry needs refresh".to_string())));
        assert!(super::default_held(&d, "ps2", "/games/Disc.iso", false, &approved, &|_| None).is_some_and(|(_, _, w)| !w.is_empty()), "a reason even where none was kept");
        assert_eq!(super::default_held(&d, "ps2", "/games/Disc.iso", false, &|_| false, &why), None, "not approved: it waits, and is asked about");
        assert_eq!(super::default_held(&d, "ps2", "/games/Disc.iso", true, &approved, &why), None, "the player chose an engine");
        assert_eq!(super::default_held(&d, "ps2", "/games/Disc.cue", false, &approved, &why), None, "PCEE2's .info lists no cue");
        let _ = std::fs::remove_dir_all(&root);
    }
}

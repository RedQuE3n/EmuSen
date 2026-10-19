//! A file's route: the console and engine that open it, and why none does, as the C# `CoreCatalog.Route` and
//! `ManagedRuntime.RouteJson` decide it, routing by content through the same probe and recognisers the library's scan
//! reads a file by (EmuSen_Backend.md §28.7, §29.2). Reads the file; runs no core.

use emusen_core_host::Discovery;
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
fn is_image_extension(catalog: &Catalog, discovery: &Discovery, extension: &str) -> bool {
    catalog.rows.iter().any(|r| r.supports(extension)) || discovery.for_extension(&emusen_dotnet::text::to_lower(extension)).next().is_some()
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
        for s in c.sidecar.info.systems.iter().filter(|s| discovery.offered(s) && s.extensions.iter().any(|e| emusen_dotnet::text::equals_ignore_case(e, &extension))) {
            let Some(entry) = emusen_systems::find(&s.id) else { continue };
            if catalog::hand_console_of(&entry.id).is_none()
                && !rows.iter().any(|(r, e)| e.is_some_and(|e| std::ptr::eq(e, entry)) || r.is_some_and(|r| catalog.rows[r].console() == entry.console))
                && entry.extensions.iter().any(|e| emusen_dotnet::text::equals_ignore_case(e, &extension))
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
    if emusen_systems::find("gbc").is_some_and(|s| s.extensions.iter().any(|e| emusen_dotnet::text::equals_ignore_case(e, extension))) {
        return true;
    }
    matches!(byte, Some(0x80 | 0xC0))
}

/// `{"system","console","engine","reason","cartridge","shelf"}`.
/// `engine_for` is the engine the settings' layers choose, given the file, the row's console and the system.
pub fn route_json(catalog: &Catalog, discovery: &Discovery, at: &str, engine_for: &dyn Fn(&str, Option<&str>, &str) -> Option<String>) -> String {
    let (system, row, decision) = routed(catalog, discovery, at);
    let extension = image_extension(catalog, discovery, at);
    let console: Option<String> = row
        .map(|r| catalog.rows[r].console().to_string())
        .or_else(|| decision.as_ref().and_then(|d| d.system.map(|s| s.console.clone())))
        .or_else(|| system.as_deref().and_then(Catalog::discovered_console_for_system).map(str::to_string));
    let engine = system.as_deref().and_then(|sys| {
        engine_for(at, row.map(|r| catalog.rows[r].console()), sys)
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
        if row.display_name == "Game Boy (Mercury)" && !row.discovered && is_game_boy_color(at) { catalog::GAME_BOY_COLOR_SHELF.to_string() } else { row.display_name.clone() }
    });
    j.str_or_null("shelf", shelf.as_deref()).end_object();
    j.finish()
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

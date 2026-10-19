//! What discovery found, as the frontend API's `emusen_crystal_cores` and `emusen_crystal_engines` shape it
//! (`emusen_crystal.h`), so that the C# host's listing and this crate's compare value for value (EmuSen_Backend.md §27.6,
//! and §29.40.5 for 1.3's keys).

use emusen_native::core::jsonw::Json;
use emusen_native::json::Value;

use crate::discovery::Discovery;
use crate::trust::{Standing, Trust};

fn directory(player: bool) -> &'static str {
    if player { "player" } else { "installation" }
}

/// `[{"library","sidecar","sha256","abi","development","engine","report","directory","trust","name","authors","license",
/// "systems"}]`, the libraries with a sidecar first, then the player's not yet read.
pub fn cores(j: &mut Json, d: &Discovery, trust: &Trust) {
    j.begin_array();
    for c in d.found() {
        let s = &c.sidecar;
        let authors: Vec<&str> = s.info_json.get("authors").and_then(Value::as_array).unwrap_or(&[]).iter().filter_map(Value::as_str).collect();
        let systems: Vec<&str> = s.info.systems.iter().map(|x| x.id.as_str()).collect();
        j.begin_object()
            .field_str("library", &s.library_path.to_string_lossy())
            .field_str("sidecar", &s.sidecar_path.to_string_lossy())
            .field_str("sha256", &s.sha256)
            .field_str("abi", &s.abi)
            .field_bool("development", s.development)
            .field_str("engine", c.engine_name())
            .field_str("report", &c.report())
            .field_str("directory", directory(c.player))
            .field_str("trust", c.standing(trust).word())
            .field_str("name", &s.info.name)
            .field_strs("authors", &authors)
            .field_str("license", &s.info.license)
            .field_strs("systems", &systems)
            .end_object();
    }
    for u in d.unread() {
        j.begin_object().field_str("library", &u.library_path.to_string_lossy());
        j.key("sidecar").null().field_str("sha256", &u.sha256);
        j.key("abi").null().field_bool("development", false);
        j.key("engine").null().field_str("report", &u.report()).field_str("directory", directory(true)).field_str("trust", Standing::NotRead.word());
        j.key("name").null().field_strs::<&str>("authors", &[]);
        j.key("license").null().field_strs::<&str>("systems", &[]).end_object();
    }
    j.end_array();
}

/// `[{"name","id","version","systems","available","report","trust"}]`, the systems those offered.
pub fn engines(j: &mut Json, d: &Discovery, trust: &Trust) {
    j.begin_array();
    for c in d.found() {
        let info = &c.sidecar.info;
        let systems: Vec<&str> = info.systems.iter().filter(|s| d.offered(s)).map(|s| s.id.as_str()).collect();
        j.begin_object()
            .field_str("name", c.engine_name())
            .field_str("id", &info.id)
            .field_str("version", &info.version)
            .field_strs("systems", &systems)
            .field_bool("available", c.available())
            .field_str("report", &c.report())
            .field_str("trust", c.standing(trust).word())
            .end_object();
    }
    j.end_array();
}

/// `{"cores":[…],"engines":[…]}`.
pub fn both(d: &Discovery, trust: &Trust) -> String {
    let mut j = Json::new();
    j.begin_object().key("cores");
    cores(&mut j, d, trust);
    j.key("engines");
    engines(&mut j, d, trust);
    j.end_object();
    j.finish()
}

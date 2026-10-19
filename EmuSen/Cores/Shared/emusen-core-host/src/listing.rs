//! What discovery found, as the frontend API's `emusen_crystal_cores` and `emusen_crystal_engines` shape it
//! (`emusen_crystal.h`), so that the C# host's listing and this crate's compare value for value (EmuSen_Backend.md §27.6).

use emusen_native::core::jsonw::Json;

use crate::discovery::Discovery;

/// `[{"library","sidecar","sha256","abi","development","engine","report"}]`.
pub fn cores(j: &mut Json, d: &Discovery) {
    j.begin_array();
    for c in d.found() {
        let s = &c.sidecar;
        j.begin_object()
            .field_str("library", &s.library_path.to_string_lossy())
            .field_str("sidecar", &s.sidecar_path.to_string_lossy())
            .field_str("sha256", &s.sha256)
            .field_str("abi", &s.abi)
            .field_bool("development", s.development)
            .field_str("engine", c.engine_name())
            .field_str("report", &c.report())
            .end_object();
    }
    j.end_array();
}

/// `[{"name","id","version","systems","available","report"}]`, the systems those offered.
pub fn engines(j: &mut Json, d: &Discovery) {
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
            .end_object();
    }
    j.end_array();
}

/// `{"cores":[…],"engines":[…]}`.
pub fn both(d: &Discovery) -> String {
    let mut j = Json::new();
    j.begin_object().key("cores");
    cores(&mut j, d);
    j.key("engines");
    engines(&mut j, d);
    j.end_object();
    j.finish()
}

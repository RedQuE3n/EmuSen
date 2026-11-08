//! What discovery found, as the frontend API's `emusen_crystal_cores` and `emusen_crystal_engines` shape it
//! (`emusen_crystal.h`), so that the C# host's listing and this crate's compare value for value (EmuSen_Backend.md §27.6,
//! and §29.40.5 for 1.3's keys; each row's `libretro` and the entries waiting for approval, EmuSen_LibretroHost.md §10m).

use emusen_native::core::jsonw::Json;
use serde_json::Value;

use crate::descriptors::LibretroFile;
use crate::discovery::Discovery;
use crate::entry::Entry;
use crate::trust::{Standing, Trust};

fn directory(player: bool) -> &'static str {
    if player { "player" } else { "installation" }
}

/// `[{"library","sidecar","sha256","abi","development","engine","report","directory","trust","name","authors","license",
/// "systems","firmware","libretro"}]`, the libraries with a sidecar first, then the player's not yet read, then the libretro entries
/// waiting for the approval that reads them.
pub fn cores(j: &mut Json, d: &Discovery, trust: &Trust) {
    j.begin_array();
    for c in d.found() {
        let s = &c.sidecar;
        let authors: Vec<&str> = s.info_json.get("authors").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
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
            .field_strs("systems", &systems);
        str_or_null(j, "firmware", firmware_need(&s.info_json));
        match &c.entry {
            Some(e) => libretro(j, e, s.info.libretro.as_ref(), Some(&s.info.version), trust),
            None => {
                j.key("libretro").null();
            }
        }
        j.end_object();
    }
    for u in d.unread() {
        j.begin_object().field_str("library", &u.library_path.to_string_lossy());
        j.key("sidecar").null().field_str("sha256", &u.sha256);
        j.key("abi").null().field_bool("development", false);
        j.key("engine").null().field_str("report", &u.report()).field_str("directory", directory(true)).field_str("trust", Standing::NotRead.word());
        j.key("name").null().field_strs::<&str>("authors", &[]);
        j.key("license").null().field_strs::<&str>("systems", &[]);
        j.key("firmware").null().key("libretro").null().end_object();
    }
    let listed = |e: &Entry| d.found().iter().any(|c| c.entry.as_ref().is_some_and(|f| f.folder == e.folder));
    for w in d.waiting().iter().filter(|w| !listed(&w.entry)) {
        let e = &w.entry;
        let proxy = e.broker_path();
        let notice = e.read_notice().ok();
        j.begin_object().field_str("library", &proxy.to_string_lossy());
        j.key("sidecar").null();
        match crate::discovery::scan_hash(&proxy) {
            Some(h) => j.field_str("sha256", &h),
            None => j.key("sha256").null(),
        };
        j.key("abi").null().field_bool("development", false);
        j.key("engine").null().field_str("report", "not yet read: its libretro core waits for approval").field_str("directory", directory(true));
        j.field_str("trust", crate::entry::standing(e, None, trust).word());
        j.field_str("name", notice.as_ref().map_or(e.stem.as_str(), |n| n.name.as_str()));
        j.field_strs("authors", notice.as_ref().map_or(&[][..], |n| n.authors.as_slice()));
        match &notice {
            Some(n) => j.field_str("license", &n.license),
            None => j.key("license").null(),
        };
        j.field_strs("systems", d.hosted_systems(e));
        j.key("firmware").null();
        libretro(j, e, None, None, trust);
        j.end_object();
    }
    j.end_array();
}

/// The firmware a core's info declares for its systems: `required` where one has no open replacement, `optional` where
/// every one has, none where it declares none (EmuSen_Cores.md §3.9).
pub fn firmware_need(info: &Value) -> Option<&'static str> {
    let systems = info.get("systems").and_then(Value::as_array).into_iter().flatten();
    let entries: Vec<&Value> = systems.flat_map(|s| s.get("firmware").and_then(Value::as_array).into_iter().flatten()).collect();
    if entries.is_empty() {
        return None;
    }
    Some(if entries.iter().any(|f| firmware_needed(f)) { "required" } else { "optional" })
}

/// Whether one firmware entry of an info is needed: declared required, or with no open replacement (EmuSen_Cores.md §3.9).
pub fn firmware_needed(entry: &Value) -> bool {
    let replaced = entry.get("replacement").and_then(|r| r.get("effect")).and_then(Value::as_str).is_some_and(|e| e != "none");
    matches!(entry.get("required"), Some(Value::Bool(true))) || !replaced
}

fn str_or_null(j: &mut Json, key: &str, value: Option<&str>) {
    match value {
        Some(v) => j.field_str(key, v),
        None => j.key(key).null(),
    };
}

/// A row's `libretro` (EmuSen_Cores.md §7.3): the entry's folder, its core's file and SHA-256 as the file is now, what
/// its `NOTICE` records, the version the core reports where it was read, and the file's own standing.
fn libretro(j: &mut Json, e: &Entry, sidecar: Option<&LibretroFile>, version: Option<&str>, trust: &Trust) {
    let core = e.core_path();
    let file = core.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let sha256 = crate::entry::core_hash(e);
    let notice = e.read_notice().ok();
    let n = |f: fn(&emusen_core_entry::Notice) -> &str| notice.as_ref().map(f);
    let own = match &sha256 {
        Some(h) if trust.approved(h) && sidecar.is_none_or(|l| l.sha256 == *h) => Standing::Approved,
        _ => Standing::NeedsApproval,
    };
    j.key("libretro").begin_object().field_str("folder", &e.folder.to_string_lossy()).field_str("file", &file);
    str_or_null(j, "sha256", sha256.as_deref());
    str_or_null(j, "corename", n(|x| &x.corename));
    str_or_null(j, "name", n(|x| &x.name));
    str_or_null(j, "version", version);
    str_or_null(j, "license", n(|x| &x.license));
    j.field_strs("authors", notice.as_ref().map_or(&[][..], |x| x.authors.as_slice()));
    str_or_null(j, "source", n(|x| &x.source));
    str_or_null(j, "build", n(|x| &x.build));
    j.field_str("trust", own.word()).end_object();
}

/// `[{"name","id","version","systems","available","report","trust"}]`, the systems those offered.
pub fn engines(j: &mut Json, d: &Discovery, trust: &Trust) {
    j.begin_array();
    for c in d.found() {
        let info = &c.sidecar.info;
        let systems: Vec<&str> = info.systems.iter().filter(|s| d.offers(c, s)).map(|s| s.id.as_str()).collect();
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

#[cfg(test)]
mod tests {
    use super::firmware_need;
    use serde_json::{Value, from_str};

    fn need(firmware: &str) -> Option<&'static str> {
        firmware_need(&from_str::<Value>(&format!(r#"{{"systems":[{{"id":"ps2","firmware":{firmware}}}]}}"#)).unwrap())
    }

    #[test]
    fn a_core_needs_firmware_it_cannot_run_without_and_can_use_firmware_it_can() {
        assert_eq!(need("[]"), None);
        assert_eq!(need(r#"[{"label":"BIOS","required":false,"replacement":{"effect":"none","cost":"runs nothing"}}]"#), Some("required"));
        assert_eq!(need(r#"[{"label":"BIOS","required":true,"replacement":{"effect":"accuracy","cost":"c"}}]"#), Some("required"));
        assert_eq!(need(r#"[{"label":"BIOS","required":false}]"#), Some("required"));
        assert_eq!(need(r#"[{"label":"BIOS","required":false,"replacement":{"effect":"accuracy","cost":"c"}}]"#), Some("optional"));
        assert_eq!(firmware_need(&from_str::<Value>("{}").unwrap()), None);
    }
}

//! One core as Cores lists it, from the runtime's `cores` listing or the core tool's `core` event, and the two joined
//! (EmuSen_Cores.md §4.1, §7.3; EmuSen_Interface.md §15.3), as C#'s `CoreRow` reads them.

use emusen_crystal_client::shapes::CoreRow as RuntimeCore;
use serde_json::Value;
use std::collections::HashMap;

/// Where a core comes from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Origin {
    /// Ships with EmuSen.
    #[default]
    Project,
    /// libretro's buildbot.
    Libretro,
    /// The player's own.
    Added,
}

/// One row of the list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Core {
    /// `libretro:<stem>`, `engine:<engine>` or `library:<path>`: the row across refreshes.
    pub key: String,
    pub name: String,
    /// A libretro core's name as its `.info` lists it, where that differs from the name shown.
    pub listed_as: Option<String>,
    pub origin: Origin,
    /// The core tool's state word (installed, update, available, unsupported, added), or for a runtime core "built in"
    /// or its trust.
    pub state: String,
    pub systems: Vec<String>,
    /// The consoles its systems belong to, as the runtime names them, and as a list shows them.
    pub consoles: Vec<String>,
    pub console_names: Vec<String>,
    pub version: String,
    pub reason: Option<String>,
    pub licence: String,
    pub authors: Vec<String>,
    pub size: Option<u64>,
    pub stem: Option<String>,
    /// The runtime's engine name, which Make Default needs.
    pub engine: Option<String>,
    pub library: Option<String>,
    /// The hash an approval names: the library's, or an installed libretro core's file's.
    pub sha256: Option<String>,
    /// approved, manifest, needs_approval, not_read, needs_refresh; none where the runtime does not list it.
    pub trust: Option<String>,
    pub is_default: bool,
    pub development: bool,
    /// The firmware its info declares: "required", "optional", or none.
    pub firmware: Option<String>,
}

impl Core {
    pub fn is_libretro(&self) -> bool {
        self.origin == Origin::Libretro
    }

    pub fn installed(&self) -> bool {
        if self.is_libretro() {
            matches!(self.state.as_str(), "installed" | "update")
        } else {
            self.library.is_some()
        }
    }

    pub fn awaits_approval(&self) -> bool {
        self.trust.as_deref() == Some("needs_approval")
    }
}

/// The systems table as Cores reads it: each system's console, and each console's display name, maker and firmware.
#[derive(Clone, Debug, Default)]
pub struct Consoles {
    pub console_of: HashMap<String, String>,
    pub name_of: HashMap<String, String>,
}

impl Consoles {
    /// The consoles of a core's systems; a system the table does not name names no console.
    pub fn of(&self, systems: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let listed = |s: &String| if self.console_of.is_empty() { Some(s.clone()) } else { self.console_of.get(s).cloned() };
        for c in systems.iter().filter_map(listed) {
            if !out.contains(&c) {
                out.push(c);
            }
        }
        out
    }

    /// A console as a list shows it, the table's display name where it has one.
    pub fn name(&self, console: &str) -> String {
        self.name_of.get(console).cloned().unwrap_or_else(|| console.to_owned())
    }

    fn names(&self, consoles: &[String]) -> Vec<String> {
        consoles.iter().map(|c| self.name(c)).collect()
    }
}

fn text(e: &Value, name: &str) -> Option<String> {
    e.get(name).and_then(Value::as_str).map(str::to_owned)
}

fn strings(e: &Value, name: &str) -> Vec<String> {
    e.get(name).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default()
}

/// A row of the core tool's `core` event (EmuSen_Cores.md §6.1); none for any other event.
pub fn of_tool(e: &Value, consoles: &Consoles) -> Option<Core> {
    if text(e, "event").as_deref() != Some("core") {
        return None;
    }
    let stem = text(e, "stem")?;
    let systems = strings(e, "systems");
    let on = consoles.of(&systems);
    let corename = text(e, "corename").filter(|c| !c.is_empty());
    let listed = text(e, "name");
    let state = text(e, "state").unwrap_or_default();
    let version: Vec<String> = [text(e, "version"), text(e, "build")].into_iter().flatten().filter(|v| !v.is_empty()).collect();
    Some(Core {
        key: format!("libretro:{stem}"),
        name: corename.clone().or_else(|| listed.clone()).unwrap_or_else(|| stem.clone()),
        listed_as: listed.filter(|l| Some(l) != corename.as_ref()),
        origin: if state == "added" { Origin::Added } else { Origin::Libretro },
        state,
        reason: text(e, "reason"),
        console_names: consoles.names(&on),
        consoles: on,
        systems,
        version: version.join(" "),
        licence: text(e, "license").unwrap_or_default(),
        authors: strings(e, "authors"),
        size: e.get("size").and_then(Value::as_u64),
        stem: Some(stem),
        ..Core::default()
    })
}

/// A row of the runtime's `cores` listing; `is_default` says whether an engine is a console's, and `size` reads a
/// library's bytes.
pub fn of_runtime(c: &RuntimeCore, consoles: &Consoles, is_default: &dyn Fn(&str, &str) -> bool, size: &dyn Fn(&str) -> Option<u64>) -> Option<Core> {
    let library = c.library.clone()?;
    let engine = c.engine.clone().unwrap_or_default();
    let shipped = c.directory.as_deref() == Some("installation");
    let on = consoles.of(&c.systems);
    let trust = c.trust.clone().unwrap_or_default();
    let file = std::path::Path::new(&library).file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    Some(Core {
        key: if engine.is_empty() { format!("library:{library}") } else { format!("engine:{engine}") },
        name: c.name.clone().unwrap_or(file),
        origin: if shipped { Origin::Project } else { Origin::Added },
        state: if shipped { "built in".to_owned() } else { trust.clone() },
        systems: c.systems.clone(),
        console_names: consoles.names(&on),
        licence: c.license.clone().unwrap_or_default(),
        authors: c.authors.clone(),
        size: size(&library),
        // A core waiting for approval cannot run, so it is no console's default until approved (EmuSen_Cores.md §3.8).
        is_default: !engine.is_empty() && trust != "needs_approval" && on.iter().any(|console| is_default(console, &engine)),
        consoles: on,
        engine: (!engine.is_empty()).then_some(engine),
        sha256: c.libretro.as_ref().map(|l| l.sha256.clone()).or_else(|| c.sha256.clone()),
        trust: Some(trust),
        development: c.development.unwrap_or(false),
        firmware: c.firmware.clone(),
        library: Some(library),
        ..Core::default()
    })
}

/// The runtime's rows and the tool's joined: a libretro core discovery lists under `libretro/<stem>` lends the tool's
/// row its engine, trust, library, default and hash, and is not listed twice (EmuSen_Cores.md §7.3).
pub fn joined(runtime: &[Core], tool: &[Core]) -> Vec<Core> {
    use std::path::Path;
    let folder = |c: &Core| c.library.as_deref().and_then(|l| Path::new(l).parent()).and_then(Path::file_name).map(|f| f.to_string_lossy().into_owned());
    let in_libretro = |c: &Core| {
        c.library.as_deref().and_then(|l| Path::new(l).parent()).and_then(Path::parent).and_then(Path::file_name).is_some_and(|f| f == "libretro")
    };
    let mut rows: Vec<Core> = runtime.iter().filter(|r| !in_libretro(r)).cloned().collect();
    for t in tool {
        let found = runtime.iter().find(|r| in_libretro(r) && folder(r) == t.stem);
        rows.push(match found {
            Some(r) => Core {
                engine: r.engine.clone(),
                trust: r.trust.clone(),
                library: r.library.clone(),
                is_default: r.is_default,
                sha256: r.sha256.clone(),
                ..t.clone()
            },
            None => t.clone(),
        });
    }
    rows
}

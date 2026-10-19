//! A console's core as the scraper needs it, joined from two owners (EmuSen_Backend.md §30.12): which systems one
//! engine runs together, the runtime's fact the host passes as `cores`; and each system's console, ES-DE name,
//! libretro folders and OpenVGDB names, the systems table's, read from `emusen-systems`. A core's console is its first
//! system's, its libretro folders and OpenVGDB names the union of its systems', as the C# `ConsoleCore` joins them.

use emusen_systems::System;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Core {
    pub console: String,
    pub systems: Vec<String>,
    pub cheat_systems: Vec<String>,
    pub openvgdb_systems: Vec<String>,
}

fn table() -> &'static [System] {
    emusen_systems::table().unwrap_or(&[])
}

/// The table's system an ES-DE name is, the first that carries it.
pub fn by_esde(esde: &str) -> Option<&'static System> {
    table().iter().find(|s| s.esde_system == esde)
}

/// Each group of system ids the host named, as a core; ids the table lacks are passed over.
pub fn cores(groups: &[Vec<String>]) -> Vec<Core> {
    let mut out = Vec::new();
    for group in groups {
        let systems: Vec<&System> = group.iter().filter_map(|id| emusen_systems::find(id)).collect();
        let Some(first) = systems.first() else { continue };
        let mut cheat_systems: Vec<String> = Vec::new();
        let mut openvgdb_systems: Vec<String> = Vec::new();
        for s in &systems {
            for c in &s.cheat_systems {
                if !cheat_systems.contains(c) {
                    cheat_systems.push(c.clone());
                }
            }
            for o in &s.openvgdb_systems {
                if !openvgdb_systems.contains(o) {
                    openvgdb_systems.push(o.clone());
                }
            }
        }
        out.push(Core { console: first.console.clone(), systems: systems.iter().map(|s| s.id.clone()).collect(), cheat_systems, openvgdb_systems });
    }
    out
}

/// The core that runs a system, by its id.
pub fn core_of<'a>(cores: &'a [Core], system_id: &str) -> Option<&'a Core> {
    cores.iter().find(|c| c.systems.iter().any(|s| s == system_id))
}

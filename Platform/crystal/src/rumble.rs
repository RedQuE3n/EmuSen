//! Each port's motors as the core last drove them, from its `OUTPUT` events, for a frontend to hand to the player's pad,
//! read from any thread without the session's gate; held at zero where the console's games have no rumble or the
//! player's switch is off (EmuSen_Input.md §9, EmuSen_CoreAPI_Extensions.md §4.4).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::machine_info::{MachineInfo, Motor};

/// The ports a frontend can seat a player at, `PlayerSlots.MaxPlayers`.
pub const PORTS: usize = 8;

#[derive(Debug, Default)]
pub struct Rumble {
    /// Each port's strong motor in the high sixteen bits and its weak one in the low.
    levels: [AtomicU32; PORTS],
    /// The systems table says the console's games drive a motor; fixed as the session opens.
    table: AtomicBool,
    /// The player's switch, read as the session opens and set again by `input_rumble_switch`.
    switch: AtomicBool,
}

impl Rumble {
    /// Whether the motors reach the pads: the systems table says the console's games drive one and the switch is on.
    pub fn set_gate(&self, table: bool, switch: bool) {
        self.table.store(table, Ordering::Relaxed);
        self.switch.store(switch, Ordering::Relaxed);
    }

    /// The player's switch turned while the game runs (frontend API 1.11); the table's half is unchanged.
    pub fn set_switch(&self, on: bool) {
        self.switch.store(on, Ordering::Relaxed);
    }

    pub fn is_open(&self) -> bool {
        self.table.load(Ordering::Relaxed) && self.switch.load(Ordering::Relaxed)
    }

    /// A port's strong and weak levels, 0 to 65535; zero while closed or past the ports.
    pub fn level(&self, port: u32) -> (u16, u16) {
        match self.levels.get(port as usize) {
            Some(both) if self.is_open() => {
                let both = both.load(Ordering::Relaxed);
                ((both >> 16) as u16, both as u16)
            }
            _ => (0, 0),
        }
    }

    /// An `OUTPUT` event: `a` is `(port << 32) | output`, `b` its level; one machine info names no motor is ignored.
    pub fn output(&self, mi: &MachineInfo, a: i64, b: i64) {
        let (port, output) = ((a as u64 >> 32) as u32, a as u32);
        let motor = mi.ports.iter().find(|p| p.port == port).and_then(|p| p.outputs.iter().find(|o| o.0 == output)).map(|o| o.1);
        let (Some(slot), Some(motor)) = (self.levels.get(port as usize), motor) else { return };
        let shift = if motor == Motor::Strong { 16 } else { 0 };
        let level = (b.clamp(0, u16::MAX as i64) as u32) << shift;
        // The engine is the one writer, under its lock.
        slot.store((slot.load(Ordering::Relaxed) & !(0xFFFF << shift)) | level, Ordering::Relaxed);
    }

    /// Machine info changed: every motor stops until the core raises its outputs again.
    pub fn clear(&self) {
        for slot in &self.levels {
            slot.store(0, Ordering::Relaxed);
        }
    }
}

/// `AppSettings.Rumble`, the player's switch, on unless the file says otherwise; read as a session opens.
pub fn switch(tree: &emusen_galaxia::tree::Tree) -> bool {
    let model = &emusen_galaxia::models::MODELS[emusen_galaxia::models::APP_SETTINGS_ID as usize];
    let (settings, _) = emusen_galaxia::config::load_or_new(tree, model, None, emusen_galaxia::models::APP_SETTINGS_FILE);
    settings.get("Rumble").and_then(|v| v.as_bool()).unwrap_or(true)
}

/// Whether the systems table says a system's games drive a motor; false for a system with no row.
pub fn in_table(system: &str) -> bool {
    emusen_systems::find(system).is_some_and(|s| s.rumble)
}

/// The gate as the table and the switch decide it, for a system by its id.
pub fn allowed(system: &str, switch: bool) -> bool {
    switch && in_table(system)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine_info::Port;

    fn info() -> MachineInfo {
        MachineInfo::read(r#"{"system":"n64","ports":[{"port":0,"outputs":[{"output":0,"kind":"motor","motor":"strong"},{"output":1,"kind":"motor","motor":"weak"},{"output":2,"kind":"light"}]},{"port":1}]}"#)
    }

    #[test]
    fn machine_info_names_each_ports_motors_and_a_light_is_none() {
        let mi = info();
        assert_eq!(mi.ports[0], Port { port: 0, controller: None, outputs: vec![(0, Motor::Strong), (1, Motor::Weak)] });
        assert!(mi.ports[1].outputs.is_empty());
    }

    #[test]
    fn each_motor_keeps_its_own_half_and_nothing_reaches_a_pad_while_closed() {
        let (r, mi) = (Rumble::default(), info());
        r.output(&mi, 0, 40000);
        r.output(&mi, 1, 7);
        r.output(&mi, 2, 65535);
        assert_eq!(r.level(0), (0, 0), "closed");
        r.set_gate(true, true);
        assert_eq!(r.level(0), (40000, 7));
        r.output(&mi, 0, 0);
        assert_eq!(r.level(0), (0, 7));
        r.output(&mi, (1 << 32) | 0, 500);
        assert_eq!((r.level(1), r.level(99)), ((0, 0), (0, 0)), "port 1 declares no motor; port 99 is past the ports");
        r.clear();
        assert_eq!(r.level(0), (0, 0));
    }

    #[test]
    fn the_switch_turned_mid_game_closes_and_opens_the_gate_but_never_past_the_table() {
        let (r, mi) = (Rumble::default(), info());
        r.set_gate(true, true);
        r.output(&mi, 0, 30000);
        r.set_switch(false);
        assert_eq!(r.level(0), (0, 0), "switched off");
        r.set_switch(true);
        assert_eq!(r.level(0), (30000, 0), "on again, the level the core last set");
        r.set_gate(false, true);
        r.set_switch(true);
        assert_eq!(r.level(0), (0, 0), "the table says no motor");
    }

    #[test]
    fn the_gate_needs_the_tables_rumble_and_the_switch() {
        assert!(allowed("n64", true) && allowed("psx", true));
        assert!(!allowed("n64", false), "switch off");
        assert!(!allowed("snes", true), "the table says the SNES has none");
        assert!(!allowed("test", true), "no row");
    }
}

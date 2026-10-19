//! The Genesis family's pack code: its cheat codes, the Sega CD's backup RAM module, and the recognisers that tell a
//! cartridge from a disc, a 32X cartridge from a Genesis one, and a Sega CD's disc from another's.

pub mod cheats;
pub mod bram;

use crate::probe::{Head, Need, Recognition, SYNC};

/// A compact disc's image rather than a cartridge's: a raw track's sync, or an .iso's volume descriptor at sector 16.
pub fn is_compact_disc(head: &[u8]) -> bool {
    (head.len() >= 12 && head[..12] == SYNC) || (head.len() >= 0x8006 && &head[0x8001..0x8006] == b"CD001")
}

/// A Sega CD's disc: SEGADISCSYSTEM at the start of its first sector's data.
pub fn is_sega_cd(head: &[u8]) -> bool {
    let data = if head.len() >= 16 && head[..12] == SYNC { if head[15] == 1 { 16 } else { 24 } } else { 0 };
    head.len() >= data + 14 && &head[data..data + 14] == b"SEGADISCSYSTEM"
}

/// A 32X cartridge's header: `32X` in its system name at $100, which a file too short to hold the header has not; Nephrite's
/// rule (`s32x::is_32x`, Nephrite_Native.md §2.2).
pub fn is_32x(head: &[u8]) -> bool {
    head.len() >= 0x200 && head[0x100..0x110].windows(3).any(|w| w == b"32X")
}

/// A cartridge, no compact disc, of the 32X or not.
pub fn is_cartridge(head: &[u8], s32x: bool) -> bool {
    !is_compact_disc(head) && is_32x(head) == s32x
}

/// The Mega Drive claims a cartridge whose header does not name the 32X, the 32X one whose header does; the Sega CD what
/// holds its system's name (Nephrite_OpenBios.md §11, EmuSen_Backend.md §32).
pub fn recognise(system: &str, head: &Head) -> Result<Option<Recognition>, Need> {
    let claim = |id: &str| Some(Recognition { system: id.to_string(), serial: None, region: None });
    Ok(match system {
        "md" => if is_cartridge(head.read(0x8010)?, false) { claim("md") } else { None },
        "32x" => if is_cartridge(head.read(0x8010)?, true) { claim("32x") } else { None },
        "mcd" => if is_sega_cd(head.read(0x40)?) { claim("mcd") } else { None },
        _ => None,
    })
}


/// Each player's pad from the two port settings: port 1's pads, then port 2's unless a 4 Way Play has it, the C#
/// `GenesisSystems.PlayerControllers` (EmuSen_Settings_Reference.md §4.101).
pub fn player_controllers(setting: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    fn plug(value: Option<&str>) -> (usize, &'static str, bool) {
        match value {
            Some("md.pad6") => (1, "md.pad6", false),
            Some("md.teamplayer3") => (4, "md.pad3", false),
            Some("md.teamplayer6") => (4, "md.pad6", false),
            Some("md.4way3") => (4, "md.pad3", true),
            Some("md.4way6") => (4, "md.pad6", true),
            _ => (1, "md.pad3", false),
        }
    }
    let first = plug(setting("pad1").as_deref());
    let mut players = vec![first.1.to_string(); first.0];
    if !first.2 {
        let second = plug(setting("pad2").as_deref());
        players.extend(std::iter::repeat_n(second.1.to_string(), if second.2 { 1 } else { second.0 }));
    }
    players
}

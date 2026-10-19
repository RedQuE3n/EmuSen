//! The PlayStation's pack code: the memory card's module, and the Shift-JIS its titles are written in.

pub mod cheats;
pub mod card;
mod cp932;
pub mod disc;
pub mod shift_jis;

/// Each player's pad from the two port settings; a port set to nothing has no player (`PsxSystem.PlayerControllers`).
pub fn player_controllers(setting: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    ["pad1", "pad2"].iter().map(|key| setting(key).unwrap_or_else(|| "psx.dualshock".to_string())).filter(|pad| pad != "none").collect()
}

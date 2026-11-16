//! The machine's battery, read from the kernel's power_supply class on every call, nothing kept (EmuSen_RustPlatform.md §58).

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The kernel's power_supply class, where Linux lists every supply.
pub const SYSTEM_ROOT: &str = "/sys/class/power_supply";

/// The machine's battery: its level from 0 to 100 and whether it is charging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Battery {
    pub level: u8,
    pub charging: bool,
}

/// The machine's battery as the kernel reports it now; none where this system has no reader or no battery of the machine's.
pub fn battery() -> Option<Battery> {
    if cfg!(target_os = "linux") { battery_at(Path::new(SYSTEM_ROOT)) } else { None }
}

/// The machine's battery read from a power_supply tree at `root`: the mean of its batteries' levels, charging while none
/// discharges and one charges or is full (§58.2).
pub fn battery_at(root: &Path) -> Option<Battery> {
    let entries = std::fs::read_dir(root).ok()?;
    let mut at = PathBuf::from(root);
    let (mut sum, mut count) = (0u32, 0u32);
    let (mut feeding, mut draining) = (false, false);
    for entry in entries.flatten() {
        at.push(entry.file_name());
        if let Some((level, status)) = system_battery(&mut at) {
            sum += u32::from(level);
            count += 1;
            feeding |= matches!(status, Status::Charging | Status::Full);
            draining |= status == Status::Discharging;
        }
        at.pop();
    }
    (count > 0).then(|| Battery { level: ((sum + count / 2) / count) as u8, charging: feeding && !draining })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Charging,
    Full,
    Discharging,
    Other,
}

/// One supply's level and status, if it is a battery of the machine's that is present and says its level (§58.1).
fn system_battery(at: &mut PathBuf) -> Option<(u8, Status)> {
    let mut buffer = [0u8; 32];
    if attribute(at, "type", &mut buffer)? != b"Battery" {
        return None;
    }
    if attribute(at, "scope", &mut buffer) == Some(b"Device") {
        return None;
    }
    if attribute(at, "present", &mut buffer) == Some(b"0") {
        return None;
    }
    let level = std::str::from_utf8(attribute(at, "capacity", &mut buffer)?).ok()?.parse::<u8>().ok()?.min(100);
    let status = match attribute(at, "status", &mut buffer) {
        Some(b"Charging") => Status::Charging,
        Some(b"Full") => Status::Full,
        Some(b"Discharging") => Status::Discharging,
        _ => Status::Other,
    };
    Some((level, status))
}

/// An attribute's value without its line end, read into `buffer`; none where the supply has no such attribute.
fn attribute<'a>(at: &mut PathBuf, name: &str, buffer: &'a mut [u8; 32]) -> Option<&'a [u8]> {
    at.push(name);
    let read = File::open(&*at).and_then(|mut f| f.read(buffer));
    at.pop();
    let n = read.ok()?;
    Some(buffer[..n].trim_ascii_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A made-up power_supply tree: each supply a folder of its attributes.
    fn tree(name: &str, supplies: &[(&str, &[(&str, &str)])]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("galaxia-power-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (supply, attributes) in supplies {
            let folder = root.join(supply);
            std::fs::create_dir_all(&folder).unwrap();
            for (key, value) in *attributes {
                std::fs::write(folder.join(key), format!("{value}\n")).unwrap();
            }
        }
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    const MOUSE: (&str, &[(&str, &str)]) =
        ("hidpp_battery_0", &[("type", "Battery"), ("scope", "Device"), ("capacity", "55"), ("status", "Discharging")]);
    const MAINS: (&str, &[(&str, &str)]) = ("ACAD", &[("type", "Mains"), ("online", "1")]);

    #[test]
    fn no_supplies_and_no_tree_are_no_battery() {
        assert_eq!(battery_at(&tree("none", &[])), None);
        assert_eq!(battery_at(Path::new("/nonexistent/power_supply")), None);
    }

    #[test]
    fn a_peripheral_s_battery_is_not_the_machine_s() {
        assert_eq!(battery_at(&tree("mouse", &[MOUSE, MAINS])), None);
    }

    #[test]
    fn a_system_battery_discharging_is_its_level_not_charging() {
        let bat = ("BAT0", &[("type", "Battery"), ("scope", "System"), ("present", "1"), ("capacity", "82"), ("status", "Discharging")][..]);
        assert_eq!(battery_at(&tree("discharging", &[MOUSE, MAINS, bat])), Some(Battery { level: 82, charging: false }));
    }

    #[test]
    fn a_battery_with_no_scope_or_an_unknown_one_is_the_machine_s() {
        let bare = ("BAT0", &[("type", "Battery"), ("capacity", "40"), ("status", "Discharging")][..]);
        assert_eq!(battery_at(&tree("bare", &[bare])), Some(Battery { level: 40, charging: false }));
        let unknown = ("BAT1", &[("type", "Battery"), ("scope", "Unknown"), ("capacity", "41"), ("status", "Discharging")][..]);
        assert_eq!(battery_at(&tree("unknown", &[unknown])), Some(Battery { level: 41, charging: false }));
    }

    #[test]
    fn charging_and_full_are_charging() {
        for (status, name) in [("Charging", "charging"), ("Full", "full")] {
            let bat = ("BAT0", &[("type", "Battery"), ("capacity", "82"), ("status", status)][..]);
            assert_eq!(battery_at(&tree(name, &[bat])), Some(Battery { level: 82, charging: true }), "{status}");
        }
        let idle = ("BAT0", &[("type", "Battery"), ("capacity", "80"), ("status", "Not charging")][..]);
        assert_eq!(battery_at(&tree("idle", &[idle])), Some(Battery { level: 80, charging: false }));
    }

    #[test]
    fn an_absent_battery_or_one_without_a_level_is_skipped() {
        let absent = ("BAT0", &[("type", "Battery"), ("present", "0"), ("capacity", "0"), ("status", "Unknown")][..]);
        let mute = ("BAT1", &[("type", "Battery"), ("status", "Discharging")][..]);
        assert_eq!(battery_at(&tree("absent", &[absent, mute])), None);
    }

    #[test]
    fn two_system_batteries_are_their_mean_and_charge_only_while_none_drains() {
        let a = ("BAT0", &[("type", "Battery"), ("capacity", "90"), ("status", "Full")][..]);
        let b = ("BAT1", &[("type", "Battery"), ("capacity", "75"), ("status", "Charging")][..]);
        assert_eq!(battery_at(&tree("two", &[a, b, MOUSE])), Some(Battery { level: 83, charging: true }));
        let c = ("BAT1", &[("type", "Battery"), ("capacity", "75"), ("status", "Discharging")][..]);
        assert_eq!(battery_at(&tree("two-draining", &[a, c])), Some(Battery { level: 83, charging: false }));
    }
}

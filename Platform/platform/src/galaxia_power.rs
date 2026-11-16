//! Galaxia's C layer for the machine's battery (EmuSen_RustPlatform.md §58).

use crate::{status, text};
use emusen_galaxia::power;
use std::path::Path;

/// The machine's battery, 0 to 100, its charging written where `charging` is not null; `ABSENT` where it has none. A
/// null `root` reads the kernel's own tree; a test names a made-up one.
///
/// # Safety
/// `root` must be valid for `root_len` bytes, or null; `charging` must be valid for one byte, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_battery(root: *const u8, root_len: usize, charging: *mut u8) -> i32 {
    let found = match unsafe { text(root, root_len) } {
        Ok(Some(root)) => power::battery_at(Path::new(root)),
        Ok(None) => power::battery(),
        Err(status) => return status,
    };
    let Some(battery) = found else { return status::ABSENT };
    if !charging.is_null() {
        unsafe { *charging = u8::from(battery.charging) };
    }
    i32::from(battery.level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_made_up_tree_answers_its_battery_and_an_empty_one_absent() {
        let root = std::env::temp_dir().join(format!("platform-power-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("BAT0")).unwrap();
        let named = root.to_str().unwrap();
        assert_eq!(unsafe { emusen_galaxia_battery(named.as_ptr(), named.len(), std::ptr::null_mut()) }, status::ABSENT);
        for (key, value) in [("type", "Battery\n"), ("capacity", "82\n"), ("status", "Charging\n")] {
            std::fs::write(root.join("BAT0").join(key), value).unwrap();
        }
        let mut charging = 0u8;
        assert_eq!(unsafe { emusen_galaxia_battery(named.as_ptr(), named.len(), &mut charging) }, 82);
        assert_eq!(charging, 1);
        assert_eq!(unsafe { emusen_galaxia_battery([0xffu8].as_ptr(), 1, &mut charging) }, status::BAD_STRING);
    }
}

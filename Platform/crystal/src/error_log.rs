//! The runtime's warnings in the player's error log, as the C#'s `ErrorLog.Warning` writes them: one entry, the time
//! formatted by this machine's calendar, in the day's file of the log folder (EmuSen_Settings_Reference.md §4.70).

use emusen_dotnet::date::{TICKS_PER_MILLISECOND, TICKS_PER_SECOND, local_text};
use emusen_galaxia::error_log;

/// A warning of `area`, best effort: a log that cannot be written is never a second fault.
pub fn warning(area: &str, message: &str) {
    let tree = emusen_galaxia::tree::tree();
    let (root, _) = error_log::root(&tree, None);
    let now = std::time::SystemTime::now();
    let local = emusen_galaxia::clock::local_ticks_now();
    let stamp = format!("{}.{:03}", local_text(local, "yyyy-MM-dd HH:mm:ss"), local.rem_euclid(TICKS_PER_SECOND) / TICKS_PER_MILLISECOND);
    let entry = error_log::format_entry(&stamp, "WARN", area, message, None, None);
    let _ = error_log::append(&root, &local_text(local, "yyyyMMdd"), &stamp, now, &entry);
}

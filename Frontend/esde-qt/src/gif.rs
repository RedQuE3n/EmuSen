//! A GIF's frame count and first frame's delay, read by Qt's QImageReader for the scene's timing (EmuSen_Qt.md §4.5.9).

use std::collections::BTreeMap;

use cxx_qt_lib::QString;
use emusen_esde::scene::System;

#[cxx_qt::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("gif_timing.h");
        fn gif_frame_count(path: &QString) -> i32;
        fn gif_first_delay(path: &QString) -> i32;
    }
}

/// The frame count and first delay in ms of a GIF; None where Qt reads no animation from it.
pub fn timing(path: &str) -> Option<(u32, u32)> {
    let q = QString::from(path);
    let frames = u32::try_from(ffi::gif_frame_count(&q)).ok().filter(|n| *n > 0)?;
    let delay = u32::try_from(ffi::gif_first_delay(&q)).ok()?;
    Some((frames, delay))
}

/// Every GIF the systems' animations name, read once each.
pub fn of_systems(systems: &[System]) -> BTreeMap<String, (u32, u32)> {
    let mut out = BTreeMap::new();
    for s in systems {
        let t = &s.theme;
        for e in t.system_view.elements.iter().chain(&t.gamelist_view.elements).filter(|e| e.kind() == "animation") {
            let Some(p) = e.path("path").filter(|p| p.exists && p.absolute.to_ascii_lowercase().ends_with(".gif")) else { continue };
            if !out.contains_key(&p.absolute) {
                if let Some(timing) = timing(&p.absolute) {
                    out.insert(p.absolute.clone(), timing);
                }
            }
        }
    }
    out
}

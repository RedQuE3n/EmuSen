//! `Battery`: the machine's battery as the platform library reads it, asked off the interface thread on each `read`, which
//! the window calls every `every` milliseconds (EmuSen_RustPlatform.md §58.3).

use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use emusen_platform_client::Platform;
use std::sync::Arc;

/// How often the window asks, in milliseconds: a constant of the design, argued in EmuSen_RustPlatform.md §58.3.
pub const EVERY_MS: i32 = 30_000;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// The platform library the battery is asked of.
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary")]
        /// A made-up power_supply tree a test names; empty, the kernel's own. A change of either asks again.
        #[qproperty(QString, root)]
        /// The level, 0 to 100, or -1 where the machine has no battery or none was read yet.
        #[qproperty(i32, level, READ, NOTIFY)]
        #[qproperty(bool, charging, READ, NOTIFY)]
        /// How often the window asks, in milliseconds.
        #[qproperty(i32, every, READ, CONSTANT)]
        type Battery = super::BatteryRust;
    }

    extern "RustQt" {
        /// Asks the platform library again, off the interface thread; the properties change when its answer does.
        #[qinvokable]
        fn read(self: Pin<&mut Battery>);
    }

    impl cxx_qt::Threading for Battery {}
}

pub struct BatteryRust {
    platform_library: QString,
    root: QString,
    level: i32,
    charging: bool,
    every: i32,
    platform: Option<Arc<Platform>>,
    /// Whether a read is out, so a slow one is never joined by another, and whether one was asked for meanwhile.
    asking: bool,
    again: bool,
}

impl Default for BatteryRust {
    fn default() -> Self {
        BatteryRust { platform_library: QString::default(), root: QString::default(), level: -1, charging: false, every: EVERY_MS, platform: None, asking: false, again: false }
    }
}

/// The level and charging the properties hold for an answer: -1 and false for none or a failure.
pub fn shown(answer: Option<emusen_platform_client::galaxia::Battery>) -> (i32, bool) {
    answer.map_or((-1, false), |b| (i32::from(b.level), b.charging))
}

impl qobject::Battery {
    fn read(mut self: Pin<&mut Self>) {
        if self.asking {
            self.as_mut().rust_mut().again = true;
            return;
        }
        if self.platform.is_none() {
            let Ok(platform) = Platform::load(self.platform_library.to_string()) else { return };
            self.as_mut().rust_mut().platform = Some(platform);
        }
        let Some(platform) = self.platform.clone() else { return };
        let root = Some(self.root.to_string()).filter(|r| !r.is_empty());
        self.as_mut().rust_mut().asking = true;
        let queue = self.qt_thread();
        std::thread::spawn(move || {
            let answer = shown(platform.battery(root.as_deref()).ok().flatten());
            let _ = queue.queue(move |battery| battery.answered(answer));
        });
    }

    fn answered(mut self: Pin<&mut Self>, (level, charging): (i32, bool)) {
        self.as_mut().rust_mut().asking = false;
        if self.level != level {
            self.as_mut().rust_mut().level = level;
            self.as_mut().level_changed();
        }
        if self.charging != charging {
            self.as_mut().rust_mut().charging = charging;
            self.as_mut().charging_changed();
        }
        if std::mem::take(&mut self.as_mut().rust_mut().again) {
            self.read();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_platform_client::galaxia::Battery;

    #[test]
    fn none_is_minus_one_and_a_battery_its_level_and_charging() {
        assert_eq!(shown(None), (-1, false));
        assert_eq!(shown(Some(Battery { level: 82, charging: true })), (82, true));
    }
}

//! `Device`, the machine's status as a theme reads it (EmuSen_Qt.md §3.2): the pads and their glyphs' family, the clock,
//! the battery, the network and the form. The host feeds it.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Device"]
        #[qml_singleton]
        #[qproperty(i32, controllers, READ, NOTIFY)]
        #[qproperty(i32, family, READ, NOTIFY)]
        #[qproperty(QString, clock, READ, NOTIFY)]
        #[qproperty(i32, battery, READ, NOTIFY)]
        #[qproperty(bool, charging, READ, NOTIFY)]
        #[qproperty(bool, network, READ, NOTIFY)]
        #[qproperty(bool, big_screen, cxx_name = "bigScreen", READ, NOTIFY)]
        #[qproperty(i32, form, READ, NOTIFY)]
        type ThemeDevice = super::ThemeDeviceRust;
    }
}

/// What the host knows of the machine: `family` and `form` are the kit's numbers (`Families`, `Forms`), `battery` a
/// percentage or -1 where there is none.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceFacts {
    pub controllers: i32,
    pub family: i32,
    pub clock: String,
    pub battery: i32,
    pub charging: bool,
    pub network: bool,
    pub big_screen: bool,
    pub form: i32,
}

impl Default for DeviceFacts {
    fn default() -> Self {
        DeviceFacts {
            controllers: 0,
            family: 0,
            clock: String::new(),
            battery: -1,
            charging: false,
            network: false,
            big_screen: true,
            form: 1,
        }
    }
}

pub struct ThemeDeviceRust {
    controllers: i32,
    family: i32,
    clock: QString,
    battery: i32,
    charging: bool,
    network: bool,
    big_screen: bool,
    form: i32,
}

impl Default for ThemeDeviceRust {
    fn default() -> Self {
        let d = DeviceFacts::default();
        ThemeDeviceRust {
            controllers: d.controllers,
            family: d.family,
            clock: QString::default(),
            battery: d.battery,
            charging: d.charging,
            network: d.network,
            big_screen: d.big_screen,
            form: d.form,
        }
    }
}

impl qobject::ThemeDevice {
    /// The host's feed; each property notifies only when it changed.
    pub fn show(mut self: Pin<&mut Self>, facts: &DeviceFacts) {
        if self.controllers != facts.controllers {
            self.as_mut().rust_mut().controllers = facts.controllers;
            self.as_mut().controllers_changed();
        }
        if self.family != facts.family {
            self.as_mut().rust_mut().family = facts.family;
            self.as_mut().family_changed();
        }
        let clock = QString::from(facts.clock.as_str());
        if self.clock != clock {
            self.as_mut().rust_mut().clock = clock;
            self.as_mut().clock_changed();
        }
        if self.battery != facts.battery {
            self.as_mut().rust_mut().battery = facts.battery;
            self.as_mut().battery_changed();
        }
        if self.charging != facts.charging {
            self.as_mut().rust_mut().charging = facts.charging;
            self.as_mut().charging_changed();
        }
        if self.network != facts.network {
            self.as_mut().rust_mut().network = facts.network;
            self.as_mut().network_changed();
        }
        if self.big_screen != facts.big_screen {
            self.as_mut().rust_mut().big_screen = facts.big_screen;
            self.as_mut().big_screen_changed();
        }
        if self.form != facts.form {
            self.as_mut().rust_mut().form = facts.form;
            self.as_mut().form_changed();
        }
    }
}

//! emusen-qt-play: a game through the ring (EmuSen_Qt.md §1.8). The window's scene graph adopts Serenity's device,
//! GamePicture samples the chain's picture where it lies, and the frame thread paces by the window's swaps.

// The modules the window imports register through their crates' initializers.
extern crate emusen_qt_endymion;

pub mod host;
pub mod window;

use cxx_qt_lib::{QMap, QMapPair_QString_QVariant};
use emusen_platform_client::Platform;
use std::sync::Arc;
use emusen_qt_models::play::Windowed;
use emusen_qt_models::settings_source::ClientSettings;

/// The program's window, in the resource system.
pub const PLAY_URL: &str = "qrc:/qt/qml/EmuSen/Play/qml/Play.qml";

/// graphics.json's window size as Play.qml's initial properties (EmuSen_Qt.md §7.7.H7), none where it was not read whole;
/// a failed read is said in its own words on standard error, as the start says its others.
pub fn windowed_initial(platform: &Arc<Platform>, initial: &mut QMap<QMapPair_QString_QVariant>) {
    let read = Windowed::read(&ClientSettings::on(platform.clone())).inspect_err(|words| eprintln!("{words}"));
    if let Some(windowed) = read.ok().flatten() {
        windowed.insert_into(initial);
    }
}

/// The instance extensions Qt's platform plugin presents with, by its name: the surface, the plugin's own, and the
/// colour spaces Qt picks a swapchain's from when they are offered.
pub fn surface_extensions(platform: &str) -> Vec<&'static str> {
    let own = match platform {
        "wayland" | "wayland-egl" => "VK_KHR_wayland_surface",
        _ => "VK_KHR_xcb_surface",
    };
    vec!["VK_KHR_surface", own, "VK_EXT_swapchain_colorspace"]
}

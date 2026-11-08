//! emusen-qt-play: a game through the ring (EmuSen_Qt.md §1.8). The window's scene graph adopts Serenity's device,
//! GamePicture samples the chain's picture where it lies, and the frame thread paces by the window's swaps.

// The modules the window imports register through their crates' initializers.
extern crate emusen_qt_endymion;

pub mod window;

/// The program's window, in the resource system.
pub const PLAY_URL: &str = "qrc:/qt/qml/EmuSen/Play/qml/Play.qml";

/// The instance extensions Qt's platform plugin presents with, by its name: the surface, the plugin's own, and the
/// colour spaces Qt picks a swapchain's from when they are offered.
pub fn surface_extensions(platform: &str) -> Vec<&'static str> {
    let own = match platform {
        "wayland" | "wayland-egl" => "VK_KHR_wayland_surface",
        _ => "VK_KHR_xcb_surface",
    };
    vec!["VK_KHR_surface", own, "VK_EXT_swapchain_colorspace"]
}

//! A window's game picture host (EmuSen_Qt.md §1.8, §7.1): Serenity's device made for Qt's platform plugin, the feed
//! drawing through the plain preset until a game's display settings ask for another, and the window adopting the
//! device before it is first shown. emusen-qt-play and emusen-qt start through it, and so do their tests in a
//! compositor of their own.

use std::path::Path;
use std::sync::Arc;

use emusen_kit_qt::picture::{adopt, Adopted};
use emusen_platform_client::serenity::{builtin_preset, Device};
use emusen_platform_client::Platform;
use emusen_qt_models::display::PLAIN;
use emusen_qt_models::picture::PictureFeed;

use crate::surface_extensions;

/// The device and the feed, made before the window so that they outlive it, on a failure too.
pub struct Host {
    pub device: Arc<Device>,
    pub feed: PictureFeed,
    /// The instance extensions Qt adopts the instance with.
    extensions: Vec<&'static str>,
}

impl Host {
    /// The device for `platform_name` (Qt's platform plugin), on the card `gpu` names or the first, validated when
    /// `validate`; the preset's files are kept under `presets`.
    pub fn new(platform: &Arc<Platform>, platform_name: &str, gpu: Option<&str>, validate: bool, presets: &Path) -> Result<Host, String> {
        let mut extensions = surface_extensions(platform_name);
        let device = Arc::new(Device::new_for(platform, gpu, validate, &extensions, &["VK_KHR_swapchain"]).map_err(|e| e.to_string())?);
        let preset = builtin_preset(platform, PLAIN, None, &presets.to_string_lossy()).map_err(|e| e.to_string())?;
        let feed = PictureFeed::new(device.clone(), preset, presets.to_path_buf());
        if validate {
            extensions.push("VK_EXT_debug_utils");
        }
        Ok(Host { device, feed, extensions })
    }

    /// What a window needs to adopt the device: plain handles, so a window's creation hook may hold them.
    pub fn adoption(&self) -> Adoption {
        let raw = self.device.raw();
        Adoption { instance: raw.instance, physical: raw.physical, device: raw.device, queue_family: raw.queue_family, extensions: self.extensions.clone() }
    }
}

/// The device's handles and the instance's extensions, as a window adopts them.
#[derive(Clone, Debug)]
pub struct Adoption {
    instance: u64,
    physical: u64,
    device: u64,
    queue_family: u32,
    extensions: Vec<&'static str>,
}

impl Adoption {
    /// The window's scene graph put on the device; the words when Qt refuses.
    ///
    /// # Safety
    /// `window` is a live `QQuickWindow` that has never been shown, and the host the handles came from is alive.
    pub unsafe fn adopt(&self, window: *mut std::ffi::c_void) -> Result<(), String> {
        let adopted = Adopted {
            instance: self.instance,
            physical: self.physical,
            device: self.device,
            queue_family: self.queue_family,
            queue_index: 0,
            extensions: &self.extensions,
        };
        unsafe { adopt(window, &adopted) }
    }
}
